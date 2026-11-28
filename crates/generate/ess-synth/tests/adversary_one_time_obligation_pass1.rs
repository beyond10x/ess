//! Adversary pass 1 on the Rust target's one-time response obligation (E1): an outcome marked
//! `one_time_response:` no longer refuses the Rust workspace; the command behaviour's existing
//! obligation carries one clause per marked outcome; Go, web and the command-line target refuse
//! with the text they refused with.
//!
//! Nothing here builds with cargo.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisPlan, Target, TargetFailureCode};

/// The refusal text Go, web and the command-line target gave before this unit, word for word.
const REFUSAL: &str = "one_time_response requires implementation-owned atomic durable \
                       consumption and fresh issuance; this code target cannot implement that \
                       policy";

/// One command, two marked outcomes: `rotated` discloses `token` only, `issued` discloses
/// `secret` and `token`, declared in that order.
const TWO_MARKED: &str = "format: ess/22\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    input: [{name: holder, type: String}]\n    response: [{name: secret, type: String}, {name: token, type: String}]\n    outcomes:\n      - {name: rotated, when: holder.utf8_bytes == 0, returns: true, one_time_response: [token]}\n      - {name: issued, returns: true, one_time_response: [secret, token]}\n";

/// A marked command invoked by a binding whose `on_failure:` selects a policy per refusal (E2).
const BOUND: &str = r"format: ess/22
system: credentials
version: v1
domain: credentials.api
types:
  - {name: credentials.api.HolderId, kind: newtype, of: String}
errors:
  - {name: credentials.api.Unavailable, fields: []}
  - {name: credentials.api.Denied, fields: []}
events:
  - name: credentials.api.Requested
    fields: [{name: holder, type: credentials.api.HolderId}]
  - name: credentials.api.IssueEscalated
    fields: [{name: holder, type: credentials.api.HolderId}]
commands:
  - name: credentials.api.Request
    input: [{name: holder, type: credentials.api.HolderId}]
    outcomes:
      - name: requested
        emits: [credentials.api.Requested]
        payload:
          credentials.api.Requested: {holder: input.holder}
  - name: credentials.api.Issue
    input: [{name: holder, type: credentials.api.HolderId}]
    response: [{name: secret, type: String}]
    outcomes:
      - {name: unavailable, external: the vault answers a server error, error: credentials.api.Unavailable}
      - {name: denied, external: the vault refuses the holder, error: credentials.api.Denied}
      - {name: issued, returns: true, one_time_response: [secret]}
components:
  - component: vault-service
    owns:
      domains: [credentials.api]
    accepts:
      commands: [credentials.api.Request, credentials.api.Issue]
    publishes:
      events: [credentials.api.Requested, credentials.api.IssueEscalated]
    reached_by: network
bindings:
  - id: issue-on-request
    when:
      event: credentials.api.Requested
    invoke:
      command: credentials.api.Issue
    mapping:
      holder: event.holder
    delivery: at_least_once
    on_failure:
      drop: [denied]
      retry: {outcomes: [unavailable], attempts: 3}
      escalate:
        emits: credentials.api.IssueEscalated
        except: [denied, unavailable]
