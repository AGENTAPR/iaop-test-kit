//! Kafka test helpers and topic assertion utilities.

use uuid::Uuid;

pub struct KafkaTestHelper;

impl KafkaTestHelper {
    /// Returns configured brokers or standard local dev fallback.
    pub fn default_brokers() -> String {
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "127.0.0.1:9092".to_string())
    }

    /// Generates a randomized isolated topic name for test execution.
    pub fn unique_topic(prefix: &str) -> String {
        format!("{}.test.{}", prefix, Uuid::now_v7().simple())
    }

    /// Asserts that a serialized event matches canonical envelope structure.
    pub fn assert_canonical_envelope(event: &serde_json::Value, expected_type: &str) {
        assert_eq!(
            event["event_type"].as_str().unwrap_or_default(),
            expected_type,
            "event_type mismatch"
        );
        assert!(
            event["event_id"].is_string(),
            "event_id must be present as a UUID string"
        );
        assert!(
            event["occurred_at"].is_string(),
            "occurred_at must be an ISO-8601 timestamp"
        );
        assert!(
            event["payload"].is_object(),
            "payload must be a valid JSON object"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_unique_topic() {
        let t1 = KafkaTestHelper::unique_topic("agent");
        let t2 = KafkaTestHelper::unique_topic("agent");
        assert_ne!(t1, t2);
        assert!(t1.starts_with("agent.test."));
    }
}
