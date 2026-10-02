//! Adversary pass 1 on beyond10x/ess#242: `{$instance: …}` inside structured inputs.
//!
//! Each case here is red on the tree it was written against, and names the defect it shows.
#![allow(clippy::too_many_lines, clippy::missing_panics_doc)]

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::ScenarioStep;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use serde_json::json;

/// Rings, and a command whose inputs hold ring identities inside a union payload and inside the
/// struct values of a map.
const MODEL: &str = r"format: ess/1
system: release
version: v1
domains:
  - release.rings

domain: release.rings

summary: Release rings, aimed at through structured inputs.

types:
  - name: release.rings.ReleaseRingId
    kind: newtype
    of: Uuid

  - name: release.rings.AimId
    kind: newtype
    of: Uuid

  - name: release.rings.RingPair
    kind: struct
    fields:
      - name: primary
        type: release.rings.ReleaseRingId
      - name: fallback
        type: Optional<release.rings.ReleaseRingId>
      - name: note
        type: String

  - name: release.rings.Stage
    kind: struct
    fields:
      - name: name
        type: String
      - name: rings
        type: List<release.rings.ReleaseRingId>

  - name: release.rings.Target
    kind: union
    tag: kind
    variants:
      ring: release.rings.ReleaseRingId
      label: String

entities:
  - name: release.rings.ReleaseRing
    identity:
      name: ring_id
      type: release.rings.ReleaseRingId
    fields:
      - name: label
        type: String
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]

  - name: release.rings.Aim
    identity:
      name: aim_id
      type: release.rings.AimId
    fields:
      - name: note
        type: String
    lifecycle:
      initial: Aimed
      states: [Aimed]
      terminal: [Aimed]

commands:
  - name: release.rings.CreateRing
    input:
      - name: label
        type: String
    outcomes:
      - name: created
        creates: release.rings.ReleaseRing
        instance: ring_id
        emits:
          - release.rings.RingCreated
        payload:
          release.rings.RingCreated:
            label: input.label
        sets:
          label: input.label

  - name: release.rings.AimAt
    input:
      - name: note
        type: String
      - name: target
        type: Optional<release.rings.Target>
      - name: maybe
        type: Optional<List<release.rings.ReleaseRingId>>
      - name: holes
        type: Optional<List<Optional<release.rings.ReleaseRingId>>>
      - name: stages
        type: Optional<List<release.rings.Stage>>
      - name: by_pair
        type: Optional<Map<String, release.rings.RingPair>>
    outcomes:
      - name: aimed
        creates: release.rings.Aim
        instance: aim_id
        emits:
          - release.rings.Aimed
        payload:
          release.rings.Aimed:
            note: input.note
        sets:
          note: input.note

events:
  - name: release.rings.RingCreated
    fields:
      - name: ring_id
        type: release.rings.ReleaseRingId
      - name: label
        type: String

  - name: release.rings.Aimed
    fields:
      - name: aim_id
        type: release.rings.AimId
      - name: note
        type: String
";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("release.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Three rings, created and captured in order: `a`, `b`, `c`.
const ARRANGED: &str = r"type: ess-scenario/1
domain: release.rings
scenario: aim-at-rings
summary: An aim names rings inside structured inputs.
arrange:
  - instance: a
    entity: release.rings.ReleaseRing
  - instance: b
    entity: release.rings.ReleaseRing
  - instance: c
    entity: release.rings.ReleaseRing
timeline:
  - at: 2026-01-05T09:00:00Z
    command: release.rings.CreateRing
    input: {label: alpha}
    outcome: created
    events:
      - event: release.rings.RingCreated
        payload: {label: alpha}
    capture: {instance: a, event: release.rings.RingCreated, field: ring_id}
  - at: 2026-01-05T09:00:01Z
    command: release.rings.CreateRing
    input: {label: beta}
    outcome: created
    events:
      - event: release.rings.RingCreated
        payload: {label: beta}
    capture: {instance: b, event: release.rings.RingCreated, field: ring_id}
  - at: 2026-01-05T09:00:02Z
    command: release.rings.CreateRing
    input: {label: gamma}
    outcome: created
    events:
      - event: release.rings.RingCreated
        payload: {label: gamma}
    capture: {instance: c, event: release.rings.RingCreated, field: ring_id}
";

