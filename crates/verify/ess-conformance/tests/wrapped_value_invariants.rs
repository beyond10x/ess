//! A value type's invariants observed where a view publishes one inside a wrapper
//! (`docs/design/wrapped-value-invariant-observations.md`), and a text's `.count` asserted on a
//! view row at a direct position.
//!
//! A `Tag` is a `String` newtype with an alphabet and invariants over `value`. A view publishes it
//! at one of four positions: directly, as the element of a `List`, inside an `Optional`, or as the
//! value of a `Map`. A direct position is asserted by the row's `satisfies`; a wrapped one by the
//! typed `value_invariants` observation, which checks every actual occurrence and requires one.
//! Each passes against the interpreter and fails against a target that answers a bad value.
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ConformanceReport, Status};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ViewExpectation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// One `Tag` at the field `{FIELD}` of type `{TYPE}` on the entity, the command's input and the
/// view. `{INVARIANTS}` is the newtype's invariant list.
const MODEL: &str = r#"format: ess/24
system: demo
version: v1
domain: demo.tags
types:
  - {name: demo.tags.BoardId, kind: newtype, of: Uuid}
  - name: demo.tags.Tag
    kind: newtype
    of: String
    alphabet: "abcdefghijklmnopqrstuvwxyz0123456789.-"
    invariants: {INVARIANTS}
entities:
  - name: demo.tags.Board
    identity: {name: board_id, type: demo.tags.BoardId}
    fields:
      - {name: {FIELD}, type: '{TYPE}'}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: demo.tags.BoardOpened
    fields:
      - {name: board_id, type: demo.tags.BoardId}
commands:
  - name: demo.tags.OpenBoard
    input:
      - {name: {FIELD}, type: '{TYPE}'}
    outcomes:
      - name: opened
        creates: demo.tags.Board
        instance: board_id
        sets: {{FIELD}: input.{FIELD}}
        emits: [demo.tags.BoardOpened]
        payload:
          demo.tags.BoardOpened: {board_id: {generated: true}}
views:
  - name: demo.tags.Boards
    source: demo.tags.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.tags.BoardId}
      - {name: {FIELD}, type: '{TYPE}'}
"#;

const COUNT: &str = "[value.count >= 1]";
const NOT_EMPTY: &str = r#"["value != \"\""]"#;

