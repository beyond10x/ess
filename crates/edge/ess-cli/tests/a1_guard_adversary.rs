//! Adversarial cases for unit A1 (beyond10x/ess#93 and #94), driven through the built `ess`
//! binary.
//!
//! The model is the one the predicate reference page carries (unit A2), inlined here because that
//! page is not in this tree. F1 and F3 are that page's adversary cases, copied as fragments; the
//! rest take the two stories' decisions as the specification.

use std::{fs, path::Path, process::Command};

use serde_yaml::Value;

const MODEL: &str = r#"
format: ess/1
system: shop
version: v1
domain: shop.order
types:
  - name: shop.order.Channel
    kind: enum
    variants: [Web, Store, Phone]
  - name: shop.order.Sku
    kind: newtype
    of: String
  - name: shop.order.Wait
    kind: newtype
    of: Duration
entities:
  - name: shop.order.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: channel, type: shop.order.Channel}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: labels, type: "Map<String, String>"}
    invariants:
      - quantity >= 1
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: shop.order.Refused
    summary: The order was not placed.
  - name: shop.order.NotOpen
    summary: The order is not open.
commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: channel, type: shop.order.Channel}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: coupon, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: gift, type: Boolean}
    outcomes:
      - name: placed
        when: quantity > 0
        creates: shop.order.Order
        instance: order_id
        emits: [shop.order.OrderPlaced]
        sets:
          channel: input.channel
          quantity: input.quantity
          sku: input.sku
          tags: input.tags
      - name: refused
        error: shop.order.Refused
  - name: shop.order.CloseOrder
    input:
      - {name: order_id, type: Uuid}
    outcomes:
      - name: closed
        moves: shop.order.Order.close
        instance: order_id
        emits: [shop.order.OrderClosed]
      - name: wrong-state
        wrong_state: true
        error: shop.order.NotOpen
events:
  - name: shop.order.OrderPlaced
    fields:
      - {name: order_id, type: Uuid}
  - name: shop.order.OrderClosed
    fields:
      - {name: order_id, type: Uuid}
views:
  - name: shop.order.OpenOrders
    source: shop.order.Order
    consistency: read_your_writes
    filter: state == Open
    fields:
      - {name: order_id, type: Uuid}
      - {name: channel, type: shop.order.Channel}
  - name: shop.order.OrderById
    source: shop.order.Order
    consistency: eventual
    fields:
      - {name: order_id, type: Uuid}
      - {name: channel, type: shop.order.Channel}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
"#;

fn model() -> Value {
    serde_yaml::from_str(MODEL).expect("the inlined model is YAML")
}

fn named<'a>(list: &'a mut Value, key: &str, name: &str) -> &'a mut Value {
    list.get_mut(key)
        .and_then(Value::as_sequence_mut)
        .and_then(|items| {
            items
                .iter_mut()
                .find(|item| item.get("name").and_then(Value::as_str) == Some(name))
        })
        .unwrap_or_else(|| panic!("the model declares no {key} entry named `{name}`"))
}

fn place_order(model: &mut Value) -> &mut Value {
    named(model, "commands", "shop.order.PlaceOrder")
}

fn push_seq(target: &mut Value, key: &str, item: &str) {
    target
        .get_mut(key)
        .and_then(Value::as_sequence_mut)
        .unwrap_or_else(|| panic!("no sequence `{key}`"))
        .push(serde_yaml::from_str(item).expect("an item"));
}

fn with_input(mut model: Value, field: &str) -> Value {
    push_seq(place_order(&mut model), "input", field);
    model
}

/// The model with `when` (a YAML fragment) on outcome `placed` of `PlaceOrder`.
fn with_when(mut model: Value, when: &str) -> Value {
    let when: Value = serde_yaml::from_str(when).expect("the guard is YAML");
    named(place_order(&mut model), "outcomes", "placed")
        .as_mapping_mut()
        .expect("an outcome is a mapping")
        .insert(Value::String("when".to_owned()), when);
    model
}

struct Run {
    code: Option<i32>,
    output: String,
}