";

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("policy.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// The declared outcome branches of `command`'s behaviour contract, keyed by outcome.
fn branches(plan: &SynthesisPlan, command: &str) -> Vec<(String, String)> {
    let contract = &plan
        .obligation_of(CapabilityKind::CommandBehavior, command)
        .unwrap_or_else(|| panic!("the behaviour of `{command}` is no obligation"))
        .contract;
    let (_, declared) = contract
        .split_once("Declared outcomes (declaration order, not selection precedence): ")
        .expect("a behaviour contract lists its declared outcomes");
    declared
        .split("; ")
        .map(|branch| {
            let name = branch[1..]
                .split_once('`')
                .expect("a branch opens with its outcome")
                .0
                .to_owned();
            (name, branch.to_owned())
        })
        .collect()
}

#[test]
fn every_marked_outcome_of_one_command_names_its_own_fields() {
    let model = ir(TWO_MARKED);
    let plan = SynthesisPlan::of(&model);
    let outcomes = branches(&plan, "credentials.api.Issue");
    let rotated = &outcomes
        .iter()
        .find(|(name, _)| name == "rotated")
        .expect("rotated")
        .1;
    let issued = &outcomes
        .iter()
        .find(|(name, _)| name == "issued")
        .expect("issued")
        .1;
    assert!(
        rotated.contains("discloses `token` once (one_time_response)"),
        "{rotated}"
    );
    assert!(!rotated.contains("`secret`"), "{rotated}");
    assert!(
        issued.contains("discloses `secret`, `token` once (one_time_response)"),
        "{issued}"
    );
    synthesize_for(&model, Target::Rust)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    let failure = synthesize_for(&model, Target::Go)
        .err()
        .expect("Go refuses both marked outcomes");
    let sources: Vec<&str> = failure
        .causes()
        .iter()
        .flat_map(|cause| cause.sources().iter().map(String::as_str))
        .collect();
    assert_eq!(
        sources,
        [
            "commands.credentials.api.Issue.outcomes.issued.one_time_response",
            "commands.credentials.api.Issue.outcomes.rotated.one_time_response",
        ]
    );
}

#[test]
fn a_marked_command_behind_a_per_refusal_binding_synthesizes_for_rust_only() {
    let model = ir(BOUND);
    let plan = SynthesisPlan::of(&model);
    assert_eq!(plan.counts().refused, 0, "{}", plan.to_canonical_json());
    synthesize_for(&model, Target::Rust)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    ess_synth::rust::workspace(&model, &plan)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    ess_synth::rust::single_crate(&model, &plan)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    let failure = synthesize_for(&model, Target::Go)
        .err()
        .expect("Go refuses the marked outcome");
    assert!(
        failure.causes().iter().any(|cause| cause
            .sources()
            .iter()
            .any(|source| source
                == "commands.credentials.api.Issue.outcomes.issued.one_time_response")),
        "{}",
        failure.to_canonical_json()
    );
}

/// The design page this unit extended, read as the contract it states.
const DESIGN: &str = include_str!("../../../../docs/design/one-time-response-values.md");

/// `one_time_response.rs`'s `SIBLINGS`: the marked `Issue` beside an unmarked `Rotate` with a typed
/// response.
const SIBLINGS: &str = "format: ess/22\nsystem: credentials\nversion: v1\ndomain: credentials.api\nerrors:\n  - name: credentials.api.Denied\ncommands:\n  - name: credentials.api.Issue\n    input: [{name: holder, type: String}]\n    response: [{name: secret, type: String}]\n    outcomes:\n      - {name: issued, returns: true, one_time_response: [secret]}\n      - {name: denied, when: holder.utf8_bytes == 0, error: credentials.api.Denied}\n  - name: credentials.api.Rotate\n    response: [{name: secret, type: String}]\n    outcomes: [{name: rotated, returns: true}]\n";

/// The page says of the synthesized Rust workspace: "The generated stub returns `UnmetObligation`,
/// never the value, and every other capability of the model is generated." Held against the
/// unit's own `SIBLINGS` model: the only capability that may be owed is the behaviour of a command
/// with a marked outcome.
#[test]
fn the_design_pages_every_other_capability_is_generated_holds_for_siblings() {
    let flat = DESIGN.split_whitespace().collect::<Vec<_>>().join(" ");
    let claim = "never the value, and every other capability of the model is generated";
    if !flat.contains(claim) {
        return;
    }
    let model = ir(SIBLINGS);
    let plan = SynthesisPlan::of(&model);
    let json: serde_json::Value = serde_json::from_str(&plan.to_canonical_json()).unwrap();
    let owed: Vec<String> = json["capabilities"]
        .as_array()
        .expect("the plan lists its capabilities")
        .iter()
        .filter(|planned| planned["disposition"]["disposition"] != "generated")
        .map(|planned| format!("{} {}", planned["kind"], planned["source"]))
        .collect();
    assert_eq!(
        owed,
        ["\"command_behavior\" \"credentials.api.Issue\""],
        "the design page claims \"{claim}\""
    );
}

/// `one_time_response.rs`'s `named` checks the refusal by `to_canonical_json()` containing
/// "atomic durable consumption". The refusal document embeds the plan, and the plan's contract
/// now carries that phrase for every target, so that check holds whatever the cause's detail
/// says. This case pins the detail itself, for every target that still refuses, and shows the
/// embedded plan alone satisfies the old check.
#[test]
fn go_web_and_clap_refuse_with_the_unchanged_detail() {
    let model = ir(TWO_MARKED);
    let plan = SynthesisPlan::of(&model);
    assert!(
        plan.to_canonical_json()
            .contains("atomic durable consumption"),
        "the plan alone carries the phrase `named` looks for"
    );
    for target in [Target::Go, Target::Web, Target::Clap] {
        let failure = synthesize_for(&model, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses"));
        assert!(!failure.causes().is_empty());
        for cause in failure.causes() {
            assert_eq!(cause.code(), TargetFailureCode::MissingRepresentation);
            assert_eq!(cause.detail(), REFUSAL, "{target:?}");
        }
    }
    for (target, failure) in [
        ("go", ess_synth::go::workspace(&model, &plan).err()),
        ("web", ess_synth::web::workspace(&model, &plan).err()),
        ("clap", ess_synth::clap::workspace(&model, &plan).err()),
    ] {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses"));
        for cause in failure.causes() {
            assert_eq!(cause.detail(), REFUSAL, "{target}");
        }
    }
}
