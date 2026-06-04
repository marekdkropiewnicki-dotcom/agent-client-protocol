//! Error handling for the Agent Client Protocol.
//!
//! This module provides error types and codes following the JSON-RPC 2.0 specification,
//! with additional protocol-specific error codes for authentication and other ACP-specific scenarios.
//!
//! All methods in the protocol follow standard JSON-RPC 2.0 error handling:
//! - Successful responses include a `result` field
//! - Errors include an `error` object with `code` and `message`
//! - Notifications never receive responses (success or error)
//!
//! See: [Error Handling](https://agentclientprotocol.com/protocol/overview#error-handling)

use std::{fmt::Display, str};

use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::IntoOption;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// JSON-RPC error object.
///
/// Represents an error that occurred during method execution, following the
/// JSON-RPC 2.0 error object specification with optional additional data.
///
/// See protocol docs: [JSON-RPC Error Object](https://www.jsonrpc.org/specification#error_object)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub struct Error {
    /// A number indicating the error type that occurred.
    /// This must be an integer as defined in the JSON-RPC specification.
    pub code: ErrorCode,
    /// A string providing a short description of the error.
    /// The message should be limited to a concise single sentence.
    pub message: String,
    /// Optional primitive or structured value that contains additional information about the error.
    /// This may include debugging information or context-specific details.
    pub data: Option<serde_json::Value>,
}

impl Error {
    /// Creates a new error with the given code and message.
    ///
    /// The code parameter can be an `ErrorCode` constant or a tuple of (code, message).
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Error {
            code: code.into(),
            message: message.into(),
            data: None,
        }
    }

    /// Adds additional data to the error.
    ///
    /// This method is chainable and allows attaching context-specific information
    /// to help with debugging or provide more details about the error.
    #[must_use]
    pub fn data(mut self, data: impl IntoOption<serde_json::Value>) -> Self {
        self.data = data.into_option();
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    #[must_use]
    pub fn parse_error() -> Self {
        ErrorCode::ParseError.into()
    }

    /// The JSON sent is not a valid Request object.
    #[must_use]
    pub fn invalid_request() -> Self {
        ErrorCode::InvalidRequest.into()
    }

    /// The method does not exist / is not available.
    #[must_use]
    pub fn method_not_found() -> Self {
        ErrorCode::MethodNotFound.into()
    }

    /// Invalid method parameter(s).
    #[must_use]
    pub fn invalid_params() -> Self {
        ErrorCode::InvalidParams.into()
    }

    /// Internal JSON-RPC error.
    #[must_use]
    pub fn internal_error() -> Self {
        ErrorCode::InternalError.into()
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Request was cancelled.
    ///
    /// Execution of the method was aborted either due to a cancellation request from the caller
    /// or because of resource constraints or shutdown.
    #[cfg(feature = "unstable_cancel_request")]
    #[must_use]
    pub fn request_cancelled() -> Self {
        ErrorCode::RequestCancelled.into()
    }

    /// Authentication required.
    #[must_use]
    pub fn auth_required() -> Self {
        ErrorCode::AuthRequired.into()
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// The agent requires user input via a URL-based elicitation before it can proceed.
    #[cfg(feature = "unstable_elicitation")]
    #[must_use]
    pub fn url_elicitation_required() -> Self {
        ErrorCode::UrlElicitationRequired.into()
    }

    /// A given resource, such as a file, was not found.
    #[must_use]
    pub fn resource_not_found(uri: Option<String>) -> Self {
        let err: Self = ErrorCode::ResourceNotFound.into();
        if let Some(uri) = uri {
            err.data(serde_json::json!({ "uri": uri }))
        } else {
            err
        }
    }

    /// Converts a standard error into an internal JSON-RPC error.
    ///
    /// The error's string representation is included as additional data.
    #[must_use]
    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().data(err.to_string())
    }
}

/// Predefined error codes for common JSON-RPC and ACP-specific errors.
///
/// These codes follow the JSON-RPC 2.0 specification for standard errors
/// and use the reserved range (-32000 to -32099) for protocol-specific errors.
#[derive(Clone, Copy, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::Display)]
#[cfg_attr(test, derive(strum::EnumIter))]
#[serde(from = "i32", into = "i32")]
#[schemars(!from, !into)]
#[non_exhaustive]
pub enum ErrorCode {
    // Standard errors
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Parse error")]
    ParseError, // -32700
    /// The JSON sent is not a valid Request object.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Invalid request")]
    InvalidRequest, // -32600
    /// The method does not exist or is not available.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Method not found")]
    MethodNotFound, // -32601
    /// Invalid method parameter(s).
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Invalid params")]
    InvalidParams, // -32602
    /// Internal JSON-RPC error.
    /// Reserved for implementation-defined server errors.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Internal error")]
    InternalError, // -32603
    #[cfg(feature = "unstable_cancel_request")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Execution of the method was aborted either due to a cancellation request from the caller or
    /// because of resource constraints or shutdown.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Request cancelled")]
    RequestCancelled, // -32800

    // Custom errors
    /// Authentication is required before this operation can be performed.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Authentication required")]
    AuthRequired, // -32000
    /// A given resource, such as a file, was not found.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "Resource not found")]
    ResourceNotFound, // -32002
    #[cfg(feature = "unstable_elicitation")]
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// The agent requires user input via a URL-based elicitation before it can proceed.
    #[schemars(transform = error_code_transform)]
    #[strum(to_string = "URL elicitation required")]
    UrlElicitationRequired, // -32042

    /// Other undefined error code.
    #[schemars(untagged)]
    #[strum(to_string = "Unknown error")]
    Other(i32),
}

impl From<i32> for ErrorCode {
    fn from(value: i32) -> Self {
        match value {
            -32700 => ErrorCode::ParseError,
            -32600 => ErrorCode::InvalidRequest,
            -32601 => ErrorCode::MethodNotFound,
            -32602 => ErrorCode::InvalidParams,
            -32603 => ErrorCode::InternalError,
            #[cfg(feature = "unstable_cancel_request")]
            -32800 => ErrorCode::RequestCancelled,
            -32000 => ErrorCode::AuthRequired,
            -32002 => ErrorCode::ResourceNotFound,
            #[cfg(feature = "unstable_elicitation")]
            -32042 => ErrorCode::UrlElicitationRequired,
            _ => ErrorCode::Other(value),
        }
    }
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        match value {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
            #[cfg(feature = "unstable_cancel_request")]
            ErrorCode::RequestCancelled => -32800,
            ErrorCode::AuthRequired => -32000,
            ErrorCode::ResourceNotFound => -32002,
            #[cfg(feature = "unstable_elicitation")]
            ErrorCode::UrlElicitationRequired => -32042,
            ErrorCode::Other(value) => value,
        }
    }
}

