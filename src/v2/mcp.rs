//! MCP-over-ACP transport types.

use std::sync::Arc;

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_with::skip_serializing_none;

use super::{McpServerAcpId, Meta};
use crate::IntoOption;

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A unique identifier for an active MCP-over-ACP connection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct McpConnectionId(pub Arc<str>);

impl McpConnectionId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for `mcp/connect`.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = MCP_CONNECT_METHOD_NAME))]
#[non_exhaustive]
pub struct ConnectMcpRequest {
    /// The ACP MCP server ID that was provided by the component declaring the MCP server.
    pub acp_id: McpServerAcpId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ConnectMcpRequest {
    #[must_use]
    pub fn new(acp_id: impl Into<McpServerAcpId>) -> Self {
        Self {
            acp_id: acp_id.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response to `mcp/connect`.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = MCP_CONNECT_METHOD_NAME))]
#[non_exhaustive]
pub struct ConnectMcpResponse {
    /// The unique identifier for this MCP-over-ACP connection.
    pub connection_id: McpConnectionId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ConnectMcpResponse {
    #[must_use]
    pub fn new(connection_id: impl Into<McpConnectionId>) -> Self {
        Self {
            connection_id: connection_id.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for `mcp/message`.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "both", "x-method" = MCP_MESSAGE_METHOD_NAME))]
#[non_exhaustive]
pub struct MessageMcpRequest {
    /// The MCP-over-ACP connection this message is sent on.
    pub connection_id: McpConnectionId,
    /// The inner MCP method name.
    pub method: String,
    /// Optional inner MCP params.
    ///
    /// If omitted or set to `null`, the inner MCP message has no params.
    #[serde(default)]
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl MessageMcpRequest {
    #[must_use]
    pub fn new(connection_id: impl Into<McpConnectionId>, method: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            method: method.into(),
            params: None,
            meta: None,
        }
    }

