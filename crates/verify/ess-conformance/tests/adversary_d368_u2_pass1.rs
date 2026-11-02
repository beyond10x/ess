//! Adversary pass 1 of wave D368 U2 (story:a-no-view-probe-tells-from-states-apart and
//! story:a-no-view-held-state-refusal-probes-the-record): the no-view probe's search and its two
//! withdrawal causes.
//!
//! * Where every send has a valid input but none tells any state apart, the withdrawal must name
//!   the states it cannot tell apart, not claim that no send has a valid input.
//! * Where a sequence of sends tells every state apart, the scenario is kept, however many sends
//!   that sequence needs within a small lifecycle.
//! * Another command whose input carries a plain text field beside the record's identity still
//!   names no other record, so its wrong-state answer tells states apart.
use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::synthesize::Synthesis;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&compiled(text))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn filed(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

/// The commands the filed scenario `id` sends, in order.
fn sent(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| {
            scenario
                .steps
                .iter()
                .filter_map(|step| match step {
                    ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

// ---- A1: every send valid, none tells a state apart -----------------------------------------

/// A lamp lit from either state: `Light` runs from `Dark` and `Lit` and lands in `Lit`, so its
/// valid send answers `lit` in both; `Dim` lands in `Dark` from both, so it tells nothing apart
/// either. A target that lights the lamp on a refused short code is not seen; the scenario must be
/// withdrawn naming the two states. `Light.lit` and `Dim.dimmed` have valid sends.
const LAMP: &str = r"format: ess/18
system: demo
version: v1
domain: demo.lamp
summary: A lamp is installed dark and lit with a code from either state.
entities:
  - name: demo.lamp.Lamp
    identity: {name: lamp_id, type: String}
    fields: []
    lifecycle:
      initial: Dark
      states: [Dark, Lit]
      transitions:
        - {name: light, from: [Dark, Lit], to: Lit}
        - {name: dim, from: [Dark, Lit], to: Dark}
actors:
  - name: demo.lamp.Keeper
    may: [demo.lamp.Install, demo.lamp.Light, demo.lamp.Dim]
errors:
  - {name: demo.lamp.ShortCode, fields: []}
events:
  - name: demo.lamp.Installed
    fields: [{name: lamp_id, type: String}]
  - name: demo.lamp.Lighted
    fields: [{name: lamp_id, type: String}]
  - name: demo.lamp.Dimmed
    fields: [{name: lamp_id, type: String}]
commands:
  - name: demo.lamp.Install
    input: []
    outcomes:
      - name: installed
        creates: demo.lamp.Lamp
        instance: lamp_id
        emits: [demo.lamp.Installed]
        payload: {demo.lamp.Installed: {lamp_id: {generated: true}}}
  - name: demo.lamp.Light
    input:
      - {name: id, type: String}
      - {name: code, type: String}
    outcomes:
      - {name: short-code, when: code.count < 4, error: demo.lamp.ShortCode}
      - name: lit
        moves: demo.lamp.Lamp.light
        instance: id
        emits: [demo.lamp.Lighted]
        payload: {demo.lamp.Lighted: {lamp_id: input.id}}
  - name: demo.lamp.Dim
    input:
      - {name: id, type: String}
    outcomes:
      - name: dimmed
        moves: demo.lamp.Lamp.dim
        instance: id
        emits: [demo.lamp.Dimmed]
        payload: {demo.lamp.Dimmed: {lamp_id: input.id}}
";

const SHORT_CODE: &str = "demo.lamp.Light/outcome/short-code";

/// Acceptance of story:a-no-view-probe-tells-from-states-apart: "or the scenario is refused with a
/// cause naming the states it cannot tell apart". `Light.lit` has a valid send that answers alike
/// in `Dark` and `Lit`, so the cause must name `Dark, Lit`, never claim that no send is valid.
#[test]
fn a1_withdrawal_where_every_send_is_valid_but_alike_names_the_states() {
    let result = synthesis(LAMP);
    let lines = refusals(&result);
    assert!(
        !filed(&result, SHORT_CODE),
        "{SHORT_CODE} filed although nothing tells `Dark` from `Lit` with no view: {lines:#?}"
    );
    let line = lines
        .iter()
        .find(|line| line.contains(SHORT_CODE))
        .unwrap_or_else(|| panic!("{SHORT_CODE} neither filed nor refused: {lines:#?}"));
    assert!(
        line.contains("Dark, Lit") && !line.contains("has no valid send"),
        "{SHORT_CODE} withdrawn with a cause that names no state and says no send is valid, \
         although `Light.lit` has a valid send that answers alike in `Dark` and `Lit` \
         (story:a-no-view-probe-tells-from-states-apart, acceptance 1): {line}"
    );
}

// ---- A2: a sequence exists, the search gives up --------------------------------------------

/// A gate at `Home` with `n` side states. `Settle` (the refused command) lands in `Home` from
/// every state, `Go<i>` lands in `S<i>` from every state, so neither tells a state apart.
/// `Check<i>` answers `on` everywhere but in `S<i>`, where it answers `off`, without moving the
/// gate: the sequence `Check1 .. Check<n>` tells `Home` apart from every side state.
fn gate(n: usize) -> String {
    let sides: Vec<String> = (1..=n).map(|i| format!("S{i}")).collect();
    let all = std::iter::once("Home".to_owned())
        .chain(sides.iter().cloned())
        .collect::<Vec<_>>()
        .join(", ");
    let mut text = String::new();
    text.push_str(
        "format: ess/18\nsystem: demo\nversion: v1\ndomain: demo.gate\n\
         summary: A gate is checked by one command per side state.\nentities:\n  \
         - name: demo.gate.Gate\n    identity: {name: gate_id, type: String}\n    fields: []\n    \
         lifecycle:\n      initial: Home\n",
    );
    write!(text, "      states: [{all}]\n      transitions:\n")
        .expect("a String takes every write");
    writeln!(text, "        - {{name: settle, from: [{all}], to: Home}}")
        .expect("a String takes every write");
    for i in 1..=n {
        writeln!(text, "        - {{name: go{i}, from: [{all}], to: S{i}}}")
            .expect("a String takes every write");
    }
    let mut commands = vec![
        "demo.gate.Install".to_owned(),
        "demo.gate.Settle".to_owned(),
    ];
    for i in 1..=n {
        commands.push(format!("demo.gate.Go{i}"));
        commands.push(format!("demo.gate.Check{i}"));
    }
    write!(
        text,
        "actors:\n  - name: demo.gate.Keeper\n    may: [{}]\n",
        commands.join(", ")
    )
    .expect("a String takes every write");
    text.push_str(
        "errors:\n  - {name: demo.gate.ShortCode, fields: []}\n  \
         - {name: demo.gate.Off, fields: []}\nevents:\n",
    );
    for event in ["Installed", "Settled", "Went", "Checked"] {
        write!(
            text,
            "  - name: demo.gate.{event}\n    fields: [{{name: gate_id, type: String}}]\n"
        )
        .expect("a String takes every write");
    }
    text.push_str(
        "commands:\n  - name: demo.gate.Install\n    input: []\n    outcomes:\n      \
         - name: installed\n        creates: demo.gate.Gate\n        instance: gate_id\n        \
         emits: [demo.gate.Installed]\n        \
         payload: {demo.gate.Installed: {gate_id: {generated: true}}}\n  \
         - name: demo.gate.Settle\n    input:\n      - {name: id, type: String}\n      \
         - {name: code, type: String}\n    outcomes:\n      \
         - {name: short-code, when: code.count < 4, error: demo.gate.ShortCode}\n      \
         - name: settled\n        moves: demo.gate.Gate.settle\n        instance: id\n        \
         emits: [demo.gate.Settled]\n        \
         payload: {demo.gate.Settled: {gate_id: input.id}}\n",
    );
    for i in 1..=n {
        let on = std::iter::once("Home".to_owned())
            .chain(
                sides
                    .iter()
                    .filter(|side| **side != format!("S{i}"))
                    .cloned(),
            )
            .collect::<Vec<_>>()
            .join(", ");
        write!(text,
            "  - name: demo.gate.Go{i}\n    input:\n      - {{name: id, type: String}}\n    \
             outcomes:\n      - name: went\n        moves: demo.gate.Gate.go{i}\n        \
             instance: id\n        emits: [demo.gate.Went]\n        \
             payload: {{demo.gate.Went: {{gate_id: input.id}}}}\n  \
             - name: demo.gate.Check{i}\n    input:\n      - {{name: id, type: String}}\n    \
             outcomes:\n      - {{name: off, when_subject_state: [S{i}], error: demo.gate.Off}}\n      \
             - name: on\n        when_subject_state: [{on}]\n        updates: demo.gate.Gate\n        \
             instance: id\n        emits: [demo.gate.Checked]\n        \
             payload: {{demo.gate.Checked: {{gate_id: input.id}}}}\n"
        ).expect("a String takes every write");
    }
    text
}

const SETTLE_SHORT: &str = "demo.gate.Settle/outcome/short-code";

/// The control: with six side states the search finds `Check1 .. Check6` and keeps the scenario.
#[test]
fn a2_control_six_side_states_are_told_apart_and_the_scenario_kept() {
    let result = synthesis(&gate(6));
    let lines = refusals(&result);
    assert!(filed(&result, SETTLE_SHORT), "{lines:#?}");
    let checks = sent(&result, SETTLE_SHORT)
        .into_iter()
        .filter(|command| command.starts_with("demo.gate.Check"))
        .count();
    assert_eq!(checks, 6, "{:#?}", sent(&result, SETTLE_SHORT));
}

/// Acceptance of story:a-no-view-probe-tells-from-states-apart: the arranged half "sends a branch
/// that tells the arranged state apart from the others, or the scenario is refused with a cause
/// naming the states it cannot tell apart". With seven side states `Check1 .. Check7` still tells
/// every one apart, each side state by one send; the search stops after 64 partial sequences
/// (`MAX_PROBE_SEARCHES`) and withdraws the scenario naming states it can tell apart.
#[test]
fn a2_seven_side_states_each_told_apart_by_one_send_keep_the_scenario() {
    let result = synthesis(&gate(7));
    let lines = refusals(&result);
    assert!(
        filed(&result, SETTLE_SHORT),
        "{SETTLE_SHORT} withdrawn although `Check1 .. Check7` tells `Home` apart from every side \
         state, each `Check<i>` answering `off` only in `S<i>`; the cause names states a single \
         send tells apart (story:a-no-view-probe-tells-from-states-apart, acceptance 1): \
         {lines:#?}"
    );
}

// ---- A3: a plain text field beside the identity ---------------------------------------------

/// The staged account model of adversary pass 1 of wave D357 U3, `Stage` given a free-text
/// `reason` input. `Stage` still names no record but the account and is decided by the input and
/// the held state only, so it tells `New` from `Staged` as before.
const STAGED_WITH_REASON: &str = r#"format: ess/14
system: vault
version: v1
domain: vault.acct
summary: An account is onboarded, optionally staged with a reason, then configured once.
entities:
  - name: vault.acct.Account
    identity: {name: account_id, type: String}
    fields:
      - {name: issuer, type: String}
    lifecycle:
      initial: New
      states: [New, Staged, Configured]
      terminal: [Configured]
      transitions:
        - {name: stage, from: [New], to: Staged}
        - {name: configure, from: [New, Staged], to: Configured}
actors:
  - name: vault.acct.Operator
    may: [vault.acct.Onboard, vault.acct.Stage, vault.acct.Configure]
errors:
  - name: vault.acct.SecretTooShort
    fields: []
  - name: vault.acct.MissingConfiguration
    fields: []
  - name: vault.acct.IdRequired
    fields: []
  - name: vault.acct.AlreadyConfigured
    fields: []
  - name: vault.acct.NotNew
    fields: []
commands:
  - name: vault.acct.Onboard
    input:
      - {name: note, type: String}
    outcomes:
      - name: onboarded
        creates: vault.acct.Account
        instance: account_id
        emits: [vault.acct.Onboarded]
        payload:
          vault.acct.Onboarded: {account_id: {generated: true}}
  - name: vault.acct.Stage
    input:
      - {name: id, type: String}
      - {name: reason, type: String}
    outcomes:
      - {name: not-new, wrong_state: true, error: vault.acct.NotNew}
      - name: staged
        moves: vault.acct.Account.stage
        instance: id
        emits: [vault.acct.Staged]
        payload:
          vault.acct.Staged: {account_id: input.id}
  - name: vault.acct.Configure
    input:
      - {name: id, type: String}
      - {name: secret, type: String}
      - {name: issuer, type: String}
    outcomes:
      - {name: id-required, when: id == "", error: vault.acct.IdRequired}
      - {name: secret-too-short, when: secret.count < 12, error: vault.acct.SecretTooShort}
      - {name: missing-configuration, when: issuer == "", error: vault.acct.MissingConfiguration}
      - {name: already-configured, wrong_state: true, error: vault.acct.AlreadyConfigured}
      - name: configured
        moves: vault.acct.Account.configure
        instance: id
        sets: {issuer: input.issuer}
        emits: [vault.acct.Configured]
        payload:
          vault.acct.Configured: {account_id: input.id}
events:
  - name: vault.acct.Onboarded
    fields:
      - {name: account_id, type: String}
  - name: vault.acct.Staged
    fields:
      - {name: account_id, type: String}
  - name: vault.acct.Configured
    fields:
      - {name: account_id, type: String}
"#;

const SECRET_TOO_SHORT: &str = "vault.acct.Configure/outcome/secret-too-short";

/// The control: without `reason` the D357 model's arranged half sends `Stage` and is kept.
#[test]
fn a3_control_stage_without_a_text_field_tells_new_from_staged() {
    let text = STAGED_WITH_REASON.replace("      - {name: reason, type: String}\n", "");
    assert_ne!(text, STAGED_WITH_REASON);
    let result = synthesis(&text);
    assert!(filed(&result, SECRET_TOO_SHORT), "{:#?}", refusals(&result));
    assert!(
        sent(&result, SECRET_TOO_SHORT).contains(&"vault.acct.Stage".to_owned()),
        "{:#?}",
        sent(&result, SECRET_TOO_SHORT)
    );
}

/// The unit's claim: other commands qualify when "they name no other record". `Stage` with a
/// free-text `reason` names none, yet `reads_plainly` refuses it because the account's identity is
/// a plain `String` too, so the half is withdrawn naming `New, Staged` as states "no send
/// synthesis can make tells apart".
#[test]
fn a3_a_free_text_field_beside_the_identity_does_not_disqualify_stage() {
    let result = synthesis(STAGED_WITH_REASON);
    let lines = refusals(&result);
    assert!(
        filed(&result, SECRET_TOO_SHORT)
            && sent(&result, SECRET_TOO_SHORT).contains(&"vault.acct.Stage".to_owned()),
        "{SECRET_TOO_SHORT} not probed through `Stage`, whose `reason` is free text and names no \
         record; `Stage` answers `not-new` for a staged account and tells it from a new one \
         (story:a-no-view-probe-tells-from-states-apart): {lines:#?}"
    );
}

/// Fix round 1, the rest of A3's class: a plain `String` input still names a row where the model
/// uses it as one. Here `Stage` stores `reason` into `referrer`, the field carrying a `references`
/// relation to another account, so `Stage` may name a second record and is not sent; nothing else
/// tells `New` from `Staged`, so the half is withdrawn naming them.
#[test]
fn a3_a_text_field_stored_into_a_relation_still_disqualifies_stage() {
    let text = STAGED_WITH_REASON
        .replace(
            "      - {name: issuer, type: String}\n    lifecycle:",
            "      - {name: issuer, type: String}\n      - {name: referrer, type: String}\n    \
             relations:\n      - {name: referred_by, kind: references, target: \
             vault.acct.Account, cardinality: one, via: referrer}\n    lifecycle:",
        )
        .replace(
            "        moves: vault.acct.Account.stage\n        instance: id\n",
            "        moves: vault.acct.Account.stage\n        instance: id\n        \
             sets: {referrer: input.reason}\n",
        );
    assert_eq!(text.matches("referrer").count(), 3, "{text}");
    let result = synthesis(&text);
    let lines = refusals(&result);
    assert!(
        !filed(&result, SECRET_TOO_SHORT),
        "{SECRET_TOO_SHORT} filed although `Stage` stores `reason` into a relation, so its send \
         may name another record (story:a-no-view-probe-tells-from-states-apart): {lines:#?} {:#?}",
        sent(&result, SECRET_TOO_SHORT)
    );
    assert!(
        lines.iter().any(|line| line.contains(SECRET_TOO_SHORT)
            && line.contains("`New, Staged`")
            && line.contains("no send synthesis can make tells apart")),
        "{SECRET_TOO_SHORT} withdrawn without naming `New, Staged` \
         (story:a-no-view-probe-tells-from-states-apart): {lines:#?}"
    );
}
