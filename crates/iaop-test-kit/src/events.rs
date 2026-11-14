//! Builders for `iaop-event-contracts`-compatible event envelopes in tests.

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Test envelope mirroring the canonical event envelope (camelCase wire
/// format of `iaop-messaging::EventEnvelope` / `envelope.schema.json`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestEventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub event_version: u32,
    pub source: String,
    pub aggregate_id: Option<Uuid>,
    pub aggregate_version: Option<u64>,
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub payload: Value,
}

impl TestEventEnvelope {
    /// Creates a valid v1 envelope for `event_type` produced by `source`.
    pub fn new(event_type: impl Into<String>, source: impl Into<String>, payload: Value) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            event_type: event_type.into(),
            event_version: 1,
            source: source.into(),
            aggregate_id: None,
            aggregate_version: None,
            occurred_at: Utc::now(),
            correlation_id: None,
            causation_id: None,
            payload,
        }
    }

    pub fn aggregate(mut self, id: Uuid, version: u64) -> Self {
        self.aggregate_id = Some(id);
        self.aggregate_version = Some(version);
        self
    }

    pub fn with_correlation(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Serializes to JSON matching the canonical envelope schema.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("envelope serialization cannot fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn envelope_serializes_camel_case() {
        let e = TestEventEnvelope::new("agent.activated", "iaop-agent-service", json!({}));
        let v = e.to_json();
        assert!(v.get("eventId").is_some());
        assert!(v.get("eventType").is_some());
        assert_eq!(v["eventVersion"], 1);
    }
}
