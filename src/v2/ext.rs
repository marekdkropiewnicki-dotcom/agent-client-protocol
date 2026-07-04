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
    use serde_json::json;
    use serde_json::value::to_raw_value;

    fn params(v: &serde_json::Value) -> Arc<RawValue> {
        Arc::from(to_raw_value(v).unwrap())
    }

    #[test]
    fn ext_request_serializes_only_params_and_drops_method() {
        let req = ExtRequest::new("_my/method", params(&json!({ "a": 1, "b": "x" })));
        let serialized = serde_json::to_value(&req).unwrap();
        assert_eq!(serialized, json!({ "a": 1, "b": "x" }));
    }

    #[test]
    fn ext_request_deserializes_to_empty_method_placeholder() {
        let raw = r#"{"anything":true}"#;
        let req: ExtRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(&*req.method, "");
        assert_eq!(req.params.get(), r#"{"anything":true}"#);
    }

    #[test]
    fn ext_notification_serializes_only_params_and_drops_method() {
        let notification = ExtNotification::new("_my/notify", params(&json!({ "seq": 42 })));
        let serialized = serde_json::to_value(&notification).unwrap();
        assert_eq!(serialized, json!({ "seq": 42 }));
    }

    #[test]
    fn ext_notification_deserializes_to_empty_method_placeholder() {
        let raw = r#"{"ok":true}"#;
        let notif: ExtNotification = serde_json::from_str(raw).unwrap();
        assert_eq!(&*notif.method, "");
        assert_eq!(notif.params.get(), r#"{"ok":true}"#);
    }

    #[test]
    fn ext_response_is_transparent_over_inner_raw_value() {
        let payload = params(&json!({ "result": [1, 2, 3] }));
        let response = ExtResponse::new(payload);

        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized, json!({ "result": [1, 2, 3] }));

        let round: ExtResponse = serde_json::from_value(serialized).unwrap();
        assert_eq!(round.0.get(), r#"{"result":[1,2,3]}"#);
    }

    #[test]
    fn ext_request_preserves_method_in_memory() {
        let req = ExtRequest::new("_foo/bar", params(&json!(null)));
        assert_eq!(&*req.method, "_foo/bar");
    }
}
