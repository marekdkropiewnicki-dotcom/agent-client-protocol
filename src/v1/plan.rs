//! Execution plans for complex tasks that require multiple steps.
//!
//! Plans are strategies that agents share with clients through session updates,
//! providing real-time visibility into their thinking and progress.
//!
//! See: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use crate::{IntoOption, Meta, SkipListener};

/// An execution plan for accomplishing complex tasks.
///
/// Plans consist of multiple entries representing individual tasks or goals.
/// Agents report plans to clients to provide visibility into their execution strategy.
/// Plans can evolve during execution as the agent discovers new requirements or completes tasks.
///
/// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Plan {
    /// The list of tasks to be accomplished.
    ///
    /// When updating a plan, the agent must send a complete list of all entries
    /// with their current status. The client replaces the entire plan with each update.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    pub entries: Vec<PlanEntry>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Plan {
    #[must_use]
    pub fn new(entries: Vec<PlanEntry>) -> Self {
        Self {
            entries,
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

/// A single entry in the execution plan.
///
/// Represents a task or goal that the assistant intends to accomplish
/// as part of fulfilling the user's request.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanEntry {
    /// Human-readable description of what this task aims to accomplish.
    pub content: String,
    /// The relative importance of this task.
    /// Used to indicate which tasks are most critical to the overall goal.
    pub priority: PlanEntryPriority,
    /// Current execution status of this task.
    pub status: PlanEntryStatus,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PlanEntry {
    #[must_use]
    pub fn new(
        content: impl Into<String>,
        priority: PlanEntryPriority,
        status: PlanEntryStatus,
    ) -> Self {
        Self {
            content: content.into(),
            priority,
            status,
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

/// Priority levels for plan entries.
///
/// Used to indicate the relative importance or urgency of different
/// tasks in the execution plan.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlanEntryPriority {
    /// High priority task - critical to the overall goal.
    High,
    /// Medium priority task - important but not critical.
    Medium,
    /// Low priority task - nice to have but not essential.
    Low,
}

/// Status of a plan entry in the execution flow.
///
/// Tracks the lifecycle of each task from planning through completion.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlanEntryStatus {
    /// The task has not started yet.
    Pending,
    /// The task is currently being worked on.
    InProgress,
    /// The task has been successfully completed.
    Completed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pending_low(content: &str) -> PlanEntry {
        PlanEntry::new(content, PlanEntryPriority::Low, PlanEntryStatus::Pending)
    }

    // ---- Wire format ----

    #[test]
    fn plan_entry_priority_serialization_is_snake_case() {
        for (variant, expected) in [
            (PlanEntryPriority::High, "high"),
            (PlanEntryPriority::Medium, "medium"),
            (PlanEntryPriority::Low, "low"),
        ] {
            let value = serde_json::to_value(&variant).unwrap();
            assert_eq!(value, json!(expected));
            let parsed: PlanEntryPriority = serde_json::from_value(json!(expected)).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn plan_entry_status_serialization_is_snake_case() {
        for (variant, expected) in [
            (PlanEntryStatus::Pending, "pending"),
            (PlanEntryStatus::InProgress, "in_progress"),
            (PlanEntryStatus::Completed, "completed"),
        ] {
            let value = serde_json::to_value(&variant).unwrap();
            assert_eq!(value, json!(expected));
            let parsed: PlanEntryStatus = serde_json::from_value(json!(expected)).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn plan_entry_priority_rejects_unknown_variants() {
        // PlanEntryPriority is a closed enum without `#[serde(other)]`, so unknown
        // priorities must fail to deserialize. This guards us from silently widening
        // the public protocol surface.
        let result: std::result::Result<PlanEntryPriority, _> =
            serde_json::from_value(json!("urgent"));
        assert!(result.is_err());
    }

    #[test]
    fn plan_entry_round_trip_includes_required_fields() {
        let entry = pending_low("Write tests");
        let value = serde_json::to_value(&entry).unwrap();
        // Each entry must always emit content, priority, and status — these are required
        // for the client to render the plan correctly.
        let map = value.as_object().unwrap();
        assert_eq!(map["content"], "Write tests");
        assert_eq!(map["priority"], "low");
        assert_eq!(map["status"], "pending");
        assert!(!map.contains_key("_meta"));

        let parsed: PlanEntry = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, entry);
    }

    #[test]
    fn plan_omits_meta_when_none_and_includes_when_set() {
        let plan = Plan::new(vec![pending_low("step 1")]);
        let value = serde_json::to_value(&plan).unwrap();
        assert!(!value.as_object().unwrap().contains_key("_meta"));

        let mut meta = Meta::new();
        meta.insert("trace".to_string(), json!("abc"));
        let plan_with_meta = Plan::new(vec![pending_low("step 1")]).meta(meta.clone());
        let value = serde_json::to_value(&plan_with_meta).unwrap();
        assert_eq!(value["_meta"]["trace"], "abc");
    }

    // ---- Tolerance: malformed entries ----

    #[test]
    fn plan_drops_individual_malformed_entries() {
        // `entries` uses DefaultOnError<VecSkipError<...>>, so per-element failures
        // (wrong shape, unknown enum, missing required field) must be skipped while
        // good entries survive — this is critical for forward-compat with future
        // priority/status values.
        let value = json!({
            "entries": [
                {"content": "ok", "priority": "high", "status": "pending"},
                "not an object at all",
                {"content": "missing fields"},
                {"content": "bad priority", "priority": "urgent", "status": "pending"},
                {"content": "also ok", "priority": "low", "status": "completed"}
            ]
        });
        let plan: Plan = serde_json::from_value(value).unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].content, "ok");
        assert_eq!(plan.entries[1].content, "also ok");
    }

    #[test]
    fn plan_with_outer_shape_error_falls_back_to_empty() {
        // DefaultOnError swallows outer shape errors when the field is present but
        // the wrong type — an explicit null or wrong outer type must produce an
        // empty plan rather than a deserialization failure.
        let cases = [
            json!({"entries": null}),
            json!({"entries": "not an array"}),
            json!({"entries": {"k": "v"}}),
        ];
        for v in cases {
            let plan: Plan = serde_json::from_value(v.clone()).unwrap();
            assert!(plan.entries.is_empty(), "expected empty plan for {v}");
        }
    }

    #[test]
    fn plan_with_missing_entries_field_is_an_error() {
        // `entries` is required (no `#[serde(default)]`), so a payload that omits the
        // field altogether must surface a parse error — `DefaultOnError` only catches
        // wrong-type values for a present field. Locking this in protects against an
        // accidental change that would silently accept malformed plans.
        let result: std::result::Result<Plan, _> = serde_json::from_value(json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn plan_entry_meta_is_optional_and_absent_by_default() {
        let entry = pending_low("c");
        assert!(entry.meta.is_none());

        let mut meta = Meta::new();
        meta.insert("k".into(), json!(1));
        let with_meta = pending_low("c").meta(meta);
        let value = serde_json::to_value(&with_meta).unwrap();
        assert_eq!(value["_meta"]["k"], 1);
    }
}
