//! Rows a filter selects, arranged by callers chosen for them (downstream report, 2026-10-18).
//!
//! An issuing command writes a token's `owner` from its caller (`{caller: user_id}`); a revoking
//! command selects tokens by `owner == input.user_id`. Synthesis used to arrange every token as the
//! one caller of its assignment, so no token sat on the other side of the filter and the branch was
//! refused with a help line about the command's input. Each arranging act is now sent as a caller
//! whose `user_id` carries the value its row needs.
//!
//! Also here: a filter over a field no command writes is refused by that field, and seeded
//! synthesis of a caller-attribute model is still refused.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::authored;
use ess_conformance::report::Status;
use ess_conformance::scenario::{InstanceName, ScenarioValue};
use ess_conformance::synthesize::{
    synthesize, synthesize_with_seeds, AdmittedSeeds, SeedSelection, Synthesis,
};
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use sha2::{Digest, Sha256};

/// Tokens whose owner is the caller that issued them, revoked together by owner.
const TOKENS: &str = "format: ess/24
system: keys
version: v1
domain: keys.tokens
summary: Tokens issued to a holder; an operator revokes every token a holder owns.
types:
  - {name: keys.tokens.TokenId, kind: newtype, of: Uuid}
  - {name: keys.tokens.UserId, kind: newtype, of: Uuid}
entities:
  - name: keys.tokens.Token
    identity: {name: token_id, type: keys.tokens.TokenId}
    fields:
      - {name: owner, type: keys.tokens.UserId}
    lifecycle:
      initial: Active
      states: [Active, Revoked]
      terminal: [Revoked]
      transitions:
        - {name: revoke, from: [Active], to: Revoked}
actors:
  - name: keys.tokens.Holder
    attributes:
      - {name: user_id, type: keys.tokens.UserId}
    may: [keys.tokens.IssueToken]
  - name: keys.tokens.Operator
    attributes:
      - {name: user_id, type: keys.tokens.UserId}
    may: [keys.tokens.RevokeOwned]
events:
  - name: keys.tokens.TokenIssued
    fields:
      - {name: token_id, type: keys.tokens.TokenId}
  - name: keys.tokens.OwnedRevoked
    fields:
      - {name: user_id, type: keys.tokens.UserId}
      - {name: count, type: Integer}
commands:
  - name: keys.tokens.IssueToken
    input: []
    outcomes:
      - name: issued
        creates: keys.tokens.Token
        instance: token_id
        sets: {owner: {caller: user_id}}
        emits: [keys.tokens.TokenIssued]
        payload:
          keys.tokens.TokenIssued: {token_id: {generated: true}}
  - name: keys.tokens.RevokeOwned
    input:
      - {name: user_id, type: keys.tokens.UserId}
    outcomes:
      - name: revoked
        moves: keys.tokens.Token.revoke
        instances: {where: owner == input.user_id}
        emits: [keys.tokens.OwnedRevoked]
        payload:
          keys.tokens.OwnedRevoked: {user_id: input.user_id, count: {count: changed}}
views:
  - name: keys.tokens.Tokens
    source: keys.tokens.Token
    consistency: read_your_writes
    fields:
      - {name: token_id, type: keys.tokens.TokenId}
      - {name: owner, type: keys.tokens.UserId}
      - {name: state, type: keys.tokens.Token.State}
";

const CALLER_WRITE: &str = "        sets: {owner: {caller: user_id}}\n";

const REVOKED: &str = "keys.tokens.RevokeOwned/outcome/revoked";

/// [`TOKENS`] with the owner sent as an input instead of read from the caller.
fn from_input() -> String {
    TOKENS
        .replace(
            "  - name: keys.tokens.IssueToken\n    input: []\n",
            "  - name: keys.tokens.IssueToken\n    input:\n      - {name: owner, type: keys.tokens.UserId}\n",
        )
        .replace(CALLER_WRITE, "        sets: {owner: input.owner}\n")
}

/// [`TOKENS`] with no command writing the owner at all.
fn unwritten() -> String {
    TOKENS.replace(CALLER_WRITE, "")
}

