//! Closed input coverage must use the producer's six real values.
mod support_go;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{flatten, when, Decision, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;
use std::collections::BTreeMap;
use std::fmt::Write as _;

const VALUES: [&str; 6] = [
    "Offline",
    "LoggingIn",
    "Waiting",
    "Idle",
    "InProgress",
    "Disposition",
];
fn model(extra: &str) -> String {
    let mut text = String::from("format: ess/1\nsystem: reporting\nversion: v1\ndomain: reporting.core\ntypes:\n  - name: reporting.core.Status\n    kind: enum\n    variants: [Offline, LoggingIn, Waiting, Idle, InProgress, Disposition]\n  - name: reporting.core.Wrapped\n    kind: newtype\n    of: reporting.core.Status\nevents:\n  - name: reporting.core.Observed\n    fields: []\ncommands:\n");
    for command in ["Report", "Refresh"] {
        write!(text, "  - name: reporting.core.{command}\n    input:\n      - name: status\n        type: reporting.core.Status\n      - name: recipient\n        type: String\n{extra}    outcomes:\n").unwrap();
        for (index, value) in VALUES.iter().enumerate() {
            write!(text, "      - name: state-{index}\n        when: status == {value}\n        emits: [reporting.core.Observed]\n").unwrap();
        }
    }
    text
}
fn assemble(text: &str) -> Result<EssIr, String> {
    let raw = RawSpecFile::parse(text).map_err(|e| e.to_string())?;
    let spec =
        Specification::assemble([(Source::new("finite.yaml"), raw)]).map_err(|e| e.to_string())?;
    compile(&spec, &SourceMap::new()).map_err(|e| e.to_string())
}
fn assert_all_witnesses(text: &str) {
    let ir = assemble(text).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    assert_eq!(result.suite.scenarios.len(), 12);
    let mut seen = BTreeMap::<String, Vec<Node>>::new();
    for (id, scenario) in &result.suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
                let input = input
                    .iter()
                    .map(|(name, value)| {
                        let ScenarioValue::Literal { value } = value else {
                            panic!("concrete witness")
                        };
                        (name.clone(), value.clone())
                    })
                    .collect();
                let declared = ir.commands().get(command.name()).unwrap();
                let facts = flatten(&ir, declared, &input).unwrap();
                let matching: Vec<_> = declared
                    .outcomes
                    .iter()
                    .filter(|outcome| {
                        when(outcome)
                            .is_some_and(|guard| matches!(facts.decide(guard), Decision::Satisfied))
                    })
                    .collect();
                assert_eq!(matching.len(), 1, "witness must select exactly one branch");
                assert!(id.to_string().ends_with(matching[0].name.as_str()));
                let status = match &input["status"] {
                    Node::Map(fields) => fields["value"].clone(),
                    value => value.clone(),
                };
                seen.entry(command.to_string()).or_default().push(status);
            }
        }
    }
    assert_eq!(seen.len(), 2);
    for values in seen.values() {
        assert_eq!(values.len(), 6);
        for value in VALUES {
            assert!(values.contains(&Node::Text(value.into())));
        }
    }
}
#[test]
fn two_six_value_shapes_have_unique_executable_branch_witnesses() {
    assert_all_witnesses(&model(""));
}
#[test]
fn transparent_wrappers_preserve_the_real_domain() {
    assert_all_witnesses(&model("").replace(
        "type: reporting.core.Status",
        "type: reporting.core.Wrapped",
    ));
}
#[test]
fn missing_variant_reports_a_real_uncovered_value() {
    let text = model("").replace("      - name: state-5\n        when: status == Disposition\n        emits: [reporting.core.Observed]\n", "");
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("Disposition"), "{errors}");
    assert!(errors.contains("uncovered"), "{errors}");
}
#[test]
fn overlap_cannot_prove_unique_coverage() {
    let text = model("").replace("when: status == Offline", "when: status != Disposition");
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("overlap"), "{errors}");
    assert!(errors.contains("LoggingIn"), "{errors}");
}
#[test]
fn optional_domain_does_not_hide_absence() {
    let text = model("").replace(
        "type: reporting.core.Status",
        "type: Optional<reporting.core.Status>",
    );
    assert!(assemble(&text).is_err());
}
#[test]
fn an_extra_open_input_conjunct_does_not_prove_coverage() {
    let text = model("").replace(
        "when: status == Offline",
        "when:\n          all: [status == Offline, recipient == alice]",
    );
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("says nothing about"), "{errors}");
}
#[test]
fn a_real_default_retains_its_reachable_witness() {
    let text = model("").replace("        when: status == Disposition\n", "");
    assert_all_witnesses_with_default(&text);
}
fn assert_all_witnesses_with_default(text: &str) {
    let ir = assemble(text).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    assert_eq!(result.suite.scenarios.len(), 12);
}
#[test]
fn retained_unreachable_branch_is_still_refused() {
    let text = model("").replace("    outcomes:\n", "    outcomes:\n      - name: impossible\n        when: false\n        emits: [reporting.core.Observed]\n");
    let ir = assemble(&text).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert_eq!(result.refusals.len(), 2, "{:?}", result.refusals);
    assert!(result.refusals.iter().all(|r| r
        .scenario
        .as_ref()
        .unwrap()
        .to_string()
        .ends_with("/impossible")));
    assert_eq!(result.suite.scenarios.len(), 12);
}
#[test]
fn external_branch_does_not_replace_or_interfere_with_enum_coverage() {
    let text = model("").replace("    outcomes:\n", "    outcomes:\n      - name: unavailable\n        external: provider unavailable\n        emits: [reporting.core.Observed]\n");
    let ir = assemble(&text).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    assert_eq!(result.suite.scenarios.len(), 14);
}
#[test]
fn struct_paths_are_populated_by_the_same_proof_assignments() {
    let text = model("").replace("events:\n", "  - name: reporting.core.Envelope\n    kind: struct\n    fields:\n      - name: value\n        type: reporting.core.Wrapped\nevents:\n")
        .replace("type: reporting.core.Status", "type: reporting.core.Envelope")
        .replace("when: status ==", "when: status.value ==");
    assert_all_witnesses(&text);
}
#[test]
fn an_extra_enum_conjunct_has_a_concrete_uncovered_assignment() {
    let text = model("      - name: mode\n        type: reporting.core.Status\n").replace(
        "when: status == Offline",
        "when:\n          all: [status == Offline, mode == Offline]",
    );
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("uncovered"), "{errors}");
    assert!(errors.contains("mode = LoggingIn"), "{errors}");
    assert!(errors.contains("status = Offline"), "{errors}");
}
#[test]
fn complete_joint_enum_domain_yields_real_witnesses() {
    let mut text = model("      - name: mode\n        type: reporting.core.Status\n");
    for value in VALUES {
        text = text.replace(&format!("when: status == {value}\n"), &format!("when:\n          all:\n            - status == {value}\n            - mode: {{any_of: [Offline, LoggingIn, Waiting, Idle, InProgress, Disposition]}}\n"));
    }
    assert_all_witnesses(&text);
}
#[test]
fn oversized_joint_domain_is_not_mistaken_for_complete_search() {
    let mut text = model("      - name: mode\n        type: reporting.core.Status\n      - name: phase\n        type: reporting.core.Status\n");
    for value in VALUES {
        text = text.replace(&format!("when: status == {value}\n"), &format!("when:\n          all:\n            - status == {value}\n            - mode: {{any_of: [Offline, LoggingIn, Waiting, Idle, InProgress, Disposition]}}\n            - phase: {{any_of: [Offline, LoggingIn, Waiting, Idle, InProgress, Disposition]}}\n"));
    }
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("says nothing about"), "{errors}");
}
#[test]
fn absence_and_null_stay_unknown_in_the_existing_evaluator() {
    let text = model("")
        .replace(
            "type: reporting.core.Status",
            "type: Optional<reporting.core.Status>",
        )
        .replace("        when: status == Disposition\n", "");
    let ir = assemble(&text).unwrap();
    let command = ir.commands().values().next().unwrap();
    let guard = command.outcomes.iter().find_map(when).unwrap();
    for mut input in [BTreeMap::new(), [("status".into(), Node::Null)].into()] {
        input.insert("recipient".into(), Node::Text("recipient".into()));
        let facts = flatten(&ir, command, &input).unwrap();
        assert!(matches!(facts.decide(guard), Decision::Unevaluable(_)));
    }
}
#[test]
fn duplicate_guards_report_both_overlap_and_the_omitted_value() {
    let text = model("").replace("when: status == LoggingIn", "when: status == Offline");
    let errors = assemble(&text).unwrap_err();
    assert!(
        errors.contains("overlap for declared input status = Offline"),
        "{errors}"
    );
    assert!(errors.contains("uncovered declared input"), "{errors}");
    assert!(errors.contains("status = LoggingIn"), "{errors}");
}
#[test]
fn predicate_node_budget_is_checked_independently_of_domain_size() {
    use ess_domain::command::finite::{paths, MAX_PREDICATE_NODES};
    use ess_primitives::predicate::Predicate;
    let equality = Predicate::parse_expression("status == Offline").unwrap();
    let within = Predicate::All(vec![equality.clone(); MAX_PREDICATE_NODES - 1]);
    let beyond = Predicate::All(vec![equality; MAX_PREDICATE_NODES]);
    assert!(paths(&[&within]).is_some());
    assert!(paths(&[&beyond]).is_none());
}

