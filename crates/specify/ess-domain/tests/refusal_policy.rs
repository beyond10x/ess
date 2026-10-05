//! A binding's failure policy selected per refusal of the invoked command (ess/22,
//! beyond10x/ess#269; `docs/design/conditional-binding-failure-policies.md`, "Refusal-selected
//! failure policy").
//!
//! Policy-keyed: `drop`, `retry` and `escalate`, each at most once, each with exactly one selector
//! (`outcomes:` or `except:`), drop and unbounded retry also as a list. Exactly one policy has
//! `except:`, and it is the explicit fallback for failures that carry no declared outcome. Names
//! resolve as `retry.final` does — an outcome that carries an error, or the error, which stands for
//! every outcome reporting it — and the resolved sets are disjoint and exhaustive. Below ess/22 the
//! shape is refused by format; every universal spelling keeps its meaning, bytes and diagnostics.

use ess_domain::binding::{BindingName, Failure};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";

fn with_policy(policy: &str) -> String {
    let model = MODEL.replace(POLICY, &format!("    on_failure:\n{policy}"));
    assert_ne!(
        model, MODEL,
        "the fixture carries the policy this test rewrites"
    );
    model
}

fn parse(body: &str) -> Result<ess_domain::spec::RawSpecFile, String> {
    ess_domain::spec::RawSpecFile::parse(body).map_err(|error| error.to_string())
}

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = parse(body).unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn assert_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

fn binding(spec: &Specification) -> serde_json::Value {
    let name = BindingName::new("notify-ledger").expect("a binding name");
    serde_json::to_value(&spec.bindings()[&name]).expect("a binding serializes")
}

#[test]
fn a_selected_policy_is_read_under_ess_22() {
    let spec = admitted(MODEL);
    let name = BindingName::new("notify-ledger").expect("a binding name");
    // The legacy word is the explicit fallback's, never a policy applied to every refusal.
    assert_eq!(spec.bindings()[&name].failure, Failure::Escalate);
    let binding = binding(&spec);
    assert_eq!(
        binding["on_refusal"],
        serde_json::json!([
            {"policy": "drop", "outcomes": ["wrong-state"]},
            {"policy": "retry", "outcomes": ["demo.ledger.Unavailable", "rejected"], "attempts": 3, "final": ["rejected"]},
            {"policy": "escalate", "except": ["wrong-state", "demo.ledger.Unavailable", "rejected"], "emits": "demo.ledger.RecordEscalated"}
        ]),
        "{binding}"
    );
}

#[test]
fn below_ess_22_the_selected_shape_is_refused_by_format_not_by_the_universal_reader() {
    let errors = refused(&MODEL.replace("format: ess/22", "format: ess/21"));
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "binding.notify-ledger.on_failure",
    );
    assert_code(&errors, ValidationCode::UnsupportedFormatVersion, "ess/22");
    // The shape that was always an error under the universal reader is refused by format as well.
    let drop_list = with_policy("      drop: [wrong-state]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state]}\n")
        .replace("format: ess/22", "format: ess/16");
    assert_code(
        &refused(&drop_list),
        ValidationCode::UnsupportedFormatVersion,
        "ess/22",
    );
}

#[test]
fn every_universal_spelling_keeps_its_meaning_and_bytes() {
    for (written, failure, extra) in [
        ("    on_failure: retry\n", "retry", None),
        ("    on_failure: drop\n", "drop", None),
        (
            "    on_failure:\n      escalate:\n        emits: demo.ledger.RecordEscalated\n",
            "escalate",
            Some((
                "escalation",
                serde_json::json!("demo.ledger.RecordEscalated"),
            )),
        ),
        (
            "    on_failure:\n      retry: {attempts: 3, final: [rejected]}\n",
            "retry",
            Some((
                "retry",
                serde_json::json!({"attempts": 3, "final": ["rejected"]}),
            )),
        ),
    ] {
        let model = MODEL.replace(POLICY, written);
        assert_ne!(model, MODEL);
        let binding = binding(&admitted(&model));
        assert_eq!(binding["failure"], failure, "{written}");
        assert!(binding.get("on_refusal").is_none(), "{written}: {binding}");
        let mut keys: Vec<&str> = binding
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        let mut expected = vec![
            "command",
            "delivery",
            "escalation",
            "event",
            "failure",
            "mapping",
            "name",
            "naming",
            "retry",
        ];
        // `escalation` is written for every binding, `null` where none escalates, as it always was.
        expected.retain(|key| match *key {
            "retry" => extra.as_ref().is_some_and(|(key, _)| *key == "retry"),
            _ => true,
        });
        assert_eq!(keys, expected, "{written}: {binding}");
        if let Some((key, value)) = extra {
            assert_eq!(binding[key], value, "{written}");
        }
    }
}

