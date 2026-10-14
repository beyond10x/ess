//! Adversary pass 1 for `trimmed:` text guards (beyond10x/ess#532, ess/25,
//! `docs/design/value-expressions.md` § E10), in conformance.
//!
//! Each case synthesizes the suite of a model holding a `trimmed` guard and runs it against the
//! native interpreter, which must pass it, and against a target wrong in one way, which must fail a
//! scenario: one that does not trim (the interpreter of the same model with `trimmed:` removed), and
//! ones that trim with a language's default trim — Rust `str::trim`, Go `strings.TrimSpace`,
//! JavaScript `String.prototype.trim` — which remove Unicode whitespace too. A mutant that passes
//! every scenario survives the suite: the suite does not test the trim it was synthesized for.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let specification = Specification::assemble([(Source::new("promo.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// A command whose `matched` branch is guarded by `guard` over the input `code: <code_type>`, and
/// whose default refuses. Nothing echoes `code`, so a target that rewrites it before deciding is
/// observed only through the branch it takes.
fn input_model(code_type: &str, guard: &str) -> String {
    format!(
        "format: ess/25
system: promo
version: v1
domain: promo.codes
errors:
  - {{name: promo.codes.Unknown, summary: Unknown., fields: []}}
events:
  - name: promo.codes.Matched
    fields:
      - {{name: n, type: Integer}}
commands:
  - name: promo.codes.Close
    input:
      - {{name: code, type: {code_type}}}
      - {{name: n, type: Integer}}
    outcomes:
      - name: matched
        when: {guard}
        emits: [promo.codes.Matched]
        payload:
          promo.codes.Matched:
            n: input.n
      - name: other
        error: promo.codes.Unknown
"
    )
}

