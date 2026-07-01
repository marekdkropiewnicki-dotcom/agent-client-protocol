//! Content blocks for representing various types of information in the Agent Client Protocol.
//!
//! This module defines the core content types used throughout the protocol for communication
//! between agents and clients. Content blocks provide a flexible, extensible way to represent
//! text, images, audio, and other resources in prompts, responses, and tool call results.
//!
//! The content block structure is designed to be compatible with the Model Context Protocol (MCP),
//! allowing seamless integration between ACP and MCP-based tools.
//!
//! See: [Content](https://agentclientprotocol.com/protocol/content)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use crate::{IntoOption, Meta, SkipListener};

/// Content blocks represent displayable information in the Agent Client Protocol.
///
/// They provide a structured way to handle various types of user-facing content—whether
/// it's text from language models, images for analysis, or embedded resources for context.
///
/// Content blocks appear in:
/// - User prompts sent via `session/prompt`
/// - Language model output streamed through `session/update` notifications
/// - Progress updates and results from tool calls
///
/// This structure is compatible with the Model Context Protocol (MCP), enabling
/// agents to seamlessly forward content from MCP tool outputs without transformation.
///
/// See protocol docs: [Content](https://agentclientprotocol.com/protocol/content)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum ContentBlock {
    /// Text content. May be plain text or formatted with Markdown.
    ///
    /// All agents MUST support text content blocks in prompts.
    /// Clients SHOULD render this text as Markdown.
    Text(TextContent),
    /// Images for visual context or analysis.
    ///
    /// Requires the `image` prompt capability when included in prompts.
    Image(ImageContent),
    /// Audio data for transcription or analysis.
    ///
    /// Requires the `audio` prompt capability when included in prompts.
    Audio(AudioContent),
    /// References to resources that the agent can access.
    ///
    /// All agents MUST support resource links in prompts.
    ResourceLink(ResourceLink),
    /// Complete resource contents embedded directly in the message.
    ///
    /// Preferred for including context as it avoids extra round-trips.
    ///
    /// Requires the `embeddedContext` prompt capability when included in prompts.
    Resource(EmbeddedResource),
}

