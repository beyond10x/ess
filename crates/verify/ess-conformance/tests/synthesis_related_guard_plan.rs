//! The `when_related:` searches ask the precedence plan which branches answer first
//! (`story:synthesis-reads-selection-plan`, wave 5 unit 4, `docs/design/selection-plan.md`).
//!
//! `PublishRelease` (`ess/22`) reads a candidate through its input and declares an accepting
//! `published` (`speed == Fast`) before the predicate refusal `not-accepted` (`state !=
//! Accepted`), beside `wrong_state:`. Where a proposed candidate is sent fast, both hold. The
//! refusal is read at step 5, before every accepting branch (beyond10x/ess#282), so it answers, and
//! its witness is the plain one, the first variant, `Fast`. With the present-related and accepting
//! phases exchanged through the plan's one test seam (`with_phase_order`), `published` answers
//! first, and the witness moves to `Slow`.
//!
//! A related refusal's phase depends on the format, so these searches build their plan in the
//! model's format. Below `ess/22` no command that validates reads the refusal at step 5: validation
//! refuses `when_related:` beside `wrong_state:` there, and a refusal and an accepting branch whose
//! overlap it decides; where it cannot decide one (`redirect_client != input.client` beside
//! `client == fast`), the refusal is read after the earlier accepting branch, and the interpreter,
//! the synthesized suite and an authored claim agree on that.
//!
//! A command reading a related row with two overlapping input refusals answers the first declared
//! where both hold, so the second's witness is one the first does not claim. An external branch's
//! witness refutes the `when_related:` predicates the plan reads before it, and only those: one
//! declared after it among the accepting branches is left to it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Authoring, Source as Authored};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue, SuiteProvenance,
};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const PUBLISH: &str = "demo.release.PublishRelease";
const NOT_ACCEPTED: &str = "demo.release.PublishRelease/outcome/not-accepted";

/// `related-guard-release.yaml` in `format`, with a `speed` input, an accepting `published`
/// (`speed == Fast`) declared before `not-accepted`, the default `published-slowly`,
/// and `wrong_state:`.
fn release(format: &str) -> String {
    let text = include_str!("fixtures/related-guard-release.yaml")
        .replace("format: ess/20", &format!("format: {format}"))
        .replace(
            "  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}\n  - {name: demo.release.Speed, kind: enum, variants: [Fast, Slow]}\n",
        )
        .replace(
            "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
            "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
        )
        .replace(
            "      - {name: candidate, type: demo.release.CandidateId}\n    outcomes:\n",
            "      - {name: candidate, type: demo.release.CandidateId}\n      - {name: speed, type: demo.release.Speed}\n    outcomes:\n",
        )
        .replace(
            "      - name: published\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n",
            "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: published-slowly\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n",
        )
        .replace(
            "      - name: not-accepted\n",
            "      - name: published\n        when: speed == Fast\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n      - name: not-accepted\n",
        );
    for part in [
        "variants: [Fast, Slow]",
        "{name: speed, type: demo.release.Speed}",
        "wrong_state: true",
        "name: published-slowly",
        "when: speed == Fast",
    ] {
        assert!(text.contains(part), "the fixture moved: {part}");
    }
    text
}

const SIGN_IN: &str = "demo.signin.InitiateSignIn";
const ODD: &str = "demo.signin.InitiateSignIn/outcome/odd";
const NO_REDIRECT: &str = "demo.signin.InitiateSignIn/outcome/no-redirect-entry";

/// `related-guard-sign-in.yaml` in `format`, with an accepting `express` (`client == fast`)
/// declared before the predicate refusal `no-redirect-entry` (`redirect_client != input.client`).
fn sign_in_express(format: &str) -> String {
    let text = include_str!("fixtures/related-guard-sign-in.yaml")
        .replace("format: ess/18", &format!("format: {format}"))
        .replace(
            "events:\n",
            "events:\n  - {name: demo.signin.Expressed, fields: []}\n",
        )
        .replace(
            "      - name: no-redirect-entry\n",
            "      - {name: express, when: client == fast, emits: [demo.signin.Expressed]}\n      - name: no-redirect-entry\n",
        );
    assert!(text.contains("name: express"), "the fixture moved");
    text
}