/// A stored `label`, written by `Create`, and `Close` refusing when the stored label satisfies
/// `predicate` over the stored row.
fn stored_model(predicate: &str) -> String {
    format!(
        "format: ess/25
system: promo
version: v1
domain: promo.codes
entities:
  - name: promo.codes.Code
    identity: {{name: code_id, type: Uuid}}
    fields:
      - {{name: label, type: String}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
errors:
  - {{name: promo.codes.Refused, summary: Refused., fields: []}}
events:
  - name: promo.codes.Created
    fields:
      - {{name: code_id, type: Uuid}}
  - name: promo.codes.Closed
    fields:
      - {{name: code_id, type: Uuid}}
commands:
  - name: promo.codes.Create
    input:
      - {{name: label, type: String}}
    outcomes:
      - name: created
        creates: promo.codes.Code
        instance: code_id
        emits: [promo.codes.Created]
        payload:
          promo.codes.Created: {{code_id: {{generated: true}}}}
        sets: {{label: input.label}}
  - name: promo.codes.Close
    input:
      - {{name: code_id, type: Uuid}}
    outcomes:
      - name: done
        when_subject: {{predicate: {predicate}}}
        error: promo.codes.Refused
      - name: closed
        moves: promo.codes.Code.close
        instance: code_id
        emits: [promo.codes.Closed]
        payload:
          promo.codes.Closed: {{code_id: input.code_id}}
"
    )
}

/// Synthesizes `model`'s suite, asserts the interpreter of `model` passes every scenario, and
/// returns the scenarios the interpreter of `mutant` does not pass.
fn killed_by(model: &str, mutant: &str) -> Vec<String> {
    let ir = ir_of(model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let statuses = run(&synthesis.suite, &Interpreted::for_model(ir));
    assert!(statuses.len() > 1, "{statuses:#?}");
    assert_eq!(
        not_passed(&statuses),
        Vec::<String>::new(),
        "the correct target"
    );
    not_passed(&run(
        &synthesis.suite,
        &Interpreted::for_model(ir_of(mutant)),
    ))
}

/// The guard without its `trimmed:` wrapper: the target that does not trim.
fn untrimmed(model: &str, trimmed: &str, plain: &str) -> String {
    let out = model.replace(trimmed, plain);
    assert_ne!(out, model, "`{trimmed}` is in the model");
    out
}

// ---- a target that does not trim ----------------------------------------------------------------

/// Control: a plain input guard's suite kills the target that does not trim, so the method below
/// can tell a surviving mutant from a broken harness.
#[test]
fn adv_m3_control_a_plain_guard_kills_the_untrimmed_target() {
    let guard = "{code: {trimmed: {equals: DONE}}}";
    let model = input_model("String", guard);
    let killed = killed_by(&model, &untrimmed(&model, guard, "{code: {equals: DONE}}"));
    assert!(
        !killed.is_empty(),
        "the untrimmed target passed every scenario"
    );
}

/// `forall` over an input list whose body is a `trimmed` operator: the target that compares each
/// element untrimmed must fail a scenario.
#[test]
fn adv_m3_forall_trimmed_guard_kills_the_untrimmed_target() {
    let guard = "{forall: {in: code, as: c, that: {c: {trimmed: {equals: DONE}}}}}";
    let model = input_model("List<String>", guard);
    let mutant = untrimmed(
        &model,
        guard,
        "{forall: {in: code, as: c, that: {c: {equals: DONE}}}}",
    );
    let killed = killed_by(&model, &mutant);
    assert!(
        !killed.is_empty(),
        "a target that does not trim the elements of a `forall` passes every scenario"
    );
}

/// `exists` over an input list whose body is a `trimmed` operator.
#[test]
fn adv_m3_exists_trimmed_guard_kills_the_untrimmed_target() {
    let guard = "{exists: {in: code, as: c, that: {c: {trimmed: {equals: DONE}}}}}";
    let model = input_model("List<String>", guard);
    let mutant = untrimmed(
        &model,
        guard,
        "{exists: {in: code, as: c, that: {c: {equals: DONE}}}}",
    );
    let killed = killed_by(&model, &mutant);
    assert!(
        !killed.is_empty(),
        "a target that does not trim the elements of an `exists` passes every scenario"
    );
}

/// A `when_subject:` predicate over the stored row: the target that compares the stored label
/// untrimmed must fail a scenario.
#[test]
fn adv_m3_when_subject_trimmed_guard_kills_the_untrimmed_target() {
    let predicate = "{label: {trimmed: {equals: DONE}}}";
    let model = stored_model(predicate);
    let mutant = untrimmed(&model, predicate, "{label: {equals: DONE}}");
    let killed = killed_by(&model, &mutant);
    assert!(
        !killed.is_empty(),
        "a target that does not trim the stored label passes every scenario"
    );
}

// ---- a target that trims with its language's default trim ---------------------------------------

/// Rust `str::trim`: the Unicode `White_Space` property.
fn rust_trim(text: &str) -> String {
    text.trim().to_owned()
}

/// Go `strings.TrimSpace`: `unicode.IsSpace`, which is `White_Space` as well, U+0085 and U+00A0
/// included.
fn go_trim_space(text: &str) -> String {
    text.trim_matches(char::is_whitespace).to_owned()
}

/// JavaScript `String.prototype.trim`: `WhiteSpace` and `LineTerminator` — U+FEFF and every `Zs`, U+2028
/// and U+2029 — but not U+0085.
fn js_trim(text: &str) -> String {
    text.trim_matches(|c: char| c != '\u{85}' && (c.is_whitespace() || c == '\u{feff}'))
        .to_owned()
}

/// The interpreter, with the input `code` (a text, or each text of a list) passed through `trim`
/// before it decides: together with the interpreter's own ASCII trim, a target trimming with
/// `trim`.
struct Pretrim {
    inner: Interpreted,
    trim: fn(&str) -> String,
}

impl Pretrim {
    fn rewrite(&self, node: Node) -> Node {
        match node {
            Node::Text(text) => Node::Text((self.trim)(&text)),
            Node::Seq(items) => {
                Node::Seq(items.into_iter().map(|item| self.rewrite(item)).collect())
            }
            other => other,
        }
    }
}

impl ConformanceTarget for Pretrim {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if let Some(code) = request.input.remove("code") {
            request.input.insert("code".to_owned(), self.rewrite(code));
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(r)
    }
}

/// Every default-trim target over `guard` (on an input `code: <code_type>`) fails a scenario of the
/// suite; the identity rewrite, through the same wrapper, passes every one.
fn default_trims_are_killed(code_type: &str, guard: &str) {
    let ir = ir_of(&input_model(code_type, guard));
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let control = Pretrim {
        inner: Interpreted::for_model(ir.clone()),
        trim: str::to_owned,
    };
    assert_eq!(
        not_passed(&run(&suite, &control)),
        Vec::<String>::new(),
        "{guard}: control"
    );
    let mut survivors = Vec::new();
    for (name, trim) in [
        ("Rust str::trim", rust_trim as fn(&str) -> String),
        ("Go strings.TrimSpace", go_trim_space),
        ("JavaScript trim()", js_trim),
    ] {
        let mutant = Pretrim {
            inner: Interpreted::for_model(ir.clone()),
            trim,
        };
        if not_passed(&run(&suite, &mutant)).is_empty() {
            survivors.push(name);
        }
    }
    assert_eq!(
        survivors,
        Vec::<&str>::new(),
        "{guard}: these targets pass every scenario"
    );
}

#[test]
fn adv_m3_default_trims_fail_a_plain_guard() {
    for guard in [
        "{code: {trimmed: {equals: DONE}}}",
        "{code: {trimmed: {in_ignore_case: [new, done]}}}",
        "{code: {trimmed: {starts_with: SP}}}",
        "{code: {trimmed: {ends_with: '24'}}}",
        "{not: {code: {trimmed: {equals: DONE}}}}",
        "{all: [{code: {trimmed: {equals: DONE}}}, 'n > 0']}",
    ] {
        default_trims_are_killed("String", guard);
    }
}

#[test]
fn adv_m3_default_trims_fail_a_quantified_guard() {
    for guard in [
        "{forall: {in: code, as: c, that: {c: {trimmed: {equals: DONE}}}}}",
        "{exists: {in: code, as: c, that: {c: {trimmed: {equals: DONE}}}}}",
    ] {
        default_trims_are_killed("List<String>", guard);
    }
}