impl std::fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {self}", i32::from(*self))
    }
}

fn error_code_transform(schema: &mut Schema) {
    let name = schema
        .get("const")
        .expect("Unexpected schema for ErrorCode")
        .as_str()
        .expect("unexpected type for schema");
    let code = match name {
        "ParseError" => ErrorCode::ParseError,
        "InvalidRequest" => ErrorCode::InvalidRequest,
        "MethodNotFound" => ErrorCode::MethodNotFound,
        "InvalidParams" => ErrorCode::InvalidParams,
        "InternalError" => ErrorCode::InternalError,
        #[cfg(feature = "unstable_cancel_request")]
        "RequestCancelled" => ErrorCode::RequestCancelled,
        "AuthRequired" => ErrorCode::AuthRequired,
        "ResourceNotFound" => ErrorCode::ResourceNotFound,
        #[cfg(feature = "unstable_elicitation")]
        "UrlElicitationRequired" => ErrorCode::UrlElicitationRequired,
        _ => panic!("Unexpected error code name {name}"),
    };
    let mut description = schema
        .get("description")
        .expect("Missing description")
        .as_str()
        .expect("Unexpected type for description")
        .to_owned();
    schema.insert("title".into(), code.to_string().into());
    description.insert_str(0, &format!("**{code}**: "));
    schema.insert("description".into(), description.into());
    schema.insert("const".into(), i32::from(code).into());
    schema.insert("type".into(), "integer".into());
    schema.insert("format".into(), "int32".into());
}

impl From<ErrorCode> for Error {
    fn from(error_code: ErrorCode) -> Self {
        Error::new(error_code.into(), error_code.to_string())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", i32::from(self.code))?;
        } else {
            write!(f, "{}", self.message)?;
        }

