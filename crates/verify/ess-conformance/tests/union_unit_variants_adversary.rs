//! Adversary cases for unit variants in the conformance lanes (ess/22, beyond10x/ess#418,
//! `docs/design/union-unit-variants.md`).
//!
//! - The design page says a unit variant makes a union whose payload variants all recurse
//!   buildable. Synthesis witnesses a union by its first variant only, so whether that holds
//!   depends on the alphabetical order of the labels.
//! - Synthesis never sends a unit variant that is not the first label, so a synthesized suite does
//!   not observe the wire form of the construct at all for the acceptance model itself.
//! - A union whose tag is `value` carries its payload under `content`; the witness builder writes
//!   a payload under `value` regardless.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("model.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// A command sending `chain`, a union `demo.work.Chain` with the variants `variants` declares
/// (YAML lines, already indented), `demo.work.Cell` a struct whose `tail` is the union again.
fn chain_model(tag: &str, variants: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.work
types:
  - name: demo.work.Cell
    kind: struct
    fields:
      - {{name: head, type: String}}
      - {{name: tail, type: demo.work.Chain}}
  - name: demo.work.Chain
    kind: union
    tag: {tag}
    variants:
{variants}events:
  - name: demo.work.ChainReported
    fields:
      - {{name: chain, type: demo.work.Chain}}
actors:
  - name: demo.work.Reporter
    may: [demo.work.ReportChain]
commands:
  - name: demo.work.ReportChain
    input:
      - {{name: chain, type: demo.work.Chain}}
    outcomes:
      - name: reported
        emits: [demo.work.ChainReported]
        payload:
          demo.work.ChainReported:
            chain: input.chain
"
    )
}

/// A flat union `demo.work.Choice` with the variants `variants` declares, sent and published.
fn choice_model(tag: &str, variants: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.work
types:
  - name: demo.work.Choice
    kind: union
    tag: {tag}
    variants:
{variants}events:
  - name: demo.work.Chosen
    fields:
      - {{name: choice, type: demo.work.Choice}}
actors:
  - name: demo.work.Chooser
    may: [demo.work.Choose]
commands:
  - name: demo.work.Choose
    input:
      - {{name: choice, type: demo.work.Choice}}
    outcomes:
      - name: chosen
        emits: [demo.work.Chosen]
        payload:
          demo.work.Chosen:
            choice: input.choice
"
    )
}

fn synthesized_and_interpreted(text: &str) {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(
        synthesis.refusals.len(),
        0,
        "synthesis refused:\n{:#?}",
        synthesis.refusals
    );
    assert_ne!(synthesis.suite.scenarios.len(), 0);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let run = ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert!(
        run.scenarios
            .iter()
            .all(|result| result.status == ess_conformance::report::Status::Passed),
        "{:#?}",
        run.scenarios
    );
}

/// The design page: "A unit variant is always inhabited, so a union whose payload variants all
/// recurse is buildable through it." The domain admits `Chain = Cons(Cell) | Nil` on that ground;
/// synthesis must then find the finite value `{"kind": "Nil"}` rather than refuse the input as
/// self-referential because `Cons` sorts before `Nil`.
#[test]
fn a_recursive_union_is_witnessed_through_its_unit_variant_whatever_the_label_order() {
    synthesized_and_interpreted(&chain_model(
        "kind",
        "      Cons: demo.work.Cell\n      Nil:\n",
    ));
}

/// The control: the same model with the unit variant sorting first synthesizes.
#[test]
fn control_a_recursive_union_whose_unit_variant_sorts_first_is_witnessed() {
    synthesized_and_interpreted(&chain_model(
        "kind",
        "      Link: demo.work.Cell\n      End:\n",
    ));
}

/// Acceptance: "a faulty target that encodes `Open` with a payload member ... fails". A synthesized
/// suite can only catch that if some scenario sends or expects `Open`; the union is witnessed by
/// its first variant (`Complete`), so nothing synthesized ever does.
#[test]
fn the_synthesized_suite_of_the_acceptance_model_sends_the_unit_variant() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert_eq!(synthesis.refusals.len(), 0, "{:?}", synthesis.refusals);
    let json = synthesis.suite.to_compact_json().unwrap();
    assert!(
        json.contains(r#"{"kind":"Open"}"#),
        "no synthesized scenario sends or expects the unit variant `Open`:\n{json}"
    );
}

/// A union tagged `value` carries its payload under `content`. With the payload variant first,
/// the witness is that payload variant; it must be `{"value": "Alpha", "content": …}`.
#[test]
fn a_union_tagged_value_whose_first_variant_carries_a_payload_synthesizes() {
    synthesized_and_interpreted(&choice_model("value", "      Alpha: String\n      Zulu:\n"));
}

