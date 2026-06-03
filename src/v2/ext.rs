//! Extension types and constants for protocol extensibility.
use derive_more::From;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Arc;

/// Value attached to a given ACP type on the `_meta` field.
///
/// The _meta property is reserved by ACP to allow clients and agents to attach
/// additional metadata to their interactions. Implementations MUST NOT make assumptions about
/// values at these keys.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
pub type Meta = serde_json::Map<String, serde_json::Value>;

/// Allows for sending an arbitrary request that is not part of the ACP spec.
/// Extension methods provide a way to add custom functionality while maintaining
/// protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtRequest {
    /// Wire method name for this extension request.
    ///
    /// Extension method names must start with `_`.
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    #[schemars(with = "serde_json::Value")]
    pub params: Arc<RawValue>,
}

impl ExtRequest {
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

/// Allows for sending an arbitrary response to an [`ExtRequest`] that is not part of the ACP spec.
/// Extension methods provide a way to add custom functionality while maintaining
/// protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, From)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtResponse(#[schemars(with = "serde_json::Value")] pub Arc<RawValue>);

impl ExtResponse {
    #[must_use]
    pub fn new(params: Arc<RawValue>) -> Self {
        Self(params)
    }
}

/// Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
/// Extension notifications provide a way to send one-way messages for custom functionality
/// while maintaining protocol compatibility.
///
/// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ExtNotification {
    /// Wire method name for this extension notification.
    ///
    /// Extension method names must start with `_`.
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    #[schemars(with = "serde_json::Value")]
    pub params: Arc<RawValue>,
}

impl ExtNotification {
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>, params: Arc<RawValue>) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(s: &str) -> Arc<RawValue> {
        serde_json::value::RawValue::from_string(s.to_string())
            .unwrap()
            .into()
    }

    #[test]
    fn ext_request_serializes_only_params_not_method() {
        let req = ExtRequest::new("_custom/ping", raw(r#"{"k":1}"#));
        let serialized = serde_json::to_string(&req).unwrap();
        assert_eq!(serialized, r#"{"k":1}"#);
        assert!(!serialized.contains("_custom/ping"));
    }

    #[test]
    fn ext_request_deserializes_into_params_with_default_method() {
        let req: ExtRequest = serde_json::from_str(r#"{"k":1}"#).unwrap();
        assert_eq!(req.params.get(), r#"{"k":1}"#);
        assert!(req.method.is_empty());
    }

    #[test]
    fn ext_notification_method_is_skipped_when_serializing() {
        let notif = ExtNotification::new("_custom/event", raw("[1,2,3]"));
        let serialized = serde_json::to_string(&notif).unwrap();
        assert_eq!(serialized, "[1,2,3]");
        assert!(!serialized.contains("_custom/event"));
    }

    #[test]
    fn ext_response_round_trips_as_raw_params() {
        let resp = ExtResponse::new(raw(r#"{"ok":true}"#));
        let serialized = serde_json::to_string(&resp).unwrap();
        assert_eq!(serialized, r#"{"ok":true}"#);
        let parsed: ExtResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed.0.get(), r#"{"ok":true}"#);
    }

    #[test]
    fn ext_request_constructors_accept_string_and_arc_str() {
        let from_str = ExtRequest::new("_x", raw("null"));
        let from_arc: Arc<str> = Arc::from("_x");
        let from_arc = ExtRequest::new(from_arc, raw("null"));
        assert_eq!(&*from_str.method, "_x");
        assert_eq!(&*from_arc.method, "_x");
    }
}