/// [`TOKENS`] with every other token of the revoked one's owner revoked beside it (`affects:`).
fn affecting() -> String {
    TOKENS
        .replace(
            "      - {name: user_id, type: keys.tokens.UserId}
    outcomes:
      - name: revoked
        moves: keys.tokens.Token.revoke
        instances: {where: owner == input.user_id}",
            "      - {name: token_id, type: keys.tokens.TokenId}
    outcomes:
      - name: revoked
        moves: keys.tokens.Token.revoke
        instance: token_id
        affects:
          - entity: keys.tokens.Token
            where: owner == subject.owner
            moves: keys.tokens.Token.revoke",
        )
        .replace(
            "keys.tokens.OwnedRevoked: {user_id: input.user_id, count: {count: changed}}",
            "keys.tokens.OwnedRevoked: {user_id: {subject: owner}, count: \"1\"}",
        )
}

/// [`TOKENS`] with a token revoked only while its owner holds another one (`when_related:`).
fn related() -> String {
    TOKENS
        .replace(
            "      - {name: user_id, type: keys.tokens.UserId}
    outcomes:
      - name: revoked
        moves: keys.tokens.Token.revoke
        instances: {where: owner == input.user_id}",
            "      - {name: token_id, type: keys.tokens.TokenId}
      - {name: user_id, type: keys.tokens.UserId}
    outcomes:
      - name: unknown-token
        unknown_instance: true
        error: keys.tokens.UnknownToken
      - name: already-revoked
        wrong_state: true
        error: keys.tokens.AlreadyRevoked
      - name: last-token
        when_related:
          entity: keys.tokens.Token
          where: {all: [owner == input.user_id, state == Active]}
          count: {lt: 2}
        error: keys.tokens.LastToken
      - name: revoked
        moves: keys.tokens.Token.revoke
        instance: token_id",
        )
        .replace(
            "keys.tokens.OwnedRevoked: {user_id: input.user_id, count: {count: changed}}",
            "keys.tokens.OwnedRevoked: {user_id: input.user_id, count: \"1\"}",
        )
        .replace(
            "events:\n",
            "errors:
  - {name: keys.tokens.LastToken, summary: The owner holds no other token., fields: []}
  - {name: keys.tokens.UnknownToken, summary: No token carries the identity., fields: []}
  - {name: keys.tokens.AlreadyRevoked, summary: The token is revoked already., fields: []}
events:\n",
        )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tokens.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn rendered(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn has(synthesis: &Synthesis, id: &str) -> bool {
    synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

/// One command step: the command, the caller it is sent as and its input.
type Send = (
    String,
    BTreeMap<String, Node>,
    BTreeMap<String, ScenarioValue>,
);

/// Every command step, in order.
fn sends(scenario: &ConformanceScenario) -> Vec<Send> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command,
                caller,
                input,
                ..
            } => Some((command.to_string(), caller.clone(), input.clone())),
            _ => None,
        })
        .collect()
}

/// Every scenario of `text`'s suite run against the interpreter of `model`, by id and status.
fn run(suite: &ConformanceSuite, model: &str) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir(model));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn all_pass(results: &[(String, Status)]) -> bool {
    !results.is_empty() && results.iter().all(|(_, status)| *status == Status::Passed)
}

// ---- 1. rows arranged by callers chosen for them -----------------------------------------------

#[test]
fn a_set_effect_over_a_caller_written_field_is_witnessed() {
    let synthesis = synthesize(&ir(TOKENS));
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", rendered(&synthesis));
    let revoked = scenario(&synthesis.suite, REVOKED);
    let sent = sends(revoked);
    let (_, _, input) = sent
        .iter()
        .find(|(command, ..)| command == "keys.tokens.RevokeOwned")
        .expect("the branch is sent");
    let Some(ScenarioValue::Literal { value: selected }) = input.get("user_id") else {
        panic!("the selecting input is a literal: {input:?}");
    };
    let owners: Vec<&Node> = sent
        .iter()
        .filter(|(command, ..)| command == "keys.tokens.IssueToken")
        .map(|(_, caller, input)| {
            assert!(
                input.is_empty(),
                "no stand-in input reaches the suite: {input:?}"
            );
            caller
                .get("user_id")
                .expect("each issuer carries a user_id")
        })
        .collect();
    let matching = owners.iter().filter(|owner| **owner == selected).count();
    assert!(matching >= 3, "three rows the filter selects: {owners:?}");
    assert!(
        owners.len() > matching,
        "and one it does not select: {owners:?}"
    );
    let text = serde_json::to_string(&synthesis.suite).unwrap();
    assert!(!text.contains("caller_attribute_"), "{text}");
}