        if let Some(data) = &self.data {
            let pretty = serde_json::to_string_pretty(data).unwrap_or_else(|_| data.to_string());
            write!(f, ": {pretty}")?;
        }

        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        match error.downcast::<Self>() {
            Ok(error) => error,
            Err(error) => Error::into_internal_error(&*error),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::invalid_params().data(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn serialize_error_code() {
        assert_eq!(
            serde_json::from_value::<ErrorCode>(serde_json::json!(-32700)).unwrap(),
            ErrorCode::ParseError
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::ParseError).unwrap(),
            serde_json::json!(-32700)
        );

        assert_eq!(
            serde_json::from_value::<ErrorCode>(serde_json::json!(1)).unwrap(),
            ErrorCode::Other(1)
        );
        assert_eq!(
            serde_json::to_value(ErrorCode::Other(1)).unwrap(),
            serde_json::json!(1)
        );
    }

    #[test]
    fn serialize_error_code_equality() {
        // Make sure this doesn't panic
        let _schema = schemars::schema_for!(ErrorCode);
        for error in ErrorCode::iter() {
            assert_eq!(
                error,
                serde_json::from_value(serde_json::to_value(error).unwrap()).unwrap()
            );
        }
    }

    // ---- Constructor helpers ----

    #[test]
    fn standard_error_constructors_carry_canonical_codes_and_messages() {
        // Each helper must produce the canonical (code, message) pair so callers and
        // peers see consistent error identities on the wire.
        for (err, expected_code) in [
            (Error::parse_error(), -32700),
            (Error::invalid_request(), -32600),
            (Error::method_not_found(), -32601),
            (Error::invalid_params(), -32602),
            (Error::internal_error(), -32603),
            (Error::auth_required(), -32000),
        ] {
            assert_eq!(i32::from(err.code), expected_code);
            assert!(
                !err.message.is_empty(),
                "error code {expected_code} must carry a message"
            );
            assert!(err.data.is_none());
        }
    }

    #[test]
    fn resource_not_found_with_uri_attaches_uri_data() {
        let err = Error::resource_not_found(Some("file:///missing.txt".into()));
        assert_eq!(i32::from(err.code), -32002);
        let data = err.data.expect("uri must be attached as data");
        assert_eq!(data["uri"], "file:///missing.txt");
    }

    #[test]
    fn resource_not_found_without_uri_omits_data() {
        let err = Error::resource_not_found(None);
        assert_eq!(i32::from(err.code), -32002);
        assert!(err.data.is_none());
    }

    #[test]
    fn data_builder_attaches_extra_information() {
        let err = Error::internal_error().data(serde_json::json!({"hint": "retry"}));
        assert_eq!(err.data.unwrap()["hint"], "retry");
    }

    #[test]
    fn into_internal_error_attaches_string_representation() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "boom");
        let err = Error::into_internal_error(&inner);
        assert_eq!(i32::from(err.code), -32603);
        assert_eq!(err.data.unwrap(), serde_json::Value::String("boom".into()));
    }

    // ---- Display branches ----

    #[test]
    fn display_uses_message_when_present() {
        let err = Error::invalid_params();
        let s = err.to_string();
        assert!(s.contains("Invalid params"), "got {s:?}");
        // No data attached -> no trailing colon.
        assert!(!s.contains(':'), "got {s:?}");
    }

    #[test]
    fn display_falls_back_to_numeric_code_for_empty_message() {
        // The `Display` impl explicitly handles the empty-message branch by writing the
        // numeric code instead. Confirm this so we don't render an empty error to users.
        let err = Error {
            code: ErrorCode::Other(-1),
            message: String::new(),
            data: None,
        };
        assert_eq!(err.to_string(), "-1");
    }

    #[test]
    fn display_appends_pretty_data_when_present() {
        let err = Error::internal_error().data(serde_json::json!({"why": "boom"}));
        let s = err.to_string();
        assert!(s.starts_with("Internal error: "), "got {s:?}");
        assert!(s.contains("\"why\""), "got {s:?}");
        assert!(s.contains("\"boom\""), "got {s:?}");
    }

    #[test]
    fn display_appends_data_when_message_is_empty() {
        let err = Error {
            code: ErrorCode::Other(-99),
            message: String::new(),
            data: Some(serde_json::json!("payload")),
        };
        // Empty-message + data branch: numeric code first, then the rendered data.
        assert_eq!(err.to_string(), "-99: \"payload\"");
    }

    // ---- Error code <-> i32 round trip ----

    #[test]
    fn error_code_round_trips_through_i32() {
        for code in ErrorCode::iter() {
            let n = i32::from(code);
            let back = ErrorCode::from(n);
            assert_eq!(back, code, "code {n} did not round-trip");
        }

        // Codes outside the documented set must collapse to `Other` rather than
        // failing — protocol peers occasionally send unknown codes and we must
        // tolerate them.
        assert_eq!(ErrorCode::from(12345), ErrorCode::Other(12345));
        assert_eq!(i32::from(ErrorCode::Other(12345)), 12345);
    }

    // ---- From<serde_json::Error> ----

    #[test]
    fn from_serde_json_error_is_invalid_params_with_data() {
        let json_err: serde_json::Error = serde_json::from_str::<i32>("not-a-number").unwrap_err();
        let err: Error = json_err.into();
        assert_eq!(err.code, ErrorCode::InvalidParams);
        assert!(
            err.data.is_some(),
            "serde error message must propagate as data"
        );
    }

    // ---- From<anyhow::Error> ----

    #[test]
    fn from_anyhow_with_acp_error_preserves_original() {
        let original = Error::auth_required().data(serde_json::json!("login"));
        let any: anyhow::Error = original.clone().into();
        let recovered: Error = any.into();
        assert_eq!(recovered, original);
    }

    #[test]
    fn from_anyhow_with_other_error_collapses_to_internal() {
        let any = anyhow::anyhow!("something went wrong");
        let err: Error = any.into();
        assert_eq!(err.code, ErrorCode::InternalError);
        let data = err.data.expect("inner error must propagate as data");
        assert!(
            data.as_str().unwrap().contains("something went wrong"),
            "got {data:?}"
        );
    }
}