fn model(field: &str, type_ref: &str, invariants: &str) -> String {
    MODEL
        .replace("{FIELD}", field)
        .replace("{TYPE}", type_ref)
        .replace("{INVARIANTS}", invariants)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(ir: &EssIr) -> Synthesis {
    ess_conformance::synthesize::synthesize(ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// The ids of the scenarios that read the type's invariants.
fn invariant_ids(suite: &ConformanceSuite) -> Vec<String> {
    suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with("demo.tags.Tag/"))
        .collect()
}

/// The interpreter, with every row of `demo.tags.Boards` rewritten by `rewrite`.
struct Rewriting {
    inner: Interpreted,
    rewrite: fn(&mut ViewRow),
}

impl ConformanceTarget for Rewriting {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("rewriting-interpreter", "1"))
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
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        for row in &mut result.rows {
            (self.rewrite)(row);
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> ConformanceReport {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
}

/// The status of the one scenario under `prefix`, with its report for a message.
fn status_of(report: &ConformanceReport, prefix: &str) -> (Status, String) {
    let found: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.scenario.to_string().starts_with(prefix))
        .collect();
    assert_eq!(found.len(), 1, "one scenario under {prefix}: {report:#?}");
    (found[0].status, format!("{:#?}", found[0]))
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

/// Part one: a text's `.count` at a direct view position is asserted by the row's `satisfies`
/// (no longer "a `.count`, which a view row does not assert"), under suite/40.
#[test]
fn a_text_count_at_a_direct_view_position_is_asserted_on_the_row() {
    let ir = ir(&model("tag", "demo.tags.Tag", COUNT));
    let synthesis = synthesis(&ir);
    assert!(
        refusals(&synthesis).is_empty(),
        "{:#?}",
        refusals(&synthesis)
    );
    assert_eq!(
        invariant_ids(&synthesis.suite),
        ["demo.tags.Tag/invariant/at/demo.tags.Boards/tag"]
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/40",
        "a text length on a row is suite/40 vocabulary"
    );
    let json = synthesis.suite.to_canonical_json().expect("serializes");
    assert!(json.contains("tag.count"), "{json}");

    let healthy = run(&synthesis.suite, &Interpreted::for_model(ir.clone()));
    let (status, about) = status_of(&healthy, "demo.tags.Tag/");
    assert_eq!(status, Status::Passed, "{about}");

    let empty = Rewriting {
        inner: Interpreted::for_model(ir),
        rewrite: |row| {
            row.insert("tag".into(), text(""));
        },
    };
    let (status, about) = status_of(&run(&synthesis.suite, &empty), "demo.tags.Tag/");
    assert_eq!(status, Status::Failed, "{about}");
}

/// The same rule over a coverage suite: `/41`.
#[test]
fn coverage_suites_take_41_for_a_direct_count_and_53_for_a_list() {
    use ess_conformance::coverage::{Origins, Scope};
    for (field, type_ref, major) in [
        ("tag", "demo.tags.Tag", 41),
        ("tags", "List<demo.tags.Tag>", 53),
    ] {
        let ir = ir(&model(field, type_ref, COUNT));
        let coverage =
            ess_conformance::coverage_build::build(&ir, &[], Scope::System, Origins::Generated)
                .expect("the coverage suite is built");
        assert_eq!(
            coverage.selected().suite().provenance.suite_version.major(),
            major,
            "{type_ref}"
        );
    }
}

/// Part two: the element of a `List` is observed by `value_invariants`, at suite/52.
#[test]
fn a_list_element_position_is_observed_and_holds_on_the_interpreter() {
    for invariants in [COUNT, NOT_EMPTY] {
        let ir = ir(&model("tags", "List<demo.tags.Tag>", invariants));
        let synthesis = synthesis(&ir);
        assert!(
            refusals(&synthesis).is_empty(),
            "{invariants}: {:#?}",
            refusals(&synthesis)
        );
        let ids = invariant_ids(&synthesis.suite);
        assert_eq!(ids.len(), 1, "{ids:?}");
        assert!(
            ids[0].starts_with("demo.tags.Tag/invariant-wrapped/at/demo.tags.Boards/tags/"),
            "{ids:?}"
        );
        assert_eq!(
            synthesis.suite.provenance.suite_version.to_string(),
            "ess-conformance/52"
        );
        let scenario = synthesis.suite.scenarios.values().find(|scenario| {
            scenario.steps.iter().any(|step| {
                matches!(
                    step,
                    ScenarioStep::ExpectView {
                        expectation: ViewExpectation::ValueInvariants(_),
                        ..
                    }
                )
            })
        });
        assert!(scenario.is_some(), "an observation is emitted");

        let report = run(&synthesis.suite, &Interpreted::for_model(ir.clone()));
        let (status, about) = status_of(&report, "demo.tags.Tag/");
        assert_eq!(status, Status::Passed, "{invariants}: {about}");
    }
}

/// A list holding an empty element after a good one fails: every occurrence is checked, not the
/// first.
#[test]
fn a_bad_later_list_element_fails() {
    for invariants in [COUNT, NOT_EMPTY] {
        let ir = ir(&model("tags", "List<demo.tags.Tag>", invariants));
        let synthesis = synthesis(&ir);
        let target = Rewriting {
            inner: Interpreted::for_model(ir),
            rewrite: |row| {
                row.insert("tags".into(), Node::Seq(vec![text("good"), text("")]));
            },
        };
        let (status, about) = status_of(&run(&synthesis.suite, &target), "demo.tags.Tag/");
        assert_eq!(status, Status::Failed, "{invariants}: {about}");
        assert!(
            about.contains("value_invariants") || about.contains("demo.tags.Tag"),
            "{about}"
        );
    }
}

/// An element outside the newtype's alphabet fails the observation's shape check.
#[test]
fn a_list_element_outside_the_alphabet_fails() {
    let ir = ir(&model("tags", "List<demo.tags.Tag>", COUNT));
    let synthesis = synthesis(&ir);
    let target = Rewriting {
        inner: Interpreted::for_model(ir),
        rewrite: |row| {
            row.insert("tags".into(), Node::Seq(vec![text("Not Allowed!")]));
        },
    };
    let (status, about) = status_of(&run(&synthesis.suite, &target), "demo.tags.Tag/");
    assert_eq!(status, Status::Failed, "{about}");
    assert!(about.contains("alphabet"), "{about}");
}

/// An empty list is legal and satisfies every element, but witnesses no occurrence: a target that
/// drops every element fails.
#[test]
fn an_empty_list_is_no_witness() {
    let ir = ir(&model("tags", "List<demo.tags.Tag>", COUNT));
    let synthesis = synthesis(&ir);
    let target = Rewriting {
        inner: Interpreted::for_model(ir),
        rewrite: |row| {
            row.insert("tags".into(), Node::Seq(Vec::new()));
        },
    };
    let (status, about) = status_of(&run(&synthesis.suite, &target), "demo.tags.Tag/");
    assert_eq!(status, Status::Failed, "{about}");
    assert!(about.contains("occurrence"), "{about}");
}

/// A row whose element is not text at all is a malformed representation.
#[test]
fn a_list_element_of_the_wrong_kind_fails() {
    let ir = ir(&model("tags", "List<demo.tags.Tag>", COUNT));
    let synthesis = synthesis(&ir);
    let target = Rewriting {
        inner: Interpreted::for_model(ir),
        rewrite: |row| {
            row.insert("tags".into(), Node::Seq(vec![text("ok"), Node::Bool(true)]));
        },
    };
    let (status, about) = status_of(&run(&synthesis.suite, &target), "demo.tags.Tag/");
    assert_ne!(status, Status::Passed, "{about}");
}

/// The same machinery covers an `Optional` and a `Map` value position.
#[test]
fn optional_and_map_value_positions_are_observed() {
    for (field, type_ref, bad) in [
        (
            "tag",
            "Optional<demo.tags.Tag>",
            (|row: &mut ViewRow| {
                row.insert("tag".into(), text(""));
            }) as fn(&mut ViewRow),
        ),
        (
            "byname",
            "Map<String, demo.tags.Tag>",
            |row: &mut ViewRow| {
                row.insert(
                    "byname".into(),
                    Node::Map(BTreeMap::from([
                        ("a".to_owned(), text("good")),
                        ("b".to_owned(), text("")),
                    ])),
                );
            },
        ),
    ] {
        let ir = ir(&model(field, type_ref, NOT_EMPTY));
        let synthesis = synthesis(&ir);
        assert!(
            refusals(&synthesis).is_empty(),
            "{type_ref}: {:#?}",
            refusals(&synthesis)
        );
        let ids = invariant_ids(&synthesis.suite);
        assert_eq!(ids.len(), 1, "{type_ref}: {ids:?}");
        assert!(ids[0].contains("/invariant-wrapped/"), "{ids:?}");
        let (status, about) = status_of(
            &run(&synthesis.suite, &Interpreted::for_model(ir.clone())),
            "demo.tags.Tag/",
        );
        assert_eq!(status, Status::Passed, "{type_ref}: {about}");
        let target = Rewriting {
            inner: Interpreted::for_model(ir),
            rewrite: bad,
        };
        let (status, about) = status_of(&run(&synthesis.suite, &target), "demo.tags.Tag/");
        assert_eq!(status, Status::Failed, "{type_ref}: {about}");
    }
}

/// A suite without a wrapped position keeps its format: an alphabet alone declares no invariant.
#[test]
fn a_model_without_invariants_keeps_its_format() {
    let ir = ir(&model("tags", "List<demo.tags.Tag>", "[]"));
    let synthesis = synthesis(&ir);
    assert_eq!(invariant_ids(&synthesis.suite), Vec::<String>::new());
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
}

/// A union variant's payload is a position too: the other variant is legal and witnesses nothing.
#[test]
fn a_union_variant_position_is_observed() {
    let source = model("holder", "demo.tags.Holder", NOT_EMPTY).replace(
        "entities:",
        "  - name: demo.tags.Holder\n    kind: union\n    tag: kind\n    variants: {named: \
         demo.tags.Tag, plain: String}\nentities:",
    );
    let ir = ir(&source);
    let synthesis = synthesis(&ir);
    assert!(
        refusals(&synthesis).is_empty(),
        "{:#?}",
        refusals(&synthesis)
    );
    let ids = invariant_ids(&synthesis.suite);
    assert_eq!(ids.len(), 1, "{ids:?}");
    assert!(ids[0].contains("/invariant-wrapped/"), "{ids:?}");
    let (status, about) = status_of(
        &run(&synthesis.suite, &Interpreted::for_model(ir.clone())),
        "demo.tags.Tag/",
    );
    assert_eq!(status, Status::Passed, "{about}");
    for (rewrite, wanted) in [
        (
            (|row: &mut ViewRow| {
                row.insert(
                    "holder".into(),
                    Node::Map(BTreeMap::from([
                        ("kind".to_owned(), text("named")),
                        ("value".to_owned(), text("")),
                    ])),
                );
            }) as fn(&mut ViewRow),
            "does not hold",
        ),
        (
            |row: &mut ViewRow| {
                row.insert(
                    "holder".into(),
                    Node::Map(BTreeMap::from([
                        ("kind".to_owned(), text("plain")),
                        ("value".to_owned(), text("anything")),
                    ])),
                );
            },
            "occurrence",
        ),
    ] {
        let target = Rewriting {
            inner: Interpreted::for_model(ir.clone()),
            rewrite,
        };
        let (status, about) = status_of(&run(&synthesis.suite, &target), "demo.tags.Tag/");
        assert_eq!(status, Status::Failed, "{about}");
        assert!(about.contains(wanted), "{wanted}: {about}");
    }
}

/// An entity invariant reading a published list through a quantifier is refused for what it is —
/// a list read element by element, which a view row does not assert — never as a field "published
/// by no view", and the help names the remedy that works: the element type's own invariants.
#[test]
fn a_quantifier_over_a_published_list_is_named_as_such() {
    let source = model("tags", "List<demo.tags.Tag>", "[]").replace(
        "    lifecycle:\n      initial: Open",
        "    invariants:\n      - {forall: {in: tags, as: t, that: 't != \"\"'}}\n    lifecycle:\n      \
         initial: Open",
    );
    let synthesis = synthesis(&ir(&source));
    let about: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-011"))
        .collect();
    assert_eq!(about.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(!about[0].contains("published by no view"), "{}", about[0]);
    assert!(about[0].contains("`tags` is published"), "{}", about[0]);
    assert!(about[0].contains("forall"), "{}", about[0]);
    assert!(about[0].contains("`invariants:` on"), "{}", about[0]);
}

/// The ESS-SYNTH-013 help names only remedies that answer: a type no view publishes is published,
/// and nothing suggests restating it as an entity invariant over a list a row cannot assert.
#[test]
fn the_unpublished_value_type_help_names_publishing_it() {
    // The view's last field is the one that published the type; without it no view does.
    let mut source = model("tags", "List<demo.tags.Tag>", COUNT);
    let published = source
        .rfind("      - {name: tags, type: 'List<demo.tags.Tag>'}")
        .expect("the view publishes the list");
    source.truncate(published);
    let synthesis = synthesis(&ir(&source));
    let about: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-013"))
        .collect();
    assert_eq!(about.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(about[0].contains("no view publishes"), "{}", about[0]);
    assert!(!about[0].contains("entity invariant"), "{}", about[0]);
    assert!(!about[0].contains("`.count`"), "{}", about[0]);
}

fn list_suite() -> (EssIr, ConformanceSuite) {
    let ir = ir(&model("tags", "List<demo.tags.Tag>", COUNT));
    let synthesis = synthesis(&ir);
    (ir, synthesis.suite)
}

/// The one wrapped observation of a suite, for a test that edits it.
fn observation(
    suite: &mut ConformanceSuite,
) -> &mut ess_conformance::value_invariant::ValueInvariants {
    suite
        .scenarios
        .values_mut()
        .flat_map(|scenario| scenario.steps.iter_mut())
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::ValueInvariants(observed),
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::ValueInvariants(observed),
                ..
            } => Some(&mut **observed),
            _ => None,
        })
        .expect("the suite observes one")
}

/// A reader below suite/52 refuses the expectation by version; the id must name the position the
/// expectation carries; the declarations are exactly the reachable closure.
#[test]
fn admission_refuses_an_older_major_a_forged_digest_and_an_unreached_declaration() {
    let (_, suite) = list_suite();
    let json = suite.to_canonical_json().expect("serializes");
    AdmittedSuite::from_json(&json).expect("the generated suite is admitted");

    let older = json.replace("\"ess-conformance/52\"", "\"ess-conformance/50\"");
    let refused = AdmittedSuite::from_json(&older).expect_err("/50 does not carry it");
    assert!(refused.to_string().contains("suite/52"), "{refused}");

    let mut forged: serde_json::Value = serde_json::from_str(&json).unwrap();
    let scenarios = forged["scenarios"].as_object_mut().unwrap();
    let id = scenarios
        .keys()
        .find(|id| id.contains("/invariant-wrapped/"))
        .cloned()
        .unwrap();
    let scenario = scenarios.remove(&id).unwrap();
    scenarios.insert(
        format!("{}{}", &id[..id.len() - 64], "0".repeat(64)),
        scenario,
    );
    let refused = AdmittedSuite::from_json(&forged.to_string()).expect_err("digest forged");
    assert!(refused.to_string().contains("filed under"), "{refused}");

    let mut extra = suite.clone();
    observation(&mut extra).declarations.insert(
        "demo.tags.Unrelated".to_owned(),
        ess_conformance::value_invariant::Declaration::Enum {
            variants: vec!["one".to_owned()],
        },
    );
    let refused = AdmittedSuite::from_suite(&extra).expect_err("an unreached declaration");
    assert!(refused.to_string().contains("not reached"), "{refused}");

    let mut unknown: serde_json::Value = serde_json::from_str(&json).unwrap();
    for scenario in unknown["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["expectation"]["expect"] == "value_invariants" {
                step["expectation"]["extra"] = serde_json::json!(1);
            }
        }
    }
    AdmittedSuite::from_json(&unknown.to_string()).expect_err("a closed member list");
}

