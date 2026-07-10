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

    /// Pin the wire-level integer codes so an accidental rename or reorder of
    /// `ErrorCode` variants cannot silently change values shipped to clients.
    #[test]
    fn error_code_to_i32_pins_wire_values() {
        assert_eq!(i32::from(ErrorCode::ParseError), -32700);
        assert_eq!(i32::from(ErrorCode::InvalidRequest), -32600);
        assert_eq!(i32::from(ErrorCode::MethodNotFound), -32601);
        assert_eq!(i32::from(ErrorCode::InvalidParams), -32602);
        assert_eq!(i32::from(ErrorCode::InternalError), -32603);
        assert_eq!(i32::from(ErrorCode::AuthRequired), -32000);
        assert_eq!(i32::from(ErrorCode::ResourceNotFound), -32002);
        assert_eq!(i32::from(ErrorCode::Other(123)), 123);
        assert_eq!(i32::from(ErrorCode::Other(-99999)), -99999);
    }

    /// Round-trip every reserved code through `i32 -> ErrorCode -> i32` and
    /// confirm an unknown code falls through to `Other(_)` rather than being
    /// remapped to a known variant.
    #[test]
    fn error_code_from_i32_roundtrips_known_codes_and_falls_back_to_other() {
        let known = [
            (-32700, ErrorCode::ParseError),
            (-32600, ErrorCode::InvalidRequest),
            (-32601, ErrorCode::MethodNotFound),
            (-32602, ErrorCode::InvalidParams),
            (-32603, ErrorCode::InternalError),
            (-32000, ErrorCode::AuthRequired),
            (-32002, ErrorCode::ResourceNotFound),
        ];
        for (code, variant) in known {
            assert_eq!(ErrorCode::from(code), variant, "from({code})");
            assert_eq!(i32::from(variant), code, "to({variant})");
        }

        // Unknown values must fall through to `Other` (not silently remapped).
        for code in [-32699, -32001, -32003, -1, 0, 1, 42, i32::MAX, i32::MIN] {
            assert_eq!(ErrorCode::from(code), ErrorCode::Other(code));
        }
    }

    #[test]
    fn error_code_debug_includes_numeric_value() {
        assert_eq!(
            format!("{:?}", ErrorCode::ParseError),
            "-32700: Parse error"
        );
        assert_eq!(
            format!("{:?}", ErrorCode::AuthRequired),
            "-32000: Authentication required"
        );
        assert_eq!(format!("{:?}", ErrorCode::Other(7)), "7: Unknown error");
    }

    /// `Error::new` accepts arbitrary i32 codes (so callers can use unknown
    /// codes received from the wire) and starts with `data: None`.
    #[test]
    fn error_new_preserves_arbitrary_code_and_message() {
        let err = Error::new(-12345, "boom");
        assert_eq!(err.code, ErrorCode::Other(-12345));
        assert_eq!(err.message, "boom");
        assert!(err.data.is_none());

        // Known codes get normalized into named variants.
        let err = Error::new(-32601, "missing");
        assert_eq!(err.code, ErrorCode::MethodNotFound);
    }

    /// Each builder helper must map to its documented `ErrorCode` and carry
    /// the corresponding strum display string as its message.
    #[test]
    fn error_builders_use_canonical_code_and_message() {
        let cases: &[(Error, ErrorCode, &str)] = &[
            (Error::parse_error(), ErrorCode::ParseError, "Parse error"),
            (
                Error::invalid_request(),
                ErrorCode::InvalidRequest,
                "Invalid request",
            ),
            (
                Error::method_not_found(),
                ErrorCode::MethodNotFound,
                "Method not found",
            ),
            (
                Error::invalid_params(),
                ErrorCode::InvalidParams,
                "Invalid params",
            ),
            (
                Error::internal_error(),
                ErrorCode::InternalError,
                "Internal error",
            ),
            (
                Error::auth_required(),
                ErrorCode::AuthRequired,
                "Authentication required",
            ),
        ];
        for (err, code, msg) in cases {
            assert_eq!(&err.code, code);
            assert_eq!(&err.message, msg);
            assert!(err.data.is_none());
        }
    }

    /// `resource_not_found` must encode the URI as `{ "uri": uri }` so
    /// downstream consumers can rely on this shape.
    #[test]
    fn resource_not_found_data_shape() {
        let none = Error::resource_not_found(None);
        assert_eq!(none.code, ErrorCode::ResourceNotFound);
        assert!(none.data.is_none());

        let with_uri = Error::resource_not_found(Some("file:///tmp/x".into()));
        assert_eq!(with_uri.code, ErrorCode::ResourceNotFound);
        assert_eq!(
            with_uri.data,
            Some(serde_json::json!({ "uri": "file:///tmp/x" }))
        );
    }

    #[test]
    fn error_data_is_chainable_and_overrides() {
        let err = Error::internal_error()
            .data(serde_json::json!({"first": true}))
            .data(serde_json::json!({"second": true}));
        assert_eq!(err.data, Some(serde_json::json!({"second": true})));

        // `IntoOption` lets callers pass the value bare or wrapped in `Some`.
        let bare = Error::internal_error().data("hello");
        let wrapped = Error::internal_error().data(Some(serde_json::Value::from("hello")));
        assert_eq!(bare.data, wrapped.data);
    }

    #[test]
    fn into_internal_error_attaches_string_representation() {
        #[derive(Debug)]
        struct MyErr;
        impl std::fmt::Display for MyErr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "boom")
            }
        }
        impl std::error::Error for MyErr {}

        let err = Error::into_internal_error(MyErr);
        assert_eq!(err.code, ErrorCode::InternalError);
        assert_eq!(err.data, Some(serde_json::Value::from("boom")));
    }

    /// Display falls back to the numeric code when the message is empty,
    /// otherwise prints the message; pretty-printed data is appended after a
    /// `: ` separator.
    #[test]
    fn error_display_handles_empty_message_and_data() {
        let with_msg = Error::method_not_found();
        assert_eq!(with_msg.to_string(), "Method not found");

        let blank = Error::new(-32603, "");
        assert_eq!(blank.to_string(), "-32603");

        let with_data = Error::internal_error().data(serde_json::json!({"k": 1}));
        let rendered = with_data.to_string();
        assert!(rendered.starts_with("Internal error: "), "{rendered}");
        assert!(rendered.contains("\"k\": 1"), "{rendered}");
    }

    /// Wire format must remain exactly `{ code, message, data? }` (data is
    /// skipped when None).
    #[test]
    fn error_serializes_to_expected_json_rpc_shape() {
        let err = Error::auth_required();
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            serde_json::json!({
                "code": -32000,
                "message": "Authentication required",
            })
        );

        let with_data = Error::resource_not_found(Some("u".into()));
        assert_eq!(
            serde_json::to_value(&with_data).unwrap(),
            serde_json::json!({
                "code": -32002,
                "message": "Resource not found",
                "data": { "uri": "u" },
            })
        );

        // `data: null` deserializes to `Some(Value::Null)` (serde default for
        // `Option<Value>`); wire absence must not be confused with explicit
        // null in tests that round-trip the type.
        let parsed: Error = serde_json::from_value(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        }))
        .unwrap();
        assert_eq!(parsed.code, ErrorCode::MethodNotFound);
        assert!(parsed.data.is_none());
    }

    #[test]
    fn from_serde_json_error_produces_invalid_params() {
        let json_err = serde_json::from_str::<i32>("not-a-number").unwrap_err();
        let msg = json_err.to_string();
        let err: Error = json_err.into();
        assert_eq!(err.code, ErrorCode::InvalidParams);
        assert_eq!(err.data, Some(serde_json::Value::from(msg)));
    }

    #[test]
    fn from_anyhow_downcasts_existing_error() {
        let original = Error::auth_required().data(serde_json::json!({"why": "no token"}));
        let anyhow_err: anyhow::Error = anyhow::Error::new(original.clone());
        let recovered: Error = anyhow_err.into();
        assert_eq!(recovered, original);
    }

    #[test]
    fn from_anyhow_wraps_unknown_error_as_internal() {
        let anyhow_err = anyhow::anyhow!("kaboom");
        let err: Error = anyhow_err.into();
        assert_eq!(err.code, ErrorCode::InternalError);
        assert_eq!(err.data, Some(serde_json::Value::from("kaboom")));
    }
}
