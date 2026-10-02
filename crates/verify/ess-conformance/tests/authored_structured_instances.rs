//! `{$instance: …}` inside a list, a map value and a struct member of an authored step's input
//! (beyond10x/ess#242).
//!
//! A scalar input took a reference; a `List<ReleaseRingId>` of the same identity type did not, and
//! was refused as `ESS-AUTHOR-015` "expected Uuid, found a mapping", so a command taking several
//! identities could not be authored at all. A reference is now admitted wherever the declared
//! type at its position is the instance's identity type, at any depth, and it resolves to the
//! identity the run bound, as a scalar one does. At any other position it is still refused, and
//! the refusal names the position.
//!
//! Every identity in the case is a different ring, and each ring sits at a different position in
//! each container, so a runner that dropped, repeated, reordered or swapped a reference sends an
//! input the assertions below tell apart from the right one.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Cause, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue, SuiteProvenance,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use ess_primitives::node::Node;
use serde_json::json;

const MODEL: &str = include_str!("fixtures/structured-instances.yaml");

/// The identity a hand-answered `Rollouts` row carries.
const ROLLOUT: &str = "00000000-0000-4000-8000-0000000000aa";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("release.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Three rings, created and captured in order: `a`, `b`, `c`.
const ARRANGED: &str = r"type: ess-scenario/1
domain: release.rings
scenario: rollout-over-three-rings
summary: A rollout names three rings inside a list, a map, a struct and a list of structs.
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

/// The rollout, with every reference at an identity-typed position.
const PLANNED: &str = r"  - at: 2026-01-05T09:00:03Z
    command: release.rings.PlanRollout
    input:
      ring_sequence: [{$instance: a}, {$instance: b}]
      by_stage: {canary: {$instance: b}, general: {$instance: c}}
      pair: {primary: {$instance: c}, fallback: {$instance: a}, note: first}
      pairs:
        - {primary: {$instance: b}, note: second}
        - {primary: {$instance: a}, fallback: {$instance: c}, note: third}
      labels: [alpha, beta]
      tags: {team: core}
    outcome: planned
";

fn document(timeline: &str) -> String {
    format!("{ARRANGED}{timeline}")
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

fn instance(name: &str) -> serde_json::Value {
    json!({"kind": "instance", "instance": name})
}

fn literal(value: serde_json::Value) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert("kind".into(), "literal".into());
    object.insert("value".into(), value);
    serde_json::Value::Object(object)
}

/// The input the rollout step compiles to, as the suite writes it.
fn planned_input(authoring: &Authoring) -> serde_json::Value {
    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let input = scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "release.rings.PlanRollout" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the rollout is invoked");
    serde_json::to_value(input).unwrap()
}

#[test]
fn references_inside_a_list_a_map_and_a_struct_author() {
    let ir = model();
    let result = authoring(&ir, &document(PLANNED));
    assert!(
        result.is_complete(),
        "{}",
        result
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        planned_input(&result),
        json!({
            "ring_sequence": {"kind": "list", "items": [instance("a"), instance("b")]},
            "by_stage": {"kind": "members", "members": {
                "canary": instance("b"),
                "general": instance("c"),
            }},
            "pair": {"kind": "members", "members": {
                "primary": instance("c"),
                "fallback": instance("a"),
                "note": literal(json!("first")),
            }},
            "pairs": {"kind": "list", "items": [
                {"kind": "members", "members": {
                    "primary": instance("b"),
                    "note": literal(json!("second")),
                }},
                {"kind": "members", "members": {
                    "primary": instance("a"),
                    "fallback": instance("c"),
                    "note": literal(json!("third")),
                }},
            ]},
            // A structured value holding no reference is the literal it always was.
            "labels": literal(json!(["alpha", "beta"])),
            "tags": literal(json!({"team": "core"})),
        })
    );
}

/// The rollout step, and each reference's position, rewritten to hold a reference where the
/// declared type is not the ring's identity.
#[test]
fn a_reference_at_a_position_that_is_not_an_identity_is_refused_by_position() {
    let ir = model();
    for (from, to, position, declared) in [
        (
            "labels: [alpha, beta]",
            "labels: [alpha, {$instance: b}]",
            "labels[1]",
            "String",
        ),
        ("note: first", "note: {$instance: a}", "pair.note", "String"),
        (
            "note: third",
            "note: {$instance: c}",
            "pairs[1].note",
            "String",
        ),
        // A map's values were not shape-checked at all, so a reference there was sent to the
        // target as the mapping `{"$instance": "a"}`.
        (
            "tags: {team: core}",
            "tags: {team: core, owner: {$instance: a}}",
            "tags[owner]",
            "String",
        ),
    ] {
        let text = document(&PLANNED.replace(from, to));
        let result = authoring(&ir, &text);
        assert!(result.scenarios.is_empty(), "{to}: a scenario compiled");
        let causes: Vec<&Cause> = result.refusals.iter().map(|it| &it.cause).collect();
        assert_eq!(causes.len(), 1, "{to}: {causes:#?}");
        match causes[0] {
            Cause::InstanceMistyped {
                field,
                declared: written,
                identity,
                ..
            } => {
                assert_eq!(field, position, "{to}");
                assert_eq!(written, declared, "{to}");
                assert_eq!(identity, "release.rings.ReleaseRingId", "{to}");
            }
            other => panic!("{to}: refused for another cause: {other:#?}"),
        }
        assert_eq!(result.refusals[0].code().to_string(), "ESS-AUTHOR-022");
        let message = result.refusals[0].to_string();
        assert!(message.contains(position), "{to}: {message}");
    }
}

#[test]
fn a_nested_reference_to_nothing_bound_is_refused_as_a_scalar_one_is() {
    let ir = model();
    let unbound = document(&PLANNED.replace(
        "ring_sequence: [{$instance: a}, {$instance: b}]",
        "ring_sequence: [{$instance: a}, {$instance: nobody}]",
    ));
    let result = authoring(&ir, &unbound);
    assert!(result.scenarios.is_empty());
    assert!(
        result
            .refusals
            .iter()
            .any(|it| matches!(&it.cause, Cause::UnarrangedInstance { instance, .. } if instance.to_string() == "nobody")),
        "{:#?}",
        result.refusals
    );
}

fn suite(ir: &EssIr, text: &str) -> ConformanceSuite {
    let result = authoring(ir, text);
    assert!(result.is_complete(), "{:#?}", result.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    suite.scenarios = result.scenarios;
    suite.select_fresh_format_for(ir);
    suite
}

#[test]
fn a_suite_carrying_structured_references_claims_the_format_that_reads_them() {
    let ir = model();
    let suite = suite(&ir, &document(PLANNED));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/32"
    );
    let json = suite.to_canonical_json().expect("admitted");
    AdmittedSuite::from_json(&json).unwrap_or_else(|error| panic!("{error}"));
    // Labelled with the major before it, the same bytes are refused rather than read by a reader
    // that would send `{"kind": "list", …}` to the target as a value.
    let older = json.replace("\"ess-conformance/32\"", "\"ess-conformance/30\"");
    assert!(AdmittedSuite::from_json(&older).is_err());
    // A suite without them keeps the format it had.
    let plain = PLANNED.replace(
        "ring_sequence: [{$instance: a}, {$instance: b}]",
        "ring_sequence: []",
    );
    let plain = plain
        .replace(
            "by_stage: {canary: {$instance: b}, general: {$instance: c}}",
            "by_stage: {}",
        )
        .replace(
            "pair: {primary: {$instance: c}, fallback: {$instance: a}, note: first}",
            "pair: {primary: 00000000-0000-4000-8000-000000000009, note: first}",
        )
        .replace(
            "        - {primary: {$instance: b}, note: second}\n        - {primary: {$instance: a}, fallback: {$instance: c}, note: third}\n",
            "        - {primary: 00000000-0000-4000-8000-000000000008, note: second}\n",
        );
    let plain = self::suite(&ir, &document(&plain));
    assert!(plain.provenance.suite_version.major() < 32, "{plain:?}");
}

/// Structured references must survive generation for every conformance runtime.
/// Runtime parity controls separately check the actual resolved callback inputs.
#[test]
fn the_generated_runners_preserve_structured_references() {
    let ir = model();
    let suite = suite(&ir, &document(PLANNED));
    ess_conformance::go::emit(&suite).expect("Go supports structured references");
    ess_conformance::ts::emit(&suite).expect("TypeScript supports structured references");
}

// ---- running it --------------------------------------------------------------------------------

/// The interpreter, with every command it was sent and every event it published written down.
struct Recording {
    inner: Interpreted,
    sent: RefCell<Vec<SemanticCommandRequest>>,
    published: RefCell<Vec<ObservedEvent>>,
    /// Where set, `Rollouts` is answered from the rollout it was sent rather than by the
    /// interpreter, which reads no view: as sent, or with its rings in reverse order.
    rows: Option<bool>,
}

impl Recording {
    fn remember(&self, events: &[ObservedEvent]) {
        let mut published = self.published.borrow_mut();
        for event in events {
            if !published.contains(event) {
                published.push(event.clone());
            }
        }
    }
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
        self.remember(&result.direct_events);
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let Some(reversed) = self.rows else {
            return self.inner.query_view(request);
        };
        let rows: Vec<BTreeMap<String, Node>> = self
            .sent
            .borrow()
            .iter()
            .filter(|sent| sent.command.to_string() == "release.rings.PlanRollout")
            .map(|sent| {
                let mut rings = sent.input["ring_sequence"].clone();
                if let (true, Node::Seq(items)) = (reversed, &mut rings) {
                    items.reverse();
                }
                [
                    ("rollout_id".to_owned(), Node::Text(ROLLOUT.into())),
                    ("ring_sequence".to_owned(), rings),
                ]
                .into_iter()
                .collect::<BTreeMap<String, Node>>()
            })
            .collect();
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let observed = self.inner.observe_events(request)?;
        self.remember(&observed);
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
fn the_reference_runner_sends_each_ring_where_the_author_put_it() {
    let ir = model();
    let suite = suite(&ir, &document(PLANNED));
    let target = Recording {
        inner: Interpreted::for_model(ir),
        sent: RefCell::default(),
        published: RefCell::default(),
        rows: None,
    };
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    for result in &report.scenarios {
        assert_eq!(
            result.status,
            Status::Passed,
            "{}: {:#?}",
            result.scenario,
            result.diagnostics().collect::<Vec<_>>()
        );
    }
    let ring = |label: &str| -> Node {
        target
            .published
            .borrow()
            .iter()
            .find(|event| event.payload.get("label") == Some(&Node::Text(label.into())))
            .and_then(|event| event.payload.get("ring_id").cloned())
            .unwrap_or_else(|| panic!("ring {label} was published"))
    };
    let (a, b, c) = (ring("alpha"), ring("beta"), ring("gamma"));
    assert!(a != b && b != c && a != c, "three different rings");
    let sent = target.sent.borrow();
    let rollout = sent
        .iter()
        .find(|request| request.command.to_string() == "release.rings.PlanRollout")
        .expect("the rollout was sent");
    let text = |value: &str| Node::Text(value.into());
    let map = |entries: Vec<(&str, Node)>| {
        Node::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    };
    let expected: BTreeMap<String, Node> = [
        ("ring_sequence", Node::Seq(vec![a.clone(), b.clone()])),
        (
            "by_stage",
            map(vec![("canary", b.clone()), ("general", c.clone())]),
        ),
        (
            "pair",
            map(vec![
                ("primary", c.clone()),
                ("fallback", a.clone()),
                ("note", text("first")),
            ]),
        ),
        (
            "pairs",
            Node::Seq(vec![
                map(vec![("primary", b.clone()), ("note", text("second"))]),
                map(vec![
                    ("primary", a.clone()),
                    ("fallback", c.clone()),
                    ("note", text("third")),
                ]),
            ]),
        ),
        ("labels", Node::Seq(vec![text("alpha"), text("beta")])),
        ("tags", map(vec![("team", text("core"))])),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value))
    .collect();
    assert_eq!(rollout.input, expected);
}

#[test]
fn a_structured_reference_is_a_value_of_its_own_kind_in_the_suite_vocabulary() {
    let list = ScenarioValue::List {
        items: vec![
            ScenarioValue::instance("a".parse().unwrap()),
            ScenarioValue::literal(Node::Text("x".into())),
        ],
    };
    assert_eq!(
        serde_json::to_value(&list).unwrap(),
        json!({"kind": "list", "items": [instance("a"), literal(json!("x"))]})
    );
    assert_eq!(list.as_literal(), None);
}

/// A row's identity-typed list names its rings by reference too, and the runner compares the
/// row against the identities it bound, in order.
#[test]
fn a_view_row_compares_the_rings_it_names_by_reference_in_order() {
    let ir = model();
    let text = document(&format!(
        "{PLANNED}assert:\n  - view: release.rings.Rollouts\n    \
         contains: {{ring_sequence: [{{$instance: a}}, {{$instance: b}}]}}\n"
    ));
    let suite = suite(&ir, &text);
    let expectation = suite
        .scenarios
        .values()
        .flat_map(|scenario| scenario.steps.iter())
        .find_map(|step| match step {
            ScenarioStep::ExpectView { expectation, .. } => Some(expectation.clone()),
            _ => None,
        })
        .expect("the view is asserted");
    assert_eq!(
        serde_json::to_value(expectation).unwrap(),
        json!({"expect": "contains", "fields": {
            "ring_sequence": {"kind": "list", "items": [instance("a"), instance("b")]},
        }})
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    for (reversed, wanted) in [(false, Status::Passed), (true, Status::Failed)] {
        let target = Recording {
            inner: Interpreted::for_model(ir.clone()),
            sent: RefCell::default(),
            published: RefCell::default(),
            rows: Some(reversed),
        };
        let report = Runner::for_suite(&suite)
            .run_admitted(&admitted, &target)
            .into_report();
        let statuses: Vec<Status> = report.scenarios.iter().map(|it| it.status).collect();
        assert_eq!(statuses, vec![wanted], "reversed: {reversed}");
    }
}

/// The coverage lane lowers the same file into its own suite, `/33`, whose bytes admit and run.
#[test]
fn a_coverage_suite_carrying_structured_references_is_33_and_runs() {
    use ess_conformance::coverage::{Origins, Scope};
    use ess_conformance::coverage_build::{build, CoverageSource};
    let ir = model();
    let source = CoverageSource::new("rollout.yaml", document(PLANNED)).unwrap();
    let input = build(&ir, &[source], Scope::System, Origins::Authored)
        .unwrap_or_else(|error| panic!("{error}"));
    let admitted = input.selected();
    assert_eq!(
        admitted.suite().provenance.suite_version.to_string(),
        "ess-conformance/33"
    );
    AdmittedSuite::from_json(admitted.original_json()).unwrap_or_else(|error| panic!("{error}"));
    let go = ess_conformance::go::emit_input(&input).expect_err("Go refuses");
    assert!(go.to_string().contains("Rust runner"), "{go}");
    let ts = ess_conformance::ts::emit_input(&input).expect_err("TypeScript refuses");
    assert!(ts.to_string().contains("Rust runner"), "{ts}");
    let target = Recording {
        inner: Interpreted::for_model(ir),
        sent: RefCell::default(),
        published: RefCell::default(),
        rows: None,
    };
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(admitted, &target)
        .into_report();
    let statuses: Vec<Status> = report.scenarios.iter().map(|it| it.status).collect();
    assert_eq!(statuses, vec![Status::Passed]);
    let sent = target.sent.borrow();
    let rollout = sent
        .iter()
        .find(|request| request.command.to_string() == "release.rings.PlanRollout")
        .expect("the rollout was sent");
    let Node::Seq(rings) = &rollout.input["ring_sequence"] else {
        panic!("a list was sent: {:?}", rollout.input["ring_sequence"]);
    };
    assert_eq!(rings.len(), 2);
    assert_ne!(rings[0], rings[1]);
    assert!(rings.iter().all(|ring| matches!(ring, Node::Text(_))));
}

/// An event's payload is compared against values the suite carries, so a reference inside one is
/// refused where it sits, as a whole-field one is — not compared as the mapping it is written as.
#[test]
fn a_reference_inside_a_compared_payload_is_refused_by_position() {
    let ir = model();
    let text = document(&PLANNED.replace(
        "    outcome: planned\n",
        "    outcome: planned\n    events:\n      - event: release.rings.RolloutPlanned\n        \
         payload: {ring_sequence: [{$instance: a}, {$instance: b}]}\n",
    ));
    let result = authoring(&ir, &text);
    assert!(result.scenarios.is_empty());
    let fields: Vec<&str> = result
        .refusals
        .iter()
        .map(|refusal| match &refusal.cause {
            Cause::NotComparable { field, .. } => field.as_str(),
            other => panic!("refused for another cause: {other:#?}"),
        })
        .collect();
    assert_eq!(fields, ["ring_sequence[0]", "ring_sequence[1]"]);
}

// ---- correction 1: unions, and positions the shape check does not walk ---------------------------

/// The rollout step with `extra` added to its input (each line indented six spaces).
fn planned_with(extra: &str) -> String {
    document(&PLANNED.replace(
        "      tags: {team: core}\n",
        &format!("      tags: {{team: core}}\n{extra}"),
    ))
}

/// A union's payload is a position typed by the variant its tag names: a reference there is
/// admitted where that variant is the identity, and resolved when the scenario runs.
#[test]
fn a_reference_as_a_union_payload_of_the_identity_variant_authors_and_runs() {
    let ir = model();
    let text = planned_with(
        "      target: {kind: ring, value: {$instance: c}}\n      \
         by_pair: {canary: {primary: {$instance: b}, note: x}}\n",
    );
    let result = authoring(&ir, &text);
    assert!(result.is_complete(), "{:#?}", result.refusals);
    let input = planned_input(&result);
    assert_eq!(
        input["target"],
        json!({"kind": "members", "members": {
            "kind": literal(json!("ring")),
            "value": instance("c"),
        }})
    );
    let suite = suite(&ir, &text);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Recording {
        inner: Interpreted::for_model(ir),
        sent: RefCell::default(),
        published: RefCell::default(),
        rows: None,
    };
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    let statuses: Vec<Status> = report.scenarios.iter().map(|it| it.status).collect();
    assert_eq!(statuses, vec![Status::Passed]);
    let ring = |label: &str| {
        target
            .published
            .borrow()
            .iter()
            .find(|event| event.payload.get("label") == Some(&Node::Text(label.into())))
            .and_then(|event| event.payload.get("ring_id").cloned())
            .expect("the ring was published")
    };
    let sent = target.sent.borrow();
    let rollout = sent
        .iter()
        .find(|request| request.command.to_string() == "release.rings.PlanRollout")
        .expect("the rollout was sent");
    let at = |field: &str, path: &[&str]| {
        path.iter().fold(rollout.input.get(field), |node, key| {
            node.and_then(Node::as_map).and_then(|map| map.get(*key))
        })
    };
    assert_eq!(at("target", &["value"]), Some(&ring("gamma")));
    assert_eq!(at("by_pair", &["canary", "primary"]), Some(&ring("beta")));
}

/// Every container kind, at a member it does not declare and in the wrong shape: each reference
/// that cannot be placed is refused, naming where it sits, and none reaches the target as the
/// mapping `{"$instance": …}` — including under a map value and a union payload, which the shape
/// check does not walk.
#[test]
fn a_reference_nothing_can_place_is_refused_in_every_container() {
    let ir = model();
    for (extra, replaced, named) in [
        // list element, wrong shape
        (
            "",
            Some(("        - {primary: {$instance: b}, note: second}\n", "        - [{$instance: b}]\n")),
            "pairs",
        ),
        // struct member, undeclared
        (
            "",
            Some(("fallback: {$instance: a}, note: first", "fallback: {$instance: a}, fallbak: {$instance: b}, note: first")),
            "fallbak",
        ),
        // Optional struct member, wrong shape
        (
            "",
            Some(("fallback: {$instance: a}, note: first", "fallback: [{$instance: a}], note: first")),
            "pair.fallback",
        ),
        // map value, undeclared member
        (
            "      by_pair: {canary: {primary: {$instance: a}, fallbak: {$instance: b}, note: x}}\n",
            None,
            "fallbak",
        ),
        // map value, wrong shape: the shape check reads a map's values by ordinal in key order
        // (beyond10x/ess#240), so it names the value there
        ("      by_pair: {canary: [{$instance: a}]}\n", None, "by_pair.0"),
        // map value of a map of identities, wrong shape
        (
            "",
            Some(("canary: {$instance: b}", "canary: [{$instance: b}]")),
            "by_stage[canary]",
        ),
        // union payload of a variant that is not the identity
        ("      target: {kind: label, value: {$instance: b}}\n", None, "target.value"),
        // union, undeclared member
        (
            "      target: {kind: ring, value: {$instance: b}, extra: {$instance: c}}\n",
            None,
            "extra",
        ),
        // union, a tag naming no variant
        ("      target: {kind: nope, value: {$instance: b}}\n", None, "target"),
        // union payload, wrong shape
        ("      target: {kind: ring, value: [{$instance: b}]}\n", None, "target.value"),
        // map value inside a union-free struct inside a map value, undeclared
        (
            "      by_pair: {canary: {primary: {$instance: a}, note: {nested: {$instance: b}}}}\n",
            None,
            "by_pair[canary].note",
        ),
    ] {
        let mut text = planned_with(extra);
        if let Some((from, to)) = replaced {
            assert!(text.contains(from), "{from}");
            text = text.replace(from, to);
        }
        let result = authoring(&ir, &text);
        let written = refusals_text(&result);
        assert!(
            result.scenarios.is_empty(),
            "{extra}{replaced:?}: compiled; the rollout sends {}",
            planned_input(&result)
        );
        assert!(
            written.contains(named),
            "{extra}{replaced:?}: refused without naming `{named}`:\n{written}"
        );
    }
}

fn refusals_text(result: &Authoring) -> String {
    result
        .refusals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