fn assert_restricted_wrapper_never_witnesses_excluded_variant(default: bool) {
    let mut text = model("")
        .replace(
            "of: reporting.core.Status\n",
            "of: reporting.core.Status\n    invariants: [value != Offline]\n",
        )
        .replace(
            "type: reporting.core.Status",
            "type: reporting.core.Wrapped",
        );
    if default {
        text = text.replace("        when: status == Disposition\n", "");
    }
    let ir = assemble(&text).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    for (id, scenario) in &result.suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                assert_ne!(
                    input.get("status"),
                    Some(&ScenarioValue::Literal { value: Node::Text("Offline".into()) }),
                    "{id} claims a reachable branch using a value excluded by Wrapped's invariant; refusals: {:?}",
                    result.refusals
                );
            }
        }
    }
    assert_eq!(
        result.suite.scenarios.len(),
        10,
        "retain every valid branch"
    );
    let rejected: Vec<_> = result
        .refusals
        .iter()
        .filter(|refusal| refusal.scenario.is_some())
        .collect();
    assert_eq!(rejected.len(), 2, "{:?}", result.refusals);
    for refusal in rejected {
        assert!(refusal
            .scenario
            .as_ref()
            .unwrap()
            .to_string()
            .ends_with("/outcome/state-0"));
        assert!(
            matches!(
                refusal.cause,
                ess_conformance::synthesize::RefusalCause::GuardUnsatisfiable { tried: 5, .. }
            ),
            "{refusal:?}"
        );
    }
    for (id, scenario) in &result.suite.scenarios {
        if id.to_string().ends_with("/outcome/state-5") {
            assert!(scenario.steps.iter().any(|step| matches!(step,
                ScenarioStep::ExecuteCommand { input, .. }
                if input.get("status") == Some(&ScenarioValue::Literal { value: Node::Text("Disposition".into()) })
            )));
        }
    }
}

