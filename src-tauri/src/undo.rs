//! What would put back what an assistant just did.
//!
//! ## The gap this closes
//!
//! [`crate::audit`] records the acts a person has to account for, and the MCP
//! surface — the one surface where the act was asked for by something that is
//! not the person at the keyboard — wrote **nothing** to it. Measured: the
//! trail is written from eighteen places, and `mcp.rs` was not one of them. So
//! *"`stackvo_stack_down` was called at 14:32"* was a sentence the app could
//! not produce, about the one caller nobody watched.
//!
//! Recording it is half. The other half is the question the record makes
//! somebody ask immediately: **can I put it back?**
//!
//! ## Why the compensation is written down rather than worked out later
//!
//! "Undo the last thing" is easy to say and impossible to do from a line of
//! text: `stackvo_stack_down` stops every container there is, and the set it
//! stopped exists only *before* the call. A compensation computed when
//! somebody presses Undo would be computed against a machine that has changed.
//!
//! So the plan is built **before the tool runs**, out of the state the call is
//! about to change, and stored on the audit line. What is offered later is not
//! a guess about the past; it is what was true at the time, written at the time.
//!
//! ## Most acts have no compensation, and saying which is the feature
//!
//! Four of the twelve writing tools can be put back — the two project
//! lifecycle pairs and Xdebug — and one more can because its pre-state is
//! recorded here. The rest cannot, each for its own reason, and the reason is
//! carried on the line instead of an Undo button that would lie:
//!
//! * a **restart** has already stopped and started the container; there is no
//!   earlier state to return to, because the call went through it;
//! * **generate** overwrote the generated tree and the previous output was not
//!   kept — the input is the thing to change, which is what the app says
//!   everywhere else;
//! * **reissuing a certificate** replaces one in a system trust store, and the
//!   old one was not kept either;
//! * **taking a snapshot** added a file and changed nothing, so there is
//!   nothing to put back — deleting it is a decision for the app;
//! * **`stack_up`** may start any of the profiles it names, and *which*
//!   containers it actually started is not knowable from before the call.
//!   Stopping every project that was down would name containers the call never
//!   touched, which is an undo that does more than the thing it undoes.
//!
//! `stack_down` is the one that reads the machine first, and it is exact
//! rather than approximate: a down stops **everything**, so everything running
//! before it is precisely what it stopped. Putting that back is starting those
//! same containers, services before projects — the order a person would use,
//! because a project that comes up without its database comes up broken.
//!
//! ## An undo is a sequence, not a transaction
//!
//! Nothing here rolls back. If the fourth of six steps fails, the first three
//! stay done and the trail says where it stopped. Pretending otherwise would
//! need a second compensation for the compensation, and the honest sentence —
//! *"three of six were put back, and here is the one that refused"* — is more
//! useful than a promise this cannot keep.

use crate::mcp::Tool;
use serde_json::{json, Value};

/// One call that would put something back.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Step {
    /// An MCP tool name, so the compensation is expressed in the same terms as
    /// the act — and is executable rather than descriptive.
    pub tool: String,
    pub arguments: Value,
}

impl Step {
    fn new(tool: &str, arguments: Value) -> Self {
        Self {
            tool: tool.to_string(),
            arguments,
        }
    }
}

/// Whether an act can be put back, and how — or why not.
///
/// One field on the audit line rather than two, because "there is no undo" is
/// an answer that has to travel with the record. A line carrying neither would
/// read as "nobody worked it out", which is the state this replaced.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Undo {
    /// In this order.
    Steps { steps: Vec<Step> },
    /// And the sentence that says why.
    None { because: String },
}

impl Undo {
    fn none(because: &str) -> Self {
        Undo::None {
            because: because.to_string(),
        }
    }

    pub fn steps(&self) -> &[Step] {
        match self {
            Undo::Steps { steps } => steps,
            Undo::None { .. } => &[],
        }
    }
}

/// The name of the argument that carries the subject of one write tool.
///
/// Used for the audit line's `subject` column, which is what somebody filters
/// on: `stackvo_project_restart` on `shop` and on `blog` are two different acts
/// and a column reading "shop" is the difference.
pub fn subject_of(tool: &Tool, args: &Value) -> String {
    // The declared project argument first, because a tool that names one has
    // said which of its arguments is the subject — `stackvo_hotspots` takes a
    // `name` and a `key`, and only one of them is a thing acts happen to.
    let declared = tool.project_arg.into_iter();
    for key in declared.chain([
        "name",
        "id",
        "service",
        "project",
        "container",
        "mode",
        "scope",
    ]) {
        if let Some(value) = args.get(key).and_then(|v| v.as_str()) {
            return value.to_string();
        }
    }
    // Not "unknown": the tools that take no argument act on the whole stack,
    // and that is the truest thing to write in the column.
    "the stack".to_string()
}

