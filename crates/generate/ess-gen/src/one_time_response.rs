//! Disclosure obligations preserved by structural projections, never enforced by them.
use ess_compiler::{ir::ResolvedCommand, EssIr};

pub(crate) const OBLIGATION: &str = "Each marked field originates a nonempty plaintext value only in its exact field of one successful response. Never disclose that value again, including retries, rotations, other actors, responses, errors, events, views, persistent fields, logs or restarts. Other marked fields cannot copy it. Structural schemas do not enforce this temporal rule. Implement atomic durable consumption and fresh issuance; verify serial observations with ESS conformance and separately verify storage, logs, response loss, restart and concurrency behavior.";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Policy {
    command: String,
    outcome: String,
    fields: Vec<String>,
    obligation: &'static str,
}

pub(crate) fn command(command: &ResolvedCommand) -> Vec<Policy> {
    command
        .outcomes
        .iter()
        .filter(|outcome| !outcome.one_time_response.is_empty())
        .map(|outcome| Policy {
            command: command.name.to_string(),
            outcome: outcome.name.to_string(),
            fields: outcome.one_time_response.clone(),
            obligation: OBLIGATION,
        })
        .collect()
}

pub(crate) fn all(ir: &EssIr) -> Vec<Policy> {
    ir.commands().values().flat_map(command).collect()
}