/// `related-guard-sign-in.yaml` with a `speed` input and two overlapping input refusals declared
/// first: `rushed` (`speed != Slow`), then `odd` (`speed != Medium`). Both hold for `Fast`.
fn sign_in_refusals() -> String {
    let text = include_str!("fixtures/related-guard-sign-in.yaml")
        .replace(
            "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n  - {name: demo.signin.Speed, kind: enum, variants: [Fast, Medium, Slow]}\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - {name: demo.signin.Rushed, summary: Too fast., fields: []}\n  - {name: demo.signin.Odd, summary: Not medium., fields: []}\n",
        )
        .replace(
            "      - {name: client, type: demo.signin.ClientId}\n    outcomes:\n",
            "      - {name: client, type: demo.signin.ClientId}\n      - {name: speed, type: demo.signin.Speed}\n    outcomes:\n      - {name: rushed, when: speed != Slow, error: demo.signin.Rushed}\n      - {name: odd, when: speed != Medium, error: demo.signin.Odd}\n",
        );
    for part in [
        "variants: [Fast, Medium, Slow]",
        "name: rushed",
        "name: odd",
    ] {
        assert!(text.contains(part), "the fixture moved: {part}");
    }
    text
}

fn spec(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = spec(text).unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The precedence order with the present-related and accepting phases exchanged.
fn exchanged() -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| match phase {
        Phase::PresentRelated => Phase::Accepting,
        Phase::Accepting => Phase::PresentRelated,
        other => other,
    })
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                let refusals: Vec<String> =
                    result.refusals.iter().map(ToString::to_string).collect();
                panic!("no scenario {id}; refusals: {refusals:#?}")
            },
            |(_, scenario)| scenario,
        )
}

/// The value of `field` in the last send of `command` in `scenario`: the one its claim is about.
fn last_sent(scenario: &ConformanceScenario, command: &str, field: &str) -> ScenarioValue {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.name().to_string() == command => Some(input[field].clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no send of {command}: {scenario:#?}"))
}

fn variant(name: &str) -> ScenarioValue {
    ScenarioValue::literal(Node::Text(name.into()))
}

/// Every scenario of `suite` run on the model interpreter of `ir`, by id, where it did not pass,
/// with its diagnostics.
fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, (Status, Vec<String>)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            (result.scenario.to_string(), (result.status, diagnostics))
        })
        .collect();
    results
        .into_iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, diagnostics))| format!("{id}: {status:?} {diagnostics:?}"))
        .collect()
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Authored::new("scenario.yaml", text)])
}

fn refusals(authoring: &Authoring) -> Vec<String> {
    authoring
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.code()))
        .collect()
}

/// The authored `text` run on the interpreter of `model`: the scenarios that did not pass.
fn authored_on_interpreter(model: &EssIr, text: &str) -> Vec<String> {
    let compiled = authoring(model, text);
    assert!(compiled.is_complete(), "{:?}", refusals(&compiled));
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(model));
    suite.scenarios = compiled.scenarios;
    not_passed(model.clone(), &suite)
}

/// A proposed candidate and a drafted release, then the release published fast for that
/// candidate, claiming `claim`.
fn fast_publish(claim: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: demo.release\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         arrange:\n  - instance: c\n    entity: demo.release.Candidate\n  \
         - instance: r\n    entity: demo.release.Release\n\
         timeline:\n\
         \x20 - at: 2026-01-05T09:00:00Z\n    command: demo.release.ProposeCandidate\n    \
         input: {{}}\n    outcome: proposed\n    \
         events:\n      - event: demo.release.CandidateProposed\n    \
         capture: {{instance: c, event: demo.release.CandidateProposed, field: candidate_id}}\n\
         \x20 - at: 2026-01-05T09:00:01Z\n    command: demo.release.DraftRelease\n    \
         input: {{}}\n    outcome: drafted\n    \
         events:\n      - event: demo.release.ReleaseDrafted\n    \
         capture: {{instance: r, event: demo.release.ReleaseDrafted, field: release_id}}\n\
         \x20 - at: 2026-01-05T09:00:02Z\n    command: {PUBLISH}\n    \
         input: {{release_id: {{$instance: r}}, candidate: {{$instance: c}}, speed: Fast}}\n{claim}"
    )
}

const NOT_ACCEPTED_CLAIM: &str =
    "    outcome: not-accepted\n    error: {name: demo.release.CandidateNotAccepted}\n";

/// A tenant configured for client `web`, then a sign-in for client `fast`, claiming `claim`.
fn fast_sign_in(claim: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: demo.signin\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         arrange:\n  - instance: c\n    entity: demo.signin.Configuration\n\
         timeline:\n\
         \x20 - at: 2026-01-05T09:00:00Z\n    command: demo.signin.ConfigureTenant\n    \
         input: {{redirect_client: web}}\n    outcome: configured\n    \
         events:\n      - event: demo.signin.TenantConfigured\n    \
         capture: {{instance: c, event: demo.signin.TenantConfigured, field: tenant}}\n\
         \x20 - at: 2026-01-05T09:00:01Z\n    command: {SIGN_IN}\n    \
         input: {{tenant: {{$instance: c}}, client: fast}}\n{claim}"
    )
}

