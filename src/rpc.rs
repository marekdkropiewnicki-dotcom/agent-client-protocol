use std::sync::Arc;

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// JSON RPC Request Id
///
/// An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null \[1\] and Numbers SHOULD NOT contain fractional parts \[2\]
///
/// The Server MUST reply with the same value in the Response object if included. This member is used to correlate the context between the two objects.
///
/// \[1\] The use of Null as a value for the id member in a Request object is discouraged, because this specification uses a value of Null for Responses with an unknown id. Also, because JSON-RPC 1.0 uses an id value of Null for Notifications this could cause confusion in handling.
///
/// \[2\] Fractional parts may be problematic, since many decimal fractions cannot be represented exactly as binary fractions.
#[derive(
    Debug,
    PartialEq,
    Clone,
    Hash,
    Eq,
    Deserialize,
    Serialize,
    PartialOrd,
    Ord,
    Display,
    JsonSchema,
    From,
)]
#[serde(untagged)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[from(String, i64)]
pub enum RequestId {
    #[display("null")]
    Null,
    Number(i64),
    Str(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[schemars(rename = "{Params}", extend("x-docs-ignore" = true))]
#[skip_serializing_none]
pub struct Request<Params> {
    pub id: RequestId,
    pub method: Arc<str>,
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[serde(untagged)]
#[schemars(rename = "{Result}", extend("x-docs-ignore" = true))]
pub enum Response<Result, Error> {
    Result { id: RequestId, result: Result },
    Error { id: RequestId, error: Error },
}

impl<R, E> Response<R, E> {
    #[must_use]
    pub fn new(id: impl Into<RequestId>, result: std::result::Result<R, E>) -> Self {
        match result {
            Ok(result) => Self::Result {
                id: id.into(),
                result,
            },
            Err(error) => Self::Error {
                id: id.into(),
                error,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[schemars(rename = "{Params}", extend("x-docs-ignore" = true))]
#[skip_serializing_none]
pub struct Notification<Params> {
    pub method: Arc<str>,
    pub params: Option<Params>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

/// A message (request, response, or notification) with `"jsonrpc": "2.0"` specified as
/// [required by JSON-RPC 2.0 Specification][1].
///
/// [1]: https://www.jsonrpc.org/specification#compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct JsonRpcMessage<M> {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    message: M,
}

impl<M> JsonRpcMessage<M> {
    /// Wraps the provided message into a versioned [`JsonRpcMessage`].
    #[must_use]
    pub fn wrap(message: M) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            message,
        }
    }

    /// Unwraps the contained message.
    #[must_use]
    pub fn into_inner(self) -> M {
        self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AgentNotification, CancelNotification, ClientNotification, ContentBlock, ContentChunk,
        SessionId, SessionNotification, SessionUpdate, TextContent,
    };
    use serde_json::{Number, Value, json};

    #[test]
    fn id_deserialization() {
        let id = serde_json::from_value::<RequestId>(Value::Null).unwrap();
        assert_eq!(id, RequestId::Null);

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_u128(1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(1));

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_i128(-1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(-1));

        let id = serde_json::from_value::<RequestId>(Value::String("id".to_owned())).unwrap();
        assert_eq!(id, RequestId::Str("id".to_owned()));
    }

    #[test]
    fn id_serialization() {
        let id = serde_json::to_value(RequestId::Null).unwrap();
        assert_eq!(id, Value::Null);

        let id = serde_json::to_value(RequestId::Number(1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_u128(1).unwrap()));

        let id = serde_json::to_value(RequestId::Number(-1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_i128(-1).unwrap()));

        let id = serde_json::to_value(RequestId::Str("id".to_owned())).unwrap();
        assert_eq!(id, Value::String("id".to_owned()));
    }

    #[test]
    fn id_display() {
        let id = RequestId::Null;
        assert_eq!(id.to_string(), "null");

        let id = RequestId::Number(1);
        assert_eq!(id.to_string(), "1");

        let id = RequestId::Number(-1);
        assert_eq!(id.to_string(), "-1");

        let id = RequestId::Str("id".to_owned());
        assert_eq!(id.to_string(), "id");
    }

    /// `RequestId::from(...)` exists for both `i64` and `String` and is what
    /// callers rely on at the JSON-RPC envelope boundary; pin both paths.
    #[test]
    fn request_id_from_conversions() {
        let from_i64: RequestId = 42i64.into();
        assert_eq!(from_i64, RequestId::Number(42));

        let from_string: RequestId = String::from("abc").into();
        assert_eq!(from_string, RequestId::Str("abc".into()));
    }

    /// Negative integer ids serialize to JSON numbers rather than getting
    /// promoted to a string; covers a case `id_serialization` does not pin.
    #[test]
    fn request_id_negative_serializes_as_number() {
        let id = RequestId::Number(-9_223_372_036_854_775_807);
        let v = serde_json::to_value(&id).unwrap();
        assert!(v.is_number(), "{v}");
        let back: RequestId = serde_json::from_value(v).unwrap();
        assert_eq!(back, id);
    }

    /// Floating-point JSON ids must be rejected — the JSON-RPC spec says
    /// numeric ids should not contain fractional parts, and the schema is
    /// `i64`. A regression here would silently accept malformed peer messages.
    #[test]
    fn request_id_rejects_fractional_number() {
        let result: Result<RequestId, _> = serde_json::from_str("1.5");
        assert!(result.is_err(), "fractional ids must be rejected");
    }

    #[test]
    fn response_new_routes_ok_and_err_into_correct_variant() {
        let ok: Response<i32, &'static str> = Response::new(7i64, Ok(123));
        match ok {
            Response::Result { id, result } => {
                assert_eq!(id, RequestId::Number(7));
                assert_eq!(result, 123);
            }
            Response::Error { .. } => panic!("Ok must produce Result variant"),
        }

        let err: Response<i32, &'static str> = Response::new(String::from("x"), Err("boom"));
        match err {
            Response::Error { id, error } => {
                assert_eq!(id, RequestId::Str("x".into()));
                assert_eq!(error, "boom");
            }
            Response::Result { .. } => panic!("Err must produce Error variant"),
        }
    }

    /// `Response<R, E>` is `#[serde(untagged)]`, so the deserializer chooses
    /// the variant by which discriminant field is present. The result variant
    /// must round-trip through `{ id, result }` and the error variant through
    /// `{ id, error }` without leakage.
    #[test]
    fn response_untagged_round_trip() {
        let ok: Response<i32, String> = Response::Result {
            id: RequestId::Number(1),
            result: 42,
        };
        let v = serde_json::to_value(&ok).unwrap();
        assert_eq!(v, json!({ "id": 1, "result": 42 }));
        let back: Response<i32, String> = serde_json::from_value(v).unwrap();
        assert!(matches!(
            back,
            Response::Result {
                id: RequestId::Number(1),
                result: 42
            }
        ));

        let err: Response<i32, String> = Response::Error {
            id: RequestId::Str("abc".into()),
            error: "nope".into(),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v, json!({ "id": "abc", "error": "nope" }));
        let back: Response<i32, String> = serde_json::from_value(v).unwrap();
        assert!(matches!(
            back,
            Response::Error {
                id: RequestId::Str(_),
                error,
            } if error == "nope"
        ));
    }

    /// The `jsonrpc` field is required by the spec and must be exactly
    /// `"2.0"`. Any other value, missing field, or non-string must fail to
    /// deserialize, otherwise a malformed peer could be treated as valid.
    #[test]
    fn json_rpc_message_enforces_version_2_0() {
        // Version 2.0 — valid.
        let body = json!({
            "jsonrpc": "2.0",
            "method": "ping",
            "params": null,
        });
        let parsed: JsonRpcMessage<Notification<Value>> =
            serde_json::from_value(body).expect("2.0 must parse");
        assert_eq!(&*parsed.message.method, "ping");

        // Version 1.0 — must be rejected.
        let body = json!({
            "jsonrpc": "1.0",
            "method": "ping",
        });
        assert!(
            serde_json::from_value::<JsonRpcMessage<Notification<Value>>>(body).is_err(),
            "version 1.0 must be rejected"
        );

        // Missing jsonrpc field — must be rejected.
        let body = json!({
            "method": "ping",
        });
        assert!(
            serde_json::from_value::<JsonRpcMessage<Notification<Value>>>(body).is_err(),
            "missing jsonrpc field must be rejected"
        );

        // Numeric (not string) version — must be rejected.
        let body = json!({
            "jsonrpc": 2.0,
            "method": "ping",
        });
        assert!(
            serde_json::from_value::<JsonRpcMessage<Notification<Value>>>(body).is_err(),
            "non-string jsonrpc must be rejected"
        );
    }

    /// `wrap` always emits `"jsonrpc": "2.0"` and `into_inner` recovers the
    /// inner payload byte-for-byte.
    #[test]
    fn json_rpc_message_wrap_and_into_inner() {
        let inner = Notification::<Value> {
            method: "x".into(),
            params: Some(json!({ "k": 1 })),
        };
        let wrapped = JsonRpcMessage::wrap(inner);
        let v = serde_json::to_value(&wrapped).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "x");
        assert_eq!(v["params"], json!({"k": 1}));

        let back = wrapped.into_inner();
        assert_eq!(&*back.method, "x");
        assert_eq!(back.params, Some(json!({ "k": 1 })));
    }

    /// JSON-RPC 2.0 lets `params` be omitted entirely; on the wire we
    /// currently emit it as `null` (because `#[skip_serializing_none]` does
    /// not skip generic `Option<Params>` fields). Pin both the current
    /// serialization behavior and the looser deserialization contract — we
    /// MUST accept both an omitted `params` field and an explicit `null`.
    #[test]
    fn notification_params_serialization_and_deserialization_contract() {
        let n = Notification::<Value> {
            method: "ping".into(),
            params: None,
        };
        let v = serde_json::to_value(&n).unwrap();
        // Current wire form: `params` is present with `null`.
        assert_eq!(v, json!({ "method": "ping", "params": Value::Null }));

        // Deserializer must accept omitted params too.
        let n: Notification<Value> = serde_json::from_value(json!({ "method": "ping" })).unwrap();
        assert_eq!(&*n.method, "ping");
        assert!(n.params.is_none());

        // ...and explicit null.
        let n: Notification<Value> =
            serde_json::from_value(json!({ "method": "ping", "params": null })).unwrap();
        assert!(n.params.is_none());
    }

    #[test]
    fn request_params_serialization_and_deserialization_contract() {
        let r = Request::<Value> {
            id: RequestId::Number(1),
            method: "ping".into(),
            params: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(
            v,
            json!({ "id": 1, "method": "ping", "params": Value::Null })
        );

        let r: Request<Value> =
            serde_json::from_value(json!({ "id": 1, "method": "ping" })).unwrap();
        assert_eq!(r.id, RequestId::Number(1));
        assert!(r.params.is_none());

        let r: Request<Value> =
            serde_json::from_value(json!({ "id": 1, "method": "ping", "params": null })).unwrap();
        assert!(r.params.is_none());
    }

    #[test]
    fn notification_wire_format() {
        // Test client -> agent notification wire format
        let outgoing_msg = JsonRpcMessage::wrap(Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("test-123".into()),
                meta: None,
            })),
        });

        let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "cancel",
                "params": {
                    "sessionId": "test-123"
                },
            })
        );

        // Test agent -> client notification wire format
        let outgoing_msg = JsonRpcMessage::wrap(Notification {
            method: "sessionUpdate".into(),
            params: Some(AgentNotification::SessionNotification(
                SessionNotification {
                    session_id: SessionId("test-456".into()),
                    update: SessionUpdate::AgentMessageChunk(ContentChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello".to_string(),
                            meta: None,
                        }),
                        #[cfg(feature = "unstable_message_id")]
                        message_id: None,
                        meta: None,
                    }),
                    meta: None,
                },
            )),
        });

        let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "sessionUpdate",
                "params": {
                    "sessionId": "test-456",
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {
                            "type": "text",
                            "text": "Hello"
                        }
                    }
                }
            })
        );
    }
}
