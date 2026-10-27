//! Adversary pass 1 over ESS 0.71.0 wave 1 (U3, U4, U12), on the integrated branch.
//!
//! Every case drives the synthesizer on a model the new code reaches, then runs the suite on the
//! model interpreter. The property each case holds the synthesizer to is the one the units' own
//! tests hold it to: what it synthesizes passes the interpreter of the same model. Where a case
//! also pins that a scenario is synthesized, it says so.
//!
//! - U3 (`related_guard::present_in`): an `exists: false` branch passed over for a captured row
//!   whatever it answers, including one that itself creates the row the arrangement wants.
//! - U4 (`binding_effects::crosses_unchanged`): a conversion at the trigger's payload rather than
//!   the binding mapping, a chain of two, and a constrained target.
//! - U12 (`set_effects::{Link, related_rows, Selection::copied, changing}`): a second creator of the
//!   rows copying another field of the related row, an update refusing the related row it is sent
//!   naming, a filter over the copied field, an enum and an `Optional` written field.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const U3_CREATING: &str =
    include_str!("fixtures/adversary-0710/affects-past-creating-not-found.yaml");

const U4_SYSTEM: &str = include_str!("fixtures/binding-input-through-conversion/system.yaml");
const U4_COMPONENTS: &str =
    include_str!("fixtures/binding-input-through-conversion/components.yaml");
const U4_DOMAIN: &str = include_str!("fixtures/binding-input-through-conversion/x.yaml");

const U12_SECOND_CREATOR: &str =
    include_str!("fixtures/adversary-0710/set-update-second-creator-copies-another-field.yaml");
const U12_REFUSES_NAMED: &str =
    include_str!("fixtures/adversary-0710/set-update-refuses-the-related-row-it-names.yaml");
const U12_COPIES: &str = include_str!("fixtures/set-update-creator-copies-related-value.yaml");
const U12_CREATOR_INPUT: &str =
    include_str!("fixtures/set-update-written-input-against-creator-input.yaml");

