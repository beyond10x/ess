//! Original suite bytes cannot disguise an infix disjunction as one quoted literal.
use ess_conformance::AdmittedSuite;
use serde_json::{json, Value};

fn document(predicate: &Value) -> String {
    json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "quoted",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"quoted.core/authored/operand": {"purpose": "Check a quoted operand",
            "steps": [{"step": "expect_view", "view": "quoted.core.Rows",
                "expectation": {"expect": "satisfies", "predicate": predicate}}], "source": []}}
    }).to_string()
}

#[test]
fn rust_suite_reader_refuses_trailing_tokens_but_preserves_structured_any() {
    let invalid = document(&json!(r#"to == "" or text == """#));
    assert!(AdmittedSuite::from_json(&invalid).is_err());
    let valid = document(&json!({"any": [r#"to == """#, r#"text == """#]}));
    AdmittedSuite::from_json(&valid).unwrap();
}

#[test]
fn source_command_guard_refuses_the_measured_disjunction() {
    let source = |predicate: Value| {
        format!(
        "format: ess/3\nsystem: quoted\nversion: v1\ndomain: quoted.core\ncommands:\n  - name: quoted.core.Send\n    input:\n      - {{name: to, type: String}}\n      - {{name: text, type: String}}\n    outcomes:\n      - name: accepted\n        when: {predicate}\n"
    )
    };
    let error = ess_domain::spec::RawSpecFile::parse(&source(json!(r#"to == "" or text == """#)))
        .unwrap_err();
    assert!(
        error.to_string().contains("structured any/all/not"),
        "{error}"
    );
    ess_domain::spec::RawSpecFile::parse(&source(json!({"any": [r#"to == """#, r#"text == """#]})))
        .unwrap();
}

#[test]
fn generated_go_reader_and_evaluator_keep_quoted_operand_boundaries() {
    let suite = AdmittedSuite::from_json(&document(&json!(r#"to == """#))).unwrap();
    let directory =
        std::env::temp_dir().join(format!("ess-gap10-predicate-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    for artifact in ess_conformance::go::emit(suite.suite()).unwrap() {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/quoted\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/quoted_test.go"),
        include_str!("fixtures/quoted-predicates.go"),
    )
    .unwrap();
    for (name, contents) in [
        (
            "adversary_test.go",
            include_str!("fixtures/adversary-quoted-specials.go"),
        ),
        (
            "boundaries_test.go",
            include_str!("fixtures/quoted-special-boundaries.go"),
        ),
    ] {
        std::fs::write(directory.join("essconform").join(name), contents).unwrap();
    }
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "Test(Quoted|StructuredSpecial)",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .unwrap();
    eprintln!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.status.success(),
        "Go predicate regression: {}",
        result.status
    );
}

fn literal_suite() -> (AdmittedSuite, Vec<&'static str>) {
    let literals = vec![
        "\"busy\" status",
        "'busy' status",
        "true",
        "1",
        "  padded  ",
        "a\\.b",
        "say \\\"hi\\\"",
        "line\nnext",
        "\0",
        "\"quoted\"",
        "Ready",
        "a.b",
        "",
    ];
    let id = "quoted.core/authored/operand";
    let mut value: Value = serde_json::from_str(&document(&json!(true))).unwrap();
    value["provenance"]["suite_version"] = json!("ess-conformance/9");
    value["scenarios"][id]["steps"] = json!(literals.iter().map(|text| json!({
        "step": "expect_view", "view": "quoted.core.Rows",
        "expectation": {"expect": "satisfies", "predicate": {"to": {"eq": format!("\"{text}\"")}}}
    })).collect::<Vec<_>>());
    value["coverage"] = json!({
        "selection": {"scope": {"kind": "system"}, "origins": "authored", "filter": {"kind": "all"}},
        "knowledge": "complete_inventory", "generated": [], "authored": [id], "outside": [], "refused": [],
        "authored_sources": {"operand.yaml": {"digest": format!("sha256:{}", "c".repeat(64)), "scenario": id, "disposition": "accepted"}},
        "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 0}
    });
    (
        AdmittedSuite::from_json(&value.to_string()).unwrap(),
        literals,
    )
}

#[test]
fn generated_suite_literals_roundtrip_in_rust_go_and_browser() {
    use ess_conformance::scenario::{ScenarioStep, ViewExpectation};
    use ess_primitives::{
        facts::FactValue,
        predicate::{Operand, Predicate},
    };
    let (admitted, literals) = literal_suite();
    let emitted = ess_conformance::coverage::compact_suite_document(
        admitted.suite(),
        admitted.coverage().unwrap(),
    )
    .unwrap();
    let reread = AdmittedSuite::from_json(&emitted).unwrap();
    assert_eq!(
        ess_conformance::coverage::compact_suite_document(
            reread.suite(),
            reread.coverage().unwrap()
        )
        .unwrap(),
        emitted
    );
    let steps = &reread.suite().scenarios.values().next().unwrap().steps;
    assert_eq!(steps.len(), literals.len());
    for (step, text) in steps.iter().zip(&literals) {
        let ScenarioStep::ExpectView {
            expectation:
                ViewExpectation::Satisfies {
                    predicate: Predicate::Compare { right, .. },
                },
            ..
        } = step
        else {
            panic!("typed comparison")
        };
        assert_eq!(right, &Operand::Literal(FactValue::Text((*text).into())));
    }
    let directory = std::env::temp_dir().join(format!(
        "ess-gap10-predicate-{}/roundtrip",
        std::process::id()
    ));
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    let input = ess_conformance::coverage::SuiteInputDocument {
        format: "ess-conformance-input/1".into(),
        suite_json: emitted,
        parent_suites: vec![],
    };
    let input =
        ess_conformance::coverage::AdmittedInput::from_json(&input.to_canonical_json().unwrap())
            .unwrap();
    for artifact in ess_conformance::go::emit_input(&input).unwrap() {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/quotedroundtrip\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/expected.json"),
        serde_json::to_vec(&literals).unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/roundtrip_test.go"),
        include_str!("fixtures/quoted-roundtrip.go"),
    )
    .unwrap();
    let go = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestQuoted(Roundtrip|VersionAuthority)",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .unwrap();
    require_success(&go);
    let node =
        std::process::Command::new(std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into()))
            .arg(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/quoted-roundtrip.mjs"),
            )
            .arg(directory.join("essconform/suite.json"))
            .arg(directory.join("essconform/expected.json"))
            .output()
            .unwrap();
    require_success(&node);
}

fn require_success(output: &std::process::Output) {
    eprintln!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{}", output.status);
}
