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
    use super::*;
    use serde_json::json;

    #[test]
    fn method_name_constants_match_wire_format() {
        assert_eq!(MCP_CONNECT_METHOD_NAME, "mcp/connect");
        assert_eq!(MCP_MESSAGE_METHOD_NAME, "mcp/message");
        assert_eq!(MCP_DISCONNECT_METHOD_NAME, "mcp/disconnect");
    }

    #[test]
    fn connect_request_round_trip() {
        let req = ConnectMcpRequest::new("project-tools-id");
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v, json!({"acpId": "project-tools-id"}));

        let parsed: ConnectMcpRequest = serde_json::from_value(v).unwrap();
        assert_eq!(parsed.acp_id, McpServerAcpId::new("project-tools-id"));
        assert!(parsed.meta.is_none());
    }

    #[test]
    fn connect_response_round_trip() {
        let resp = ConnectMcpResponse::new("conn-42");
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v, json!({"connectionId": "conn-42"}));

        let parsed: ConnectMcpResponse = serde_json::from_value(v).unwrap();
        assert_eq!(parsed.connection_id, McpConnectionId::new("conn-42"));
    }

    #[test]
    fn message_request_omits_unset_params_on_the_wire() {
        let req = MessageMcpRequest::new("conn-1", "tools/list");
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v, json!({"connectionId": "conn-1", "method": "tools/list"}));
        assert!(
            !v.as_object().unwrap().contains_key("params"),
            "unset params must be omitted, got {v}"
        );
    }

    #[test]
    fn message_request_serializes_provided_params() {
        let mut params = serde_json::Map::new();
        params.insert("cursor".into(), json!("abc"));

        let req = MessageMcpRequest::new("conn-1", "tools/list").params(params.clone());
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v,
            json!({
                "connectionId": "conn-1",
                "method": "tools/list",
                "params": {"cursor": "abc"},
            })
        );

        let parsed: MessageMcpRequest = serde_json::from_value(v).unwrap();
        assert_eq!(parsed.params, Some(params));
    }

    #[test]
    fn message_request_explicit_null_params_decode_to_none() {
        let omitted: MessageMcpRequest =
            serde_json::from_value(json!({"connectionId": "conn-1", "method": "ping"})).unwrap();
        let explicit_null: MessageMcpRequest = serde_json::from_value(
            json!({"connectionId": "conn-1", "method": "ping", "params": null}),
        )
        .unwrap();

        assert_eq!(omitted.params, None);
        assert_eq!(explicit_null.params, None);
    }

    #[test]
    fn message_notification_round_trip_matches_request_shape() {
        let note = MessageMcpNotification::new("conn-1", "notifications/progress");
        let v = serde_json::to_value(&note).unwrap();
        assert_eq!(
            v,
            json!({
                "connectionId": "conn-1",
                "method": "notifications/progress",
            })
        );

        let parsed: MessageMcpNotification = serde_json::from_value(json!({
            "connectionId": "conn-1",
            "method": "notifications/progress",
            "params": {"progressToken": "tok", "progress": 1}
        }))
        .unwrap();
        assert_eq!(
            parsed.params.unwrap().get("progressToken").unwrap(),
            &json!("tok")
        );
    }

    #[test]
    fn message_response_preserves_arbitrary_inner_json_verbatim() {
        let raw = serde_json::value::RawValue::from_string(
            r#"{"tools":[{"name":"echo"}],"nextCursor":null}"#.to_string(),
        )
        .unwrap();
        let resp = MessageMcpResponse::new(raw.into());

        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v, json!({"tools": [{"name": "echo"}], "nextCursor": null}));

        let reparsed: MessageMcpResponse = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(serde_json::to_value(&reparsed).unwrap(), v);
    }

    #[test]
    fn disconnect_request_round_trip() {
        let req = DisconnectMcpRequest::new("conn-7");
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v, json!({"connectionId": "conn-7"}));

        let parsed: DisconnectMcpRequest = serde_json::from_value(v).unwrap();
        assert_eq!(parsed.connection_id, McpConnectionId::new("conn-7"));
    }

    #[test]
    fn disconnect_response_default_is_empty_object() {
        let resp = DisconnectMcpResponse::new();
        assert_eq!(serde_json::to_value(&resp).unwrap(), json!({}));
        let parsed: DisconnectMcpResponse = serde_json::from_value(json!({})).unwrap();
        assert_eq!(parsed, DisconnectMcpResponse::default());
    }

    #[test]
    fn meta_is_round_tripped_under_underscored_key() {
        let mut meta = serde_json::Map::new();
        meta.insert("trace_id".into(), json!("xyz"));

        let req = ConnectMcpRequest::new("acp").meta(meta.clone());
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["_meta"], json!({"trace_id": "xyz"}));

        let parsed: ConnectMcpRequest = serde_json::from_value(v).unwrap();
        assert_eq!(parsed.meta.as_ref().unwrap(), &meta);
    }
}
