//! OIDC and Keycloak test helpers.
//!
//! Generates structured mock JWTs and authorization headers for testing
//! valid tokens, expired tokens, audience mismatches, issuer mismatches,
//! and role/tenant scope assertions.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

/// Standard claims matching IAOP Keycloak tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
    pub agent_scope: Option<String>,
}

pub struct OidcTestHelper;

impl OidcTestHelper {
    /// Generates a serialized mock JWT with unverified header/signature (for dev/test bypass or decoding tests).
    pub fn create_token(
        subject: &str,
        roles: Vec<&str>,
        issuer: &str,
        audience: &str,
        validity: Duration,
    ) -> String {
        let now = Utc::now();
        let claims = MockClaims {
            sub: subject.to_string(),
            iss: issuer.to_string(),
            aud: audience.to_string(),
            exp: (now + validity).timestamp(),
            iat: now.timestamp(),
            preferred_username: Some(subject.to_string()),
            email: Some(format!("{}@iaop.internal", subject)),
            roles: roles.into_iter().map(String::from).collect(),
            tenant_id: Some("default-tenant".to_string()),
            agent_scope: None,
        };

        // Construct standard Base64URL-encoded JWT
        let header = serde_json::json!({
            "alg": "none",
            "typ": "JWT"
        });

        let h_b64 = base64_url_encode(&serde_json::to_vec(&header).unwrap());
        let c_b64 = base64_url_encode(&serde_json::to_vec(&claims).unwrap());
        let sig = "mock_test_signature";

        format!("{}.{}.{}", h_b64, c_b64, sig)
    }

    /// Generates a valid standard test token.
    pub fn valid_token(subject: &str, roles: Vec<&str>) -> String {
        Self::create_token(
            subject,
            roles,
            "http://keycloak:8080/realms/iaop",
            "iaop-services",
            Duration::hours(1),
        )
    }

    /// Generates an expired test token for security rejection tests.
    pub fn expired_token(subject: &str, roles: Vec<&str>) -> String {
        Self::create_token(
            subject,
            roles,
            "http://keycloak:8080/realms/iaop",
            "iaop-services",
            -Duration::hours(2),
        )
    }

    /// Generates a token with an unauthorized audience.
    pub fn wrong_audience_token(subject: &str, roles: Vec<&str>) -> String {
        Self::create_token(
            subject,
            roles,
            "http://keycloak:8080/realms/iaop",
            "unauthorized-audience",
            Duration::hours(1),
        )
    }

    /// Generates a token with an invalid issuer.
    pub fn wrong_issuer_token(subject: &str, roles: Vec<&str>) -> String {
        Self::create_token(
            subject,
            roles,
            "http://malicious-issuer.com/realms/fake",
            "iaop-services",
            Duration::hours(1),
        )
    }

    /// Formats a Bearer Authorization header value.
    pub fn bearer(token: &str) -> String {
        format!("Bearer {}", token)
    }
}

fn base64_url_encode(data: &[u8]) -> String {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    URL_SAFE_NO_PAD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_token_has_three_segments() {
        let tok = OidcTestHelper::valid_token("test-user", vec!["agent:read"]);
        let parts: Vec<&str> = tok.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn bearer_prepends_bearer() {
        let b = OidcTestHelper::bearer("abc");
        assert_eq!(b, "Bearer abc");
    }
}
