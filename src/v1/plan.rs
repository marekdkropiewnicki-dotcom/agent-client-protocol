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

    /// Each `PlanEntryPriority` variant must serialize to the exact
    /// `snake_case` string the spec calls out. Renaming or reordering
    /// these would silently break every existing agent.
    #[test]
    fn priority_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(PlanEntryPriority::High).unwrap(),
            json!("high")
        );
        assert_eq!(
            serde_json::to_value(PlanEntryPriority::Medium).unwrap(),
            json!("medium")
        );
        assert_eq!(
            serde_json::to_value(PlanEntryPriority::Low).unwrap(),
            json!("low")
        );
    }

    /// Each `PlanEntryStatus` variant must serialize to the exact
    /// `snake_case` string the spec calls out. The `in_progress` form is
    /// especially easy to break (e.g. `inProgress`, `in-progress`) so it
    /// is pinned explicitly.
    #[test]
    fn status_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(PlanEntryStatus::Pending).unwrap(),
            json!("pending")
        );
        assert_eq!(
            serde_json::to_value(PlanEntryStatus::InProgress).unwrap(),
            json!("in_progress")
        );
        assert_eq!(
            serde_json::to_value(PlanEntryStatus::Completed).unwrap(),
            json!("completed")
        );
    }

    /// Unknown priority / status values must fail loudly: the protocol
    /// does not define a catch-all variant here, and silently coercing
    /// an unknown priority would mislead the UI about what the agent is
    /// actually planning.
    #[test]
    fn unknown_priority_and_status_fail_to_deserialize() {
        assert!(serde_json::from_value::<PlanEntryPriority>(json!("urgent")).is_err());
        assert!(serde_json::from_value::<PlanEntryStatus>(json!("blocked")).is_err());
    }

    /// A non-empty plan must round-trip through JSON with all fields
    /// preserved. This is the message agents emit on every plan update.
    #[test]
    fn plan_round_trips_through_json() {
        let plan = Plan::new(vec![
            PlanEntry::new(
                "investigate",
                PlanEntryPriority::High,
                PlanEntryStatus::InProgress,
            ),
            PlanEntry::new("write up", PlanEntryPriority::Low, PlanEntryStatus::Pending),
        ]);

        let value = serde_json::to_value(&plan).unwrap();
        let parsed: Plan = serde_json::from_value(value).unwrap();
        assert_eq!(plan, parsed);
    }

    /// `Plan` and `PlanEntry` must skip the optional `_meta` field when
    /// it is `None`, otherwise every plan update would carry a
    /// distracting `_meta: null`.
    #[test]
    fn meta_is_omitted_when_none() {
        let plan = Plan::new(vec![PlanEntry::new(
            "step",
            PlanEntryPriority::Medium,
            PlanEntryStatus::Completed,
        )]);
        let value = serde_json::to_value(&plan).unwrap();
        let obj = value.as_object().unwrap();
        assert!(
            !obj.contains_key("_meta"),
            "plan _meta should be skipped when None"
        );
        let entry = value["entries"][0].as_object().unwrap();
        assert!(
            !entry.contains_key("_meta"),
            "entry _meta should be skipped when None"
        );
    }

    /// Malformed entries inside the `entries` array must be silently
    /// skipped (per `VecSkipError`) so one bad entry never drops a
    /// whole plan update mid-stream.
    #[test]
    fn malformed_entries_are_skipped_not_fatal() {
        let raw = json!({
            "entries": [
                {
                    "content": "ok",
                    "priority": "high",
                    "status": "pending"
                },
                {
                    "content": "bad - unknown priority",
                    "priority": "urgent",
                    "status": "pending"
                },
                {
                    "content": "also ok",
                    "priority": "low",
                    "status": "completed"
                }
            ]
        });
        let plan: Plan = serde_json::from_value(raw).unwrap();
        assert_eq!(
            plan.entries.len(),
            2,
            "the bad middle entry should be skipped, leaving 2 good ones"
        );
        assert_eq!(plan.entries[0].content, "ok");
        assert_eq!(plan.entries[1].content, "also ok");
    }
}
