//! Shared validation logic for protocol versions.
//!
//! The validation rules for authentication methods and elicitation requests
//! are identical between protocol v1 and v2 — the only differences are the
//! concrete types resolved from `super::` and the version-specific
//! `ElicitationMode` enum. Rather than maintaining two parallel copies of
//! ~250 lines of validation logic (which is prone to drift), this macro
//! generates the `Validate` implementations once per version.
//!
//! Each version's `validation_impl.rs` invokes `crate::impl_protocol_validation!();`
//! after bringing the version's types into scope via `use super::*;`. The macro
//! expansion resolves the in-scope types (e.g. `AuthenticateRequest`,
//! `ElicitationMode`) relative to the call site, so those imports determine
//! which protocol version's types are validated.

/// Generates the `Validate` impls for a protocol version's auth + elicitation
/// types.
///
/// All types referenced inside the macro (`AuthenticateRequest`, `AuthMethod`,
/// `ElicitationMode`, etc.) are looked up by their bare names and must be
/// brought into scope by the calling module via `use super::*;` (or explicit
/// imports). This is how a single macro can generate v1 and v2 impls without
/// passing the version as a parameter.
#[macro_export]
#[doc(hidden)]
macro_rules! impl_protocol_validation {
    () => {
        impl $crate::validation::Validate for AuthenticateRequest {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                $crate::validation::validate_auth_method_id(&self.method_id.0)?;
                Ok(())
            }
        }

        impl $crate::validation::Validate for AuthMethodId {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                $crate::validation::validate_auth_method_id(&self.0)
            }
        }

        impl $crate::validation::Validate for AuthMethodAgent {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                $crate::validation::Validate::validate(&self.id)?;

                let name_constraints = $crate::validation::StringConstraints {
                    min_length: Some(1),
                    max_length: Some(256),
                    allow_empty: false,
                    ..Default::default()
                };
                $crate::validation::validate_string_field(&self.name, "name", &name_constraints)?;

                let description_constraints = $crate::validation::StringConstraints {
                    min_length: None,
                    max_length: Some(1024),
                    allow_empty: true,
                    ..Default::default()
                };
                $crate::validation::validate_optional_string_field(
                    &self.description,
                    "description",
                    &description_constraints,
                )?;

                Ok(())
            }
        }

        #[cfg(feature = "unstable_auth_methods")]
        impl $crate::validation::Validate for AuthMethodEnvVar {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                $crate::validation::Validate::validate(&self.id)?;

                let name_constraints = $crate::validation::StringConstraints {
                    min_length: Some(1),
                    max_length: Some(256),
                    allow_empty: false,
                    ..Default::default()
                };
                $crate::validation::validate_string_field(&self.name, "name", &name_constraints)?;

                let description_constraints = $crate::validation::StringConstraints {
                    min_length: None,
                    max_length: Some(1024),
                    allow_empty: true,
                    ..Default::default()
                };
                $crate::validation::validate_optional_string_field(
                    &self.description,
                    "description",
                    &description_constraints,
                )?;

                for env_var in &self.vars {
                    $crate::validation::Validate::validate(env_var)?;
                }

                $crate::validation::validate_unique_ids(&self.vars, |var| &var.name)?;

                if let Some(link) = &self.link {
                    $crate::validation::validate_url(link)?;
                }

                Ok(())
            }
        }

        #[cfg(feature = "unstable_auth_methods")]
        impl $crate::validation::Validate for AuthEnvVar {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                $crate::validation::validate_env_var_name(&self.name)?;

                let label_constraints = $crate::validation::StringConstraints {
                    min_length: None,
                    max_length: Some(256),
                    allow_empty: true,
                    ..Default::default()
                };
                $crate::validation::validate_optional_string_field(
                    &self.label,
                    "label",
                    &label_constraints,
                )?;

                Ok(())
            }
        }

        impl $crate::validation::Validate for AuthMethod {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                match self {
                    AuthMethod::Agent(agent) => $crate::validation::Validate::validate(agent),
                    #[cfg(feature = "unstable_auth_methods")]
                    AuthMethod::EnvVar(env_var) => $crate::validation::Validate::validate(env_var),
                    #[cfg(feature = "unstable_auth_methods")]
                    AuthMethod::Terminal(terminal) => {
                        $crate::validation::Validate::validate(&terminal.id)?;

                        let name_constraints = $crate::validation::StringConstraints {
                            min_length: Some(1),
                            max_length: Some(256),
                            allow_empty: false,
                            ..Default::default()
                        };
                        $crate::validation::validate_string_field(
                            &terminal.name,
                            "name",
                            &name_constraints,
                        )?;

                        let description_constraints = $crate::validation::StringConstraints {
                            min_length: None,
                            max_length: Some(1024),
                            allow_empty: true,
                            ..Default::default()
                        };
                        $crate::validation::validate_optional_string_field(
                            &terminal.description,
                            "description",
                            &description_constraints,
                        )?;

                        Ok(())
                    }
                }
            }
        }

        /// Validates a collection of authentication methods.
        pub fn validate_auth_methods(
            methods: &[AuthMethod],
        ) -> ::core::result::Result<(), $crate::validation::ValidationError> {
            for method in methods {
                $crate::validation::Validate::validate(method)?;
            }

            $crate::validation::validate_unique_ids(methods, |method| &method.id().0)?;

            Ok(())
        }

        #[cfg(feature = "unstable_elicitation")]
        impl $crate::validation::Validate for ElicitationId {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                let constraints = $crate::validation::StringConstraints {
                    min_length: Some(1),
                    max_length: Some(128),
                    allow_empty: false,
                    pattern: None,
                };
                $crate::validation::validate_string_field(&self.0, "elicitation_id", &constraints)?;

                if !$crate::validation::IDENTIFIER_PATTERN.is_match(&self.0) {
                    return Err($crate::validation::ValidationError::InvalidCharacters {
                        field: "elicitation_id".to_string(),
                    });
                }

                Ok(())
            }
        }

        #[cfg(feature = "unstable_elicitation")]
        impl $crate::validation::Validate for CreateElicitationRequest {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                let message_constraints = $crate::validation::StringConstraints {
                    min_length: Some(1),
                    max_length: Some(2048),
                    allow_empty: false,
                    ..Default::default()
                };
                $crate::validation::validate_string_field(
                    &self.message,
                    "message",
                    &message_constraints,
                )?;

                match &self.mode {
                    ElicitationMode::Form(form) => $crate::validation::Validate::validate(form)?,
                    ElicitationMode::Url(url) => $crate::validation::Validate::validate(url)?,
                }

                Ok(())
            }
        }

        #[cfg(feature = "unstable_elicitation")]
        impl $crate::validation::Validate for ElicitationFormMode {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                if self.requested_schema.properties.is_empty() {
                    return Err($crate::validation::ValidationError::RequiredFieldMissing {
                        field: "requested_schema.properties".to_string(),
                    });
                }

                if let Some(title) = &self.requested_schema.title {
                    let title_constraints = $crate::validation::StringConstraints {
                        min_length: Some(1),
                        max_length: Some(256),
                        allow_empty: false,
                        ..Default::default()
                    };
                    $crate::validation::validate_string_field(title, "title", &title_constraints)?;
                }

                if let Some(description) = &self.requested_schema.description {
                    let description_constraints = $crate::validation::StringConstraints {
                        min_length: None,
                        max_length: Some(1024),
                        allow_empty: true,
                        ..Default::default()
                    };
                    $crate::validation::validate_string_field(
                        description,
                        "description",
                        &description_constraints,
                    )?;
                }

                Ok(())
            }
        }

        #[cfg(feature = "unstable_elicitation")]
        impl $crate::validation::Validate for ElicitationUrlMode {
            fn validate(&self) -> ::core::result::Result<(), $crate::validation::ValidationError> {
                $crate::validation::Validate::validate(&self.elicitation_id)?;
                $crate::validation::validate_url(&self.url)?;
                Ok(())
            }
        }
    };
}