#[test]
fn adversary_finite_wrapper_witness_respects_declared_invariant() {
    assert_restricted_wrapper_never_witnesses_excluded_variant(false);
}

#[test]
fn adversary_legacy_default_wrapper_witness_respects_declared_invariant() {
    assert_restricted_wrapper_never_witnesses_excluded_variant(true);
}

#[test]
fn adversary_optional_container_cannot_prove_required_leaf_coverage() {
    let text = model("")
        .replace("events:\n", "  - name: reporting.core.Envelope\n    kind: struct\n    fields:\n      - name: value\n        type: reporting.core.Wrapped\nevents:\n")
        .replace("type: reporting.core.Status", "type: Optional<reporting.core.Envelope>")
        .replace("when: status ==", "when: status.value ==");
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("says nothing about"), "{errors}");
}

#[test]
fn adversary_raw_named_scalar_deferral_does_not_escape_full_validation() {
    let text = model("").replace(
        "kind: enum\n    variants: [Offline, LoggingIn, Waiting, Idle, InProgress, Disposition]",
        "kind: newtype\n    of: String",
    );
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("says nothing about"), "{errors}");
}

#[test]
fn adversary_repeated_same_fact_is_one_correlated_domain() {
    let mut text = model("");
    for value in VALUES {
        text = text.replace(&format!("when: status == {value}\n"), &format!("when:\n          all:\n            - status == {value}\n            - status: {{any_of: [{value}]}}\n"));
    }
    assert_all_witnesses(&text);
}