/// The plan for one call, built **before** it runs.
///
/// Async because two of the twelve are about the whole stack and the answer is
/// what the stack is doing right now. The other ten never touch the engine.
pub async fn before(tool: &Tool, args: &Value) -> Undo {
    let string = |key: &str| args.get(key).and_then(|v| v.as_str()).map(str::to_string);

    match tool.name {
        "stackvo_xdebug_set" => {
            match (string("name"), args.get("enabled").and_then(Value::as_bool)) {
                (Some(name), Some(enabled)) => Undo::Steps {
                    steps: vec![Step::new(
                        "stackvo_xdebug_set",
                        json!({ "name": name, "enabled": !enabled }),
                    )],
                },
                // The dispatch arm will refuse this call for the same reason; the
                // plan says so rather than inventing a value to invert.
                _ => Undo::none("the call did not say which project, or on or off"),
            }
        }

        "stackvo_project_start" | "stackvo_project_stop" => match string("name") {
            Some(name) => {
                let back = if tool.name == "stackvo_project_start" {
                    "stackvo_project_stop"
                } else {
                    "stackvo_project_start"
                };
                Undo::Steps {
                    steps: vec![Step::new(back, json!({ "name": name }))],
                }
            }
            None => Undo::none("the call did not name a project"),
        },

        "stackvo_service_start" | "stackvo_service_stop" => match string("id") {
            Some(id) => {
                let back = if tool.name == "stackvo_service_start" {
                    "stackvo_service_stop"
                } else {
                    "stackvo_service_start"
                };
                Undo::Steps {
                    steps: vec![Step::new(back, json!({ "id": id }))],
                }
            }
            None => Undo::none("the call did not name an instance"),
        },

        "stackvo_project_restart" | "stackvo_service_restart" => Undo::none(
            "a restart has already stopped and started it — the call went through the state \
             an undo would return to",
        ),

        "stackvo_generate" => Undo::none(
            "the generated tree was overwritten and the previous output is not kept. Change \
             the input and generate again — that is the repair everywhere else in this app",
        ),

        "stackvo_certificates_reissue" => Undo::none(
            "the certificate it replaced was not kept, and the trust store now holds the new \
             one",
        ),

        "stackvo_snapshot_take" => Undo::none(
            "taking a snapshot added a file and changed nothing — there is nothing to put \
             back, and deleting the file is a decision for the app",
        ),

        "stackvo_stack_up" => Undo::none(
            "an up starts what its profiles name, and which containers it actually started is \
             not knowable before the call. Stopping everything that was down would name \
             containers this call never touched",
        ),

        "stackvo_stack_down" => running_now().await,

        // A read, or a tool added without a decision being made here. The
        // second is why this is not `unreachable!`: a thirteenth writing tool
        // must not panic the surface, and `no_writing_tool_is_left_undecided`
        // below fails the build instead.
        _ => Undo::none("nothing was changed"),
    }
}

