//! The predicate reference page cannot drift from the grammar (beyond10x/ess#92).
//!
//! `website/docs/reference/predicates.md` is the one place an adopter reads every predicate form
//! ESS accepts. Every YAML block on it is an example, and every example carries attributes on its
//! code fence that say how it is checked:
//!
//! | attribute | meaning |
//! |---|---|
//! | `ess-check="model"` | the page's one complete specification; every fragment is spliced into it |
//! | `ess-check="when"` | a `when:` fragment, put on outcome `placed` of `shop.order.PlaceOrder` |
//! | `ess-check="invariants"` | an `invariants:` fragment, the invariants of entity `shop.order.Order` |
//! | `ess-check="filter"` | a `filter:` fragment, the filter of view `shop.order.OpenOrders` |
//! | `ess-expect="synthesizes"` | validates, and synthesis witnesses every branch with no refusal |
//! | `ess-expect="valid"` | validates; the page makes no claim about synthesis |
//! | `ess-expect="unwitnessed:<code>"` | validates, and synthesis refuses a scenario with `<code>` |
//! | `ess-expect="refused"` / `"refused:<code>"` | validate refuses it, naming `<code>` if given |
//! | `ess-says="<text>"` | the output of the deciding command contains `<text>` |
//! | `ess-pending="<issue>"` | documents behaviour that lands with `<issue>`; a strict expected failure |
//!
//! A YAML block without `ess-check` is refused rather than skipped, so an example cannot be added
//! to the page without being run. An `ess-pending` example runs too: it must disagree with the page
//! today, it prints the issue it waits for, and the test fails as soon as it agrees, so the marker
//! is removed when the change lands rather than whenever somebody remembers.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_yaml::Value;

const PAGE: &str = "website/docs/reference/predicates.md";
const SIDEBAR: &str = "website/sidebars.ts";
const PARSER: &str = "crates/specify/ess-primitives/src/predicate.rs";

const COMMAND: &str = "shop.order.PlaceOrder";
const OUTCOME: &str = "placed";
const ENTITY: &str = "shop.order.Order";
const VIEW: &str = "shop.order.OpenOrders";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

/// One fenced YAML block of the page and the attributes on its fence.
struct Example {
    line: usize,
    attributes: BTreeMap<String, String>,
    body: String,
}

impl Example {
    fn at(&self) -> String {
        format!("{PAGE}:{}", self.line)
    }

    fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    fn check(&self) -> &str {
        self.attribute("ess-check").unwrap_or("")
    }
}

/// Reads `key="value"` pairs from a fence's meta string. A token that is not one is ignored, so
/// Docusaurus's own `title="…"` and line ranges sit beside these.
fn attributes(meta: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    let mut rest = meta;
    while let Some(equals) = rest.find("=\"") {
        let key = rest[..equals]
            .rsplit(char::is_whitespace)
            .next()
            .unwrap_or_default()
            .to_owned();
        let after = &rest[equals + 2..];
        let Some(close) = after.find('"') else { break };
        found.insert(key, after[..close].to_owned());
        rest = &after[close + 1..];
    }
    found
}

fn examples(page: &str) -> Vec<Example> {
    let mut found = Vec::new();
    let mut open: Option<(usize, bool, BTreeMap<String, String>, String)> = None;
    for (index, line) in page.lines().enumerate() {
        let trimmed = line.trim_start();
        match open.take() {
            None => {
                if let Some(info) = trimmed.strip_prefix("```") {
                    // CommonMark does not open a backtick fence whose info string holds a
                    // backtick, so the site would render this line as text and pair every later
                    // fence with the wrong partner, while this reader went on seeing examples.
                    assert!(
                        !info.contains('`'),
                        "{PAGE}:{}: a fence's info string holds a backtick, which CommonMark \
                         does not read as a fence; the site renders the rest of the page \
                         shifted by one fence",
                        index + 1
                    );
                    let language = info.split_whitespace().next().unwrap_or_default();
                    let meta = info[language.len()..].trim();
                    let yaml = matches!(language, "yaml" | "yml");
                    open = Some((index + 1, yaml, attributes(meta), String::new()));
                }
            }
            Some((line_number, yaml, attributes, mut body)) => {
                if trimmed.starts_with("```") {
                    if yaml {
                        found.push(Example {
                            line: line_number,
                            attributes,
                            body,
                        });
                    }
                } else {
                    body.push_str(line);
                    body.push('\n');
                    open = Some((line_number, yaml, attributes, body));
                }
            }
        }
    }
    assert!(open.is_none(), "{PAGE} ends inside a code fence");
    found
}

