//! Validation implementations for v2 protocol types.
//!
//! The actual validation logic is shared with v1 and lives in the
//! `impl_protocol_validation!` macro defined in `crate::validation_macros`.

use super::{AuthMethod, AuthMethodAgent, AuthMethodId, AuthenticateRequest};

#[cfg(feature = "unstable_auth_methods")]
use super::{AuthEnvVar, AuthMethodEnvVar};

#[cfg(feature = "unstable_elicitation")]
use super::{
    CreateElicitationRequest, ElicitationFormMode, ElicitationId, ElicitationMode,
    ElicitationUrlMode,
};

crate::impl_protocol_validation!();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::Validate;

    #[test]
    fn test_authenticate_request_validation() {
        let valid_request = AuthenticateRequest {
            method_id: AuthMethodId::new("valid_method"),
            meta: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AuthenticateRequest {
            method_id: AuthMethodId::new(""),
            meta: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_auth_method_agent_validation() {
        let valid_method = AuthMethodAgent {
            id: AuthMethodId::new("test_method"),
            name: "Test Method".to_string(),
            description: Some("A test authentication method".to_string()),
            meta: None,
        };
        assert!(valid_method.validate().is_ok());

        let invalid_method = AuthMethodAgent {
            id: AuthMethodId::new(""),
            name: "Test Method".to_string(),
            description: None,
            meta: None,
        };
        assert!(invalid_method.validate().is_err());

        let empty_name_method = AuthMethodAgent {
            id: AuthMethodId::new("test_method"),
            name: "".to_string(),
            description: None,
            meta: None,
        };
        assert!(empty_name_method.validate().is_err());
    }

    #[cfg(feature = "unstable_elicitation")]
    #[test]
    fn test_elicitation_id_validation() {
        let valid_id = ElicitationId::new("registration_form");
        assert!(valid_id.validate().is_ok());

        let valid_id_with_dots = ElicitationId::new("com.example.registration");
        assert!(valid_id_with_dots.validate().is_ok());

        let invalid_empty_id = ElicitationId::new("");
        assert!(invalid_empty_id.validate().is_err());

        let invalid_id_with_spaces = ElicitationId::new("registration form");
        assert!(invalid_id_with_spaces.validate().is_err());
    }

    #[cfg(feature = "unstable_elicitation")]
    #[test]
    fn test_elicitation_url_mode_validation() {
        use crate::v2::{ElicitationScope, ElicitationSessionScope};

        let valid_url_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v2::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new("test_elicitation"),
            url: "https://example.com/register".to_string(),
        };
        assert!(valid_url_mode.validate().is_ok());

        let invalid_url_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v2::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new("test_elicitation"),
            url: "not_a_url".to_string(),
        };
        assert!(invalid_url_mode.validate().is_err());

        let invalid_id_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v2::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new(""),
            url: "https://example.com/register".to_string(),
        };
        assert!(invalid_id_mode.validate().is_err());
    }
}