/// Everything running, as the sequence that would start it again.
///
/// Services first, then projects: a project that comes up without its database
/// comes up broken, and this is the order a person putting a stack back would
/// use.
async fn running_now() -> Undo {
    let Ok(root) = crate::workspace::resolve().require_root() else {
        return Undo::none("no workspace was open, so what was running was not read");
    };

    let Ok(containers) = crate::engine::stackvo_containers().await else {
        return Undo::none(
            "the engine could not be reached before the call, so what was running was not read",
        );
    };

    let mut steps = Vec::new();

    if let Ok(table) = crate::instances::Table::load(&root) {
        for instance in &table.instances {
            if containers.get(&instance.id).is_some_and(|c| c.running) {
                steps.push(Step::new(
                    "stackvo_service_start",
                    json!({ "id": instance.id }),
                ));
            }
        }
    }

    if let Ok(projects) = crate::commands::list_projects(&root).await {
        for project in projects.iter().filter(|p| p.running) {
            steps.push(Step::new(
                "stackvo_project_start",
                json!({ "name": project.name }),
            ));
        }
    }

    if steps.is_empty() {
        return Undo::none("nothing was running, so there is nothing to start again");
    }

    Undo::Steps { steps }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(name: &str) -> &'static Tool {
        crate::mcp::TOOLS
            .iter()
            .find(|t| t.name == name)
            .expect(name)
    }

    async fn plan(name: &str, args: Value) -> Undo {
        before(tool(name), &args).await
    }

    #[tokio::test]
    async fn a_lifecycle_call_is_undone_by_its_pair() {
        assert_eq!(
            plan("stackvo_project_start", json!({ "name": "shop" })).await,
            Undo::Steps {
                steps: vec![Step::new("stackvo_project_stop", json!({ "name": "shop" }))]
            }
        );
        assert_eq!(
            plan("stackvo_service_stop", json!({ "id": "mysql-8-4" })).await,
            Undo::Steps {
                steps: vec![Step::new(
                    "stackvo_service_start",
                    json!({ "id": "mysql-8-4" })
                )]
            }
        );
    }

    #[tokio::test]
    async fn xdebug_is_undone_by_the_other_value_rather_than_by_off() {
        assert_eq!(
            plan(
                "stackvo_xdebug_set",
                json!({ "name": "shop", "enabled": false })
            )
            .await,
            Undo::Steps {
                steps: vec![Step::new(
                    "stackvo_xdebug_set",
                    json!({ "name": "shop", "enabled": true })
                )]
            },
            "turning it off is undone by turning it on, not by turning it off again"
        );
    }

    /// The half that keeps the button honest.
    #[tokio::test]
    async fn what_cannot_be_undone_says_why_rather_than_saying_nothing() {
        for (name, args, word) in [
            (
                "stackvo_project_restart",
                json!({ "name": "shop" }),
                "restart",
            ),
            ("stackvo_generate", json!({}), "generated tree"),
            ("stackvo_certificates_reissue", json!({}), "certificate"),
            (
                "stackvo_snapshot_take",
                json!({ "service": "mysql", "name": "x" }),
                "added a file",
            ),
            ("stackvo_stack_up", json!({}), "profiles"),
        ] {
            let Undo::None { because } = plan(name, args).await else {
                panic!("{name} claims an undo it cannot perform");
            };
            assert!(
                because.contains(word),
                "{name} gives no usable reason: {because}"
            );
        }
    }

    /// A call missing its argument is refused by the dispatch arm, and the plan
    /// must not invent one to invert.
    #[tokio::test]
    async fn a_call_without_its_subject_plans_nothing() {
        assert!(matches!(
            plan("stackvo_project_start", json!({})).await,
            Undo::None { .. }
        ));
        assert!(matches!(
            plan("stackvo_xdebug_set", json!({ "name": "shop" })).await,
            Undo::None { .. }
        ));
    }

    /// Every step names a tool that exists and can perform it.
    ///
    /// A compensation naming `stackvo_project_halt` would sit on the audit line
    /// looking like an undo and fail the moment anybody pressed it.
    #[tokio::test]
    async fn every_step_names_a_writing_tool_this_server_has() {
        for (name, args) in [
            ("stackvo_project_start", json!({ "name": "shop" })),
            ("stackvo_project_stop", json!({ "name": "shop" })),
            ("stackvo_service_start", json!({ "id": "redis-7-2" })),
            ("stackvo_service_stop", json!({ "id": "redis-7-2" })),
            (
                "stackvo_xdebug_set",
                json!({ "name": "shop", "enabled": true }),
            ),
        ] {
            for step in plan(name, args).await.steps() {
                let found = crate::mcp::TOOLS
                    .iter()
                    .find(|t| t.name == step.tool)
                    .unwrap_or_else(|| panic!("{name} compensates with an unknown {}", step.tool));
                assert!(
                    found.writes,
                    "{name} compensates with {}, which changes nothing",
                    step.tool
                );
            }
        }
    }

    /// The gate that makes a thirteenth writing tool a build failure rather
    /// than a silent "nothing was changed" on the audit line.
    #[tokio::test]
    async fn no_writing_tool_is_left_undecided() {
        for t in crate::mcp::TOOLS.iter().filter(|t| t.writes) {
            let plan = before(t, &json!({})).await;
            if let Undo::None { because } = &plan {
                assert_ne!(
                    because, "nothing was changed",
                    "{} reaches the fallback arm — decide whether it can be undone",
                    t.name
                );
            }
        }
    }

    #[test]
    fn the_subject_column_names_what_was_acted_on() {
        assert_eq!(
            subject_of(tool("stackvo_project_stop"), &json!({ "name": "shop" })),
            "shop"
        );
        assert_eq!(
            subject_of(tool("stackvo_service_start"), &json!({ "id": "mysql-8-4" })),
            "mysql-8-4"
        );
        assert_eq!(
            subject_of(tool("stackvo_stack_down"), &json!({})),
            "the stack"
        );
    }
}
