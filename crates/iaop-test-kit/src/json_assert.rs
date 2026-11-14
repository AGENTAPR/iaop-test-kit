//! JSON assertion helpers for API/contract tests.

use serde_json::Value;

/// Asserts `actual` contains all key/value pairs of `expected` (deep).
/// Returns a readable diff on failure.
pub fn assert_json_subset(actual: &Value, expected: &Value) -> Result<(), String> {
    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => {
            for (k, ev) in e {
                match a.get(k) {
                    Some(av) => assert_json_subset(av, ev)?,
                    None => return Err(format!("missing key '{k}'")),
                }
            }
            Ok(())
        }
        (Value::Array(a), Value::Array(e)) => {
            if a.len() != e.len() {
                return Err(format!("array length {} != {}", a.len(), e.len()));
            }
            for (i, (av, ev)) in a.iter().zip(e.iter()).enumerate() {
                assert_json_subset(av, ev).map_err(|m| format!("[{i}]: {m}"))?;
            }
            Ok(())
        }
        (a, e) if a == e => Ok(()),
        (a, e) => Err(format!("value mismatch: {a} != {e}")),
    }
}

/// Asserts a Problem Details response has the expected `code` and `status`.
pub fn assert_problem_details(
    body: &Value,
    expected_code: &str,
    expected_status: u64,
) -> Result<(), String> {
    let code = body.get("code").and_then(Value::as_str).unwrap_or("");
    let status = body.get("status").and_then(Value::as_u64).unwrap_or(0);
    if code != expected_code {
        return Err(format!("expected code '{expected_code}', got '{code}'"));
    }
    if status != expected_status {
        return Err(format!("expected status {expected_status}, got {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn subset_matches() {
        let actual = json!({"a": 1, "b": {"c": 2, "d": 3}});
        let expected = json!({"b": {"c": 2}});
        assert!(assert_json_subset(&actual, &expected).is_ok());
    }

    #[test]
    fn subset_fails_on_missing() {
        let actual = json!({"a": 1});
        let expected = json!({"b": 2});
        assert!(assert_json_subset(&actual, &expected).is_err());
    }
}
