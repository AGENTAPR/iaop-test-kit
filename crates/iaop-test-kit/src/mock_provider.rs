//! Mock external HTTP provider simulator for integration tests.
//!
//! Spawns an in-memory Axum HTTP server on an ephemeral port (127.0.0.1:0)
//! to simulate external provider behavior:
//! - Success responses (200, 201)
//! - Validation rejections (400, 422)
//! - Authentication rejections (401, 403)
//! - Rate limiting (429 with Retry-After)
//! - Timeouts / slow responses
//! - Upstream server errors (500, 502, 503)
//! - Recorded request history inspection

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: Method,
    pub path: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub enum MockBehavior {
    RespondJson(StatusCode, serde_json::Value),
    RespondText(StatusCode, String),
    RateLimit {
        retry_after_secs: u32,
    },
    Delay {
        duration: Duration,
        and_then: Box<MockBehavior>,
    },
}

#[derive(Clone, Default)]
struct MockState {
    behaviors: Arc<Mutex<VecDeque<MockBehavior>>>,
    recorded: Arc<Mutex<Vec<RecordedRequest>>>,
}

pub struct MockHttpProvider {
    addr: SocketAddr,
    state: MockState,
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl MockHttpProvider {
    /// Starts a new mock provider listening on 127.0.0.1:0.
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");

        let state = MockState::default();
        let app = Router::new()
            .route("/{*path}", any(mock_handler))
            .route("/", any(mock_handler))
            .with_state(state.clone());

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await;
        });

        Self {
            addr,
            state,
            _shutdown: tx,
        }
    }

    /// Base URL (e.g. `http://127.0.0.1:54321`).
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Queues a JSON response.
    pub fn enqueue_json(&self, status: StatusCode, json: serde_json::Value) {
        self.state
            .behaviors
            .lock()
            .unwrap()
            .push_back(MockBehavior::RespondJson(status, json));
    }

    /// Queues a rate-limit response (429 Too Many Requests).
    pub fn enqueue_rate_limit(&self, retry_after_secs: u32) {
        self.state
            .behaviors
            .lock()
            .unwrap()
            .push_back(MockBehavior::RateLimit { retry_after_secs });
    }

    /// Queues a delayed response to simulate timeouts or latency.
    pub fn enqueue_delay(&self, duration: Duration, and_then: MockBehavior) {
        self.state
            .behaviors
            .lock()
            .unwrap()
            .push_back(MockBehavior::Delay {
                duration,
                and_then: Box::new(and_then),
            });
    }

    /// Returns all recorded requests.
    pub fn recorded_requests(&self) -> Vec<RecordedRequest> {
        self.state.recorded.lock().unwrap().clone()
    }

    /// Clears recorded requests.
    pub fn clear_recorded(&self) {
        self.state.recorded.lock().unwrap().clear();
    }
}

async fn mock_handler(
    State(state): State<MockState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Record request
    state.recorded.lock().unwrap().push(RecordedRequest {
        method,
        path: uri.path().to_string(),
        headers,
        body: body.to_vec(),
    });

    let behavior = state.behaviors.lock().unwrap().pop_front();
    match behavior {
        Some(b) => execute_behavior(b).await,
        None => (
            StatusCode::OK,
            [("content-type", "application/json")],
            "{\"status\":\"default_mock_ok\"}",
        )
            .into_response(),
    }
}

async fn execute_behavior(b: MockBehavior) -> Response {
    match b {
        MockBehavior::RespondJson(status, json) => (
            status,
            [("content-type", "application/json")],
            json.to_string(),
        )
            .into_response(),
        MockBehavior::RespondText(status, text) => (status, text).into_response(),
        MockBehavior::RateLimit { retry_after_secs } => (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("retry-after", retry_after_secs.to_string()),
                ("content-type", "application/json".to_string()),
            ],
            "{\"error\":\"rate_limited\"}",
        )
            .into_response(),
        MockBehavior::Delay { duration, and_then } => {
            tokio::time::sleep(duration).await;
            Box::pin(execute_behavior(*and_then)).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_provider_serves_json_and_records_request() {
        let server = MockHttpProvider::start().await;
        server.enqueue_json(StatusCode::CREATED, serde_json::json!({"created": true}));

        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/api/v1/test", server.base_url()))
            .json(&serde_json::json!({"ping": "pong"}))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);
        let reqs = server.recorded_requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].path, "/api/v1/test");
    }
}
