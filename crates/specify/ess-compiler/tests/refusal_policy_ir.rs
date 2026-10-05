//! A refusal-selected failure policy (ess/22, beyond10x/ess#269) lands in the IR as one typed
//! table: every declared refusal of the invoked command, in the command's order, mapped to the
//! concrete policy its aliases resolved to, plus the explicit fallback for a failure that carries
//! no declared outcome. `ResolvedBinding::on_failure` answers it as its own arm, so no consumer
//! can read the legacy fields and apply the fallback to every refusal. A universal policy keeps
//! its IR bytes.

use ess_compiler::ir::{EssIr, ResolvedBinding};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::binding::BindingName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("refusal-policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn binding(ir: &EssIr) -> &ResolvedBinding {
    &ir.bindings()[&BindingName::new("notify-ledger").unwrap()]
}

#[test]
fn every_declared_refusal_resolves_to_one_policy_in_command_order() {
    let ir = ir(MODEL);
    let binding = binding(&ir);
    assert!(
        format!("{:?}", binding.on_failure()).starts_with("ByRefusal"),
        "{:?}",
        binding.on_failure()
    );
    let json = serde_json::to_value(binding).unwrap();
    let bound = serde_json::json!({"attempts": 3, "final": ["rejected"]});
    let retry = |outcome: &str| serde_json::json!({"outcome": outcome, "policy": "retry", "bound": bound.clone()});
    assert_eq!(
        json["on_refusal"],
        serde_json::json!({
            "refusals": [
                retry("unavailable"),
                retry("busy"),
                retry("rejected"),
                {"outcome": "at-limit", "policy": "escalate", "emits": json["escalation"].clone()},
                {"outcome": "wrong-state", "policy": "drop"}
            ],
            "fallback": {"policy": "escalate", "emits": json["escalation"].clone()}
        }),
        "{json:#}"
    );
    // The legacy view names the fallback's word and the policies the table uses, never a policy
    // that would apply to every refusal.
    assert_eq!(json["failure"], "escalate");
    assert_eq!(json["retry"], bound);
    let ess_compiler::ir::ResolvedFailure::ByRefusal { policy } = binding.on_failure() else {
        panic!("a selected policy is its own arm");
    };
    assert!(policy.agrees_with(binding));
    // The actual refusal selects; a failure with no declared outcome takes the fallback.
    let select = |outcome: Option<&str>| {
        policy
            .select(outcome.map(|outcome| outcome.parse().unwrap()).as_ref())
            .word()
            .to_string()
    };
    assert_eq!(select(Some("wrong-state")), "drop");
    assert_eq!(select(Some("busy")), "retry");
    assert_eq!(select(Some("at-limit")), "escalate");
    assert_eq!(select(None), "escalate");
}

#[test]
fn an_error_alias_and_its_outcome_names_select_the_same_policy() {
    // outcome_aliases_expand_to_same_policy: `demo.ledger.Unavailable` stands for both outcomes
    // that report it, and writing the two names instead resolves to the identical table.
    let by_error = serde_json::to_value(binding(&ir(MODEL))).unwrap();
    let by_name = serde_json::to_value(binding(&ir(&MODEL.replace(
        POLICY,
        "    on_failure:
      drop: [demo.ledger.WrongState]
      retry: {outcomes: [unavailable, busy, demo.ledger.Unknown], attempts: 3, final: [demo.ledger.Unknown]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [demo.ledger.WrongState, unavailable, busy, rejected]
",
    ))))
    .unwrap();
    assert_eq!(by_error["on_refusal"], by_name["on_refusal"]);
}

#[test]
fn a_bounded_fallback_resolves_its_final_refusals_inside_the_complement() {
    let ir = ir(&MODEL.replace(
        POLICY,
        "    on_failure:
      drop: [wrong-state, at-limit]
      retry: {except: [wrong-state, at-limit], attempts: 4, final: [rejected]}
",
    ));
    let json = serde_json::to_value(binding(&ir)).unwrap();
    assert_eq!(
        json["on_refusal"]["fallback"],
        serde_json::json!({"policy": "retry", "bound": {"attempts": 4, "final": ["rejected"]}})
    );
    assert_eq!(json["failure"], "retry");
    assert!(json.get("escalation").is_none(), "{json:#}");
}

#[test]
fn a_universal_policy_keeps_its_ir_bytes() {
    for written in [
        "    on_failure: retry\n",
        "    on_failure: drop\n",
        "    on_failure:\n      escalate: {emits: demo.ledger.RecordEscalated}\n",
        "    on_failure:\n      retry: {attempts: 3, final: [rejected]}\n",
    ] {
        let ir = ir(&MODEL.replace(POLICY, written));
        let json = serde_json::to_value(binding(&ir)).unwrap();
        assert!(json.get("on_refusal").is_none(), "{written}: {json:#}");
        assert!(
            !format!("{:?}", binding(&ir).on_failure()).starts_with("ByRefusal"),
            "{written}"
        );
    }
}