const EXPRESS: &str = "    outcome: express\n    events:\n      - event: demo.signin.Expressed\n";
const NO_REDIRECT_CLAIM: &str =
    "    outcome: no-redirect-entry\n    error: {name: demo.signin.NoRedirectEntry}\n";

#[test]
fn ess22_the_present_related_refusal_answers_before_the_earlier_accepting_when() {
    let model = ir(&release("ess/22"));
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(
        last_sent(scenario(&result, NOT_ACCEPTED), PUBLISH, "speed"),
        variant("Fast"),
        "ess/22 reads `not-accepted` at step 5, before `published`: its witness is the plain one"
    );
    assert_eq!(
        not_passed(model.clone(), &result.suite),
        Vec::<String>::new()
    );
    assert_eq!(
        authored_on_interpreter(&model, &fast_publish(NOT_ACCEPTED_CLAIM)),
        Vec::<String>::new(),
        "an authored act claiming `not-accepted` for a proposed candidate sent fast passes"
    );
}

#[test]
fn exchanging_present_related_and_accepting_moves_the_related_refusal_witness() {
    let model = ir(&release("ess/22"));
    let swapped = with_phase_order(exchanged(), || {
        ess_conformance::synthesize::synthesize(&model)
    });
    assert_eq!(
        last_sent(scenario(&swapped, NOT_ACCEPTED), PUBLISH, "speed"),
        variant("Slow"),
        "with the phases exchanged `published` answers before `not-accepted`, so its witness is \
         sent the way `published` does not claim"
    );
    let failed = with_phase_order(exchanged(), || not_passed(model.clone(), &swapped.suite));
    let publishing: Vec<&String> = failed
        .iter()
        .filter(|failure| failure.starts_with(PUBLISH))
        .collect();
    assert_eq!(
        publishing,
        Vec::<&String>::new(),
        "the interpreter, reading the same exchanged order, passes every `{PUBLISH}` scenario"
    );
}

#[test]
fn below_ess22_a_related_refusal_beside_wrong_state_does_not_validate() {
    let refused = spec(&release("ess/21")).expect_err("ess/21 refuses the composition");
    assert!(
        refused.contains("conflicting_declaration")
            && refused.contains(
                "selects on a related row (`when_related`) and on a `wrong_state` branch"
            ),
        "{refused}"
    );
}

#[test]
fn ess21_the_related_refusal_reads_after_the_earlier_accepting_when() {
    let model = ir(&sign_in_express("ess/21"));
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_ne!(
        last_sent(scenario(&result, NO_REDIRECT), SIGN_IN, "client"),
        ScenarioValue::literal(Node::Text("fast".into())),
        "`express` answers `fast` first, so `no-redirect-entry` is witnessed for another client"
    );
    assert_eq!(
        not_passed(model.clone(), &result.suite),
        Vec::<String>::new()
    );
    assert_eq!(
        authored_on_interpreter(&model, &fast_sign_in(EXPRESS)),
        Vec::<String>::new(),
        "the interpreter answers `express` for client `fast` on a row registering another"
    );
    let claimed = authoring(&model, &fast_sign_in(NO_REDIRECT_CLAIM));
    let found = refusals(&claimed);
    assert!(
        found.len() == 1
            && found[0].starts_with("ESS-AUTHOR-041")
            && found[0].contains("`express`"),
        "an act claiming `no-redirect-entry` there is refused, naming `express`: {found:?}"
    );
}