const SYNCED: &str = "demo.z.SyncFlag/outcome/synced";
const RESET: &str = "demo.w.ResetMember/outcome/reset";
const BINDING: &str = "enrol-on-member-created";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ir_of(files: &[(&str, &str)]) -> EssIr {
    let parsed: Vec<_> = files
        .iter()
        .map(|(label, text)| {
            let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
            (Source::new(*label), raw)
        })
        .collect();
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `text` with `part`, which it holds exactly once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

struct Run {
    synthesis: Synthesis,
    statuses: BTreeMap<String, (Status, Vec<String>)>,
}

impl Run {
    fn refusals(&self) -> Vec<String> {
        self.synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    /// Every synthesized scenario that does not pass the interpreter of its own model, with the
    /// interpreter's diagnostics and the scenario's steps.
    fn failing(&self) -> Vec<String> {
        self.statuses
            .iter()
            .filter(|(_, (status, _))| *status != Status::Passed)
            .map(|(id, (status, diagnostics))| {
                let steps = self
                    .synthesis
                    .suite
                    .scenarios
                    .iter()
                    .find(|(scenario, _)| scenario.to_string() == *id)
                    .map(|(_, scenario)| serde_json::to_string(&scenario.steps).unwrap())
                    .unwrap_or_default();
                format!("{id} {status:?}: {diagnostics:#?}\n  steps: {steps}")
            })
            .collect()
    }

    fn has(&self, id: &str) -> bool {
        self.statuses.contains_key(id)
    }
}

fn run(model: &EssIr) -> Run {
    let synthesis = synthesize(model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result.diagnostics().map(|d| format!("{d:?}")).collect();
            (result.scenario.to_string(), (result.status, diagnostics))
        })
        .collect();
    Run {
        synthesis,
        statuses,
    }
}

/// Everything synthesized passes the interpreter of the same model.
fn assert_sound(run: &Run) {
    let failing = run.failing();
    assert!(
        failing.is_empty(),
        "the model interpreter fails what synthesis wrote:\n{}\nrefusals: {:#?}",
        failing.join("\n"),
        run.refusals()
    );
}

// ---------------------------------------------------------------------------------------------
// U3
// ---------------------------------------------------------------------------------------------

/// The creator's `exists: false` branch creates a ticket itself (a walk-in). A captured member
/// never takes it, so the ticket of `reset` must be arranged through `opened`, and the arranged
/// step must expect `TicketOpened`, not `WalkInOpened`.
#[test]
fn u3_an_exists_false_branch_that_creates_the_row_is_not_the_arrangement_s_creator() {
    let run = run(&ir(U3_CREATING));
    assert_sound(&run);
    assert!(
        run.has(RESET),
        "reset is synthesized: {:#?}",
        run.refusals()
    );
}

// ---------------------------------------------------------------------------------------------
// U4
// ---------------------------------------------------------------------------------------------

fn u4(domain: &str, components: &str) -> EssIr {
    ir_of(&[
        ("system.yaml", U4_SYSTEM),
        ("components.yaml", components),
        ("x.yaml", domain),
    ])
}

const MEMBER_CREATED: &str =
    "  - name: demo.x.MemberCreated\n    fields:\n      - {name: group_ref, type: demo.x.GroupRef}";

/// The conversion sits at the trigger's payload (`CreateMember`'s `GroupRef` input into a
/// `GroupId` event field) and the binding maps the field unconverted.
#[test]
fn u4_a_conversion_at_the_trigger_payload_is_followed_and_the_suite_passes() {
    let domain = replaced(
        U4_DOMAIN,
        MEMBER_CREATED,
        "  - name: demo.x.MemberCreated\n    fields:\n      - {name: group_ref, type: demo.x.GroupId}",
    );
    let run = run(&u4(&domain, U4_COMPONENTS));
    assert_sound(&run);
    assert_eq!(run.refusals(), Vec::<String>::new());
    assert!(run.has(&format!("{BINDING}/binding/flow")));
}

/// Two conversions on the way: `GroupRef -> GroupKey` at the payload, `GroupKey -> GroupId` at
/// the mapping, every type an unconstrained newtype over `String`.
#[test]
fn u4_a_chain_of_two_unchanged_conversions_is_followed_and_the_suite_passes() {
    let domain = replaced(
        U4_DOMAIN,
        "  - {name: demo.x.GroupRef, kind: newtype, of: String}",
        "  - {name: demo.x.GroupRef, kind: newtype, of: String}\n  - {name: demo.x.GroupKey, kind: newtype, of: String}",
    );
    let domain = replaced(
        &domain,
        MEMBER_CREATED,
        "  - name: demo.x.MemberCreated\n    fields:\n      - {name: group_ref, type: demo.x.GroupKey}",
    );
    let components = replaced(
        U4_COMPONENTS,
        "conversions:\n  - from: demo.x.GroupRef\n    to: demo.x.GroupId\n",
        "conversions:\n  - from: demo.x.GroupRef\n    to: demo.x.GroupKey\n    because: A reference is a key.\n  - from: demo.x.GroupKey\n    to: demo.x.GroupId\n",
    );
    let run = run(&u4(&domain, &components));
    assert_sound(&run);
    assert_eq!(run.refusals(), Vec::<String>::new());
    assert!(run.has(&format!("{BINDING}/binding/flow")));
}

fn u4_prefixed() -> EssIr {
    let domain = replaced(
        U4_DOMAIN,
        "{name: demo.x.GroupId, kind: newtype, of: String}",
        "{name: demo.x.GroupId, kind: newtype, of: String, prefix: \"g-\"}",
    );
    u4(&domain, U4_COMPONENTS)
}

/// The target newtype declares a prefix: what the conversion makes of a reference is the
/// implementation's, so the binding is refused and the refusal names the conversion.
#[test]
fn u4_a_conversion_into_a_prefixed_newtype_is_refused_naming_it() {
    let run = run(&u4_prefixed());
    let flow: Vec<String> = run
        .refusals()
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{BINDING}/binding/flow`")))
        .collect();
    assert!(
        flow.iter()
            .any(|refusal| refusal.contains("`demo.x.GroupRef` -> `demo.x.GroupId`")),
        "{flow:#?}"
    );
}

/// The same prefixed model: the scenarios synthesis does write for it (the trigger's own outcome
/// and `binding/mapping`) send `CreateMember` a `GroupRef` the binding cannot convert into a
/// `GroupId` starting `g-`, and the interpreter cannot carry them out.
///
/// The 0.70.0 binary gives the same, so this pins today's result: the two scenarios are
/// `Unsupported`. It goes red when the story below lands, and `assert_sound` replaces the pin.
#[test]
fn u4_the_prefixed_model_s_remaining_scenarios_pass_the_interpreter() {
    let run = run(&u4_prefixed());
    let failing: Vec<(&str, Status)> = run
        .statuses
        .iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, _))| (id.as_str(), *status))
        .collect();
    assert_eq!(
        failing,
        [
            ("demo.x.CreateMember/outcome/created", Status::Unsupported),
            (
                "enrol-on-member-created/binding/mapping",
                Status::Unsupported
            ),
        ],
        "story:a-scenario-beside-a-prefixed-conversion-target-runs-on-the-interpreter makes every \
         scenario synthesis writes for the prefixed model pass the interpreter, or refuses it \
         naming the conversion; once it lands, assert `assert_sound` here instead:\n{}",
        run.failing().join("\n")
    );
}

// ---------------------------------------------------------------------------------------------
// U12
// ---------------------------------------------------------------------------------------------

/// Two commands create children: `AddChild` (first) copies the parent's `flag`, `AddMirror`
/// copies the parent's `mirror_flag`, and only mirror children are selected. The parent holds
/// `flag: false, mirror_flag: true`, so a mirror child holds `true` already and `SyncFlag`
/// (`flag: true`) changes none of them. Whatever synthesis writes must pass the interpreter.
#[test]
fn u12_rows_created_by_a_second_creator_copy_the_field_that_creator_reads() {
    let run = run(&ir(U12_SECOND_CREATOR));
    assert_sound(&run);
}

/// The shelf opens `Locked`, parcels are added to any shelf, and `RelabelShelf` refuses a locked
/// shelf before it relabels. The relabel the `instances:` scenario sends names the shelf the rows
/// were created on, so that shelf must be unlocked first, or the branch refused.
#[test]
fn u12_the_update_is_not_sent_naming_a_related_row_it_refuses() {
    let run = run(&ir(U12_REFUSES_NAMED));
    assert_sound(&run);
}

/// The filter reads the copied field as well: `parent_id == input.parent_id, flag == false`. The
/// parent is created `flag: false`, so every child of it is selected and holds `false`, and the
/// update writes `true`: each selected row would change. What is missing is a child of the same
/// parent the `flag == false` conjunct leaves out, which no arrangement can make. The refusal may
/// stand, but it must not say every selected row already holds what the effect writes.
#[test]
fn u12_a_refusal_over_a_copied_filter_field_does_not_claim_the_rows_are_unchanged() {
    let model = replaced(
        U12_COPIES,
        "instances: {where: parent_id == input.parent_id}",
        "instances: {where: [parent_id == input.parent_id, flag == false]}",
    );
    let run = run(&ir(&model));
    assert_sound(&run);
    let claimed: Vec<String> = run
        .refusals()
        .into_iter()
        .filter(|refusal| refusal.contains("already holding what the effect writes"))
        .collect();
    assert_eq!(claimed, Vec::<String>::new());
}

/// The creator and the update both write an enum field from their input.
#[test]
fn u12_an_enum_written_from_both_inputs_is_varied_and_passes() {
    let model = replaced(
        U12_CREATOR_INPUT,
        "  - {name: demo.z.Key, kind: newtype, of: Uuid}",
        "  - {name: demo.z.Key, kind: newtype, of: Uuid}\n  - {name: demo.z.Mode, kind: enum, variants: [Low, Mid, High]}",
    );
    let model = model.replace(
        "{name: flag, type: Boolean}",
        "{name: flag, type: demo.z.Mode}",
    );
    let run = run(&ir(&model));
    assert_sound(&run);
    assert!(run.has(SYNCED), "{:#?}", run.refusals());
}

/// [`U12_CREATOR_INPUT`] with the update moving the rows (`Active -> Synced`) as well as writing
/// `flag`. The base (0.70.0) already witnessed `synced` by the move: its rows are created sending
/// `AddChild` `flag: false, true, false, true` by their distinctions, and the suite passes. U12
/// is meant to reach only updates the plain path refused, so this suite keeps its bytes, and every
/// row of it is still created as it was.
#[test]
fn u12_a_moving_set_update_the_base_witnessed_keeps_its_arrangement() {
    let model = replaced(
        U12_CREATOR_INPUT,
        "    lifecycle: {initial: Active, states: [Active], terminal: [Active]}",
        "    lifecycle:\n      initial: Active\n      states: [Active, Synced]\n      terminal: [Synced]\n      transitions:\n        - {name: sync, from: [Active], to: Synced}",
    );
    let model = replaced(
        &model,
        "        updates: demo.z.Child\n",
        "        moves: demo.z.Child.sync\n",
    );
    let run = run(&ir(&model));
    assert_sound(&run);
    let document: serde_json::Value =
        serde_json::from_str(&run.synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    let steps = document["scenarios"][SYNCED]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("{SYNCED} is synthesized: {:#?}", run.refusals()));
    let sent: Vec<&serde_json::Value> = steps
        .iter()
        .filter(|step| step["step"] == "execute_command" && step["command"] == "demo.z.AddChild")
        .map(|step| &step["input"]["flag"]["value"])
        .collect();
    // As `ess 0.70.0 verify conform synthesize --suite-format 5` writes it for this model.
    let base = [false, true, false, true].map(serde_json::Value::Bool);
    assert_eq!(
        sent,
        base.iter().collect::<Vec<_>>(),
        "every row is created as the base created it"
    );
}

/// The creator and the update both write an `Optional<Boolean>` field from their input.
#[test]
fn u12_an_optional_written_from_both_inputs_passes() {
    let model = U12_CREATOR_INPUT.replace(
        "{name: flag, type: Boolean}",
        "{name: flag, type: Optional<Boolean>}",
    );
    let run = run(&ir(&model));
    assert_sound(&run);
}