/// One `AimAt` step, with `inputs` as extra input lines (each already indented six spaces).
fn aim(inputs: &str) -> String {
    format!(
        "{ARRANGED}  - at: 2026-01-05T09:00:03Z\n    command: release.rings.AimAt\n    input:\n      \
         note: first\n{inputs}    outcome: aimed\n"
    )
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

fn refusals(result: &Authoring) -> String {
    result
        .refusals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The input the `AimAt` step compiles to, as the suite writes it.
fn aim_input(result: &Authoring) -> serde_json::Value {
    let scenario = result.scenarios.values().next().expect("one scenario");
    let input = scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "release.rings.AimAt" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the aim is invoked");
    serde_json::to_value(input).unwrap()
}

/// The story's outcome: "wherever the declared type at that position is an identity type". A
/// union's payload is a position whose declared type is the variant's, here the ring's identity
/// (`release.rings.Target`, variant `ring: ReleaseRingId`, written `{kind: ring, value: …}` as
/// every union value is). `Container::of` classifies every union as opaque, so the reference is
/// refused as `ESS-AUTHOR-022` naming `target.value` with declared `release.rings.Target`.
#[test]
fn a_reference_as_the_payload_of_an_identity_variant_of_a_union_authors() {
    let ir = model();
    let result = authoring(
        &ir,
        &aim("      target: {kind: ring, value: {$instance: b}}\n"),
    );
    assert!(result.is_complete(), "refused:\n{}", refusals(&result));
    assert_eq!(
        aim_input(&result)["target"],
        json!({"kind": "members", "members": {
            "kind": {"kind": "literal", "value": "ring"},
            "value": {"kind": "instance", "instance": "b"},
        }})
    );
}

/// `structured` leaves a reference at a member the model does not declare to "the shape check,
/// which names the member" (authored.rs doc on `structured`). The shape check does not walk into
/// a map's values (input.rs, `ResolvedTypeRef::Map`: "Its entries are not facts"), so under a
/// `Map<String, RingPair>` a misspelt member holding a reference is refused by nobody: the step
/// compiles and the target is sent the mapping `{"$instance": "b"}` as a value.
#[test]
fn a_reference_at_an_undeclared_member_inside_a_map_value_is_refused() {
    let ir = model();
    let result = authoring(
        &ir,
        &aim("      by_pair: {canary: {primary: {$instance: a}, fallbak: {$instance: b}, note: x}}\n"),
    );
    assert!(
        !result.is_complete(),
        "compiled; the aim sends {}",
        aim_input(&result)["by_pair"]
    );
    assert!(
        refusals(&result).contains("fallbak"),
        "refused, but not naming the member:\n{}",
        refusals(&result)
    );
}

/// The same hole with the wrong container: a `RingPair` written as a list inside a map value. The
/// walk classifies the list's elements as `Slot::Undeclared` ("where the shape check reports the
/// mismatch"), and no shape check reaches a map value, so the reference is sent as a literal.
#[test]
fn a_reference_inside_a_map_value_of_the_wrong_shape_is_refused() {
    let ir = model();
    let result = authoring(&ir, &aim("      by_pair: {canary: [{$instance: a}]}\n"));
    assert!(
        !result.is_complete(),
        "compiled; the aim sends {}",
        aim_input(&result)["by_pair"]
    );
}

// ---- probes: attacks that held on this tree (green), kept as regression cases -------------------

mod probes {
    use std::cell::RefCell;

    use ess_conformance::interpret::Interpreted;
    use ess_conformance::report::Status;
    use ess_conformance::target::*;
    use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, SuiteProvenance};
    use ess_primitives::node::Node;

    use super::*;

    /// Optional<List<Id>> with one ring twice, List<Optional<Id>> with a null, a list of structs
    /// of lists, and a map of structs: every ring at a different position in each container.
    const ALL: &str = "      maybe: [{$instance: a}, {$instance: a}, {$instance: c}]\n      \
         holes: [null, {$instance: c}, {$instance: b}]\n      \
         stages: [{name: early, rings: [{$instance: b}, {$instance: c}]}, {name: late, rings: [{$instance: a}]}]\n      \
         by_pair: {canary: {primary: {$instance: c}, fallback: {$instance: b}, note: y}}\n";

    fn suite(ir: &EssIr, text: &str) -> ConformanceSuite {
        let result = authoring(ir, text);
        assert!(result.is_complete(), "{}", refusals(&result));
        let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
        suite.scenarios = result.scenarios;
        suite.select_fresh_format_for(ir);
        suite
    }

    struct Recording {
        inner: Interpreted,
        sent: RefCell<Vec<SemanticCommandRequest>>,
        published: RefCell<Vec<ObservedEvent>>,
    }

    impl ConformanceTarget for Recording {
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
            request: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            self.sent.borrow_mut().push(request.clone());
            let result = self.inner.execute_command(request)?;
            self.published
                .borrow_mut()
                .extend(result.direct_events.iter().cloned());
            Ok(result)
        }
        fn query_view(
            &self,
            request: SemanticViewRequest,
        ) -> Result<SemanticViewResult, TargetError> {
            self.inner.query_view(request)
        }
        fn observe_events(
            &self,
            request: EventObservationRequest,
        ) -> Result<Vec<ObservedEvent>, TargetError> {
            let observed = self.inner.observe_events(request)?;
            self.published.borrow_mut().extend(observed.iter().cloned());
            Ok(observed)
        }
        fn configure_external_outcome(
            &self,
            control: ExternalOutcomeControl,
        ) -> Result<(), TargetError> {
            self.inner.configure_external_outcome(control)
        }
        fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
            self.inner.redeliver_event(request)
        }
    }

    #[test]
    fn nested_optional_and_repeated_references_run_where_written() {
        let ir = model();
        let suite = suite(&ir, &aim(ALL));
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/32"
        );
        let target = Recording {
            inner: Interpreted::for_model(ir),
            sent: RefCell::default(),
            published: RefCell::default(),
        };
        let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|e| panic!("{e}"));
        let report = Runner::for_suite(&suite)
            .run_admitted(&admitted, &target)
            .into_report();
        let statuses: Vec<Status> = report.scenarios.iter().map(|it| it.status).collect();
        assert_eq!(statuses, vec![Status::Passed], "{report:#?}");
        let ring = |label: &str| -> Node {
            target
                .published
                .borrow()
                .iter()
                .find(|e| e.payload.get("label") == Some(&Node::Text(label.into())))
                .and_then(|e| e.payload.get("ring_id").cloned())
                .unwrap_or_else(|| panic!("ring {label}"))
        };
        let (a, b, c) = (ring("alpha"), ring("beta"), ring("gamma"));
        assert!(a != b && b != c && a != c);
        let sent = target.sent.borrow();
        let aim = sent
            .iter()
            .find(|r| r.command.to_string() == "release.rings.AimAt")
            .expect("sent");
        let text = |v: &str| Node::Text(v.into());
        let map = |entries: Vec<(&str, Node)>| {
            Node::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v))
                    .collect(),
            )
        };
        assert_eq!(
            aim.input["maybe"],
            Node::Seq(vec![a.clone(), a.clone(), c.clone()])
        );
        assert_eq!(
            aim.input["holes"],
            Node::Seq(vec![Node::Null, c.clone(), b.clone()])
        );
        assert_eq!(
            aim.input["stages"],
            Node::Seq(vec![
                map(vec![
                    ("name", text("early")),
                    ("rings", Node::Seq(vec![b.clone(), c.clone()]))
                ]),
                map(vec![
                    ("name", text("late")),
                    ("rings", Node::Seq(vec![a.clone()]))
                ]),
            ])
        );
        assert_eq!(
            aim.input["by_pair"],
            map(vec![(
                "canary",
                map(vec![
                    ("primary", c.clone()),
                    ("fallback", b.clone()),
                    ("note", text("y"))
                ])
            )])
        );
        // Each entry point preserves the structured references for its executing runtime.
        ess_conformance::go::emit(&suite).expect("Go structured references");
        ess_conformance::ts::emit(&suite).expect("TypeScript structured references");
        let ir = model();
        ess_conformance::go::emit_with_model(&suite, &ir).expect("Go model and references");
        ess_conformance::ts::emit_with_model(&suite, &ir).expect("TypeScript model and references");
    }

    /// A reference to an instance the timeline has not captured yet at that step.
    #[test]
    fn a_nested_reference_before_its_capture_is_refused() {
        let ir = model();
        let (head, tail) = ARRANGED
            .split_once("  - at: 2026-01-05T09:00:02Z\n")
            .unwrap();
        let text = format!(
            "{head}  - at: 2026-01-05T09:00:02Z\n    command: release.rings.AimAt\n    input:\n      \
             note: first\n      maybe: [{{$instance: a}}, {{$instance: c}}]\n    outcome: aimed\n  \
             - at: 2026-01-05T09:00:03Z\n{tail}"
        );
        let result = authoring(&ir, &text);
        assert!(!result.is_complete());
        assert!(
            result.refusals.iter().any(|r| matches!(
                &r.cause,
                ess_conformance::authored::Cause::UnboundInstance { instance }
                    if instance.to_string() == "c"
            )),
            "{}",
            refusals(&result)
        );
    }

    /// The shape filter drops only what a reference answered: a sibling mistake is still reported.
    #[test]
    fn a_sibling_shape_error_beside_a_reference_is_still_reported() {
        let ir = model();
        let result = authoring(
            &ir,
            &aim("      stages: [{name: 5, rings: [{$instance: a}]}]\n"),
        );
        assert!(!result.is_complete());
        assert!(
            refusals(&result).contains("stages.0.name"),
            "{}",
            refusals(&result)
        );
    }

    /// Hand-written suite bytes: malformed structured values, and the kinds under an older major.
    #[test]
    fn hand_written_structured_values_are_admitted_only_when_well_formed() {
        let ir = model();
        let suite = suite(&ir, &aim(ALL));
        let json = suite.to_canonical_json().unwrap();
        AdmittedSuite::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let (id, _) = value["scenarios"]
            .as_object()
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let id = id.clone();
        let steps = value["scenarios"][&id]["steps"].as_array().unwrap().clone();
        let index = steps
            .iter()
            .position(|s| s["command"] == "release.rings.AimAt")
            .unwrap();
        let mutate = |f: &dyn Fn(&mut serde_json::Value)| {
            let mut v = value.clone();
            f(&mut v["scenarios"][&id]["steps"][index]["input"]);
            v.to_string()
        };
        for (what, bytes) in [
            (
                "items not an array",
                mutate(&|i| i["maybe"]["items"] = json!({"x": 1})),
            ),
            (
                "items missing",
                mutate(&|i| i["maybe"] = json!({"kind": "list"})),
            ),
            ("an extra key", mutate(&|i| i["maybe"]["extra"] = json!(1))),
            (
                "a now_offset element",
                mutate(&|i| i["maybe"]["items"][0] = json!({"kind": "now_offset", "seconds": 1})),
            ),
            (
                "an observed element",
                mutate(&|i| {
                    i["maybe"]["items"][0] = json!({"kind": "observed", "event": "release.rings.RingCreated", "field": "ring_id"});
                }),
            ),
            (
                "a fixture element",
                mutate(&|i| i["maybe"]["items"][0] = json!({"kind": "fixture", "fixture": "x"})),
            ),
            (
                "members as an array",
                mutate(&|i| i["by_pair"]["members"] = json!([])),
            ),
            (
                "an element without kind",
                mutate(&|i| i["maybe"]["items"][0] = json!({"instance": "a"})),
            ),
        ] {
            assert!(
                AdmittedSuite::from_json(&bytes).is_err(),
                "{what}: admitted"
            );
        }
        value["provenance"]["suite_version"] = json!("ess-conformance/30");
        assert!(AdmittedSuite::from_json(&value.to_string()).is_err());
    }

    /// The coverage lane: /33, and the same bytes relabelled /31 (which carries coverage) refused
    /// for the vocabulary (`UnsupportedScenarioValue`), not for a missing inventory.
    #[test]
    fn a_coverage_suite_relabelled_31_is_refused_for_its_vocabulary() {
        use ess_conformance::coverage::{Origins, Scope};
        use ess_conformance::coverage_build::{build, CoverageSource};
        let ir = model();
        let source = CoverageSource::new("aim.yaml", aim(ALL)).unwrap();
        let input = build(&ir, &[source], Scope::System, Origins::Authored)
            .unwrap_or_else(|e| panic!("{e}"));
        let admitted = input.selected();
        assert_eq!(
            admitted.suite().provenance.suite_version.to_string(),
            "ess-conformance/33"
        );
        let older = admitted
            .original_json()
            .replace("\"ess-conformance/33\"", "\"ess-conformance/31\"");
        let error = AdmittedSuite::from_json(&older).expect_err("/31 refused");
        assert!(
            error.to_string().contains("UnsupportedScenarioValue"),
            "{error}"
        );
        ess_conformance::go::emit_input_with_model(&input, &ir).expect("Go coverage references");
        ess_conformance::ts::emit_input_with_model(&input, &ir)
            .expect("TypeScript coverage references");
    }
}