#[test]
fn the_suite_passes_the_interpreter_and_fails_a_target_that_ignores_the_caller() {
    let suite = synthesize(&ir(TOKENS)).suite;
    let results = run(&suite, TOKENS);
    assert!(all_pass(&results), "{results:#?}");
    // A target that stores no owner selects no row by it.
    let ignoring = run(&suite, &unwritten());
    assert!(
        ignoring
            .iter()
            .any(|(id, status)| id == REVOKED && *status != Status::Passed),
        "{ignoring:#?}"
    );
}

/// A model's own name that merely starts like a lifted input is not one: the scenario naming it is
/// still taken (review 2026-10-19, B11).
#[test]
fn a_field_named_like_a_lifted_input_keeps_its_scenario() {
    let model = TOKENS
        .replace(
            "{name: count, type: Integer}",
            "{name: caller_attribute_count, type: Integer}",
        )
        .replace(
            "count: {count: changed}",
            "caller_attribute_count: {count: changed}",
        );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", rendered(&synthesis));
    assert!(has(&synthesis, REVOKED), "{:#?}", rendered(&synthesis));
    let results = run(&synthesis.suite, &model);
    assert!(all_pass(&results), "{results:#?}");
}

#[test]
fn an_affects_entry_over_a_caller_written_field_is_witnessed() {
    let model = affecting();
    let synthesis = synthesize(&ir(&model));
    assert!(has(&synthesis, REVOKED), "{:#?}", rendered(&synthesis));
    assert!(
        !rendered(&synthesis)
            .iter()
            .any(|refusal| refusal.contains(REVOKED)),
        "{:#?}",
        rendered(&synthesis)
    );
    let results = run(&synthesis.suite, &model);
    assert!(all_pass(&results), "{results:#?}");
}

#[test]
fn a_related_row_set_over_a_caller_written_field_is_witnessed() {
    let model = related();
    let synthesis = synthesize(&ir(&model));
    for id in [
        REVOKED,
        "keys.tokens.RevokeOwned/outcome/last-token",
        "keys.tokens.Token/transition/revoke/by/keys.tokens.RevokeOwned/revoked",
    ] {
        assert!(has(&synthesis, id), "{id}: {:#?}", rendered(&synthesis));
    }
    let results = run(&synthesis.suite, &model);
    assert!(all_pass(&results), "{results:#?}");
}

#[test]
fn a_served_command_witnessed_on_such_rows_owes_its_grant_refusal() {
    let model = format!(
        "{TOKENS}components:
  - component: token-service
    summary: Issues and revokes tokens.
    owns:
      domains: [keys.tokens]
    accepts:
      commands: [keys.tokens.IssueToken, keys.tokens.RevokeOwned]
    publishes:
      events: [keys.tokens.TokenIssued, keys.tokens.OwnedRevoked]
    reached_by: network
"
    );
    let synthesis = synthesize(&ir(&model));
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", rendered(&synthesis));
    assert!(has(&synthesis, "keys.tokens.RevokeOwned/grant/denied"));
    let results = run(&synthesis.suite, &model);
    assert!(all_pass(&results), "{results:#?}");
}

