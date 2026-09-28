//! Adversary pass 2 for structured precondition literals (beyond10x/ess#205): what `ess-domain`
//! admits against what the conformance setup reader runs.
//!
//! The property every case holds: a model `ess-domain` admits compiles, and each of its
//! preconditions, executed by the interpreter from the IR (`interpret::execute`, which reads the
//! input through `input::flatten` and holds the created row to its entity's invariants), takes the
//! branch the IR records for it. A model the domain refuses passes trivially, so each case pairs a
//! control that runs with the literal the domain and the reader disagree on.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// A control: admitted by `ess-domain`, and every precondition runs.
fn control(body: &str) {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("parses: {error}\n{body}"));
    if let Err(errors) = Specification::assemble(vec![(Source::new("probe.yaml"), raw)]) {
        panic!("the control is admitted, got:\n{errors}\n{body}");
    }
    runs(body);
}

/// Admitted by `ess-domain` implies every precondition runs, in order, as the IR says it does.
fn runs(body: &str) {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("parses: {error}\n{body}"));
    let Ok(specification) = Specification::assemble(vec![(Source::new("probe.yaml"), raw)]) else {
        return;
    };
    let mut sources = SourceMap::new();
    sources.insert("probe.yaml", body);
    let ir = compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("an admitted model compiles:\n{diagnostics}"));
    assert!(
        !ir.preconditions().is_empty(),
        "the model has a precondition"
    );
    let mut store = Store::default();
    for precondition in ir.preconditions() {
        let command = precondition.command.name().clone();
        let steps = execute(
            &ir,
            &store,
            &command,
            &precondition.input,
            &Externals::Withheld,
        )
        .unwrap_or_else(|why| {
            panic!(
                "ess-domain admitted the precondition on `{command}` with input {:?}, and the \
                 interpreter does not run it: {why}\n{body}",
                precondition.input
            )
        });
        assert_eq!(steps.len(), 1, "one step for `{command}`");
        let wanted = format!("{command}/{}", precondition.outcome);
        let taken = steps[0].outcome.as_ref().map(ToString::to_string);
        assert_eq!(
            taken.as_deref(),
            Some(wanted.as_str()),
            "the IR records `{wanted}` for the precondition, and the interpreter takes another \
             branch\n{body}"
        );
        store = steps[0].next.clone();
    }
}

/// One creating command over `probe.core.User` (`rank: Integer`, invariant `rank >= 1`), whose
/// input `probe` is typed `probe_type`, sent as `probe`. `guard` is the `when:` of the creating
/// branch `guarded`; the default `plain` only emits. `sets` is the creating branch's `sets:`.
fn system(probe_type: &str, probe: &str, guard: &str, sets: &str) -> String {
    format!(
        "format: ess/15
system: probe
version: v1
preconditions:
  - command: probe.core.Open
    as: probe.core.Server
    input: {{user_id: 00000000-0000-4000-8000-000000000001{probe}}}
domain: probe.core
types:
  - name: probe.core.Window
    kind: struct
    fields:
      - {{name: low, type: Integer}}
  - name: probe.core.Maybe
    kind: newtype
    of: 'Optional<String>'
  - name: probe.core.Account
    kind: struct
    fields:
      - {{name: name, type: String}}
      - {{name: note, type: probe.core.Maybe}}
entities:
  - name: probe.core.User
    identity: {{name: user_id, type: Uuid}}
    fields:
      - {{name: rank, type: Integer}}
    invariants: [rank >= 1]
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - {{name: probe.core.Server, may: [probe.core.Open]}}
events:
  - {{name: probe.core.Opened, fields: [{{name: user_id, type: Uuid}}]}}
  - {{name: probe.core.Plain}}
commands:
  - name: probe.core.Open
    input:
      - {{name: user_id, type: Uuid}}
      - {{name: probe, type: '{probe_type}'}}
    outcomes:
      - name: guarded
{guard}        creates: probe.core.User
        instance: user_id
        sets: {sets}
        emits: [probe.core.Opened]
        payload: {{probe.core.Opened: {{user_id: input.user_id}}}}
      - name: plain
        emits: [probe.core.Plain]
"
    )
}

const PRESENT: &str = "        when: defined(probe)\n";
const ALWAYS: &str = "        when: 'user_id == \"00000000-0000-4000-8000-000000000001\"'\n";

/// `defined(probe)` over an `Optional<Window>` the precondition sends as a struct literal: the setup
/// reader marks it present and takes `guarded`; the domain binds no fact for a map and selects the
/// default `plain`, which the IR records.
#[test]
fn a_presence_guard_over_a_struct_literal_selects_the_branch_the_interpreter_takes() {
    control(
        &system(
            "Optional<probe.core.Window>",
            ", probe: null",
            PRESENT,
            "{rank: 1}",
        )
        .replace("format: ess/15", "format: ess/16"),
    );
    runs(
        &system(
            "Optional<probe.core.Window>",
            ", probe: {low: 1}",
            PRESENT,
            "{rank: 1}",
        )
        .replace("format: ess/15", "format: ess/16"),
    );
}

/// An input typed by a newtype over `Optional<…>`, left out: the domain counts it supplied, the
/// reader's `bind` counts only a literal `Optional<…>` as optional and refuses the request.
#[test]
fn an_omitted_input_typed_by_an_optional_newtype_runs() {
    control(&system(
        "probe.core.Maybe",
        ", probe: null",
        ALWAYS,
        "{rank: 1}",
    ));
    runs(&system("probe.core.Maybe", "", ALWAYS, "{rank: 1}"));
}

/// A struct literal leaving out a member typed by a newtype over `Optional<…>`: admitted by the
/// correction to F3, refused by the reader's struct walk as a missing field.
#[test]
fn a_struct_literal_leaving_out_an_optional_newtype_member_runs() {
    control(&system(
        "probe.core.Account",
        ", probe: {name: n, note: null}",
        ALWAYS,
        "{rank: 1}",
    ));
    runs(&system(
        "probe.core.Account",
        ", probe: {name: n}",
        ALWAYS,
        "{rank: 1}",
    ));
}

/// The creating branch writes the literal `rank: 0`: the interpreter holds the row to `rank >= 1`
/// and refuses the step, and the domain never looked at a literal `sets:` source.
#[test]
fn a_precondition_whose_branch_sets_a_literal_the_invariant_forbids_is_not_admitted() {
    control(&system("String", ", probe: x", ALWAYS, "{rank: 2}"));
    runs(&system("String", ", probe: x", ALWAYS, "{rank: 0}"));
}
