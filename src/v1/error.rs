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

    /// Every convenience constructor must round-trip its code through the i32
    /// representation and emit the strum-defined default message. If any of
    /// these regress, the wire `code`/`message` clients see will be wrong.
    #[test]
    fn convenience_constructors_carry_code_and_default_message() {
        let cases: Vec<(Error, ErrorCode, i32, &str)> = vec![
            (
                Error::parse_error(),
                ErrorCode::ParseError,
                -32700,
                "Parse error",
            ),
            (
                Error::invalid_request(),
                ErrorCode::InvalidRequest,
                -32600,
                "Invalid request",
            ),
            (
                Error::method_not_found(),
                ErrorCode::MethodNotFound,
                -32601,
                "Method not found",
            ),
            (
                Error::invalid_params(),
                ErrorCode::InvalidParams,
                -32602,
                "Invalid params",
            ),
            (
                Error::internal_error(),
                ErrorCode::InternalError,
                -32603,
                "Internal error",
            ),
            (
                Error::auth_required(),
                ErrorCode::AuthRequired,
                -32000,
                "Authentication required",
            ),
        ];

        for (err, code, code_i32, message) in cases {
            assert_eq!(err.code, code, "code mismatch for {message}");
            assert_eq!(i32::from(err.code), code_i32, "i32 mismatch for {message}");
            assert_eq!(err.message, message);
            assert!(
                err.data.is_none(),
                "default constructor should not carry data"
            );
        }
    }

    #[cfg(feature = "unstable_cancel_request")]
    #[test]
    fn request_cancelled_constructor_carries_code_and_message() {
        let err = Error::request_cancelled();
        assert_eq!(err.code, ErrorCode::RequestCancelled);
        assert_eq!(i32::from(err.code), -32800);
        assert_eq!(err.message, "Request cancelled");
    }

    #[cfg(feature = "unstable_elicitation")]
    #[test]
    fn url_elicitation_required_constructor_carries_code_and_message() {
        let err = Error::url_elicitation_required();
        assert_eq!(err.code, ErrorCode::UrlElicitationRequired);
        assert_eq!(i32::from(err.code), -32042);
        assert_eq!(err.message, "URL elicitation required");
    }

    /// `Error::new` accepts a raw i32 code and arbitrary message — unknown
    /// codes must round-trip through `ErrorCode::Other` (no silent coercion).
    #[test]
    fn new_constructor_accepts_arbitrary_codes_and_messages() {
        let err = Error::new(-1234, "boom");
        assert_eq!(err.code, ErrorCode::Other(-1234));
        assert_eq!(i32::from(err.code), -1234);
        assert_eq!(err.message, "boom");
        assert!(err.data.is_none());

        // A well-known code passed through `new` still resolves to the typed variant.
        let err = Error::new(-32601, "custom-message");
        assert_eq!(err.code, ErrorCode::MethodNotFound);
        assert_eq!(err.message, "custom-message");
    }

    /// `data(...)` accepts both raw JSON values and strings via `IntoOption`,
    /// and must overwrite any previously-attached data when chained.
    #[test]
    fn data_builder_attaches_and_overwrites() {
        let err = Error::internal_error().data(serde_json::json!({"reason": "x"}));
        assert_eq!(err.data, Some(serde_json::json!({"reason": "x"})));

        // String slices go through `IntoOption<serde_json::Value> for &str`.
        let err = Error::invalid_params().data("bad payload");
        assert_eq!(
            err.data,
            Some(serde_json::Value::String("bad payload".into()))
        );

        // Chained calls replace earlier data.
        let err = Error::invalid_params()
            .data("first")
            .data(serde_json::json!(42));
        assert_eq!(err.data, Some(serde_json::json!(42)));
    }

    /// `From<ErrorCode>` matches the public `Error::new(code, code.to_string())`
    /// shape; this keeps `?` on an `ErrorCode` aligned with the named helpers.
    #[test]
    fn error_code_into_error_matches_default_constructor() {
        let from_code: Error = ErrorCode::InvalidRequest.into();
        let from_helper = Error::invalid_request();
        assert_eq!(from_code.code, from_helper.code);
        assert_eq!(from_code.message, from_helper.message);
        assert!(from_code.data.is_none());
    }

    /// `Display` has three branches; lock all three down so user-visible
    /// formatting cannot silently regress.
    #[test]
    fn display_uses_message_then_data_when_present() {
        let err = Error::method_not_found();
        assert_eq!(err.to_string(), "Method not found");

        // With a non-string data payload, Display appends pretty JSON.
        let err = Error::invalid_params().data(serde_json::json!({"field": "name"}));
        let rendered = err.to_string();
        assert!(
            rendered.starts_with("Invalid params: "),
            "unexpected prefix in {rendered:?}"
        );
        assert!(
            rendered.contains("\"field\""),
            "missing JSON body in {rendered:?}"
        );
        assert!(
            rendered.contains("\"name\""),
            "missing JSON body in {rendered:?}"
        );
    }

    /// Empty-message branch of `Display`: falls back to the numeric code.
    #[test]
    fn display_falls_back_to_code_for_empty_messages() {
        let err = Error::new(-32603, "");
        assert_eq!(err.to_string(), "-32603");

        let err = Error::new(-32603, "").data("aux");
        assert_eq!(err.to_string(), "-32603: \"aux\"");
    }

    /// `Debug` on `ErrorCode` follows the "code: name" shape and is used in
    /// log lines all over the SDK; it must not regress to the strum default.
    #[test]
    fn error_code_debug_uses_code_then_name() {
        assert_eq!(
            format!("{:?}", ErrorCode::ParseError),
            "-32700: Parse error"
        );
        assert_eq!(format!("{:?}", ErrorCode::Other(-1)), "-1: Unknown error");
    }

    /// `From<serde_json::Error>` is the conventional bridge between serde
    /// failures and JSON-RPC; it must produce `InvalidParams` and preserve
    /// the underlying error string in `data`.
    #[test]
    fn from_serde_json_error_yields_invalid_params() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let serde_msg = serde_err.to_string();
        let acp_err: Error = serde_err.into();

        assert_eq!(acp_err.code, ErrorCode::InvalidParams);
        assert_eq!(acp_err.message, "Invalid params");
        assert_eq!(acp_err.data, Some(serde_json::Value::String(serde_msg)));
    }

    /// The `From<anyhow::Error>` impl has two branches: if the source is
    /// already an `Error`, it is unwrapped via downcast; otherwise it is
    /// converted to `InternalError`. Both branches must work, otherwise
    /// errors propagated through `anyhow::Result` would be double-wrapped
    /// or lose their original code.
    #[test]
    fn from_anyhow_downcasts_existing_acp_error() {
        let original = Error::auth_required().data("token expired");
        let anyhow_err: anyhow::Error = original.clone().into();
        let recovered: Error = anyhow_err.into();
        assert_eq!(recovered, original);
    }

    #[test]
    fn from_anyhow_wraps_foreign_error_as_internal() {
        #[derive(Debug)]
        struct Custom;
        impl std::fmt::Display for Custom {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("custom-error-display")
            }
        }
        impl std::error::Error for Custom {}

        let anyhow_err = anyhow::Error::new(Custom);
        let acp_err: Error = anyhow_err.into();

        assert_eq!(acp_err.code, ErrorCode::InternalError);
        assert_eq!(
            acp_err.data,
            Some(serde_json::Value::String("custom-error-display".to_owned()))
        );
    }

    /// `into_internal_error` is the public alternative to the anyhow bridge —
    /// it must preserve the source's Display in `data`.
    #[test]
    fn into_internal_error_captures_display_in_data() {
        #[derive(Debug)]
        struct E;
        impl std::fmt::Display for E {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("oops")
            }
        }
        impl std::error::Error for E {}

        let err = Error::into_internal_error(E);
        assert_eq!(err.code, ErrorCode::InternalError);
        assert_eq!(err.data, Some(serde_json::Value::String("oops".to_owned())));
    }

    /// `resource_not_found` has both a no-URI and a with-URI branch; the
    /// with-URI branch must place the URI under the `uri` key (clients rely
    /// on this exact shape to surface "go to file" affordances).
    #[test]
    fn resource_not_found_attaches_uri_when_provided() {
        let bare = Error::resource_not_found(None);
        assert_eq!(bare.code, ErrorCode::ResourceNotFound);
        assert_eq!(bare.message, "Resource not found");
        assert!(bare.data.is_none());

        let with_uri = Error::resource_not_found(Some("file:///x".to_owned()));
        assert_eq!(with_uri.code, ErrorCode::ResourceNotFound);
        assert_eq!(with_uri.data, Some(serde_json::json!({"uri": "file:///x"})));
    }

    /// `ErrorCode` is `#[non_exhaustive]` plus an explicit `Other(i32)` escape
    /// hatch. Unknown codes must always round-trip through `Other` so newer
    /// agents talking to older SDKs (or vice versa) do not lose the wire
    /// value.
    #[test]
    fn unknown_error_codes_round_trip_through_other() {
        for value in [0, -1, -32099, -33000, i32::MIN, i32::MAX] {
            let code: ErrorCode = value.into();
            assert_eq!(code, ErrorCode::Other(value));
            assert_eq!(i32::from(code), value);

            let json = serde_json::to_value(code).unwrap();
            assert_eq!(json, serde_json::json!(value));
            let parsed: ErrorCode = serde_json::from_value(json).unwrap();
            assert_eq!(parsed, ErrorCode::Other(value));
        }
    }

    /// The full `Error` payload must round-trip through JSON in the JSON-RPC
    /// shape (`code`, `message`, optional `data`).
    #[test]
    fn error_json_round_trip_preserves_all_fields() {
        let err = Error::invalid_params().data(serde_json::json!({"field": "uri"}));

        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], serde_json::json!(-32602));
        assert_eq!(json["message"], "Invalid params");
        assert_eq!(json["data"], serde_json::json!({"field": "uri"}));

        let round_tripped: Error = serde_json::from_value(json).unwrap();
        assert_eq!(round_tripped, err);

        // `data: None` must be omitted via `skip_serializing_none`, not
        // serialized as `null` — otherwise older SDKs that reject `null` on
        // `data` would break.
        let bare = Error::method_not_found();
        let json = serde_json::to_value(&bare).unwrap();
        let obj = json.as_object().unwrap();
        assert!(
            !obj.contains_key("data"),
            "data should be omitted, got {obj:?}"
        );
    }
}