#[test]
fn adversary_undeclared_variant_inside_membership_is_not_coverage() {
    let text = model("").replace(
        "when: status == Offline",
        "when:\n          status: {any_of: [Offline, Fictional]}",
    );
    let errors = assemble(&text).unwrap_err();
    assert!(errors.contains("Fictional"), "{errors}");
}

#[test]
fn invariant_filter_preserves_unconstrained_candidate_order_and_bytes() {
    use ess_conformance::witness::{candidates, Distinction};
    let expected = format!(
        "[{}]",
        VALUES
            .iter()
            .map(|value| format!("{{\"recipient\":\"recipient\",\"status\":\"{value}\"}}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    for default in [false, true] {
        let text = if default {
            model("").replace("        when: status == Disposition\n", "")
        } else {
            model("")
        };
        let ir = assemble(&text).unwrap();
        for command in ir.commands().values() {
            let guards: Vec<_> = command.outcomes.iter().filter_map(when).collect();
            let inputs = candidates(&ir, command, &guards, Distinction::PLAIN).unwrap();
            assert_eq!(serde_json::to_string(&inputs).unwrap(), expected);
        }
    }
}

#[test]
fn invariant_filter_with_no_admitted_candidate_reports_zero_guard_trials() {
    for default in [false, true] {
        let mut text = model("")
            .replace("of: reporting.core.Status\n", "of: reporting.core.Status\n    invariants: ['value == Offline', 'value != Offline']\n")
            .replace("type: reporting.core.Status", "type: reporting.core.Wrapped");
        if default {
            text = text.replace("        when: status == Disposition\n", "");
        }
        let ir = assemble(&text).unwrap();
        let result = ess_conformance::synthesize::synthesize(&ir);
        assert!(result.suite.scenarios.is_empty());
        let rejected: Vec<_> = result
            .refusals
            .iter()
            .filter(|refusal| refusal.scenario.is_some())
            .collect();
        assert_eq!(rejected.len(), 12, "{:?}", result.refusals);
        for refusal in rejected {
            assert!(
                matches!(
                    refusal.cause,
                    ess_conformance::synthesize::RefusalCause::GuardUnsatisfiable { tried: 0, .. }
                ),
                "{refusal:?}"
            );
        }
    }
}

mod issue_298 {
    use super::*;
    use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};
    use ess_domain::command::finite::{self, FieldGuard, StateGuard};
    use ess_primitives::{facts::FactValue, predicate::Predicate};

    fn boolean_model() -> String {
        "format: ess/1\nsystem: reporting\nversion: v1\ndomain: reporting.core\ntypes:\n  - name: reporting.core.Flag\n    kind: newtype\n    of: Boolean\n  - name: reporting.core.WrappedFlag\n    kind: newtype\n    of: reporting.core.Flag\n  - name: reporting.core.Envelope\n    kind: struct\n    fields:\n      - {name: value, type: reporting.core.WrappedFlag}\nevents:\n  - name: reporting.core.Observed\n    fields: []\ncommands:\n  - name: reporting.core.Report\n    input:\n      - {name: pause, type: Boolean}\n    outcomes:\n      - name: paused\n        when: pause == true\n        emits: [reporting.core.Observed]\n      - name: resumed\n        when: pause == false\n        emits: [reporting.core.Observed]\n".into()
    }

    fn assert_executable(text: &str, expected: usize) -> EssIr {
        let ir = assemble(text).unwrap();
        let result = ess_conformance::synthesize::synthesize(&ir);
        assert!(result.refusals.is_empty(), "{:?}", result.refusals);
        assert_eq!(result.suite.scenarios.len(), expected);
        let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
        let report = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
        assert_eq!(report.scenarios.len(), expected);
        assert!(
            report.scenarios.iter().all(|s| s.status == Status::Passed),
            "{report:?}"
        );
        ir
    }

    #[test]
    fn boolean_no_default_partition_has_two_typed_witnesses() {
        let ir = assert_executable(&boolean_model(), 2);
        let command = ir.commands().values().next().unwrap();
        let guards: Vec<_> = command.outcomes.iter().filter_map(when).collect();
        let cases = finite::analyze(
            &ess_compiler::expression::Environment::new(&ir, &command.input),
            &guards,
        )
        .unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(
            serde_json::to_string(&cases[0].values).unwrap(),
            r#"{"pause":false}"#
        );
        assert_eq!(
            serde_json::to_string(&cases[1].values).unwrap(),
            r#"{"pause":true}"#
        );
        let inputs = ess_conformance::witness::candidates(
            &ir,
            command,
            &guards,
            ess_conformance::witness::Distinction::PLAIN,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&inputs).unwrap(),
            r#"[{"pause":false},{"pause":true}]"#
        );
    }

    #[test]
    fn boolean_transparent_wrapper_preserves_closed_domain() {
        assert_executable(
            &boolean_model().replace(
                "name: pause, type: Boolean",
                "name: pause, type: reporting.core.WrappedFlag",
            ),
            2,
        );
    }

    #[test]
    fn boolean_struct_path_preserves_closed_domain() {
        let text = boolean_model()
            .replace(
                "name: pause, type: Boolean",
                "name: pause, type: reporting.core.Envelope",
            )
            .replace("when: pause ==", "when: pause.value ==");
        assert_executable(&text, 2);
        for optional in [
            text.replace(
                "name: pause, type: reporting.core.Envelope",
                "name: pause, type: Optional<reporting.core.Envelope>",
            ),
            text.replace(
                "name: value, type: reporting.core.WrappedFlag",
                "name: value, type: Optional<reporting.core.WrappedFlag>",
            ),
            boolean_model().replace(
                "name: pause, type: Boolean",
                "name: pause, type: Optional<Boolean>",
            ),
        ] {
            assert!(assemble(&optional).is_err());
        }
    }

    #[test]
    fn mixed_enum_boolean_partition_covers_the_product() {
        let text = model("      - name: pause\n        type: Boolean\n");
        let mut joint = text.clone();
        for (index, value) in VALUES.iter().enumerate() {
            let original = format!("      - name: state-{index}\n        when: status == {value}\n        emits: [reporting.core.Observed]\n");
            let mut branches = String::new();
            for flag in [false, true] {
                write!(branches, "      - name: state-{index}-{flag}\n        when:\n          all: [status == {value}, pause == {flag}]\n        emits: [reporting.core.Observed]\n").unwrap();
            }
            joint = joint.replace(&original, &branches);
        }
        assert_executable(&joint, 24);
        let missing = joint.replace("      - name: state-0-false\n        when:\n          all: [status == Offline, pause == false]\n        emits: [reporting.core.Observed]\n", "");
        let error = assemble(&missing).unwrap_err();
        assert!(
            error.contains("uncovered")
                && error.contains("pause = false")
                && error.contains("status = Offline"),
            "{error}"
        );
        let overlap = joint.replace(
            "all: [status == Offline, pause == true]",
            "all: [status == Offline, pause == false]",
        );
        let error = assemble(&overlap).unwrap_err();
        assert!(
            error.contains("overlap") && error.contains("pause = false"),
            "{error}"
        );
    }

    #[test]
    fn boolean_membership_negation_and_truthiness_are_typed() {
        for (yes, no) in [
            ("pause != false", "pause != true"),
            ("{pause: {any_of: [true]}}", "{pause: {none_of: [true]}}"),
            ("pause", "{not: pause}"),
            (
                "{any: [pause == true, false]}",
                "{all: [pause == false, true]}",
            ),
        ] {
            assert_executable(
                &boolean_model()
                    .replace("pause == true", yes)
                    .replace("pause == false", no),
                2,
            );
        }
        let ir = assemble(&boolean_model().replace("        when: pause == false\n", "")).unwrap();
        let command = ir.commands().values().next().unwrap();
        let reversed = Predicate::Compare {
            left: ess_primitives::predicate::Operand::Literal(FactValue::Bool(true)),
            op: ess_primitives::predicate::CompareOp::Eq,
            right: ess_primitives::predicate::Operand::Fact("pause".parse().unwrap()),
        };
        let cases = finite::analyze(
            &ess_compiler::expression::Environment::new(&ir, &command.input),
            &[&reversed],
        )
        .unwrap();
        assert!(cases[0].selected.is_empty());
        assert_eq!(cases[1].selected, [0]);
    }

    #[test]
    fn mixed_domain_bounds_remain_64_assignments_and_128_nodes() {
        let mut fields = String::new();
        for i in 0..7 {
            writeln!(fields, "      - {{name: b{i}, type: Boolean}}").unwrap();
        }
        let text = boolean_model()
            .replace("      - {name: pause, type: Boolean}", &fields)
            .replace("pause == true", "b0 == true")
            .replace("        when: pause == false\n", "");
        let ir = assemble(&text).unwrap();
        let command = ir.commands().values().next().unwrap();
        let environment = ess_compiler::expression::Environment::new(&ir, &command.input);
        let tautologies = (0..7)
            .map(|i| Predicate::parse_expression(&format!("b{i} == true")).unwrap())
            .collect::<Vec<_>>();
        let six = Predicate::All(tautologies[..6].to_vec());
        let seven = Predicate::All(tautologies.clone());
        assert_eq!(finite::analyze(&environment, &[&six]).unwrap().len(), 64);
        assert!(finite::analyze(&environment, &[&seven]).is_none());
        let nodes = Predicate::All(vec![tautologies[0].clone(); 127]);
        assert_eq!(finite::analyze(&environment, &[&nodes]).unwrap().len(), 2);
        let too_many = Predicate::All(vec![tautologies[0].clone(); 128]);
        assert!(finite::analyze(&environment, &[&too_many]).is_none());
        let states = ["One".parse().unwrap(), "Two".parse().unwrap()].into();
        let five = Predicate::All(tautologies[..5].to_vec());
        assert_eq!(
            finite::analyze_with_states(
                &environment,
                &[StateGuard {
                    states: None,
                    predicate: Some(&five)
                }],
                &states
            )
            .unwrap()
            .len(),
            64
        );
        assert!(finite::analyze_with_states(
            &environment,
            &[StateGuard {
                states: None,
                predicate: Some(&six)
            }],
            &states
        )
        .is_none());
        assert_eq!(
            finite::analyze_with_fields(
                &environment,
                &environment,
                &[FieldGuard {
                    fields: Some(&tautologies[0]),
                    input: Some(&five)
                }]
            )
            .unwrap()
            .len(),
            64
        );
        assert!(finite::analyze_with_fields(
            &environment,
            &environment,
            &[FieldGuard {
                fields: Some(&tautologies[0]),
                input: Some(&six)
            }]
        )
        .is_none());
        for variants in [32, 33] {
            let text = boolean_model().replace("types:\n", &format!("types:\n  - name: reporting.core.Mode\n    kind: enum\n    variants: [{}]\n", (0..variants).map(|i| format!("V{i}")).collect::<Vec<_>>().join(", ")))
                .replace("    input:\n", "    input:\n      - {name: mode, type: reporting.core.Mode}\n")
                .replace("        when: pause == false\n", "");
            let ir = assemble(&text).unwrap();
            let command = ir.commands().values().next().unwrap();
            let guard = Predicate::All(vec![
                Predicate::parse_expression("mode == V0").unwrap(),
                Predicate::parse_expression("pause == true").unwrap(),
            ]);
            let cases = finite::analyze(
                &ess_compiler::expression::Environment::new(&ir, &command.input),
                &[&guard],
            );
            if variants == 32 {
                assert_eq!(cases.unwrap().len(), 64);
            } else {
                assert!(cases.is_none());
            }
        }
    }

    #[test]
    fn boolean_finite_witness_respects_wrapper_invariants() {
        let text = boolean_model()
            .replace(
                "of: Boolean\n",
                "of: Boolean\n    invariants: [value == true]\n",
            )
            .replace(
                "name: pause, type: Boolean",
                "name: pause, type: reporting.core.WrappedFlag",
            );
        let ir = assemble(&text).unwrap();
        let result = ess_conformance::synthesize::synthesize(&ir);
        assert_eq!(result.suite.scenarios.len(), 1);
        let refusals: Vec<_> = result
            .refusals
            .iter()
            .filter(|refusal| refusal.scenario.is_some())
            .collect();
        assert_eq!(refusals.len(), 1, "{:?}", result.refusals);
        assert!(refusals[0]
            .scenario
            .as_ref()
            .unwrap()
            .to_string()
            .ends_with("/resumed"));
        assert!(matches!(
            refusals[0].cause,
            ess_conformance::synthesize::RefusalCause::GuardUnsatisfiable { tried: 1, .. }
        ));
        for step in &result.suite.scenarios.values().next().unwrap().steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                assert_eq!(
                    input["pause"],
                    ScenarioValue::Literal {
                        value: Node::Bool(true)
                    }
                );
            }
        }
    }

    #[test]
    fn boolean_shared_proof_callers_preserve_default_semantics() {
        let stored = include_str!("fixtures/stored-field-guards.yaml")
            .replace("          predicate:\n            all:\n              - service == Express\n              - weight_kg > 20", "          predicate: service == Express")
            .replace("      - name: dispatched", "      - name: duplicate-refusal\n        when_subject: {predicate: service == Express}\n        error: shipping.parcel.ExpressOverweight\n      - name: dispatched");
        let related = include_str!("fixtures/related-guard-sign-in.yaml")
            .replace("{name: demo.signin.ClientId, kind: newtype, of: String}", "{name: demo.signin.ClientId, kind: enum, variants: [Standard, Express]}")
            .replace("redirect_client != input.client", "redirect_client == Express")
            .replace("      - name: initiated", "      - name: duplicate-refusal\n        when_related: {via: input.tenant, predicate: redirect_client == Express}\n        error: demo.signin.NoRedirectEntry\n      - name: initiated");
        let state = include_str!("fixtures/subject-state.yaml")
            .replace("      - name: enriched", "      - name: duplicate-preserved\n        when_subject_state: Bridged\n        when: incoming == Ringing\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n        sets: {note: input.note}\n      - name: enriched");
        for (enumerated, boolean) in [
            (
                stored.clone(),
                stored
                    .replace(
                        "kind: enum\n    variants: [Standard, Express]",
                        "kind: newtype\n    of: Boolean",
                    )
                    .replace("service == Express", "service == true"),
            ),
            (
                related.clone(),
                related
                    .replace(
                        "kind: enum, variants: [Standard, Express]",
                        "kind: newtype, of: Boolean",
                    )
                    .replace("redirect_client == Express", "redirect_client == true"),
            ),
            (
                state.clone(),
                state
                    .replace(
                        "kind: enum\n    variants: [Ringing, Unspecified]",
                        "kind: newtype\n    of: Boolean",
                    )
                    .replace("incoming == Ringing", "incoming == true"),
            ),
        ] {
            let error = assemble(&enumerated).unwrap_err();
            assert!(error.contains("select 2 branches"), "{error}");
            assemble(&boolean)
                .unwrap_or_else(|error| panic!("existing Boolean/default policy changed: {error}"));
        }
    }

    #[test]
    fn boolean_shared_no_default_partitions_use_typed_values() {
        let stored = include_str!("fixtures/stored-field-guards.yaml")
            .replace("kind: enum\n    variants: [Standard, Express]", "kind: newtype\n    of: Boolean")
            .replace("          predicate:\n            all:\n              - service == Express\n              - weight_kg > 20", "          predicate: service == true")
            .replace("      - name: dispatched\n", "      - name: dispatched\n        when_subject: {predicate: service == false}\n");
        let related = include_str!("fixtures/related-guard-sign-in.yaml")
            .replace("{name: demo.signin.ClientId, kind: newtype, of: String}", "{name: demo.signin.ClientId, kind: newtype, of: Boolean}")
            .replace("redirect_client != input.client", "redirect_client == true")
            .replace("      - name: initiated\n", "      - name: initiated\n        when_related: {via: input.tenant, predicate: redirect_client == false}\n");
        let state = include_str!("fixtures/subject-state.yaml")
            .replace(
                "kind: enum\n    variants: [Ringing, Unspecified]",
                "kind: newtype\n    of: Boolean",
            )
            .replace("incoming == Ringing", "incoming == true")
            .replace(
                "      - name: enriched\n",
                "      - name: enriched\n        when: incoming == false\n",
            );
        for text in [stored, related, state] {
            assemble(&text).unwrap_or_else(|error| panic!("{error}"));
        }
    }

    #[test]
    fn boolean_extension_keeps_other_predicate_fragments_outside_the_proof() {
        let ir = assemble(&model("")).unwrap();
        let command = ir.commands().values().next().unwrap();
        let env = ess_compiler::expression::Environment::new(&ir, &command.input);
        for guard in [
            Predicate::Truthy("status".parse().unwrap()),
            Predicate::Defined("status".parse().unwrap()),
        ] {
            assert!(finite::analyze(&env, &[&guard]).is_none());
        }
        for replacement in ["String", "Integer", "Optional<Boolean>"] {
            assert!(assemble(&boolean_model().replace(
                "name: pause, type: Boolean",
                &format!("name: pause, type: {replacement}")
            ))
            .is_err());
        }
    }

    #[test]
    fn generated_go_runner_executes_boolean_partition() {
        let ir = assemble(&boolean_model()).unwrap();
        let result = ess_conformance::synthesize::synthesize(&ir);
        assert!(result.refusals.is_empty());
        let (rust, go) =
            support_go::compare("finite-boolean", &result.suite, Interpreted::for_model(ir));
        eprintln!("{}", go.go.log);
        assert!(go.go.success, "{}", go.go.log);
        let outcomes = support_go::assert_compared("finite-boolean", (rust, go));
        assert_eq!(outcomes.len(), 2);
        assert!(support_go::not_passed(&outcomes).is_empty());
    }

    #[test]
    fn generated_typescript_runner_executes_boolean_partition_and_rejects_ignored_flag() {
        use std::process::Command;
        let ir = assemble(&boolean_model()).unwrap();
        let result = ess_conformance::synthesize::synthesize(&ir);
        assert!(result.refusals.is_empty());
        let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("finite-boolean-ts-{}", std::process::id()));
        for artifact in ess_conformance::ts::emit(&result.suite).unwrap() {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        let directory = root.join(ess_conformance::ts::PACKAGE);
        // Match the existing runtime parity harness: transpilation runs executable semantics;
        // TypeScript declaration checking is a separate repository lane.
        std::fs::write(
            directory.join("runtime-test.tsconfig.json"),
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        )
        .unwrap();
        let built = Command::new("tsc")
            .args(["--project", "runtime-test.tsconfig.json"])
            .current_dir(&directory)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}{}",
            String::from_utf8_lossy(&built.stdout),
            String::from_utf8_lossy(&built.stderr)
        );
        // Only a boundary adapter: branch selection is deliberately independent of the proof.
        let driver = r"
