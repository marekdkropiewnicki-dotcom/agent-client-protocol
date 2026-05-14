//! Input validation utilities for Agent Client Protocol types.
//!
//! This module provides validation functions for authentication-related protocol types
//! to ensure data integrity and security for user registration and authentication flows.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Pattern for identifier-style strings (auth method IDs, elicitation IDs, etc.).
///
/// Compiled once via `LazyLock` to avoid re-parsing the regex on every
/// validation call (validation runs in hot paths like `validate_auth_methods`
/// which iterates over a collection).
pub(crate) static IDENTIFIER_PATTERN: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_\-\.]+$").unwrap());

/// Pattern for POSIX-style environment variable names.
static ENV_VAR_NAME_PATTERN: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap());

/// Validation errors that can occur when validating protocol types.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("Field '{field}' is required but was empty or missing")]
    RequiredFieldMissing { field: String },
    
    #[error("Field '{field}' is too short (minimum length: {min_length})")]
    TooShort { field: String, min_length: usize },
    
    #[error("Field '{field}' is too long (maximum length: {max_length})")]
    TooLong { field: String, max_length: usize },
    
    #[error("Field '{field}' contains invalid characters")]
    InvalidCharacters { field: String },
    
    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },
    
    #[error("Duplicate identifier found: {id}")]
    DuplicateId { id: String },
    
    #[error("Environment variable name '{name}' is invalid: {reason}")]
    InvalidEnvVarName { name: String, reason: String },
    
    #[error("URL '{url}' is invalid: {reason}")]
    InvalidUrl { url: String, reason: String },
}

pub type ValidationResult<T> = Result<T, ValidationError>;

/// Trait for types that can be validated.
pub trait Validate {
    /// Validates the type and returns any validation errors.
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Validation constraints for string fields.
#[derive(Debug, Clone)]
pub struct StringConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub allow_empty: bool,
    pub pattern: Option<regex::Regex>,
}

impl Default for StringConstraints {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            allow_empty: false,
            pattern: None,
        }
    }
}