/// The universal reader's diagnostics, as they were before a selector existed.
#[test]
fn the_universal_readers_diagnostics_are_unchanged() {
    for (policy, message) in [
        (
            "      drop: {}\n",
            "`drop` is written as a bare word — `on_failure: drop`. Only `escalate` and `retry` take a block: `escalate` names what it publishes, and `retry` states its bound",
        ),
        (
            "      retry: {attempts: 3}\n      drop: x\n",
            "`on_failure` says `retry` and `drop`; a binding has one policy for a command that does not run",
        ),
        (
            "      retry: {tries: 3}\n",
            "`retry:` takes `attempts:` and `final:`, and `tries` is neither; only `escalate` publishes anything, and no timing is a claim here",
        ),
        (
            "      retry: {final: [rejected]}\n",
            "`retry:` with a block states `attempts:`, the invocations in all; write `on_failure: retry` for a retry with no bound",
        ),
        ("      escalate: {publishes: x}\n", "unknown field `publishes`, expected `emits`"),
        ("      resend: x\n", "unknown variant `resend`, expected one of `retry`, `escalate`, `drop`"),
    ] {
        let error = parse(&with_policy(policy)).expect_err(policy);
        assert!(error.contains(message), "{policy}: {error}");
    }
}

#[test]
fn an_error_name_and_the_list_shorthand_select_like_outcome_names() {
    admitted(&with_policy(
        "      drop: [demo.ledger.WrongState]\n      retry: [unavailable, busy, rejected]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, demo.ledger.Unavailable, demo.ledger.Unknown]}\n",
    ));
    // An empty `except` is the fallback for every refusal.
    admitted(&with_policy(
        "      escalate: {emits: demo.ledger.RecordEscalated, except: []}\n",
    ));
    // The fallback may be a bounded retry.
    admitted(&with_policy(
        "      drop: [wrong-state, at-limit]\n      retry: {except: [wrong-state, at-limit], attempts: 3, final: [rejected]}\n",
    ));
}

#[test]
fn a_name_that_is_no_refusal_of_the_invoked_command_is_refused() {
    for name in ["wrong_state", "recorded", "demo.ledger.Missing", "placed"] {
        let errors = refused(&with_policy(&format!(
            "      drop: [{name}]\n      escalate: {{emits: demo.ledger.RecordEscalated, except: [{name}]}}\n"
        )));
        let code = if name == "recorded" {
            ValidationCode::ConflictingDeclaration
        } else {
            ValidationCode::UndeclaredReference
        };
        assert_code(&errors, code, &format!("`{name}`"));
        assert_code(&errors, code, "binding.notify-ledger.on_failure.drop");
    }
}