import test from 'node:test';
import {run} from './dist/runtime.js';
await test('boolean partition', t => run(t, () => ({
 identity() { return {name:'boolean-fixture', version:'1'}; },
 beginScenario() {}, endScenario() {},
 executeCommand({input}) {
   if (typeof input.pause !== 'boolean') throw new Error('Boolean witness required');
   return {outcome: process.env.ESS_BOOLEAN_MUTANT === 'yes' || input.pause ? 'paused' : 'resumed', directEvents:[{event:'reporting.core.Observed', payload:{}}]};
 }
})));
";
        std::fs::write(directory.join("partition.test.mjs"), driver).unwrap();
        for mutant in [false, true] {
            let report = directory.join(format!("report-{mutant}.json"));
            let output = Command::new("node")
                .args(["--test", "partition.test.mjs"])
                .env("ESS_BOOLEAN_MUTANT", if mutant { "yes" } else { "no" })
                .env("ESS_REPORT_FORMAT", "2")
                .env("ESS_REPORT_OUT", &report)
                .current_dir(&directory)
                .output()
                .unwrap();
            let log = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!("TypeScript mutant={mutant}: {log}");
            assert_eq!(output.status.success(), !mutant, "{log}");
            let report: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(report).unwrap()).unwrap();
            let passed = report["outcomes"]["passed"].as_array().map_or(0, Vec::len);
            let failed = report["outcomes"]["failed"].as_array().map_or(0, Vec::len);
            assert_eq!(
                (passed, failed),
                if mutant { (1, 1) } else { (2, 0) },
                "{report}"
            );
        }
    }
}