/// A witness form this runner does not execute is reported unsupported, naming the expectation.
#[test]
fn an_identity_query_witness_is_reported_unsupported() {
    use ess_conformance::value_invariant::{Selector, Witness};
    let (ir, mut suite) = list_suite();
    let observed = observation(&mut suite);
    let Witness::ArrangedRow {
        selector:
            Selector::ProjectedIdentity {
                entity,
                identity,
                expected,
            },
    } = observed.witness.clone()
    else {
        panic!("the generated witness projects the identity: {observed:#?}");
    };
    observed.witness = Witness::ArrangedRow {
        selector: Selector::IdentityQuery {
            entity,
            identity: identity.clone(),
            parameter: identity,
            filter: ess_primitives::predicate::Predicate::Always,
            expected,
        },
    };
    let report = run(&suite, &Interpreted::for_model(ir));
    let (status, about) = status_of(&report, "demo.tags.Tag/");
    assert_eq!(status, Status::Unsupported, "{about}");
    assert!(about.contains("value_invariants"), "{about}");
}

/// An exhausted work budget is a nonpassing observation error, never a pass.
#[test]
fn an_exhausted_work_budget_is_an_error() {
    let (ir, mut suite) = list_suite();
    observation(&mut suite).work_limit = 3;
    let report = run(&suite, &Interpreted::for_model(ir.clone()));
    let (status, about) = status_of(&report, "demo.tags.Tag/");
    assert_eq!(status, Status::Error, "{about}");
    assert!(about.contains("work budget"), "{about}");
    // The default budget is far from this one.
    let (_, suite) = list_suite();
    let (status, _) = status_of(&run(&suite, &Interpreted::for_model(ir)), "demo.tags.Tag/");
    assert_eq!(status, Status::Passed);
}

