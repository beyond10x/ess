//! Adversary, w1fix pass 1: the sentence the unit rewrote on `website/docs/reference/predicates.md`,
//! "a list set from the input is one of them: `tags: input.tags` sends a list, so `tags.count` and a
//! quantifier over `tags` are decided". The page runs only the `.count` half; these run the
//! quantifier half against the page's own model, the way `predicate_reference_page.rs` runs a
//! `filter:` fragment.

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

fn with_filter(mut model: Value, filter: &str) -> Value {
    let filter: Value = serde_yaml::from_str(filter).expect("the filter is YAML");
    model
        .get_mut("views")
        .and_then(Value::as_sequence_mut)
        .and_then(|views| {
            views.iter_mut().find(|view| {
                view.get("name").and_then(Value::as_str) == Some("shop.order.OpenOrders")
            })
        })
        .expect("the page's model declares shop.order.OpenOrders")
        .as_mapping_mut()
        .expect("a view is a mapping")
        .insert(Value::String("filter".to_owned()), filter);
    model
}

fn ess(arguments: &[&str], path: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .arg("--path")
        .arg(path)
        .output()
        .expect("run the built ess binary");
    (
        output.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn synthesizes(name: &str, filter: &str) {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-w1fix-page-{name}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("scratch");
    let path = directory.join("model.yaml");
    fs::write(
        &path,
        serde_yaml::to_string(&with_filter(page_model(), filter)).expect("serialize"),
    )
    .expect("write");
    let (code, validated) = ess(&["specify", "validate"], &path);
    let (synth_code, synthesized) = ess(&["verify", "conform", "synthesize"], &path);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(code, Some(0), "`{filter}` validates\n{validated}");
    assert_eq!(synth_code, Some(0), "{synthesized}");
    let refusals: Vec<&str> = synthesized
        .lines()
        .filter(|line| line.starts_with("refused:"))
        .collect();
    assert!(
        refusals.is_empty(),
        "the page says a quantifier over `tags` is decided; `{filter}` was refused\n{synthesized}"
    );
}

#[test]
fn adversary_w1fix_a_filter_exists_over_an_input_list_synthesizes() {
    synthesizes("exists", "exists: {in: tags, as: t, that: t == vip}");
}

#[test]
fn adversary_w1fix_a_filter_forall_over_an_input_list_synthesizes() {
    synthesizes("forall", "forall: {in: tags, as: t, that: t != spam}");
}