/// Text provided to or from an LLM.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct TextContent {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub annotations: Option<Annotations>,
    pub text: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl TextContent {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            annotations: None,
            text: text.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn annotations(mut self, annotations: impl IntoOption<Annotations>) -> Self {
        self.annotations = annotations.into_option();
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

impl<T: Into<String>> From<T> for ContentBlock {
    fn from(value: T) -> Self {
        Self::Text(TextContent::new(value))
    }
}

/// An image provided to or from an LLM.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageContent {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub annotations: Option<Annotations>,
    pub data: String,
    pub mime_type: String,
    pub uri: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ImageContent {
    #[must_use]
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            annotations: None,
            data: data.into(),
            mime_type: mime_type.into(),
            uri: None,
            meta: None,
        }
    }

    #[must_use]
    pub fn annotations(mut self, annotations: impl IntoOption<Annotations>) -> Self {
        self.annotations = annotations.into_option();
        self
    }

    #[must_use]
    pub fn uri(mut self, uri: impl IntoOption<String>) -> Self {
        self.uri = uri.into_option();
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

/// Audio provided to or from an LLM.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AudioContent {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub annotations: Option<Annotations>,
    pub data: String,
    pub mime_type: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl AudioContent {
    #[must_use]
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            annotations: None,
            data: data.into(),
            mime_type: mime_type.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn annotations(mut self, annotations: impl IntoOption<Annotations>) -> Self {
        self.annotations = annotations.into_option();
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

/// The contents of a resource, embedded into a prompt or tool call result.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct EmbeddedResource {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub annotations: Option<Annotations>,
    pub resource: EmbeddedResourceResource,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl EmbeddedResource {
    #[must_use]
    pub fn new(resource: EmbeddedResourceResource) -> Self {
        Self {
            annotations: None,
            resource,
            meta: None,
        }
    }

    #[must_use]
    pub fn annotations(mut self, annotations: impl IntoOption<Annotations>) -> Self {
        self.annotations = annotations.into_option();
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

/// Resource content that can be embedded in a message.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EmbeddedResourceResource {
    TextResourceContents(TextResourceContents),
    BlobResourceContents(BlobResourceContents),
}

/// Text-based resource contents.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TextResourceContents {
    pub mime_type: Option<String>,
    pub text: String,
    pub uri: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl TextResourceContents {
    #[must_use]
    pub fn new(text: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            mime_type: None,
            text: text.into(),
            uri: uri.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn mime_type(mut self, mime_type: impl IntoOption<String>) -> Self {
        self.mime_type = mime_type.into_option();
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

/// Binary resource contents.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BlobResourceContents {
    pub blob: String,
    pub mime_type: Option<String>,
    pub uri: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl BlobResourceContents {
    #[must_use]
    pub fn new(blob: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            blob: blob.into(),
            mime_type: None,
            uri: uri.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn mime_type(mut self, mime_type: impl IntoOption<String>) -> Self {
        self.mime_type = mime_type.into_option();
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

/// A resource that the server is capable of reading, included in a prompt or tool call result.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResourceLink {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub annotations: Option<Annotations>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub name: String,
    pub size: Option<i64>,
    pub title: Option<String>,
    pub uri: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ResourceLink {
    #[must_use]
    pub fn new(name: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            annotations: None,
            description: None,
            mime_type: None,
            name: name.into(),
            size: None,
            title: None,
            uri: uri.into(),
            meta: None,
        }
    }

    #[must_use]
    pub fn annotations(mut self, annotations: impl IntoOption<Annotations>) -> Self {
        self.annotations = annotations.into_option();
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    #[must_use]
    pub fn mime_type(mut self, mime_type: impl IntoOption<String>) -> Self {
        self.mime_type = mime_type.into_option();
        self
    }

    #[must_use]
    pub fn size(mut self, size: impl IntoOption<i64>) -> Self {
        self.size = size.into_option();
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
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

/// Optional annotations for the client. The client can use annotations to inform how objects are used or displayed
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Annotations {
    #[serde_as(deserialize_as = "DefaultOnError<Option<VecSkipError<_, SkipListener>>>")]
    #[serde(default)]
    pub audience: Option<Vec<Role>>,
    pub last_modified: Option<String>,
    pub priority: Option<f64>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Annotations {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn audience(mut self, audience: impl IntoOption<Vec<Role>>) -> Self {
        self.audience = audience.into_option();
        self
    }

    #[must_use]
    pub fn last_modified(mut self, last_modified: impl IntoOption<String>) -> Self {
        self.last_modified = last_modified.into_option();
        self
    }

    #[must_use]
    pub fn priority(mut self, priority: impl IntoOption<f64>) -> Self {
        self.priority = priority.into_option();
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

/// The sender or recipient of messages and data in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Role {
    Assistant,
    User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_content_roundtrip() {
        let content = TextContent::new("hello world");
        let json = serde_json::to_value(&content).unwrap();
        let parsed: TextContent = serde_json::from_value(json).unwrap();
        assert_eq!(content, parsed);
    }

    #[test]
    fn test_text_content_omits_optional_fields() {
        let content = TextContent::new("hello");
        let json = serde_json::to_value(&content).unwrap();
        assert!(!json.as_object().unwrap().contains_key("annotations"));
        assert!(!json.as_object().unwrap().contains_key("meta"));
    }

    #[test]
    fn test_text_content_from_string() {
        let block: ContentBlock = "hello".into();
        match block {
            ContentBlock::Text(c) => assert_eq!(c.text, "hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_image_content_roundtrip() {
        let content = ImageContent::new("base64data", "image/png");
        let json = serde_json::to_value(&content).unwrap();
        let parsed: ImageContent = serde_json::from_value(json).unwrap();
        assert_eq!(content, parsed);
    }

    #[test]
    fn test_image_content_omits_optional_fields() {
        let content = ImageContent::new("data", "image/png");
        let json = serde_json::to_value(&content).unwrap();
        assert!(!json.as_object().unwrap().contains_key("uri"));
        assert!(!json.as_object().unwrap().contains_key("annotations"));
        assert!(!json.as_object().unwrap().contains_key("meta"));
    }

    #[test]
    fn test_image_content_with_uri() {
        let content = ImageContent::new("data", "image/png").uri("https://example.com/image.png");
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["uri"], "https://example.com/image.png");
    }

    #[test]
    fn test_audio_content_roundtrip() {
        let content = AudioContent::new("base64audio", "audio/mp3");
        let json = serde_json::to_value(&content).unwrap();
        let parsed: AudioContent = serde_json::from_value(json).unwrap();
        assert_eq!(content, parsed);
    }

    #[test]
    fn test_audio_content_omits_optional_fields() {
        let content = AudioContent::new("data", "audio/mp3");
        let json = serde_json::to_value(&content).unwrap();
        assert!(!json.as_object().unwrap().contains_key("annotations"));
        assert!(!json.as_object().unwrap().contains_key("meta"));
    }

    #[test]
    fn test_content_block_text_wire_shape() {
        // Content blocks are #[serde(tag = "type", rename_all = "snake_case")]
        // so the discriminator is a snake_case string. Downstream clients
        // dispatch on this exact value.
        let block = ContentBlock::Text(TextContent::new("hi"));
        let value = serde_json::to_value(&block).unwrap();
        assert_eq!(value["type"], "text");
        assert_eq!(value["text"], "hi");

        let round_trip: ContentBlock = serde_json::from_value(value).unwrap();
        assert_eq!(round_trip, block);
    }

    #[test]
    fn test_content_block_resource_link_wire_shape() {
        let block = ContentBlock::ResourceLink(ResourceLink::new("readme", "file:///README.md"));
        let value = serde_json::to_value(&block).unwrap();
        assert_eq!(value["type"], "resource_link");
        assert_eq!(value["name"], "readme");
        assert_eq!(value["uri"], "file:///README.md");
    }

    #[test]
    fn test_content_block_resource_wire_shape() {
        let embedded = EmbeddedResource::new(EmbeddedResourceResource::TextResourceContents(
            TextResourceContents::new("body", "file:///a.txt"),
        ));
        let block = ContentBlock::Resource(embedded);
        let value = serde_json::to_value(&block).unwrap();
        assert_eq!(value["type"], "resource");
        assert_eq!(value["resource"]["text"], "body");
        assert_eq!(value["resource"]["uri"], "file:///a.txt");
    }

    #[test]
    fn test_role_wire_format_is_camel_case() {
        assert_eq!(serde_json::to_value(&Role::Assistant).unwrap(), "assistant");
        assert_eq!(serde_json::to_value(&Role::User).unwrap(), "user");
        assert_eq!(
            serde_json::from_value::<Role>(serde_json::json!("assistant")).unwrap(),
            Role::Assistant,
        );
        assert_eq!(
            serde_json::from_value::<Role>(serde_json::json!("user")).unwrap(),
            Role::User,
        );
    }

    #[test]
    fn test_annotations_builder_and_wire_shape() {
        let annotations = Annotations::new()
            .audience(vec![Role::Assistant, Role::User])
            .last_modified("2026-01-02T03:04:05Z")
            .priority(0.75);

        let value = serde_json::to_value(&annotations).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "audience": ["assistant", "user"],
                "lastModified": "2026-01-02T03:04:05Z",
                "priority": 0.75
            })
        );

        // Round-trip
        let parsed: Annotations = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, annotations);
    }

    #[test]
    fn test_annotations_omits_none_fields() {
        let annotations = Annotations::new();
        let value = serde_json::to_value(&annotations).unwrap();
        let obj = value.as_object().unwrap();
        assert!(!obj.contains_key("audience"));
        assert!(!obj.contains_key("lastModified"));
        assert!(!obj.contains_key("priority"));
        assert!(!obj.contains_key("_meta"));
    }

    #[test]
    fn test_annotations_audience_skips_unknown_role_variants() {
        // `audience` uses VecSkipError so a future/unknown role entry must be
        // silently dropped instead of failing the whole payload. This is the
        // documented forward-compat contract; regressing it would break older
        // clients when new roles are added.
        let raw = serde_json::json!({
            "audience": ["user", "future_role", "assistant"]
        });
        let parsed: Annotations = serde_json::from_value(raw).unwrap();
        assert_eq!(
            parsed.audience.as_deref(),
            Some(&[Role::User, Role::Assistant][..])
        );
    }

    #[test]
    fn test_annotations_field_falls_back_to_default_on_bad_type() {
        // The parent `annotations` field on content types uses `DefaultOnError`,
        // so a garbage annotations value must degrade to None rather than
        // rejecting the whole content block. Locking this in prevents future
        // strictness regressions from breaking forward compatibility.
        let raw = serde_json::json!({
            "text": "hi",
            "annotations": 42
        });
        let parsed: TextContent = serde_json::from_value(raw).unwrap();
        assert_eq!(parsed.text, "hi");
        assert!(parsed.annotations.is_none());
    }

    #[test]
    fn test_annotations_priority_supports_f64_edges() {
        // priority is `Option<f64>` — sanity-check that it survives round-trip
        // for the documented [0.0, 1.0] range that MCP uses.
        for p in [0.0_f64, 0.25, 0.5, 1.0] {
            let annotations = Annotations::new().priority(p);
            let value = serde_json::to_value(&annotations).unwrap();
            let parsed: Annotations = serde_json::from_value(value).unwrap();
            assert_eq!(parsed.priority, Some(p));
        }
    }

    #[test]
    fn test_resource_link_builder_full_wire_shape() {
        let link = ResourceLink::new("logo", "https://example.com/logo.png")
            .description("Brand mark")
            .mime_type("image/png")
            .size(1024)
            .title("Logo");

        let value = serde_json::to_value(&link).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "name": "logo",
                "uri": "https://example.com/logo.png",
                "description": "Brand mark",
                "mimeType": "image/png",
                "size": 1024,
                "title": "Logo"
            })
        );
        assert!(!value.as_object().unwrap().contains_key("_meta"));

        let parsed: ResourceLink = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, link);
    }

    #[test]
    fn test_resource_link_omits_optional_fields() {
        let link = ResourceLink::new("n", "file:///x");
        let value = serde_json::to_value(&link).unwrap();
        let obj = value.as_object().unwrap();
        for key in [
            "description",
            "mimeType",
            "size",
            "title",
            "_meta",
            "annotations",
        ] {
            assert!(!obj.contains_key(key), "unexpected key: {key}");
        }
    }

    #[test]
    fn test_text_resource_contents_builder_and_wire_shape() {
        let contents = TextResourceContents::new("hello", "file:///a.txt").mime_type("text/plain");
        let value = serde_json::to_value(&contents).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "text": "hello",
                "uri": "file:///a.txt",
                "mimeType": "text/plain"
            })
        );

        // Round-trip
        let parsed: TextResourceContents = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, contents);
    }

    #[test]
    fn test_blob_resource_contents_builder_and_wire_shape() {
        let contents = BlobResourceContents::new("YmxvYg==", "file:///b.bin")
            .mime_type("application/octet-stream");
        let value = serde_json::to_value(&contents).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "blob": "YmxvYg==",
                "uri": "file:///b.bin",
                "mimeType": "application/octet-stream"
            })
        );

        let parsed: BlobResourceContents = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, contents);
    }

    #[test]
    fn test_embedded_resource_untagged_dispatches_on_shape() {
        // `EmbeddedResourceResource` is #[serde(untagged)]. The variant is
        // chosen purely by which required field ("text" or "blob") is present.
        // If either arm ever loses/renames its required field, this test
        // catches the ambiguity before it ships.
        let text_json = serde_json::json!({
            "text": "hi",
            "uri": "file:///a.txt"
        });
        match serde_json::from_value::<EmbeddedResourceResource>(text_json).unwrap() {
            EmbeddedResourceResource::TextResourceContents(t) => {
                assert_eq!(t.text, "hi");
                assert_eq!(t.uri, "file:///a.txt");
            }
            EmbeddedResourceResource::BlobResourceContents(_) => {
                panic!("untagged deserialize picked wrong variant")
            }
        }

        let blob_json = serde_json::json!({
            "blob": "YmxvYg==",
            "uri": "file:///b.bin"
        });
        match serde_json::from_value::<EmbeddedResourceResource>(blob_json).unwrap() {
            EmbeddedResourceResource::BlobResourceContents(b) => {
                assert_eq!(b.blob, "YmxvYg==");
                assert_eq!(b.uri, "file:///b.bin");
            }
            EmbeddedResourceResource::TextResourceContents(_) => {
                panic!("untagged deserialize picked wrong variant")
            }
        }
    }

    #[test]
    fn test_embedded_resource_round_trip_text_variant() {
        let resource = EmbeddedResource::new(EmbeddedResourceResource::TextResourceContents(
            TextResourceContents::new("body", "file:///a.txt").mime_type("text/plain"),
        ));

        let value = serde_json::to_value(&resource).unwrap();
        assert_eq!(value["resource"]["text"], "body");
        assert_eq!(value["resource"]["uri"], "file:///a.txt");
        assert_eq!(value["resource"]["mimeType"], "text/plain");

        let parsed: EmbeddedResource = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, resource);
    }

    #[test]
    fn test_embedded_resource_round_trip_blob_variant() {
        let resource = EmbeddedResource::new(EmbeddedResourceResource::BlobResourceContents(
            BlobResourceContents::new("YmxvYg==", "file:///b.bin"),
        ));
        let value = serde_json::to_value(&resource).unwrap();
        assert_eq!(value["resource"]["blob"], "YmxvYg==");
        assert_eq!(value["resource"]["uri"], "file:///b.bin");
        assert!(
            !value["resource"]
                .as_object()
                .unwrap()
                .contains_key("mimeType")
        );

        let parsed: EmbeddedResource = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, resource);
    }
}
