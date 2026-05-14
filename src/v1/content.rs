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
    /// A template that supports variable substitution using {{variable_name}} syntax.
    ///
    /// Allows dynamic content generation by substituting variables into template strings.
    /// Variables are resolved at processing time and can include values from context,
    /// user input, or system state.
    ///
    /// Requires the `promptVariables` prompt capability when included in prompts.
    PromptTemplate(PromptTemplateContent),
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

/// A template content block that supports variable substitution.
///
/// Templates use {{variable_name}} syntax for variable placeholders that can be
/// substituted with actual values at processing time. This enables dynamic content
/// generation and reusable prompt templates.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct PromptTemplateContent {
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub annotations: Option<Annotations>,
    /// The template string with {{variable_name}} placeholders.
    pub template: String,
    /// Variables available for substitution in this template.
    pub variables: Vec<PromptVariable>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PromptTemplateContent {
    #[must_use]
    pub fn new(template: impl Into<String>, variables: Vec<PromptVariable>) -> Self {
        Self {
            annotations: None,
            template: template.into(),
            variables,
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

    /// Substitute variables in the template and return the resolved text.
    ///
    /// This method processes the template string and replaces {{variable_name}}
    /// placeholders with their corresponding values from the variables vector.
    /// If a variable is not found or has no value, the placeholder is left unchanged.
    #[must_use]
    pub fn substitute(&self) -> String {
        let mut result = self.template.clone();

        for variable in &self.variables {
            if let Some(value) = &variable.value {
                let placeholder = format!("{{{{{}}}}}", variable.name);
                result = result.replace(&placeholder, value);
            }
        }

        result
    }
}

/// A variable that can be substituted in a prompt template.
///
/// Variables define named placeholders that can be replaced with actual values
/// during template processing. They can include metadata about expected types,
/// descriptions for user interfaces, and validation constraints.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct PromptVariable {
    /// The variable name (used in {{variable_name}} placeholders).
    pub name: String,
    /// The current value of the variable (if set).
    pub value: Option<String>,
    /// Human-readable description of this variable.
    pub description: Option<String>,
    /// The expected type of this variable's value.
    #[serde(rename = "type")]
    pub variable_type: Option<PromptVariableType>,
    /// Whether this variable is required for template processing.
    #[serde(default)]
    pub required: bool,
    /// Default value to use if no value is provided.
    pub default_value: Option<String>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PromptVariable {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            description: None,
            variable_type: None,
            required: false,
            default_value: None,
            meta: None,
        }
    }

    #[must_use]
    pub fn value(mut self, value: impl IntoOption<String>) -> Self {
        self.value = value.into_option();
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl IntoOption<String>) -> Self {
        self.description = description.into_option();
        self
    }

    #[must_use]
    pub fn variable_type(mut self, variable_type: impl IntoOption<PromptVariableType>) -> Self {
        self.variable_type = variable_type.into_option();
        self
    }

    #[must_use]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    #[must_use]
    pub fn default_value(mut self, default_value: impl IntoOption<String>) -> Self {
        self.default_value = default_value.into_option();
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

    /// Get the effective value for this variable, considering default values.
    #[must_use]
    pub fn effective_value(&self) -> Option<&String> {
        self.value.as_ref().or(self.default_value.as_ref())
    }
}

/// The expected type of a prompt variable's value.
///
/// This helps clients provide appropriate input interfaces and validation
/// for prompt variables.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PromptVariableType {
    /// A string value (default if not specified).
    String,
    /// A numeric value (integer or float).
    Number,
    /// A boolean value (true/false).
    Boolean,
    /// A date/time value in ISO 8601 format.
    DateTime,
    /// A URL or URI reference.
    Url,
    /// An email address.
    Email,
    /// A multiline text value.
    Text,
    /// A value selected from a predefined list (enum-like).
    Select { options: Vec<String> },
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
    fn test_prompt_variable_creation() {
        let var = PromptVariable::new("username")
            .value("alice")
            .description("The user's name")
            .variable_type(PromptVariableType::String)
            .required(true);

        assert_eq!(var.name, "username");
        assert_eq!(var.value, Some("alice".to_string()));
        assert_eq!(var.description, Some("The user's name".to_string()));
        assert_eq!(var.variable_type, Some(PromptVariableType::String));
        assert!(var.required);
    }

    #[test]
    fn test_prompt_template_substitution() {
        let variables = vec![
            PromptVariable::new("name").value("Alice"),
            PromptVariable::new("task").value("code review"),
        ];

        let template =
            PromptTemplateContent::new("Hello {{name}}, please help me with {{task}}.", variables);

        let result = template.substitute();
        assert_eq!(result, "Hello Alice, please help me with code review.");
    }

    #[test]
    fn test_content_block_prompt_template_v1() {
        let variables = vec![PromptVariable::new("language").value("Rust")];
        let template_content = PromptTemplateContent::new("Write {{language}} code", variables);
        let content_block = ContentBlock::PromptTemplate(template_content);

        // Test serialization
        let json = serde_json::to_value(&content_block).unwrap();
        assert_eq!(json["type"], "prompt_template");
        assert_eq!(json["template"], "Write {{language}} code");

        // Test deserialization
        let parsed: ContentBlock = serde_json::from_value(json).unwrap();
        if let ContentBlock::PromptTemplate(template) = parsed {
            assert_eq!(template.template, "Write {{language}} code");
            assert_eq!(template.variables.len(), 1);
            assert_eq!(template.variables[0].name, "language");
        } else {
            panic!("Expected PromptTemplate variant");
        }
    }
}