fn page_examples() -> Vec<Example> {
    examples(&read(PAGE))
}

fn model(examples: &[Example]) -> Value {
    let models: Vec<&Example> = examples.iter().filter(|e| e.check() == "model").collect();
    assert_eq!(
        models.len(),
        1,
        "{PAGE} carries exactly one `ess-check=\"model\"` block; found {}",
        models.len()
    );
    serde_yaml::from_str(&models[0].body)
        .unwrap_or_else(|error| panic!("{}: the model is not YAML: {error}", models[0].at()))
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

/// The page's model with one fragment put where its `ess-check` says.
fn spliced(model: &Value, example: &Example) -> Result<Value, String> {
    let fragment: Value = serde_yaml::from_str(&example.body)
        .map_err(|error| format!("{}: the example is not YAML: {error}", example.at()))?;
    let key = example.check();
    let Some(value) = fragment
        .as_mapping()
        .filter(|map| map.len() == 1)
        .and_then(|map| map.get(key))
        .cloned()
    else {
        return Err(format!(
            "{}: an `ess-check=\"{key}\"` example is one mapping with the single key `{key}:`",
            example.at()
        ));
    };
    let mut document = model.clone();
    let target = match key {
        "when" => named(
            named(&mut document, "commands", COMMAND),
            "outcomes",
            OUTCOME,
        ),
        "invariants" => named(&mut document, "entities", ENTITY),
        "filter" => named(&mut document, "views", VIEW),
        other => return Err(format!("{}: unknown ess-check `{other}`", example.at())),
    };
    target
        .as_mapping_mut()
        .expect("a named entry is a mapping")
        .insert(Value::String(key.to_owned()), value);
    Ok(document)
}

struct Run {
    code: Option<i32>,
    output: String,
}

fn ess(arguments: &[&str], path: &Path) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .arg("--path")
        .arg(path)
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

/// Runs one example and says what disagreed with the page, if anything.
fn verdict(example: &Example, document: &Value, directory: &Path) -> Result<(), String> {
    let path = directory.join(format!("example-{}.yaml", example.line));
    fs::write(&path, serde_yaml::to_string(document).expect("serialize")).expect("write");
    let expect = example.attribute("ess-expect").unwrap_or("");
    let validate = ess(&["specify", "validate"], &path);
    let (kind, code) = expect.split_once(':').unwrap_or((expect, ""));
    let deciding = match kind {
        "refused" => {
            if validate.code != Some(1) {
                return Err(format!(
                    "{}: the page says validate refuses this; it exited {:?}\n{}",
                    example.at(),
                    validate.code,
                    validate.output
                ));
            }
            if !validate.output.contains(code) {
                return Err(format!(
                    "{}: validate refused it without naming `{code}`\n{}",
                    example.at(),
                    validate.output
                ));
            }
            validate
        }
        "valid" | "synthesizes" | "unwitnessed" => {
            if validate.code != Some(0) {
                return Err(format!(
                    "{}: the page says this validates; it exited {:?}\n{}",
                    example.at(),
                    validate.code,
                    validate.output
                ));
            }
            if kind == "valid" {
                validate
            } else {
                let synthesize = ess(&["verify", "conform", "synthesize"], &path);
                if synthesize.code != Some(0) {
                    return Err(format!(
                        "{}: synthesize exited {:?}\n{}",
                        example.at(),
                        synthesize.code,
                        synthesize.output
                    ));
                }
                let refusals: Vec<&str> = synthesize
                    .output
                    .lines()
                    .filter(|line| line.starts_with("refused:"))
                    .collect();
                if kind == "synthesizes" && !refusals.is_empty() {
                    return Err(format!(
                        "{}: the page says every branch is synthesized; synthesis refused {}\n{}",
                        example.at(),
                        refusals.len(),
                        synthesize.output
                    ));
                }
                if kind == "unwitnessed" {
                    let marker = format!("refusal[{code}]");
                    if code.is_empty() || !refusals.iter().any(|line| line.contains(&marker)) {
                        return Err(format!(
                            "{}: the page says synthesis refuses a scenario with `{code}`; it \
                             did not\n{}",
                            example.at(),
                            synthesize.output
                        ));
                    }
                }
                synthesize
            }
        }
        other => {
            return Err(format!(
                "{}: unknown ess-expect `{other}`; one of synthesizes, valid, \
                 unwitnessed:<code>, refused[:<code>]",
                example.at()
            ))
        }
    };
    if let Some(says) = example.attribute("ess-says") {
        if !deciding.output.contains(says) {
            return Err(format!(
                "{}: the page quotes `{says}`, which the output does not contain\n{}",
                example.at(),
                deciding.output
            ));
        }
    }
    Ok(())
}

#[test]
fn the_reference_sidebar_lists_the_predicate_page_beside_the_other_references() {
    let sidebar = read(SIDEBAR);
    let reference = sidebar
        .find("label: 'Reference'")
        .map(|start| &sidebar[start..])
        .and_then(|rest| rest.find(']').map(|end| &rest[..end]))
        .expect("the sidebar has a Reference category with an items list");
    for page in [
        "'reference/cli'",
        "'reference/formats'",
        "'reference/predicates'",
    ] {
        assert!(
            reference.contains(page),
            "{SIDEBAR}: the Reference items do not list {page}: {reference}"
        );
    }
}

#[test]
fn every_yaml_block_on_the_predicate_page_says_how_it_is_checked() {
    let examples = page_examples();
    assert!(!examples.is_empty(), "{PAGE} has no YAML examples");
    let mut undeclared = Vec::new();
    for example in &examples {
        let check = example.check();
        let declared = match check {
            "model" => true,
            "when" | "invariants" | "filter" => example.attribute("ess-expect").is_some(),
            _ => false,
        };
        if !declared {
            undeclared.push(format!(
                "{}: ess-check={check:?} ess-expect={:?}",
                example.at(),
                example.attribute("ess-expect")
            ));
        }
    }
    assert!(
        undeclared.is_empty(),
        "every YAML block needs `ess-check` of model/when/invariants/filter and a fragment needs \
         `ess-expect`:\n{}",
        undeclared.join("\n")
    );
}

#[test]
fn every_example_on_the_predicate_page_behaves_as_the_page_says() {
    let examples = page_examples();
    let model = model(&examples);
    let directory = tempfile::tempdir().expect("a temporary directory");

    let whole = Example {
        line: examples
            .iter()
            .find(|e| e.check() == "model")
            .map_or(0, |e| e.line),
        attributes: [("ess-expect".to_owned(), "synthesizes".to_owned())]
            .into_iter()
            .collect(),
        body: String::new(),
    };
    let mut failures = Vec::new();
    if let Err(failure) = verdict(&whole, &model, directory.path()) {
        failures.push(failure);
    }

    let mut ran = 1;
    let mut pending = Vec::new();
    for example in examples.iter().filter(|e| e.check() != "model") {
        ran += 1;
        let document = match spliced(&model, example) {
            Ok(document) => document,
            Err(failure) => {
                failures.push(failure);
                continue;
            }
        };
        let outcome = verdict(example, &document, directory.path());
        match (example.attribute("ess-pending"), outcome) {
            (None, Ok(())) => {}
            (None, Err(failure)) => failures.push(failure),
            // A strict expected failure: the example runs, and the day it agrees with the page is
            // the day its marker has to go, so agreement is what fails here.
            (Some(issue), Ok(())) => failures.push(format!(
                "{}: marked `ess-pending=\"{issue}\"`, and it now behaves as the page says; \
                 remove the marker",
                example.at()
            )),
            (Some(issue), Err(_)) => pending.push(format!(
                "expected failure {}: disagrees with the page until {issue} lands",
                example.at()
            )),
        }
    }
    for line in &pending {
        println!("{line}");
    }
    println!(
        "{PAGE}: {ran} example(s) run, {} of them expected failures pending an issue",
        pending.len()
    );
    assert!(
        failures.is_empty(),
        "{} example(s) disagree with the page:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// The keywords the parser dispatches on, read from its source rather than listed here, so an
/// operator added to the grammar is an operator this page has to name.
fn parser_keywords() -> BTreeSet<String> {
    let source = read(PARSER);
    let mut keywords = BTreeSet::new();
    for function in [
        "fn from_entry(",
        "fn from_operator(",
        "pub fn from_keyword(",
        "fn parse_expression_nested(",
    ] {
        let start = source
            .find(function)
            .unwrap_or_else(|| panic!("{PARSER} has no `{function}`"));
        let body = &source[start..];
        let end = body[1..]
            .find("\n    fn ")
            .map_or(body.len(), |end| end + 1);
        let end = end.min(
            body[1..]
                .find("\n    pub fn ")
                .map_or(body.len(), |e| e + 1),
        );
        for line in body[..end].lines() {
            let dispatch = line.contains("=>") || line.contains("for (function, negate)");
            if !dispatch {
                continue;
            }
            for (index, piece) in line.split('"').enumerate() {
                let literal = index % 2 == 1;
                if literal
                    && !piece.is_empty()
                    && piece.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                {
                    keywords.insert(piece.to_owned());
                }
            }
        }
    }
    keywords
}

#[test]
fn every_keyword_the_predicate_parser_dispatches_on_is_named_on_the_page() {
    let keywords = parser_keywords();
    for expected in [
        "all", "any_of", "none_of", "forall", "truthy", "gte", "missing",
    ] {
        assert!(
            keywords.contains(expected),
            "the keyword reader lost `{expected}`; it found {keywords:?}"
        );
    }
    let page = read(PAGE);
    let unnamed: Vec<&String> = keywords
        .iter()
        .filter(|keyword| !page.contains(&format!("`{keyword}`")))
        .collect();
    assert!(
        unnamed.is_empty(),
        "{PARSER} accepts these keywords and {PAGE} never names them in backticks: {unnamed:?}"
    );
}

fn collect(value: &Value, keys: &mut BTreeSet<String>, texts: &mut Vec<String>) {
    match value {
        Value::Mapping(map) => {
            for (key, value) in map {
                if let Some(key) = key.as_str() {
                    keys.insert(key.to_owned());
                }
                collect(value, keys, texts);
            }
        }
        Value::Sequence(items) => items.iter().for_each(|item| collect(item, keys, texts)),
        Value::String(text) => texts.push(text.clone()),
        Value::Tagged(tagged) => collect(&tagged.value, keys, texts),
        _ => {}
    }
}

#[test]
fn every_form_the_story_names_is_run_by_at_least_one_example_today() {
    let mut keys = BTreeSet::new();
    let mut texts = Vec::new();
    let mut sites = BTreeSet::new();
    let mut whole = Vec::new();
    for example in page_examples()
        .iter()
        .filter(|e| e.check() != "model" && e.attribute("ess-pending").is_none())
    {
        sites.insert(example.check().to_owned());
        let value: Value = serde_yaml::from_str(&example.body).unwrap_or(Value::Null);
        match value.get(example.check()) {
            Some(Value::String(text)) => whole.push(text.clone()),
            Some(Value::Sequence(items)) => {
                whole.extend(items.iter().filter_map(Value::as_str).map(str::to_owned));
            }
            _ => {}
        }
        collect(&value, &mut keys, &mut texts);
    }
    let mut missing = Vec::new();
    for site in ["when", "invariants", "filter"] {
        if !sites.contains(site) {
            missing.push(format!("an example at `{site}`"));
        }
    }
    for key in [
        "all", "any", "not", "none", "eq", "ne", "lt", "lte", "gt", "gte", "any_of", "in",
        "one_of", "none_of", "not_in", "exists", "defined", "truthy", "forall", "as", "that",
    ] {
        if !keys.contains(key) {
            missing.push(format!("the key `{key}:`"));
        }
    }
    for (what, found) in [
        (
            "a compact comparison",
            texts
                .iter()
                .any(|t| t.contains(" == ") || t.contains(" > ")),
        ),
        (
            "`defined(…)`",
            texts.iter().any(|t| t.starts_with("defined(")),
        ),
        (
            "a `not …` prefix",
            texts.iter().any(|t| t.starts_with("not ")),
        ),
        ("a `.count`", texts.iter().any(|t| t.contains(".count"))),
        (
            "a bare path as a whole predicate",
            whole.iter().any(|t| {
                !t.is_empty()
                    && !["true", "false", "always", "never"].contains(&t.as_str())
                    && t.chars()
                        .all(|c| c.is_ascii_lowercase() || c == '_' || c == '.')
            }),
        ),
    ] {
        if !found {
            missing.push(what.to_owned());
        }
    }
    assert!(
        missing.is_empty(),
        "{PAGE} runs no example with {}",
        missing.join(", ")
    );
}
