//! Validation implementations for v1 protocol types.
//!
//! The actual validation logic is shared with v2 and lives in the
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

    #[cfg(feature = "unstable_auth_methods")]
    #[test]
    fn test_auth_env_var_validation() {
        let valid_env_var = AuthEnvVar {
            name: "API_KEY".to_string(),
            label: Some("API Key".to_string()),
            secret: true,
            optional: false,
            meta: None,
        };
        assert!(valid_env_var.validate().is_ok());

        let invalid_env_var = AuthEnvVar {
            name: "invalid_name".to_string(), // lowercase not allowed
            label: None,
            secret: true,
            optional: false,
            meta: None,
        };
        assert!(invalid_env_var.validate().is_err());

        let reserved_name_env_var = AuthEnvVar {
            name: "PATH".to_string(), // reserved name
            label: None,
            secret: false,
            optional: false,
            meta: None,
        };
        assert!(reserved_name_env_var.validate().is_err());
    }

    #[cfg(feature = "unstable_auth_methods")]
    #[test]
    fn test_auth_method_env_var_validation() {
        let env_vars = vec![
            AuthEnvVar {
                name: "API_KEY".to_string(),
                label: Some("API Key".to_string()),
                secret: true,
                optional: false,
                meta: None,
            },
            AuthEnvVar {
                name: "SECRET_TOKEN".to_string(),
                label: Some("Secret Token".to_string()),
                secret: true,
                optional: true,
                meta: None,
            },
        ];

        let valid_method = AuthMethodEnvVar {
            id: AuthMethodId::new("env_method"),
            name: "Environment Variables".to_string(),
            description: Some("Authenticate using environment variables".to_string()),
            vars: env_vars,
            link: Some("https://example.com/setup".to_string()),
            meta: None,
        };
        assert!(valid_method.validate().is_ok());

        // Test with duplicate variable names
        let duplicate_vars = vec![
            AuthEnvVar {
                name: "API_KEY".to_string(),
                label: Some("API Key".to_string()),
                secret: true,
                optional: false,
                meta: None,
            },
            AuthEnvVar {
                name: "API_KEY".to_string(), // duplicate name
                label: Some("Another API Key".to_string()),
                secret: true,
                optional: true,
                meta: None,
            },
        ];

        let invalid_method = AuthMethodEnvVar {
            id: AuthMethodId::new("env_method"),
            name: "Environment Variables".to_string(),
            description: None,
            vars: duplicate_vars,
            link: None,
            meta: None,
        };
        assert!(invalid_method.validate().is_err());
    }

    #[test]
    fn test_validate_auth_methods_collection() {
        let methods = vec![
            AuthMethod::Agent(AuthMethodAgent {
                id: AuthMethodId::new("method1"),
                name: "Method 1".to_string(),
                description: None,
                meta: None,
            }),
            AuthMethod::Agent(AuthMethodAgent {
                id: AuthMethodId::new("method2"),
                name: "Method 2".to_string(),
                description: None,
                meta: None,
            }),
        ];
        assert!(validate_auth_methods(&methods).is_ok());

        // Test with duplicate IDs
        let duplicate_methods = vec![
            AuthMethod::Agent(AuthMethodAgent {
                id: AuthMethodId::new("method1"),
                name: "Method 1".to_string(),
                description: None,
                meta: None,
            }),
            AuthMethod::Agent(AuthMethodAgent {
                id: AuthMethodId::new("method1"), // duplicate ID
                name: "Method 1 Copy".to_string(),
                description: None,
                meta: None,
            }),
        ];
        assert!(validate_auth_methods(&duplicate_methods).is_err());
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
        use crate::v1::{ElicitationScope, ElicitationSessionScope};

        let valid_url_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v1::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new("test_elicitation"),
            url: "https://example.com/register".to_string(),
        };
        assert!(valid_url_mode.validate().is_ok());

        let invalid_url_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v1::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new("test_elicitation"),
            url: "not_a_url".to_string(),
        };
        assert!(invalid_url_mode.validate().is_err());

        let invalid_id_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v1::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new(""),
            url: "https://example.com/register".to_string(),
        };
        assert!(invalid_id_mode.validate().is_err());
    }
}
