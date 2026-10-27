//! Adversary pass 1 over ESS 0.72.0 wave 1 (U1 to U4), on the integrated branch.
//!
//! Every case drives the synthesizer on a model the new code reaches, then runs the suite on the
//! model interpreter. The property each case holds the synthesizer to is the one the units' own
//! tests hold it to: what it synthesizes passes the interpreter of the same model. Where a case
//! also pins that a scenario is synthesized, it says so.
//!
//! - U2 (`related_on_row`, `related_run`, `related_missing`): the trigger's own guard compares the
//!   input the related row is sent through with its row's stored field.
//! - U4 area (`related_decided`): the trigger reads a row of another entity through the input a
//!   declared conversion carries into the invoked command's guard.
//! - U1 (`shared_selection`, `append`): the follow-up's set effect publishes one event per changed
//!   row (`each: changed`).
//! - U3 (`set_guard::arranged_for`): the trigger names a related row of its own through the input
//!   the set guard's filter reads; and a trigger that itself creates the row the guard selects.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const U2_SYSTEM: &str = include_str!("fixtures/binding-into-updating-command/system.yaml");
const U2_COMPONENTS: &str = include_str!("fixtures/binding-into-updating-command/components.yaml");
const U2_DOMAIN: &str = include_str!("fixtures/binding-into-updating-command/b.yaml");

const U4_SYSTEM: &str = include_str!("fixtures/binding-trigger-shares-row-guard/system.yaml");
const ACROSS_COMPONENTS: &str =
    include_str!("fixtures/adversary-0720/related-row-across-entities/components.yaml");
const ACROSS_DOMAIN: &str =
    include_str!("fixtures/adversary-0720/related-row-across-entities/x.yaml");

const U1_SYSTEM: &str = include_str!("fixtures/set-effect-fresh-input/system.yaml");
const U1_KEYS: &str = include_str!("fixtures/set-effect-fresh-input/domains/keys.yaml");
const U1_SEATS: &str = include_str!("fixtures/set-effect-fresh-input/domains/seats.yaml");

const U3_SYSTEM: &str = include_str!("fixtures/binding-set-guard/system.yaml");
const NAMES_COMPONENTS: &str =
    include_str!("fixtures/adversary-0720/set-guard-trigger-names-holder/components.yaml");
const NAMES_DOMAIN: &str =
    include_str!("fixtures/adversary-0720/set-guard-trigger-names-holder/a.yaml");
const CREATES_COMPONENTS: &str =
    include_str!("fixtures/adversary-0720/set-guard-trigger-creates-the-row/components.yaml");