/// The owner sent as an input reads no caller, so nothing is lifted: its suite is the one ESS
/// 0.62.0 wrote for it, byte for byte.
#[test]
fn a_model_without_a_caller_written_field_keeps_its_bytes() {
    let suite = synthesize(&ir(&from_input())).suite;
    let json = suite.to_canonical_json().unwrap();
    let digest = Sha256::digest(json.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            use std::fmt::Write as _;
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    assert_eq!(digest, INPUT_SUITE_DIGEST);
}

const INPUT_SUITE_DIGEST: &str = "4aa1151b9f0782dda76055c353807bef954d8974dda366623c1bbb81e25dd8a4";

// ---- 2. a filter over a field no command writes ------------------------------------------------

#[test]
fn a_filter_over_an_unwritten_field_names_the_field_and_what_to_declare() {
    let synthesis = synthesize(&ir(&unwritten()));
    let refusals = rendered(&synthesis);
    let refusal = refusals
        .iter()
        .find(|refusal| refusal.contains(REVOKED))
        .unwrap_or_else(|| panic!("{refusals:#?}"));
    assert!(
        refusal.contains("`keys.tokens.Token.owner`")
            && refusal.contains("no declared command writes"),
        "{refusal}"
    );
    let help = synthesis
        .refusals
        .iter()
        .find(|refusal| refusal.to_string().contains(REVOKED))
        .unwrap()
        .cause
        .hint();
    assert!(help.contains("declare a `sets:`"), "{help}");
    assert!(!help.contains("drop it from the command's input"), "{help}");
}

#[test]
fn a_filter_no_arrangement_satisfies_is_not_told_to_drop_an_input() {
    // The tier is written, but from a literal: every row holds `basic`, none the `premium` the
    // filter selects.
    let model = TOKENS
        .replace(
            "      - {name: owner, type: keys.tokens.UserId}\n    lifecycle:",
            "      - {name: owner, type: keys.tokens.UserId}\n      - {name: tier, type: String}\n    lifecycle:",
        )
        .replace(
            CALLER_WRITE,
            "        sets: {owner: {caller: user_id}, tier: basic}\n",
        )
        .replace(
            "instances: {where: owner == input.user_id}",
            "instances: {where: {all: [owner == input.user_id, tier == \"premium\"]}}",
        );
    let synthesis = synthesize(&ir(&model));
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| refusal.to_string().contains(REVOKED))
        .unwrap_or_else(|| panic!("{:#?}", rendered(&synthesis)));
    let help = refusal.cause.hint();
    assert!(
        help.contains("from an input or from a caller attribute"),
        "{help}"
    );
    assert!(!help.contains("drop it from the command's input"), "{help}");
}

/// A `when_related:` row set whose rows no arrangement gives the branch is told what the filter
/// needs written, not to drop an input (review 2026-10-19, B1): its reason is the row-set one.
#[test]
fn a_related_row_set_no_arrangement_satisfies_is_not_told_to_drop_an_input() {
    let model = related()
        .replace(
            "      - {name: owner, type: keys.tokens.UserId}\n    lifecycle:",
            "      - {name: owner, type: keys.tokens.UserId}\n      - {name: tier, type: String}\n    lifecycle:",
        )
        .replace(
            CALLER_WRITE,
            "        sets: {owner: {caller: user_id}, tier: basic}\n",
        )
        .replace(
            "where: {all: [owner == input.user_id, state == Active]}\n          count: {lt: 2}",
            "where: {all: [owner == input.user_id, state == Active, tier == \"premium\"]}\n          exists: true",
        );
    let synthesis = synthesize(&ir(&model));
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .to_string()
                .contains("on which the rows its selector selects take this branch")
        })
        .unwrap_or_else(|| panic!("{:#?}", rendered(&synthesis)));
    let help = refusal.cause.hint();
    assert!(
        help.contains("from an input or from a caller attribute"),
        "{help}"
    );
    assert!(!help.contains("drop it from the command's input"), "{help}");
}

// ---- 3. seeds and caller attributes -------------------------------------------------------------

/// A seed row holding a field its creator sets from the caller implies that caller: the seeded
/// reading sends the creator as a holder carrying the row's `owner` as its `user_id`, so the row
/// reads as one that holder made.
#[test]
fn a_seed_for_a_caller_attribute_model_is_sent_as_the_caller_its_row_implies() {
    let model = ir(TOKENS);
    let source = authored::Source::new(
        "held.yaml",
        "type: ess-scenario/2
domain: keys.tokens
scenario: held-token
summary: One token a holder owns.
arrange:
  - instance: held
    entity: keys.tokens.Token
    setup:
      identity: 00000000-0000-4000-8000-00000000b001
      fields: {owner: 00000000-0000-4000-8000-00000000b002}
      state: Active
assert:
  - view: keys.tokens.Tokens
    contains: {token_id: {$instance: held}, owner: 00000000-0000-4000-8000-00000000b002}
",
    );
    let seeds = AdmittedSeeds::compile(
        &model,
        &[SeedSelection {
            source,
            instance: InstanceName::new("held").unwrap(),
        }],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let synthesis = synthesize_with_seeds(&model, &seeds).unwrap_or_else(|error| panic!("{error}"));
    let owner = Node::Text("00000000-0000-4000-8000-00000000b002".to_owned());
    let issued_as_owner = synthesis.suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, caller, .. }
                if command.to_string() == "keys.tokens.IssueToken"
                    && caller.get("user_id") == Some(&owner))
        })
    });
    assert!(issued_as_owner, "{:#?}", synthesis.suite.scenarios.keys());
}
