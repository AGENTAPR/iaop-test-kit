//! Evidence capture for contract verification.
//!
//! Captures sanitized request/response payloads with provenance metadata so
//! generated examples and conformance checks trace back to verifiable sources
//! (live DEV calls or isolated DB fixtures). Secrets and credentials are
//! redacted before anything is persisted.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Where the evidence came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// Captured from a live running service (e.g. InstaBox DEV).
    Live,
    /// Captured from an isolated PostgreSQL test fixture.
    Fixture,
}

/// Provenance record attached to every captured payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMeta {
    pub service: String,
    pub operation_id: String,
    pub source: EvidenceSource,
    /// Git SHA of the service repo at capture time, if known.
    pub source_commit: Option<String>,
    pub captured_at: String,
    pub environment: String,
}

/// A sanitized evidence record: meta + payloads, safe to persist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub meta: EvidenceMeta,
    pub request: Option<Value>,
    pub response_status: u16,
    pub response_body: Option<Value>,
}

const SENSITIVE_KEYS: &[&str] = &[
    "password", "passwd", "secret", "token", "access_token", "refresh_token",
    "id_token", "authorization", "cookie", "set-cookie", "api_key", "apikey",
    "client_secret", "private_key", "credential",
];

fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_lowercase();
    SENSITIVE_KEYS.iter().any(|s| k == *s || k.ends_with(&format!("_{s}")) || k.ends_with(s))
}

/// Recursively redact sensitive values in a JSON payload.
pub fn sanitize(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(k) {
                    out.insert(k.clone(), Value::String("[REDACTED]".into()));
                } else {
                    out.insert(k.clone(), sanitize(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sanitize).collect()),
        other => other.clone(),
    }
}

/// Build a sanitized evidence record.
pub fn capture(
    service: &str,
    operation_id: &str,
    source: EvidenceSource,
    environment: &str,
    request: Option<Value>,
    response_status: u16,
    response_body: Option<Value>,
) -> EvidenceRecord {
    EvidenceRecord {
        meta: EvidenceMeta {
            service: service.to_string(),
            operation_id: operation_id.to_string(),
            source,
            source_commit: std::env::var("GIT_COMMIT").ok(),
            captured_at: chrono::Utc::now().to_rfc3339(),
            environment: environment.to_string(),
        },
        request: request.map(|r| sanitize(&r)),
        response_status,
        response_body: response_body.map(|r| sanitize(&r)),
    }
}

/// Serialize an evidence record to canonical JSON (sorted keys) for stable diffs.
pub fn to_canonical_json(record: &EvidenceRecord) -> String {
    let v = serde_json::to_value(record).expect("evidence serializes");
    let sorted = sort_value(v);
    serde_json::to_string_pretty(&sorted).expect("evidence serializes")
}

fn sort_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, sort_value(v)))
                .collect();
            serde_json::to_value(sorted).expect("map serializes")
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}