const CREATES_DOMAIN: &str =
    include_str!("fixtures/adversary-0720/set-guard-trigger-creates-the-row/a.yaml");

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
    /// interpreter's first diagnostics.
    fn failing(&self) -> Vec<String> {
        self.statuses
            .iter()
            .filter(|(_, (status, _))| *status != Status::Passed)
            .map(|(id, (status, diagnostics))| {
                let first: Vec<&String> = diagnostics.iter().take(2).collect();
                format!("{id} {status:?}: {first:#?}")
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

/// U2. `MoveTicket` moves a ticket only when the grant it is sent is the one the ticket holds
/// (`when_subject: grant_id != input.grant_id` refuses otherwise). Its `moved` scenario arranges
/// the ticket with a grant and sends that same grant. The binding into `Refresh`, which updates
/// the ticket and branches on the grant, has the grant arranged before the trigger and the trigger
/// sent naming it (`related_run` adopting `related_on_row`), with no check that the trigger keeps
/// its branch: the ticket still holds the first grant, so `MoveTicket` answers `mismatch`. The
/// trigger's own `MoveTicket/outcome/moved` scenario fails, and so does every binding scenario
/// sent the same way, `related_missing`'s refusal scenario included.
#[test]
fn a_binding_into_an_updating_command_keeps_the_trigger_on_its_own_branch() {
    let domain = replaced(
        U2_DOMAIN,
        "      - name: moved\n",
        "      - name: mismatch\n        when_subject: {predicate: grant_id != input.grant_id}\n        \
         error: demo.b.Mismatch\n      - name: moved\n",
    );
    let domain = replaced(
        &domain,
        "  - name: demo.b.Missing\n",
        "  - name: demo.b.Missing\n  - name: demo.b.Mismatch\n",
    );
    let model = ir_of(&[
        ("system.yaml", U2_SYSTEM),
        ("components.yaml", U2_COMPONENTS),
        ("b.yaml", &domain),
    ]);
    let run = run(&model);
    assert!(
        run.has("demo.b.MoveTicket/outcome/moved"),
        "{:#?}",
        run.refusals()
    );
    assert_sound(&run);
}

/// U4 area. `Enrol` refuses a missing team (`when_related: {via: input.team_id, exists: false}`)
/// and publishes the team's identity; the binding carries it across a declared `TeamId -> GroupId`
/// conversion that hands the value on as it is into `Grant`, which reads a group through it.
/// `related_decided` arranges a group and sends `Enrol` naming the group, so `Enrol` finds no team
/// and refuses; `refuses_only_missing` reads "publishes while that row exists, whichever row it
/// is" without comparing the entity the trigger reads with the one the invoked command reads.
#[test]
fn a_trigger_reading_another_entity_through_a_converted_input_keeps_its_branch() {
    let model = ir_of(&[
        ("system.yaml", U4_SYSTEM),
        ("components.yaml", ACROSS_COMPONENTS),
        ("x.yaml", ACROSS_DOMAIN),
    ]);
    let run = run(&model);
    assert_sound(&run);
}

/// U1. The repro's own model with one change: `Park` publishes `Parked` once per row it moves
/// (`each: changed`). The disclosure cell appends a bare `Park` sent an owner the origin never
/// sent, and arranges no seat for it: the set effect selects no row, so no `Parked` is published,
/// and the cell expects one. `shared_selection` admits the follow-up because no row of the origin
/// can match; nothing checks that any row matches. The repro's acceptance test holds only because
/// a plain `emits:` publishes on zero rows: its appended run parks nothing either.
#[test]
fn a_per_row_event_follow_up_composed_after_a_disclosure_passes() {
    let seats = replaced(
        U1_SEATS,
        "        emits: [demo.w.seats.Parked]\n",
        "        emits:\n          - {event: demo.w.seats.Parked, each: changed}\n",
    );
    let model = ir_of(&[
        ("system.yaml", U1_SYSTEM),
        ("domains/keys.yaml", U1_KEYS),
        ("domains/seats.yaml", &seats),
    ]);
    let run = run(&model);
    assert!(
        run.has(
            "demo.w.keys.Issue/disclosure/issued/secret/command/demo.w.seats.Park/parked/as/anonymous"
        ),
        "{:#?}",
        run.refusals()
    );
    assert_sound(&run);
}

/// U3. `Request` refuses a missing holder (`when_related: {via: input.holder_id, exists: false}`),
/// so its own scenario registers a holder and sends the captured identity. `Check`'s set guard
/// selects seats by that holder; `set_guard::arranged_for` arranges a seat for a literal holder
/// and overwrites the trigger's captured holder with that literal (`keeps_branch` reads no
/// related row), so `Request` answers `no-holder` and publishes nothing.
#[test]
fn a_set_guard_arrangement_keeps_the_trigger_naming_its_own_related_row() {
    let model = ir_of(&[
        ("system.yaml", U3_SYSTEM),
        ("components.yaml", NAMES_COMPONENTS),
        ("a.yaml", NAMES_DOMAIN),
    ]);
    let run = run(&model);
    assert!(
        run.has("check-on-requested/binding/flow"),
        "{:#?}",
        run.refusals()
    );
    assert_sound(&run);
}

/// U3, a refusal. `OpenSeat` creates the seat `Check`'s set guard selects, copying the holder from
/// its own input unconverted, as the refusal's help asks; it also refuses a missing holder, so the
/// rows cannot be arranged by literal. The trigger alone decides `kept`, yet the binding is
/// refused `BranchSetUnarranged`, with a help the model already follows.
#[test]
fn a_set_guard_the_trigger_alone_decides_flows() {
    let model = ir_of(&[
        ("system.yaml", U3_SYSTEM),
        ("components.yaml", CREATES_COMPONENTS),
        ("a.yaml", CREATES_DOMAIN),
    ]);
    let run = run(&model);
    assert!(
        run.has("check-on-seat-opened/binding/flow"),
        "{:#?}",
        run.refusals()
    );
    assert_sound(&run);
}
