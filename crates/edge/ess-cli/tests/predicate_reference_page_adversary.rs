//! Adversarial cases for the predicate reference page (beyond10x/ess#92).
//!
//! Each case takes one sentence of `website/docs/reference/predicates.md` as the specification,
//! builds the smallest fragment that sentence rules on, splices it into the page's own model, and
//! asks the built `ess` binary. A red case here is a sentence on the page the code does not keep.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_yaml::Value;

const PAGE: &str = "website/docs/reference/predicates.md";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The page's one `ess-check="model"` block.
fn page_model() -> Value {
    let page = fs::read_to_string(root().join(PAGE)).expect("read the page");
    let start = page
        .find("```yaml ess-check=\"model\"")
        .expect("the page has a model block");
    let body = &page[start..];
    let body = &body[body.find('\n').expect("fence line ends") + 1..];
    let body = &body[..body.find("\n```").expect("the model block closes")];
    serde_yaml::from_str(body).expect("the page's model is YAML")
}

fn named<'a>(list: &'a mut Value, key: &str, name: &str) -> &'a mut Value {
    list.get_mut(key)
        .and_then(Value::as_sequence_mut)
        .and_then(|items| {
            items
                .iter_mut()
                .find(|item| item.get("name").and_then(Value::as_str) == Some(name))
        })
        .unwrap_or_else(|| panic!("the page's model declares no {key} entry named `{name}`"))
}

fn place_order(model: &mut Value) -> &mut Value {
    named(model, "commands", "shop.order.PlaceOrder")
}

/// The page's model with `when` (a YAML fragment) on outcome `placed` of `PlaceOrder`.
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

fn validate(document: &Value, name: &str) -> Run {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let path = directory.path().join(format!("{name}.yaml"));
    fs::write(&path, serde_yaml::to_string(document).expect("serialize")).expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["specify", "validate", "--path"])
        .arg(&path)
        .output()
        .expect("run the built ess binary");
    Run {
        code: output.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

/// predicates.md, "Ordering": "`Duration` has no ordering."
#[test]
fn the_page_says_duration_has_no_ordering_so_validate_refuses_ordering_a_duration() {
    let mut model = page_model();
    place_order(&mut model)
        .get_mut("input")
        .and_then(Value::as_sequence_mut)
        .expect("PlaceOrder has an input list")
        .push(serde_yaml::from_str("{name: wait, type: Duration}").expect("a field"));
    let document = with_when(model, r#"wait > "PT5M""#);
    let run = validate(&document, "duration-ordering");
    assert_eq!(
        run.code,
        Some(1),
        "{PAGE} says `Duration` has no ordering, and validate admitted `wait > \"PT5M\"` on a \
         Duration input:\n{}",
        run.output
    );
}

/// predicates.md, "`.count` and ordinals": "One element of a list is reachable by its position,
/// counted from `0`: `tags.0` is the first element."
///
/// Rewritten after coordinator decision F2: ordinals are admitted and documented, not refused.
#[test]
fn the_page_says_a_list_element_is_reachable_by_its_ordinal_so_validate_admits_one() {
    let document = with_when(page_model(), "tags.0 == vip");
    let run = validate(&document, "list-ordinal");
    assert_eq!(
        run.code,
        Some(0),
        "decision F2: list ordinals are admitted and documented on {PAGE}, and validate refused \
         `tags.0 == vip`:\n{}",
        run.output
    );
}

/// predicates.md, "Compact forms": "The compact form has no `&&`, `||` or `in [...]`. Write a
/// conjunction, disjunction or membership in the structured form below." — followed by an example
/// that is refused.
#[test]
fn the_page_says_the_compact_form_has_no_conjunction_so_validate_refuses_one_over_text() {
    let mut admitted = Vec::new();
    for guard in ["sku == A1 && gift", "sku == A1 || sku == B2"] {
        let run = validate(&with_when(page_model(), guard), "compact-conjunction");
        if run.code != Some(1) {
            admitted.push(format!("`{guard}` exited {:?}\n{}", run.code, run.output));
        }
    }
    assert!(
        admitted.is_empty(),
        "{PAGE} says the compact form has no `&&` or `||`, and validate admitted these, reading \
         everything after `==` as one text literal:\n{}",
        admitted.join("\n")
    );
}

/// predicates.md, "Compact forms": "An equality whose text literal is the name of a declared field
/// is refused rather than read as text."
///
/// Rewritten after coordinator decision F4: the refusal is kept and documented, not relaxed.
#[test]
fn the_page_says_a_word_naming_a_field_is_refused_as_a_literal_with_command_002() {
    let run = validate(
        &with_when(page_model(), "sku == gift"),
        "bare-word-field-name",
    );
    assert_eq!(
        run.code,
        Some(1),
        "decision F4: {PAGE} documents that `sku == gift` is refused because `gift` names a \
         declared field, and validate admitted it:\n{}",
        run.output
    );
    assert!(
        run.output.contains("ESS-COMMAND-002"),
        "decision F4: {PAGE} documents the refusal of `sku == gift` as ESS-COMMAND-002, and \
         validate named another code:\n{}",
        run.output
    );
}

/// predicates.md, "The equality shorthand reads a literal": "`{sku: A.1}` compares `sku` with the
/// text `A.1`, while `{sku: {eq: A.1}}` and `sku == A.1` both read a path `A.1` and are refused
/// here."
///
/// Rewritten after coordinator decision F5: the three spellings differ, and the page says so.
#[test]
fn the_equality_shorthand_reads_a_dotted_word_as_text_while_eq_and_compact_read_a_path() {
    let expected: [(&str, Option<i32>, Option<&str>); 3] = [
        ("sku == A.1", Some(1), Some("ESS-COMMAND-003")),
        ("{sku: A.1}", Some(0), None),
        ("{sku: {eq: A.1}}", Some(1), Some("ESS-COMMAND-003")),
    ];
    let mut disagreements = Vec::new();
    for (guard, code, named) in expected {
        let run = validate(&with_when(page_model(), guard), "equality-spellings");
        let named_ok = named.is_none_or(|named| run.output.contains(named));
        if run.code != code || !named_ok {
            disagreements.push(format!(
                "`{guard}` exited {:?}, expected {code:?}{}\n{}",
                run.code,
                named.map(|n| format!(" naming {n}")).unwrap_or_default(),
                run.output
            ));
        }
    }
    assert!(
        disagreements.is_empty(),
        "pre-existing difference documented on the page; change it here and on the page \
         together ({PAGE}):\n{}",
        disagreements.join("\n")
    );
}
