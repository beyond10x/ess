//! Closed input coverage must use the producer's six real values.
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
