//! Test identifiers.

use uuid::Uuid;

/// Generates a unique test identifier (UUIDv7).
pub fn unique_id() -> Uuid {
    Uuid::now_v7()
}

/// Deterministic-ish unique suffix for names/keys in tests.
pub fn unique_suffix() -> String {
    Uuid::now_v7().simple().to_string()[..12].to_string()
}