#[test]
fn the_first_declared_of_two_overlapping_input_refusals_answers() {
    let model = ir(&sign_in_refusals());
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(
        last_sent(scenario(&result, ODD), SIGN_IN, "speed"),
        variant("Slow"),
        "`rushed`, declared first, answers `Fast`: `odd` is witnessed where `rushed` does not hold"
    );
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

/// `external_beside_held_guard.rs`'s `RELATED`: `CheckPick` refuses a list no row carries (`no-list`)
/// and a list whose `revision` differs from the input's (`stale`), declared before the external
/// `unlisted`.
const DESK: &str = r"format: ess/20
system: demo
version: v1
domain: demo.desk
summary: A pick checked against its list's revision before it is accepted.
types:
  - {name: demo.desk.PickId, kind: newtype, of: Uuid}
  - {name: demo.desk.ListId, kind: newtype, of: Uuid}
entities:
  - name: demo.desk.List
    identity: {name: list_id, type: demo.desk.ListId}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.desk.Pick
    identity: {name: pick_id, type: demo.desk.PickId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Picked
      states: [Picked, Accepted, Refused]
      terminal: [Accepted, Refused]
      transitions:
        - {name: accept, from: [Picked], to: Accepted}
        - {name: refuse, from: [Picked], to: Refused}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.OpenList, demo.desk.MakePick, demo.desk.CheckPick]
commands:
  - name: demo.desk.OpenList
    input:
      - {name: revision, type: Integer}
    outcomes:
      - name: opened
        creates: demo.desk.List
        instance: list_id
        sets: {revision: input.revision}
        emits: [demo.desk.ListOpened]
        payload:
          demo.desk.ListOpened: {list_id: {generated: true}}
  - name: demo.desk.MakePick
    input:
      - {name: note, type: String}
    outcomes:
      - name: picked
        creates: demo.desk.Pick
        instance: pick_id
        sets: {note: input.note}
        emits: [demo.desk.Picked]
        payload:
          demo.desk.Picked: {pick_id: {generated: true}}
  - name: demo.desk.CheckPick
    input:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: list_id, type: demo.desk.ListId}
      - {name: revision, type: Integer}
    outcomes:
      - name: no-list
        when_related: {via: input.list_id, exists: false}
        error: demo.desk.NoList
      - name: stale
        when_related:
          via: input.list_id
          predicate: revision != input.revision
        error: demo.desk.Stale
      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
      - name: accepted
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
events:
  - name: demo.desk.ListOpened
    fields: [{name: list_id, type: demo.desk.ListId}]
  - name: demo.desk.Picked
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickUnlisted
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickAccepted
    fields: [{name: pick_id, type: demo.desk.PickId}]
errors:
  - name: demo.desk.NotPicked
  - name: demo.desk.NoList
  - name: demo.desk.Stale
views:
  - name: demo.desk.Picks
    source: demo.desk.Pick
    consistency: read_your_writes
    fields:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: note, type: String}
      - {name: state, type: demo.desk.Pick.State}
  - name: demo.desk.Lists
    source: demo.desk.List
    consistency: read_your_writes
    fields:
      - {name: list_id, type: demo.desk.ListId}
      - {name: revision, type: Integer}
";

const CHECK: &str = "demo.desk.CheckPick";
const UNLISTED: &str = "demo.desk.CheckPick/outcome/unlisted";

const STALE: &str = "      - name: stale
        when_related:
          via: input.list_id
          predicate: revision != input.revision
        error: demo.desk.Stale
";

/// [`DESK`] with `stale` declared after `unlisted`.
fn desk_stale_after() -> String {
    let text = DESK.replacen(STALE, "", 1).replacen(
        "      - name: accepted\n",
        &format!("{STALE}      - name: accepted\n"),
        1,
    );
    assert!(
        text.find("name: unlisted") < text.find("name: stale"),
        "the fixture moved"
    );
    text
}

/// The `revision` the list named by the forced `unlisted` send was opened with, and the one sent.
fn unlisted_against_list(scenario: &ConformanceScenario) -> (Option<ScenarioValue>, ScenarioValue) {
    let mut lists = BTreeMap::new();
    let mut opened = None;
    let mut forced = false;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                let name = command.name().to_string();
                if name == "demo.desk.OpenList" {
                    opened = input.get("revision").cloned();
                } else if name == CHECK && forced {
                    let held = match &input["list_id"] {
                        ScenarioValue::Instance { instance } => lists.get(instance).cloned(),
                        _ => None,
                    };
                    return (held, input["revision"].clone());
                } else {
                    opened = None;
                }
            }
            ScenarioStep::CaptureInstance { instance, .. } => {
                if let Some(revision) = opened.take() {
                    lists.insert(instance.clone(), revision);
                }
            }
            ScenarioStep::ConfigureExternalOutcome { force, .. } => {
                forced = force.command.name().to_string() == CHECK
                    && force.outcome.to_string() == "unlisted";
            }
            _ => {}
        }
    }
    panic!("no forced `unlisted` send: {scenario:#?}")
}

#[test]
fn a_related_refusal_declared_after_the_external_is_left_to_it() {
    // Declared before `unlisted`, `stale` is read first, so the witness names a list holding the
    // revision it is sent.
    let model = ir(DESK);
    let result = ess_conformance::synthesize::synthesize(&model);
    let (held, sent) = unlisted_against_list(scenario(&result, UNLISTED));
    assert_eq!(held, Some(sent), "`stale`, read first, is refuted");
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
    // Declared after it, `stale` is read after `unlisted` among the accepting branches: nothing
    // over the row answers first, and the witness is the plain one, which `stale` would claim.
    let model = ir(&desk_stale_after());
    let result = ess_conformance::synthesize::synthesize(&model);
    let (held, sent) = unlisted_against_list(scenario(&result, UNLISTED));
    assert!(
        held.is_some() && held != Some(sent),
        "`stale` is read after `unlisted`, so the witness need not refute it"
    );
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}