/// The same with the unit variant first: the witness is the tag alone, `{"value": "Alpha"}`.
#[test]
fn a_union_tagged_value_whose_first_variant_is_a_unit_variant_synthesizes() {
    synthesized_and_interpreted(&choice_model("value", "      Alpha:\n      Zulu: String\n"));
}

/// A union of only unit variants — the design page's alternative to an enum.
#[test]
fn a_union_of_only_unit_variants_synthesizes_and_runs() {
    synthesized_and_interpreted(&choice_model("kind", "      Alpha:\n      Zulu:\n"));
}

/// `demo.work.Status` (`Open | Complete`) inside a list and an optional, sent and published.
const NESTED: &str = "format: ess/22
system: demo
version: v1
domain: demo.work
types:
  - name: demo.work.Completion
    kind: struct
    fields:
      - {name: outcome, type: String}
  - name: demo.work.Status
    kind: union
    tag: kind
    variants:
      Open:
      Complete: demo.work.Completion
events:
  - name: demo.work.StatusesReported
    fields:
      - {name: statuses, type: List<demo.work.Status>}
      - {name: maybe, type: Optional<demo.work.Status>}
actors:
  - name: demo.work.Reporter
    may: [demo.work.ReportStatuses]
commands:
  - name: demo.work.ReportStatuses
    input:
      - {name: statuses, type: List<demo.work.Status>}
      - {name: maybe, type: Optional<demo.work.Status>}
    outcomes:
      - name: reported
        emits: [demo.work.StatusesReported]
        payload:
          demo.work.StatusesReported:
            statuses: input.statuses
            maybe: input.maybe
";

#[test]
fn a_list_and_an_optional_of_a_union_with_a_unit_variant_synthesize_and_run() {
    synthesized_and_interpreted(NESTED);
}

/// One authored act against `NESTED`, sending `statuses` and `maybe` as written.
fn authored_nested(statuses: &str, maybe: &str) -> ess_conformance::authored::Authoring {
    let text = format!(
        "type: ess-scenario/1\ndomain: demo.work\nscenario: nested\nsummary: Nested \
         statuses.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    command: \
         demo.work.ReportStatuses\n    actor: demo.work.Reporter\n    input:\n      statuses: \
         {statuses}\n      maybe: {maybe}\n    outcome: reported\n"
    );
    ess_conformance::authored::compile(
        &ir(NESTED),
        &[ess_conformance::authored::Source::new("nested.yaml", text)],
    )
}

/// The control: well-formed unit variants inside a list and an optional compile.
#[test]
fn control_authored_unit_variants_inside_a_list_and_an_optional_compile() {
    let authoring = authored_nested("[{kind: Open}]", "{kind: Open}");
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
}

/// The design page: "authored scenarios | a unit variant written with a content member is refused
/// as an undeclared field". The implementor's case sends the union as a top-level input; inside a
/// list element it must be refused the same way.
#[test]
fn an_authored_unit_variant_with_a_payload_inside_a_list_is_refused() {
    let authoring = authored_nested("[{kind: Open, value: {outcome: shipped}}]", "{kind: Open}");
    assert!(
        !authoring.is_complete(),
        "a list element `{{kind: Open, value: …}}` must be refused before it runs"
    );
}

/// The same inside an optional input.
#[test]
fn an_authored_unit_variant_with_a_payload_inside_an_optional_is_refused() {
    let authoring = authored_nested("[{kind: Open}]", "{kind: Open, value: null}");
    assert!(
        !authoring.is_complete(),
        "an optional `{{kind: Open, value: null}}` must be refused before it runs"
    );
}

/// An authored *expected* event payload holding a unit variant with a content member is the same
/// malformed value as an input holding one.
#[test]
fn an_authored_expected_event_with_a_unit_variant_payload_is_refused() {
    let text = "type: ess-scenario/1
domain: demo.work
scenario: open-expected-with-payload
summary: An expected event spells Open with a payload.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.work.ReportStatus
    actor: demo.work.Reporter
    input:
      status: {kind: Open}
    outcome: reported
    events:
      - event: demo.work.StatusReported
        payload: {status: {kind: Open, value: {outcome: shipped}}}
";
    let authoring = ess_conformance::authored::compile(
        &ir(MODEL),
        &[ess_conformance::authored::Source::new(
            "expected.yaml",
            text,
        )],
    );
    assert!(
        !authoring.is_complete(),
        "an expected `{{kind: Open, value: …}}` must be refused before it runs"
    );
}