/// The browser product runs the same admitted suite through the shared runner.
struct Browser;

impl ess_conformance::web_execution::Installation for Browser {
    type Target = Interpreted;
    type Clock = ess_conformance::AdvancingClock;
    fn create(
        _: &ess_conformance::web_execution::RunContext,
    ) -> ess_conformance::web_execution::Result<
        ess_conformance::web_execution::Installed<Self::Target, Self::Clock>,
    > {
        Ok(ess_conformance::web_execution::Installed {
            target: Interpreted::for_model(ir(&model("tags", "List<demo.tags.Tag>", COUNT))),
            clock: ess_conformance::AdvancingClock::default(),
            config: ess_conformance::RunnerConfig::default(),
        })
    }
}

#[test]
fn the_browser_product_admits_and_runs_the_observation() {
    use ess_conformance::web_execution::{bundle, Product};
    let (_, suite) = list_suite();
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let (manifest, blobs) = bundle::create(
        &[bundle::SourceDocument {
            path: "sources/0000.yaml".into(),
            text: model("tags", "List<demo.tags.Tag>", COUNT),
        }],
        &bundle::Execution::Ordinary(admitted),
    )
    .expect("bundles");
    let mut product = Product::<Browser>::new();
    let handle = product
        .load(&manifest, blobs)
        .expect("a /52 suite is admitted");
    let completed = product.run(handle, [7; 16], 1).expect("runs");
    let report: serde_json::Value = serde_json::from_str(&completed.report).expect("JSON");
    let passed = report["outcomes"]["passed"].to_string();
    assert!(passed.contains("/invariant-wrapped/"), "{report:#}");
    for (status, ids) in report["outcomes"].as_object().unwrap() {
        if status != "passed" {
            assert_eq!(
                ids.as_array().map(Vec::len),
                Some(0),
                "{status}: {report:#}"
            );
        }
    }
}