fn ess(args: &[&str], document: &Value, extra: &[&Path]) -> Run {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join("model.yaml");
    fs::write(&path, serde_yaml::to_string(document).expect("serialize")).expect("write");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.args(args).arg("--path").arg(&path);
    for arg in extra {
        command.arg(arg);
    }
    let output = command.output().expect("run the built ess binary");
    Run {
        code: output.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

fn validate(document: &Value) -> Run {
    ess(&["specify", "validate"], document, &[])
}

/// Synthesizes the IR suite; returns the run and the suite JSON when one was written.
fn synthesize(document: &Value) -> (Run, Option<serde_json::Value>) {
    let out_dir = tempfile::tempdir().expect("a temporary directory");
    let out = out_dir.path().join("suite.json");
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join("model.yaml");
    fs::write(&path, serde_yaml::to_string(document).expect("serialize")).expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args([
            "verify",
            "conform",
            "synthesize",
            "--target",
            "ir",
            "--path",
        ])
        .arg(&path)
        .arg("--out")
        .arg(&out)
        .output()
        .expect("run the built ess binary");
    let run = Run {
        code: output.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    };
    let suite = fs::read_to_string(&out)
        .ok()
        .map(|text| serde_json::from_str(&text).expect("the suite is JSON"));
    (run, suite)
}

/// Every `execute_command` input for `PlaceOrder` in the suite.
fn place_order_inputs(
    suite: &serde_json::Value,
) -> Vec<serde_json::Map<String, serde_json::Value>> {
    fn walk(value: &serde_json::Value, into: &mut Vec<serde_json::Map<String, serde_json::Value>>) {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("step").and_then(serde_json::Value::as_str) == Some("execute_command")
                    && map
                        .get("command")
                        .is_some_and(|command| command.to_string().contains("PlaceOrder"))
                {
                    into.push(
                        map.get("input")
                            .and_then(serde_json::Value::as_object)
                            .cloned()
                            .unwrap_or_default(),
                    );
                }
                for child in map.values() {
                    walk(child, into);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    walk(item, into);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    walk(suite, &mut found);
    found
}

// ------------------------------------------------------------------------------------------------
// F1 and F3, copied from the A2 reference-page adversary
// ------------------------------------------------------------------------------------------------

/// A2 F1, predicates.md "Ordering": "`Duration` has no ordering."
#[test]
fn f1_the_page_says_duration_has_no_ordering_so_validate_refuses_ordering_a_duration() {
    let model = with_input(model(), "{name: wait, type: Duration}");
    let run = validate(&with_when(model, r#"wait > "PT5M""#));
    assert_eq!(
        run.code,
        Some(1),
        "validate admitted `wait > \"PT5M\"` on a Duration input:\n{}",
        run.output
    );
}

/// A2 F3, predicates.md "Compact forms": "The compact form has no `&&`, `||` …".
#[test]
fn f3_the_page_says_the_compact_form_has_no_conjunction_so_validate_refuses_one_over_text() {
    let mut admitted = Vec::new();
    for guard in ["sku == A1 && gift", "sku == A1 || sku == B2"] {
        let run = validate(&with_when(model(), guard));
        if run.code != Some(1) {
            admitted.push(format!("`{guard}` exited {:?}\n{}", run.code, run.output));
        }
    }
    assert!(admitted.is_empty(), "{}", admitted.join("\n"));
}

// ------------------------------------------------------------------------------------------------
// Duration ordering: every spelling and every place an ordering can be written
// ------------------------------------------------------------------------------------------------

#[test]
fn every_spelling_of_a_duration_ordering_is_refused_at_validate() {
    let cases: Vec<(&str, Value)> = vec![
        (
            "newtype over Duration",
            with_when(
                with_input(model(), "{name: wait, type: shop.order.Wait}"),
                "wait > PT5M",
            ),
        ),
        (
            "Optional<Duration>",
            with_when(
                with_input(model(), "{name: wait, type: Optional<Duration>}"),
                "wait >= PT5M",
            ),
        ),
        (
            "mapping form",
            with_when(
                with_input(model(), "{name: wait, type: Duration}"),
                "{wait: {lt: PT5M}}",
            ),
        ),
        (
            "two Duration facts",
            with_when(
                with_input(
                    with_input(model(), "{name: wait, type: Duration}"),
                    "{name: other, type: Duration}",
                ),
                "wait <= other",
            ),
        ),
        (
            "element of a List<Duration>",
            with_when(
                with_input(model(), "{name: waits, type: List<Duration>}"),
                "{exists: {in: waits, as: w, that: w > PT5M}}",
            ),
        ),
        ("entity invariant", {
            // `ttl` is set by the creating branch, so the only refusal left to name `Duration` is
            // the ordering one this case is about (ess#112 refuses an invariant over an unset
            // field, and its hint names the type too).
            let mut model = with_input(model(), "{name: ttl, type: Duration}");
            named(place_order(&mut model), "outcomes", "placed")
                .get_mut("sets")
                .and_then(Value::as_mapping_mut)
                .expect("`placed` sets fields")
                .insert(
                    Value::String("ttl".to_owned()),
                    Value::String("input.ttl".to_owned()),
                );
            let order = named(&mut model, "entities", "shop.order.Order");
            push_seq(order, "fields", "{name: ttl, type: Duration}");
            push_seq(order, "invariants", "ttl > PT5M");
            model
        }),
    ];
    let mut admitted = Vec::new();
    for (name, document) in cases {
        let run = validate(&document);
        if run.code != Some(1) || !run.output.contains("Duration") {
            admitted.push(format!("{name}: exited {:?}\n{}", run.code, run.output));
        }
    }
    let control = validate(&with_when(
        with_input(model(), "{name: wait, type: Duration}"),
        "wait == PT5M",
    ));
    assert_eq!(
        control.code,
        Some(0),
        "equality on a Duration stays admitted:\n{}",
        control.output
    );
    assert!(
        admitted.is_empty(),
        "a Duration ordering validate admitted:\n{}",
        admitted.join("\n")
    );
}

// ------------------------------------------------------------------------------------------------
// ess#93: unquoted null is refused at validate with a stable code and a hint
// ------------------------------------------------------------------------------------------------

#[test]
fn an_unquoted_null_comparison_is_refused_at_validate_with_a_code_and_the_presence_hint() {
    let mut wrong = Vec::new();
    for guard in [
        "coupon == null",
        "coupon != null",
        "{coupon: null}",
        "{coupon: {ne: null}}",
    ] {
        let run = validate(&with_when(model(), guard));
        let named_code = run.output.contains("ESS-");
        if run.code != Some(1) || !named_code || !run.output.contains("defined(coupon)") {
            wrong.push(format!("`{guard}` exited {:?}\n{}", run.code, run.output));
        }
    }
    let quoted = validate(&with_when(model(), r#"coupon == "null""#));
    assert_eq!(
        quoted.code,
        Some(0),
        "a quoted \"null\" stays text and validates:\n{}",
        quoted.output
    );
    assert!(
        wrong.is_empty(),
        "the story: validate refuses `note == null` with a stable code and a hint naming \
         `defined(note)`:\n{}",
        wrong.join("\n")
    );
}

/// A quoted operand followed by `&&` is the same mistake as the unquoted one.
#[test]
fn a_quoted_operand_followed_by_a_conjunction_is_refused_too() {
    let run = validate(&with_when(model(), r#"sku == "A1" && gift"#));
    assert_eq!(run.code, Some(1), "{}", run.output);
}

// ------------------------------------------------------------------------------------------------
// Synthesis through the CLI: what the emitted suite carries
// ------------------------------------------------------------------------------------------------

#[test]
fn both_sides_of_a_presence_guard_reach_the_emitted_suite_and_absence_is_an_omitted_key() {
    for guard in [
        "defined(coupon)",
        "not defined(coupon)",
        "{coupon: {exists: false}}",
    ] {
        let (run, suite) = synthesize(&with_when(model(), guard));
        assert_eq!(run.code, Some(0), "`{guard}`: {}", run.output);
        let inputs = place_order_inputs(&suite.expect("a suite was written"));
        assert!(
            !inputs.is_empty(),
            "`{guard}`: no PlaceOrder input in the suite"
        );
        assert!(
            inputs.iter().any(|input| !input.contains_key("coupon")),
            "`{guard}`: no input omits `coupon`: {inputs:?}"
        );
        assert!(
            inputs.iter().any(|input| input.contains_key("coupon")),
            "`{guard}`: no input carries `coupon`: {inputs:?}"
        );
        assert!(
            inputs.iter().all(|input| input
                .get("coupon")
                .is_none_or(|value| !value.is_null() && !value.to_string().contains("null"))),
            "`{guard}`: an input sends `coupon` as null: {inputs:?}"
        );
    }
}

#[test]
fn list_and_text_guards_synthesize_through_the_cli() {
    for guard in [
        "{exists: {in: tags, as: t, that: t == vip}}",
        "{forall: {in: tags, as: t, that: t == vip}}",
        "tags.count > 0",
        "sku < M",
        "sku >= M",
    ] {
        let (run, suite) = synthesize(&with_when(model(), guard));
        assert_eq!(
            run.code,
            Some(0),
            "`{guard}` did not synthesize:\n{}",
            run.output
        );
        let inputs = place_order_inputs(&suite.expect("a suite was written"));
        assert!(!inputs.is_empty(), "`{guard}`: no PlaceOrder input");
        // Not for `forall`: `[]` satisfies it vacuously, so the story's `[vip]` is not needed there.
        if guard.contains("vip") && !guard.contains("forall") {
            assert!(
                inputs.iter().any(|input| input
                    .get("tags")
                    .is_some_and(|tags| tags.to_string().contains("vip"))),
                "`{guard}`: no input carries a `vip` element: {inputs:?}"
            );
        }
    }
}

/// A list ordinal on command input validates (A2 decision F2 documents `tags.0` as admitted), and
/// `input.rs`'s module doc says `lines.0.quantity` over an input list is "decided rather than
/// refused". The base witness keeps the list `[]`, where the ordinal is absent and `Unknown`, and
/// an `Unknown` ends the search — so each outcome is refused `ESS-SYNTH-002`, calling a `String`
/// element "a collection". That is ess#94's defect class: valid at validate, refused at synthesize.
#[test]
fn a_list_ordinal_guard_on_command_input_gets_both_scenarios() {
    let mut refused = Vec::new();
    for guard in ["tags.0 == vip", "tags.0 != vip"] {
        let (run, suite) = synthesize(&with_when(model(), guard));
        let validated = validate(&with_when(model(), guard));
        assert_eq!(
            validated.code,
            Some(0),
            "`{guard}` validates:\n{}",
            validated.output
        );
        let inputs = suite.as_ref().map(place_order_inputs).unwrap_or_default();
        if run.code != Some(0) || inputs.is_empty() || run.output.contains("ESS-SYNTH-002") {
            refused.push(format!("`{guard}` exited {:?}\n{}", run.code, run.output));
        }
    }
    assert!(refused.is_empty(), "{}", refused.join("\n"));
}
