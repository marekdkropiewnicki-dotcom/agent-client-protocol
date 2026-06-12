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

    /// Locks the snake_case wire spelling of every priority. `Medium` and
    /// `Low` are not exercised anywhere else in the test suite, and a
    /// rename to camelCase would silently downgrade every entry to the
    /// default priority on the receiver side.
    #[test]
    fn plan_entry_priority_wire_format_is_snake_case() {
        let cases = [
            (PlanEntryPriority::High, "high"),
            (PlanEntryPriority::Medium, "medium"),
            (PlanEntryPriority::Low, "low"),
        ];

        for (priority, wire) in cases {
            assert_eq!(
                serde_json::to_value(&priority).unwrap(),
                json!(wire),
                "wire spelling for {priority:?}"
            );

            let round: PlanEntryPriority = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(round, priority);
        }
    }

    /// `InProgress` is the easy one to break: `rename_all = "snake_case"`
    /// must emit `in_progress`, not `inProgress` or `InProgress`. Pin all
    /// three variants so a rename surfaces as a test failure instead of
    /// silently mis-classifying every running plan entry on the wire.
    #[test]
    fn plan_entry_status_wire_format_is_snake_case() {
        let cases = [
            (PlanEntryStatus::Pending, "pending"),
            (PlanEntryStatus::InProgress, "in_progress"),
            (PlanEntryStatus::Completed, "completed"),
        ];

        for (status, wire) in cases {
            assert_eq!(serde_json::to_value(&status).unwrap(), json!(wire));
            let round: PlanEntryStatus = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(round, status);
        }
    }

    /// A round trip over a representative plan locks the camelCase field
    /// layout (`entries`, `content`, `priority`, `status`) and confirms
    /// that the default empty `meta` is elided via `skip_serializing_none`.
    #[test]
    fn plan_round_trips_through_json() {
        let plan = Plan::new(vec![
            PlanEntry::new("draft", PlanEntryPriority::High, PlanEntryStatus::Pending),
            PlanEntry::new(
                "review",
                PlanEntryPriority::Medium,
                PlanEntryStatus::InProgress,
            ),
        ]);

        let value = serde_json::to_value(&plan).unwrap();
        assert_eq!(
            value,
            json!({
                "entries": [
                    {
                        "content": "draft",
                        "priority": "high",
                        "status": "pending",
                    },
                    {
                        "content": "review",
                        "priority": "medium",
                        "status": "in_progress",
                    },
                ],
            })
        );

        let back: Plan = serde_json::from_value(value).unwrap();
        assert_eq!(back, plan);
    }

    /// Plan entries with unknown variants for `priority` or `status` must
    /// not blow up the whole plan: `VecSkipError` drops just the malformed
    /// entries, and good entries are preserved. This is the forward-compat
    /// contract for adding new priority/status variants on the agent side.
    #[test]
    fn plan_deserialization_skips_unknown_priority_or_status_entries() {
        let value = json!({
            "entries": [
                {
                    "content": "first",
                    "priority": "high",
                    "status": "pending",
                },
                {
                    "content": "broken-priority",
                    "priority": "extreme",
                    "status": "pending",
                },
                {
                    "content": "broken-status",
                    "priority": "low",
                    "status": "abandoned",
                },
                {
                    "content": "last",
                    "priority": "low",
                    "status": "completed",
                },
            ],
        });

        let plan: Plan = serde_json::from_value(value).unwrap();
        let kept: Vec<&str> = plan.entries.iter().map(|e| e.content.as_str()).collect();
        assert_eq!(kept, vec!["first", "last"]);
    }

    /// Wrong outer types for `entries` (null, string, object, ...) must
    /// collapse to an empty `Vec` via `DefaultOnError` rather than failing
    /// the whole plan. This matches the documented `DefaultOnError +
    /// VecSkipError` pattern enforced everywhere in the protocol types.
    #[test]
    fn plan_deserialization_recovers_from_invalid_entries_shape() {
        let null_entries: Plan = serde_json::from_value(json!({ "entries": null })).unwrap();
        assert!(null_entries.entries.is_empty());

        let stringy_entries: Plan =
            serde_json::from_value(json!({ "entries": "oops" })).unwrap();
        assert!(stringy_entries.entries.is_empty());

        let object_entries: Plan =
            serde_json::from_value(json!({ "entries": { "not": "a list" } })).unwrap();
        assert!(object_entries.entries.is_empty());
    }
}