/// Validates a string field against constraints.
pub fn validate_string_field(
    value: &str,
    field_name: &str,
    constraints: &StringConstraints,
) -> ValidationResult<()> {
    // Check if empty when not allowed
    if value.is_empty() && !constraints.allow_empty {
        return Err(ValidationError::RequiredFieldMissing {
            field: field_name.to_string(),
        });
    }
    
    // Check length constraints in characters (not bytes) so that multi-byte
    // UTF-8 content (CJK, emoji, accented letters) is measured against the
    // limit the caller intended.
    if constraints.min_length.is_some() || constraints.max_length.is_some() {
        let char_count = value.chars().count();

        if let Some(min_len) = constraints.min_length {
            if char_count < min_len {
                return Err(ValidationError::TooShort {
                    field: field_name.to_string(),
                    min_length: min_len,
                });
            }
        }

        if let Some(max_len) = constraints.max_length {
            if char_count > max_len {
                return Err(ValidationError::TooLong {
                    field: field_name.to_string(),
                    max_length: max_len,
                });
            }
        }
    }
    
    // Check pattern match
    if let Some(pattern) = &constraints.pattern {
        if !pattern.is_match(value) {
            return Err(ValidationError::InvalidCharacters {
                field: field_name.to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validates an optional string field.
pub fn validate_optional_string_field(
    value: &Option<String>,
    field_name: &str,
    constraints: &StringConstraints,
) -> ValidationResult<()> {
    if let Some(val) = value {
        validate_string_field(val, field_name, constraints)?;
    }
    Ok(())
}

/// Validates an authentication method ID.
pub fn validate_auth_method_id(id: &str) -> ValidationResult<()> {
    let constraints = StringConstraints {
        min_length: Some(1),
        max_length: Some(128),
        allow_empty: false,
        pattern: None,
    };

    validate_string_field(id, "method_id", &constraints)?;

    if !IDENTIFIER_PATTERN.is_match(id) {
        return Err(ValidationError::InvalidCharacters {
            field: "method_id".to_string(),
        });
    }

    Ok(())
}

/// Validates an environment variable name.
pub fn validate_env_var_name(name: &str) -> ValidationResult<()> {
    if name.is_empty() {
        return Err(ValidationError::InvalidEnvVarName {
            name: name.to_string(),
            reason: "Environment variable name cannot be empty".to_string(),
        });
    }
    
    // Environment variable names should follow POSIX standards
    if !ENV_VAR_NAME_PATTERN.is_match(name) {
        return Err(ValidationError::InvalidEnvVarName {
            name: name.to_string(),
            reason: "Environment variable name must start with a letter and contain only uppercase letters, numbers, and underscores".to_string(),
        });
    }
    
    // Check for reserved names
    let reserved_names = [
        "PATH", "HOME", "USER", "USERNAME", "SHELL", "TERM", "PWD", "TMPDIR",
        "TMP", "TEMP", "LANG", "LC_ALL", "LC_CTYPE", "TZ", "DISPLAY",
    ];
    
    if reserved_names.contains(&name) {
        return Err(ValidationError::InvalidEnvVarName {
            name: name.to_string(),
            reason: "This environment variable name is reserved by the system".to_string(),
        });
    }
    
    Ok(())
}

/// Validates a URL string.
pub fn validate_url(url: &str) -> ValidationResult<()> {
    if url.is_empty() {
        return Err(ValidationError::InvalidUrl {
            url: url.to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }
    
    // Basic URL validation - check for protocol
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::InvalidUrl {
            url: url.to_string(),
            reason: "URL must start with http:// or https://".to_string(),
        });
    }
    
    // Check for obvious malformed URLs
    if url.contains(' ') || url.contains('\n') || url.contains('\t') {
        return Err(ValidationError::InvalidUrl {
            url: url.to_string(),
            reason: "URL cannot contain whitespace characters".to_string(),
        });
    }
    
    Ok(())
}

/// Validates that all IDs in a collection are unique.
pub fn validate_unique_ids<T>(items: &[T], get_id: impl Fn(&T) -> &str) -> ValidationResult<()> {
    let mut seen_ids = HashSet::new();
    
    for item in items {
        let id = get_id(item);
        if seen_ids.contains(id) {
            return Err(ValidationError::DuplicateId {
                id: id.to_string(),
            });
        }
        seen_ids.insert(id.to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string_field_empty_not_allowed() {
        let constraints = StringConstraints::default();
        let result = validate_string_field("", "test_field", &constraints);
        assert!(matches!(result, Err(ValidationError::RequiredFieldMissing { .. })));
    }
    
    #[test]
    fn test_validate_string_field_empty_allowed() {
        let constraints = StringConstraints {
            allow_empty: true,
            ..Default::default()
        };
        let result = validate_string_field("", "test_field", &constraints);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_string_field_too_short() {
        let constraints = StringConstraints {
            min_length: Some(5),
            ..Default::default()
        };
        let result = validate_string_field("abc", "test_field", &constraints);
        assert!(matches!(result, Err(ValidationError::TooShort { .. })));
    }
    
    #[test]
    fn test_validate_string_field_too_long() {
        let constraints = StringConstraints {
            max_length: Some(5),
            allow_empty: true,
            ..Default::default()
        };
        let result = validate_string_field("abcdefgh", "test_field", &constraints);
        assert!(matches!(result, Err(ValidationError::TooLong { .. })));
    }

    #[test]
    fn test_validate_string_field_length_is_measured_in_characters() {
        // Each Japanese character is 3 bytes in UTF-8, so `len()` (bytes) would
        // report 9 and incorrectly reject a 5-character max. `chars().count()`
        // reports 3 and accepts the value.
        let constraints = StringConstraints {
            min_length: Some(1),
            max_length: Some(5),
            ..Default::default()
        };
        assert!(validate_string_field("日本語", "test_field", &constraints).is_ok());

        // And the limit is still enforced by character count.
        let too_long = "あいうえおか"; // 6 characters, 18 bytes
        assert!(matches!(
            validate_string_field(too_long, "test_field", &constraints),
            Err(ValidationError::TooLong { .. })
        ));
    }
    
    #[test]
    fn test_validate_auth_method_id_valid() {
        assert!(validate_auth_method_id("my_auth_method").is_ok());
        assert!(validate_auth_method_id("auth-method-1").is_ok());
        assert!(validate_auth_method_id("com.example.auth").is_ok());
    }
    
    #[test]
    fn test_validate_auth_method_id_invalid() {
        assert!(validate_auth_method_id("").is_err());
        assert!(validate_auth_method_id("auth method").is_err()); // spaces not allowed
        assert!(validate_auth_method_id("auth@method").is_err()); // @ not allowed
    }
    
    #[test]
    fn test_validate_env_var_name_valid() {
        assert!(validate_env_var_name("API_KEY").is_ok());
        assert!(validate_env_var_name("MY_SECRET_TOKEN").is_ok());
        assert!(validate_env_var_name("CONFIG_VAR_123").is_ok());
    }
    
    #[test]
    fn test_validate_env_var_name_invalid() {
        assert!(validate_env_var_name("").is_err());
        assert!(validate_env_var_name("api_key").is_err()); // must be uppercase
        assert!(validate_env_var_name("123_KEY").is_err()); // must start with letter
        assert!(validate_env_var_name("PATH").is_err()); // reserved name
    }
    
    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080/path").is_ok());
    }
    
    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err()); // not http/https
        assert!(validate_url("https://example .com").is_err()); // contains space
    }
    
    #[test]
    fn test_validate_unique_ids() {
        let items = vec!["id1", "id2", "id3"];
        assert!(validate_unique_ids(&items, |item| item).is_ok());
        
        let items_with_duplicate = vec!["id1", "id2", "id1"];
        assert!(validate_unique_ids(&items_with_duplicate, |item| item).is_err());
    }
}