#[test]
fn an_accepting_outcome_is_refused_by_name() {
    let errors = refused(&with_policy(
        "      drop: [recorded]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [recorded]}\n",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "accepting");
}

#[test]
fn overlapping_selectors_are_refused_even_through_an_alias() {
    // Two policies select `busy`: one by its error, one by its name.
    let errors = refused(&with_policy(
        "      drop: [busy]\n      retry: [demo.ledger.Unavailable]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [busy, demo.ledger.Unavailable]}\n",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "`busy`");
    // One policy selects `unavailable` twice, by name and by its error.
    let errors = refused(&with_policy(
        "      drop: [unavailable, demo.ledger.Unavailable]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [demo.ledger.Unavailable]}\n",
    ));
    assert_code(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "`unavailable`",
    );
    // A positive selection the fallback also covers.
    let errors = refused(&with_policy(
        "      drop: [wrong-state]\n      escalate: {emits: demo.ledger.RecordEscalated, except: []}\n",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "`wrong-state`",
    );
}

#[test]
fn an_excepted_refusal_no_policy_selects_is_refused() {
    let errors = refused(&with_policy(
        "      drop: [wrong-state]\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, at-limit]}\n",
    ));
    assert_code(&errors, ValidationCode::MissingDeclaration, "`at-limit`");
}

#[test]
fn exactly_one_policy_is_the_explicit_fallback() {
    let errors = refused(&with_policy(
        "      drop: [wrong-state]\n      escalate: {emits: demo.ledger.RecordEscalated, outcomes: [at-limit]}\n      retry: [unavailable, busy, rejected]\n",
    ));
    assert_code(&errors, ValidationCode::MissingDeclaration, "except");
    let errors = refused(&with_policy(
        "      drop: {except: [at-limit]}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state]}\n",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "except");
}

#[test]
fn a_final_refusal_outside_the_retry_selection_is_refused() {
    let errors = refused(&with_policy(
        "      drop: [wrong-state]\n      retry: {outcomes: [unavailable, busy], attempts: 3, final: [rejected]}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, unavailable, busy]}\n",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "`rejected`",
    );
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "final");
}

#[test]
fn a_final_without_a_bound_and_a_bound_below_two_are_refused() {
    let errors = refused(&with_policy(
        "      retry: {outcomes: [rejected], final: [rejected]}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [rejected]}\n",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "attempts");
    let errors = refused(&with_policy(
        "      retry: {outcomes: [rejected], attempts: 1}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [rejected]}\n",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "attempts: 1",
    );
}

#[test]
fn shape_errors_of_a_selected_policy_are_refused() {
    // Escalation still names what it emits.
    assert_code(
        &refused(&with_policy(
            "      drop: [wrong-state]\n      escalate: {except: [wrong-state]}\n",
        )),
        ValidationCode::MissingDeclaration,
        "emits",
    );
    // Both selectors on one policy.
    assert_code(
        &refused(&with_policy(
            "      drop: {outcomes: [wrong-state], except: []}\n",
        )),
        ValidationCode::ConflictingDeclaration,
        "`outcomes:` and `except:`",
    );
    // A policy with no selector beside one with a selector.
    assert_code(
        &refused(&with_policy(
            "      drop: {except: [at-limit]}\n      escalate: {emits: demo.ledger.RecordEscalated}\n",
        )),
        ValidationCode::MissingDeclaration,
        "binding.notify-ledger.on_failure.escalate",
    );
    // An empty positive list selects nothing.
    assert_code(
        &refused(&with_policy(
            "      drop: []\n      escalate: {emits: demo.ledger.RecordEscalated, except: []}\n",
        )),
        ValidationCode::MissingDeclaration,
        "binding.notify-ledger.on_failure.drop",
    );
    // Escalate has no list shorthand.
    assert!(parse(&with_policy(
        "      drop: {except: [at-limit]}\n      escalate: [at-limit]\n",
    ))
    .is_err_and(|error| error.contains("emits")));
    // A key a policy does not take.
    assert!(parse(&with_policy(
        "      drop: {outcomes: [wrong-state], emits: demo.ledger.RecordEscalated}\n      retry: {except: [wrong-state]}\n",
    ))
    .is_err_and(|error| error.contains("`emits`")));
}

#[test]
fn a_selected_binding_assembled_in_code_must_agree_with_its_table() {
    let model = admitted(MODEL);
    let name = BindingName::new("notify-ledger").expect("a binding name");
    let binding = model.bindings()[&name].clone();
    assert!(binding.refusals.is_some());
    assert_eq!(
        binding.validate().as_slice().len(),
        0,
        "{}",
        binding.validate()
    );
    // The legacy word names the fallback; a binding that says `drop` there while its table falls
    // back to `escalate` would let a consumer apply the wrong policy to every untyped failure.
    let mut disagreeing = binding.clone();
    disagreeing.failure = Failure::Drop;
    assert_code(
        &disagreeing.validate(),
        ValidationCode::ConflictingDeclaration,
        "binding.notify-ledger.on_failure",
    );
    let mut disagreeing = binding;
    disagreeing.escalation = None;
    assert_code(
        &disagreeing.validate(),
        ValidationCode::ConflictingDeclaration,
        "binding.notify-ledger.on_failure",
    );
}

/// The committed document schema's `BindingFailure`, as an editor validates against it.
fn failure_schema() -> jsonschema::Validator {
    let text = include_str!("../../../../schemas/generated/ess.schema.json");
    let schema: serde_json::Value = serde_json::from_str(text).expect("the schema is JSON");
    jsonschema::validator_for(&serde_json::json!({
        "$schema": schema["$schema"],
        "definitions": schema["definitions"],
        "allOf": [{ "$ref": "#/definitions/BindingFailure" }],
    }))
    .expect("a usable schema")
}

fn on_failure(yaml: &str) -> serde_json::Value {
    serde_yaml::from_str(yaml).expect("YAML")
}

#[test]
fn the_published_schema_admits_the_selected_shape_and_keeps_every_universal_one() {
    let schema = failure_schema();
    for admitted in [
        "drop: [wrong-state]\nretry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}\nescalate: {emits: demo.ledger.RecordEscalated, except: [wrong-state, demo.ledger.Unavailable, rejected]}",
        "escalate: {emits: demo.ledger.RecordEscalated, except: []}",
        "drop: [wrong-state]\nretry: {except: [wrong-state]}",
        "retry",
        "escalate: {emits: demo.ledger.RecordEscalated}",
        "retry: {attempts: 3, final: [rejected]}",
    ] {
        assert!(schema.is_valid(&on_failure(admitted)), "{admitted}");
    }
    for refused in [
        // A policy beside a selected one with no selector of its own.
        "drop: [wrong-state]\nescalate: {emits: demo.ledger.RecordEscalated}",
        // A key a policy does not take, and a word that is no policy.
        "drop: {outcomes: [wrong-state], emits: demo.ledger.RecordEscalated}",
        "resend: [wrong-state]",
        // Escalate has no list shorthand.
        "escalate: [at-limit]",
    ] {
        assert!(!schema.is_valid(&on_failure(refused)), "{refused}");
    }
}
