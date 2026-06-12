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
    use serde_json::{from_value, json, to_value};

    /// `ExtRequest` carries an extension method name **and** a JSON payload
    /// internally, but only the payload travels on the wire because:
    ///
    /// - `method` is marked `#[serde(skip)]` (it lives in the JSON-RPC
    ///   envelope's `method` field, not in `params`).
    /// - the struct itself is `#[serde(transparent)]`, so the wire shape is
    ///   exactly the inner `params` value.
    ///
    /// If anyone removes the `#[serde(skip)]` (or the `#[serde(transparent)]`)
    /// the method name would either leak into `params` or wrap the payload
    /// in an extra object, both of which silently break every extension.
    #[test]
    fn ext_request_serializes_only_params_transparently() {
        let raw: Arc<RawValue> = serde_json::value::to_raw_value(&json!({"x": 1})).unwrap().into();
        let request = ExtRequest::new("_custom/ping", raw);

        let wire = to_value(&request).unwrap();
        assert_eq!(wire, json!({ "x": 1 }));
    }

    /// Deserializing back from the on-wire payload gives an `ExtRequest`
    /// whose `params` round-trip to the same JSON, but the `method` field
    /// is reset to the empty string because it doesn't travel on the wire.
    /// Routing code is responsible for filling `method` from the JSON-RPC
    /// envelope. Locking this behaviour stops anyone from accidentally
    /// adding a default that hides the missing routing key.
    #[test]
    fn ext_request_deserializes_with_empty_method_when_only_params_are_on_the_wire() {
        let request: ExtRequest = from_value(json!({"x": 1})).unwrap();
        assert_eq!(&*request.method, "");
        assert_eq!(from_value::<serde_json::Value>(request.params.get().parse().unwrap()).unwrap(),
            json!({"x": 1})
        );
    }

    /// `ExtResponse` is `#[serde(transparent)]` over an `Arc<RawValue>`, so
    /// the wire shape is exactly the inner JSON. Any wrapper object would
    /// be an incompatible change for every existing extension.
    #[test]
    fn ext_response_is_transparent_over_raw_value() {
        let raw: Arc<RawValue> = serde_json::value::to_raw_value(&json!([1, 2, 3])).unwrap().into();
        let response = ExtResponse::new(raw);
        assert_eq!(to_value(&response).unwrap(), json!([1, 2, 3]));
    }

    /// Mirror of [`ext_request_serializes_only_params_transparently`] for
    /// notifications. Notifications and requests both carry a `method` /
    /// `params` pair internally and must serialize transparently as just
    /// the `params`.
    #[test]
    fn ext_notification_serializes_only_params_transparently() {
        let raw: Arc<RawValue> = serde_json::value::to_raw_value(&json!({"event": "tick"}))
            .unwrap()
            .into();
        let notification = ExtNotification::new("_custom/tick", raw);

        let wire = to_value(&notification).unwrap();
        assert_eq!(wire, json!({ "event": "tick" }));
    }
}