    /// Optional inner MCP params.
    ///
    /// If omitted or set to `null`, the inner MCP message has no params.
    #[must_use]
    pub fn params(
        mut self,
        params: impl IntoOption<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        self.params = params.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Notification parameters for `mcp/message`.
///
/// This is used when the wrapped MCP message is a notification and the outer JSON-RPC
/// envelope has no `id`.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "both", "x-method" = MCP_MESSAGE_METHOD_NAME))]
#[non_exhaustive]
pub struct MessageMcpNotification {
    /// The MCP-over-ACP connection this message is sent on.
    pub connection_id: McpConnectionId,
    /// The inner MCP method name.
    pub method: String,
    /// Optional inner MCP params.
    ///
    /// If omitted or set to `null`, the inner MCP message has no params.
    #[serde(default)]
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl MessageMcpNotification {
    #[must_use]
    pub fn new(connection_id: impl Into<McpConnectionId>, method: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            method: method.into(),
            params: None,
            meta: None,
        }
    }

    /// Optional inner MCP params.
    ///
    /// If omitted or set to `null`, the inner MCP message has no params.
    #[must_use]
    pub fn params(
        mut self,
        params: impl IntoOption<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        self.params = params.into_option();
        self
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response to `mcp/message`.
///
/// This is the inner MCP response result payload. Any JSON value is valid.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, From)]
#[serde(transparent)]
#[schemars(extend("x-side" = "both", "x-method" = MCP_MESSAGE_METHOD_NAME))]
#[non_exhaustive]
pub struct MessageMcpResponse(#[schemars(with = "serde_json::Value")] pub Arc<RawValue>);

impl MessageMcpResponse {
    #[must_use]
    pub fn new(result: Arc<RawValue>) -> Self {
        Self(result)
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for `mcp/disconnect`.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = MCP_DISCONNECT_METHOD_NAME))]
#[non_exhaustive]
pub struct DisconnectMcpRequest {
    /// The MCP-over-ACP connection to close.
    pub connection_id: McpConnectionId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl DisconnectMcpRequest {
    #[must_use]
    pub fn new(connection_id: impl Into<McpConnectionId>) -> Self {
        Self {
            connection_id: connection_id.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response to `mcp/disconnect`.
#[skip_serializing_none]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "client", "x-method" = MCP_DISCONNECT_METHOD_NAME))]
#[non_exhaustive]
pub struct DisconnectMcpResponse {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl DisconnectMcpResponse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// Method name for opening an MCP-over-ACP connection.
pub(crate) const MCP_CONNECT_METHOD_NAME: &str = "mcp/connect";
/// Method name for exchanging MCP-over-ACP messages.
pub(crate) const MCP_MESSAGE_METHOD_NAME: &str = "mcp/message";
/// Method name for closing an MCP-over-ACP connection.
pub(crate) const MCP_DISCONNECT_METHOD_NAME: &str = "mcp/disconnect";

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use serde_json::value::RawValue;

    use super::*;
    use crate::v2::Meta;

    fn sample_meta() -> Meta {
        let mut meta = serde_json::Map::new();
        meta.insert("trace".to_string(), json!("abc-123"));
        meta
    }

    #[test]
    fn mcp_connection_id_from_impls() {
        let from_str: McpConnectionId = "conn-1".into();
        let from_string: McpConnectionId = String::from("conn-1").into();
        let from_arc: McpConnectionId = Arc::<str>::from("conn-1").into();
        assert_eq!(from_str, from_string);
        assert_eq!(from_string, from_arc);
    }

    #[test]
    fn mcp_connection_id_serializes_transparently() {
        let id = McpConnectionId::new("conn-1");
        assert_eq!(serde_json::to_value(&id).unwrap(), json!("conn-1"));
        let deserialized: McpConnectionId = serde_json::from_value(json!("conn-1")).unwrap();
        assert_eq!(deserialized, id);
    }

    #[test]
    fn connect_mcp_request_camel_case_and_meta() {
        let request = ConnectMcpRequest::new("srv-1").meta(sample_meta());
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "acpId": "srv-1",
                "_meta": { "trace": "abc-123" }
            })
        );

        // Absent `_meta` must not be emitted (skip_serializing_none).
        let plain = ConnectMcpRequest::new("srv-1");
        let value = serde_json::to_value(&plain).unwrap();
        assert_eq!(value, json!({ "acpId": "srv-1" }));
        assert!(!value.as_object().unwrap().contains_key("_meta"));

        // Round-trip with meta preserved.
        let round: ConnectMcpRequest =
            serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap();
        assert_eq!(round, request);
    }

    #[test]
    fn connect_mcp_response_round_trip() {
        let response = ConnectMcpResponse::new("conn-1").meta(sample_meta());
        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(
            value,
            json!({
                "connectionId": "conn-1",
                "_meta": { "trace": "abc-123" }
            })
        );
        let round: ConnectMcpResponse = serde_json::from_value(value).unwrap();
        assert_eq!(round, response);
    }

    /// Documents the invariant from `MessageMcpRequest::params`:
    /// "If omitted or set to `null`, the inner MCP message has no params."
    #[test]
    fn message_mcp_request_params_absent_or_null_becomes_none() {
        let absent: MessageMcpRequest = serde_json::from_value(json!({
            "connectionId": "conn-1",
            "method": "tools/list"
        }))
        .unwrap();
        assert_eq!(absent.params, None);

        let explicit_null: MessageMcpRequest = serde_json::from_value(json!({
            "connectionId": "conn-1",
            "method": "tools/list",
            "params": null
        }))
        .unwrap();
        assert_eq!(explicit_null.params, None);

        // Present params survive round-trip.
        let params: serde_json::Map<String, serde_json::Value> =
            [("cursor".to_string(), json!("abc"))].into_iter().collect();
        let with_params = MessageMcpRequest::new("conn-1", "tools/list").params(params.clone());
        assert_eq!(with_params.params.as_ref(), Some(&params));
    }

    /// `None` params must round-trip as an omitted key, not `null`, so the wire
    /// shape matches what MCP clients emit on the other side of the transport.
    #[test]
    fn message_mcp_request_omits_none_params_on_serialize() {
        let request = MessageMcpRequest::new("conn-1", "tools/list");
        let value = serde_json::to_value(&request).unwrap();
        let object = value.as_object().unwrap();
        assert!(
            !object.contains_key("params"),
            "params must be omitted, not null; got: {value}"
        );
        assert_eq!(
            value,
            json!({ "connectionId": "conn-1", "method": "tools/list" })
        );
    }

    #[test]
    fn message_mcp_notification_shares_wire_shape_with_request() {
        // Notifications and requests share the same wire representation on the
        // outer envelope; only the JSON-RPC `id` distinguishes them.
        let params: serde_json::Map<String, serde_json::Value> =
            [("progress".to_string(), json!(0.5))].into_iter().collect();
        let notification =
            MessageMcpNotification::new("conn-1", "notifications/progress").params(params.clone());
        assert_eq!(
            serde_json::to_value(&notification).unwrap(),
            json!({
                "connectionId": "conn-1",
                "method": "notifications/progress",
                "params": { "progress": 0.5 }
            })
        );

        let absent: MessageMcpNotification = serde_json::from_value(json!({
            "connectionId": "conn-1",
            "method": "notifications/progress"
        }))
        .unwrap();
        assert_eq!(absent.params, None);
    }

    /// `MessageMcpResponse` is the only MCP-over-ACP type without a `PartialEq`
    /// impl (its payload is an [`Arc<RawValue>`]), so we compare via the raw
    /// JSON text. This guards the `#[serde(transparent)]` wrapper: any change
    /// that stops emitting the payload verbatim (e.g. accidentally wrapping in
    /// a `{ "result": ... }` object) breaks this test.
    #[test]
    fn message_mcp_response_is_transparent_wrapper_over_raw_value() {
        let raw: Arc<RawValue> = serde_json::from_str(r#"{"tools":[{"name":"echo"}]}"#).unwrap();
        let response = MessageMcpResponse::new(raw);

        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized, json!({ "tools": [ { "name": "echo" } ] }));

        // Non-object payloads (arrays, primitives) also work since RawValue
        // holds any JSON value.
        let array_raw: Arc<RawValue> = serde_json::from_str("[1, 2, 3]").unwrap();
        let array_response = MessageMcpResponse::new(array_raw);
        assert_eq!(
            serde_json::to_value(&array_response).unwrap(),
            json!([1, 2, 3])
        );

        // Round-trip: deserialize -> serialize preserves shape.
        let round: MessageMcpResponse =
            serde_json::from_value(json!({ "tools": [ { "name": "echo" } ] })).unwrap();
        assert_eq!(
            serde_json::to_value(&round).unwrap(),
            json!({ "tools": [ { "name": "echo" } ] })
        );
    }

    #[test]
    fn disconnect_mcp_request_round_trip() {
        let request = DisconnectMcpRequest::new("conn-1");
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "connectionId": "conn-1" })
        );
        let round: DisconnectMcpRequest =
            serde_json::from_value(json!({ "connectionId": "conn-1" })).unwrap();
        assert_eq!(round, request);
    }

    /// The [`Default`] impl is important because `DisconnectMcpResponse` is
    /// used with `#[serde(default)]` inside the `ClientResponse` enum so that
    /// an empty `{}` result payload deserializes cleanly.
    #[test]
    fn disconnect_mcp_response_defaults_to_empty_object() {
        let response = DisconnectMcpResponse::new();
        assert_eq!(response, DisconnectMcpResponse::default());
        assert_eq!(serde_json::to_value(&response).unwrap(), json!({}));

        let round: DisconnectMcpResponse = serde_json::from_value(json!({})).unwrap();
        assert_eq!(round, response);

        // Meta is preserved when present.
        let with_meta = DisconnectMcpResponse::new().meta(sample_meta());
        assert_eq!(
            serde_json::to_value(&with_meta).unwrap(),
            json!({ "_meta": { "trace": "abc-123" } })
        );
    }

    #[test]
    fn method_name_constants_match_wire_spec() {
        assert_eq!(MCP_CONNECT_METHOD_NAME, "mcp/connect");
        assert_eq!(MCP_MESSAGE_METHOD_NAME, "mcp/message");
        assert_eq!(MCP_DISCONNECT_METHOD_NAME, "mcp/disconnect");
    }
}
