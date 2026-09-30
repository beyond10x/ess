//! The reference as a format-neutral list of blocks, built once from the schema and written by
//! [`crate::html`] and [`crate::markdown`], so both pages say the same thing.

use std::collections::BTreeSet;

use serde_yaml::Value;

use crate::model::{Construct, Schema};
use crate::yaml::flow;

/// A run of inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Inline {
    /// Literal text.
    Text(String),
    /// Author prose from the schema, in which backticks delimit code.
    Prose(String),
    /// Code.
    Code(String),
    /// A link to a heading of this page.
    Link { text: String, anchor: String },
}

/// One block of the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Block {
    /// Opens a region (`chapter` or `construct`) whose heading follows.
    Open { class: &'static str, id: String },
    /// Closes the innermost open region.
    Close,
    /// A heading; `level` 1 is the page title.
    Heading { level: u8, text: String },
    /// A small label heading, never linked to.
    Label(String),
    /// A paragraph.
    Para(Vec<Inline>),
    /// A table.
    Table {
        class: &'static str,
        head: Vec<&'static str>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    /// A YAML value, shown as an example.
    Yaml(Value),
}

/// The id of the HTML page's sidebar filter; reserved, so no heading takes it.
pub(crate) const FILTER_ID: &str = "ess-ui-filter";

/// Where the schema lives, as the generated pages name it.
pub(crate) const SOURCE: &str = "schemas/ui/ess-ui.schema.yaml";

/// The heading id a documentation site derives from `text` (lowercase, punctuation dropped,
/// spaces as `-`), used as the anchor in both formats.
pub(crate) fn slug(text: &str) -> String {
    text.chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                Some(c.to_lowercase().collect::<String>())
            } else if c == ' ' {
                Some("-".to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn text(s: impl Into<String>) -> Inline {
    Inline::Text(s.into())
}

fn prose(s: impl Into<String>) -> Inline {
    Inline::Prose(s.into())
}

fn code(s: impl Into<String>) -> Inline {
    Inline::Code(s.into())
}

/// A scalar as prose, anything else as inline YAML.
fn prose_of(value: &Value) -> Inline {
    match value {
        Value::String(s) => prose(s.clone()),
        other => code(flow(other)),
    }
}

/// Builds the page, refusing it when two linked headings would share an anchor.
pub(crate) fn build(schema: &Schema<'_>) -> Result<Vec<Block>, Vec<String>> {
    let mut page = Page {
        schema,
        blocks: Vec::new(),
        anchors: BTreeSet::from([FILTER_ID.to_owned()]),
        problems: Vec::new(),
    };
    page.title();
    page.foundations();
    for group in &schema.groups {
        page.open("chapter", 2, group.title);
        page.blocks.push(Block::Para(vec![prose(group.summary)]));
        let mut members: Vec<&Construct<'_>> = schema
            .constructs
            .iter()
            .filter(|construct| construct.group == group.name)
            .collect();
        members.sort_by_key(|construct| construct.order);
        for construct in members {
            page.construct(construct);
        }
        page.blocks.push(Block::Close);
    }
    page.checks();
    page.retrofits();
    page.others();
    page.quick_reference();
    if page.problems.is_empty() {
        Ok(page.blocks)
    } else {
        Err(page.problems)
    }
}

struct Page<'s, 'a> {
    schema: &'s Schema<'a>,
    blocks: Vec<Block>,
    anchors: BTreeSet<String>,
    problems: Vec<String>,
}

/// The top-level keys this page lays out itself; any other is shown as YAML under "Other
/// declarations", so nothing the schema says is dropped.
const LAID_OUT: &[&str] = &[
    "format",
    "type_rule",
    "expressions",
    "unmapped_marker",
    "shorthands",
    "groups",
    "layers",
    "constructs",
    "lowering",
    "traceability",
    "checks",
];

impl Page<'_, '_> {
    fn open(&mut self, class: &'static str, level: u8, heading: &str) {
        let id = slug(heading);
        if id.is_empty() {
            self.problems.push(format!(
                "the heading `{heading}` has no letter or digit, so it gets no anchor"
            ));
        } else if !self.anchors.insert(id.clone()) {
            self.problems.push(format!(
                "two headings share the anchor `#{id}` (`{heading}`)"
            ));
        }
        self.blocks.push(Block::Open { class, id });
        self.blocks.push(Block::Heading {
            level,
            text: heading.to_owned(),
        });
    }

    fn para(&mut self, inlines: Vec<Inline>) {
        self.blocks.push(Block::Para(inlines));
    }

    fn label(&mut self, text: &str) {
        self.blocks.push(Block::Label(text.to_owned()));
    }

    fn title(&mut self) {
        let format = self.schema.format;
        self.blocks.push(Block::Heading {
            level: 1,
            text: format!("{format} reference"),
        });
        self.anchors.insert(slug(&format!("{format} reference")));
        self.para(vec![
            text(format!("Every construct of {} ", article(format))),
            code(format),
            text(
                " document: what it is for, its properties, the short forms it accepts, an \
                  example and the checks that apply to it. This page is generated from ",
            ),
            code(SOURCE),
            text(" by "),
            code("ess ui docs"),
            text("; change the schema, not the page."),
        ]);
    }

    fn foundations(&mut self) {
        let root = self.schema.root;
        self.open("chapter", 2, "Foundations");
        self.para(vec![text(
            "How types, expressions, short forms and nesting work in every construct below.",
        )]);
        self.type_rule(&root["type_rule"]);
        self.expressions(&root["expressions"]);
        self.short_forms(&root["shorthands"]);
        self.layers(&root["layers"]);
        self.blocks.push(Block::Close);
    }

    fn type_rule(&mut self, rule: &Value) {
        self.open("section", 3, "Type rule");
        self.para(vec![prose_of(&rule["summary"])]);
        self.para(vec![prose_of(&rule["doc"])]);
        self.label("Primitives");
        let rows = mapping(&rule["primitives"])
            .map(|(name, spec)| vec![vec![code(name)], described(spec, "note", &[])])
            .collect();
        self.blocks.push(Block::Table {
            class: "plain",
            head: vec!["Primitive", "Meaning"],
            rows,
        });
        self.label("Constructors");
        let rows = mapping(&rule["constructors"])
            .map(|(name, spec)| {
                let mut notes = described(spec, "note", &["form", "example"]);
                if notes.is_empty() {
                    notes.push(text(""));
                }
                vec![
                    vec![code(name)],
                    vec![value_code(&spec["form"])],
                    vec![value_code(&spec["example"])],
                    notes,
                ]
            })
            .collect();
        self.blocks.push(Block::Table {
            class: "plain",
            head: vec!["Constructor", "Form", "Example", "Notes"],
            rows,
        });
        self.blocks.push(Block::Close);
    }

    fn expressions(&mut self, expressions: &Value) {
        if !expressions.is_null() {
            self.open("section", 3, "Expressions");
            self.para(vec![prose_of(&expressions["summary"])]);
            self.para(vec![prose_of(&expressions["doc"])]);
            let rows = expressions["forms"]
                .as_sequence()
                .into_iter()
                .flatten()
                .map(|form| {
                    vec![
                        vec![code(scalar(&form["form"]))],
                        vec![prose_of(&form["note"])],
                    ]
                })
                .collect();
            self.blocks.push(Block::Table {
                class: "plain",
                head: vec!["Form", "Meaning"],
                rows,
            });
            self.blocks.push(Block::Close);
        }
    }

    fn short_forms(&mut self, shorthands: &Value) {
        if !shorthands.is_null() {
            self.open("section", 3, "Short forms");
            self.para(vec![prose_of(&shorthands["summary"])]);
            if let Some(validation) = shorthands.get("validation") {
                self.para(vec![text("Validation: "), prose_of(validation)]);
            }
            for (key, head, label) in [
                (
                    "placeholders",
                    "Placeholder",
                    "Placeholders in an expansion",
                ),
                ("operators", "Operator", "Operators in an expansion"),
            ] {
                if shorthands.get(key).is_some() {
                    self.label(label);
                    let rows = mapping(&shorthands[key])
                        .map(|(name, meaning)| vec![vec![code(name)], vec![prose_of(meaning)]])
                        .collect();
                    self.blocks.push(Block::Table {
                        class: "plain",
                        head: vec![head, "Meaning"],
                        rows,
                    });
                }
            }
            if let Some(absent) = shorthands.get("absent_placeholder") {
                self.para(vec![
                    text("A placeholder that resolves to nothing: "),
                    prose_of(absent),
                ]);
            }
            let inheritance = &shorthands["inheritance"];
            if !inheritance.is_null() {
                self.label("Inheritance");
                if let Some(summary) = inheritance.get("summary") {
                    self.para(vec![prose_of(summary)]);
                }
                self.blocks
                    .push(Block::Yaml(without(inheritance, &["summary"])));
            }
            self.blocks.push(Block::Close);
        }
    }

    fn layers(&mut self, layers: &Value) {
        if let Some(layers) = layers.as_sequence() {
            self.open("section", 3, "Layers");
            self.para(vec![text("What may nest inside what.")]);
            let rows = layers
                .iter()
                .map(|layer| {
                    vec![
                        vec![code(scalar(&layer["name"]))],
                        codes(&layer["contains"]),
                        vec![prose_of(&layer["note"])],
                    ]
                })
                .collect();
            self.blocks.push(Block::Table {
                class: "plain",
                head: vec!["Layer", "Contains", "Note"],
                rows,
            });
            self.blocks.push(Block::Close);
        }
    }

    fn construct(&mut self, construct: &Construct<'_>) {
        self.open("construct", 3, construct.name);
        self.blocks
            .push(Block::Para(vec![prose(construct.summary)]));
        self.para(vec![prose(construct.doc.trim_end())]);

        self.label("Properties");
        let rows = construct
            .fields
            .iter()
            .map(|field| {
                let name = if field.name.starts_with('(') {
                    text(field.name)
                } else {
                    code(field.name)
                };
                let mut note = vec![prose(field.note)];
                for (key, value) in &field.extras {
                    note.push(text(format!("; {}: ", key.replace('_', " "))));
                    note.push(code(flow(value)));
                }
                vec![
                    vec![name],
                    self.type_inlines(field.ty),
                    vec![text(if field.required { "yes" } else { "" })],
                    vec![field
                        .default
                        .map_or_else(|| text(""), |value| code(flow(value)))],
                    note,
                ]
            })
            .collect();
        self.blocks.push(Block::Table {
            class: "fields",
            head: vec!["Property", "Type", "Required", "Default", "Note"],
            rows,
        });

        if !construct.shorthands.is_empty() {
            self.label("You may also write");
            let rows = construct
                .shorthands
                .iter()
                .map(|shorthand| {
                    let at = match shorthand["at"].as_str() {
                        Some(at) => vec![code(at)],
                        None => vec![text("the value itself")],
                    };
                    let accepts = match shorthand["accepts"].as_str() {
                        Some("absent") => vec![text("nothing: leave it out")],
                        _ => self.type_inlines(&shorthand["accepts"]),
                    };
                    vec![at, accepts, vec![code(flow(&shorthand["expands_to"]))]]
                })
                .collect();
            self.blocks.push(Block::Table {
                class: "plain",
                head: vec!["Where", "You write", "It means"],
                rows,
            });
        }

        for (key, value) in &construct.extras {
            self.label(&heading_of(key));
            match value.as_sequence() {
                Some(items) if items.iter().all(Value::is_string) => {
                    self.para(join_codes(items.iter().map(scalar)));
                }
                _ => self.blocks.push(Block::Yaml((*value).clone())),
            }
        }

        self.label("Example");
        self.blocks.push(Block::Yaml(construct.example.clone()));

        let checks = self.checks_of(construct.name);
        if !checks.is_empty() {
            self.label("Checks");
            self.blocks.push(Block::Table {
                class: "plain",
                head: vec!["Check", "Subject", "Rule", "Severity"],
                rows: checks,
            });
        }
        self.blocks.push(Block::Close);
    }

    /// The rows of every check whose subject resolves to `construct`.
    fn checks_of(&self, construct: &str) -> Vec<Vec<Vec<Inline>>> {
        self.schema.root["checks"]["list"]
            .as_sequence()
            .into_iter()
            .flatten()
            .filter(|check| {
                subjects(&check["subject"])
                    .iter()
                    .any(|subject| self.schema.check_owners(subject).contains(&construct))
            })
            .map(check_row)
            .collect()
    }

    fn checks(&mut self) {
        let checks = &self.schema.root["checks"];
        let Some(list) = checks["list"].as_sequence() else {
            return;
        };
        self.open("chapter", 2, "Checks");
        let mut intro = vec![text(
            "What a validator checks in a document, and the construct each check applies to.",
        )];
        if let Some(by) = checks["report_by"].as_str() {
            intro.push(text(" Every finding names the failing node by its "));
            intro.push(match self.schema.construct(by) {
                Some(_) => link(by),
                None => code(by),
            });
            intro.push(text("."));
        }
        self.para(intro);
        let rows = list
            .iter()
            .map(|check| {
                let mut row = check_row(check);
                let mut owners: Vec<&str> = Vec::new();
                for subject in subjects(&check["subject"]) {
                    for owner in self.schema.check_owners(&subject) {
                        if !owners.contains(&owner) {
                            owners.push(owner);
                        }
                    }
                }
                let applies = if owners.is_empty() {
                    vec![text("every node")]
                } else {
                    let mut inlines = Vec::new();
                    for (i, owner) in owners.iter().enumerate() {
                        if i > 0 {
                            inlines.push(text(", "));
                        }
                        inlines.push(link(owner));
                    }
                    inlines
                };
                row.insert(1, applies);
                row
            })
            .collect();
        self.blocks.push(Block::Table {
            class: "plain",
            head: vec!["Check", "Applies to", "Subject", "Rule", "Severity"],
            rows,
        });
        self.blocks.push(Block::Close);
    }

    fn retrofits(&mut self) {
        let root = self.schema.root;
        let marker = &root["unmapped_marker"];
        let traceability = &root["traceability"];
        let lowering = &root["lowering"];
        if marker.is_null() && traceability.is_null() && lowering.is_null() {
            return;
        }
        self.open("chapter", 2, "Retrofits and lowering");
        self.para(vec![text(
            "How a document records what a retrofit could not determine, where it came from, and \
             what each construct becomes in the ESS model.",
        )]);
        if !marker.is_null() {
            self.open("section", 3, "Unmapped marker");
            self.para(vec![prose_of(&marker["summary"])]);
            self.blocks.push(Block::Yaml(without(marker, &["summary"])));
            self.blocks.push(Block::Close);
        }
        if traceability.is_mapping() {
            self.open("section", 3, "Traceability");
            let rows = mapping(traceability)
                .map(|(key, value)| vec![vec![code(key)], vec![prose_of(value)]])
                .collect();
            self.blocks.push(Block::Table {
                class: "plain",
                head: vec!["Key", "Carried"],
                rows,
            });
            self.blocks.push(Block::Close);
        }
        if let Some(entries) = lowering.as_sequence() {
            self.open("section", 3, "Lowering");
            let rows = entries
                .iter()
                .map(|entry| {
                    vec![
                        vec![prose_of(&entry["construct"])],
                        vec![prose_of(&entry["lowers_to"])],
                    ]
                })
                .collect();
            self.blocks.push(Block::Table {
                class: "plain",
                head: vec!["Construct", "Lowers to"],
                rows,
            });
            self.blocks.push(Block::Close);
        }
        self.blocks.push(Block::Close);
    }

    fn others(&mut self) {
        let others: Vec<(&str, &Value)> = mapping(self.schema.root)
            .filter(|(key, _)| !LAID_OUT.contains(key))
            .collect();
        if others.is_empty() {
            return;
        }
        self.open("chapter", 2, "Other declarations");
        for (key, value) in others {
            self.label(&heading_of(key));
            self.blocks.push(Block::Yaml(value.clone()));
        }
        self.blocks.push(Block::Close);
    }

    fn quick_reference(&mut self) {
        self.open("chapter", 2, "Quick reference");
        self.para(vec![text(
            "Every construct with its one-line summary, chapter by chapter.",
        )]);
        let mut rows = Vec::new();
        for group in &self.schema.groups {
            let mut members: Vec<&Construct<'_>> = self
                .schema
                .constructs
                .iter()
                .filter(|construct| construct.group == group.name)
                .collect();
            members.sort_by_key(|construct| construct.order);
            for construct in members {
                rows.push(vec![
                    vec![link(construct.name)],
                    vec![Inline::Link {
                        text: group.title.to_owned(),
                        anchor: slug(group.title),
                    }],
                    vec![prose(construct.summary)],
                ]);
            }
        }
        self.blocks.push(Block::Table {
            class: "quick",
            head: vec!["Construct", "Chapter", "Summary"],
            rows,
        });
        self.blocks.push(Block::Close);
    }

    /// A type expression in words, linking every construct it names.
    fn type_inlines(&self, ty: &Value) -> Vec<Inline> {
        let mut out = Vec::new();
        self.type_into(ty, false, &mut out);
        out
    }

    fn type_into(&self, ty: &Value, nested: bool, out: &mut Vec<Inline>) {
        match ty {
            Value::String(name) => {
                if self.schema.construct(name).is_some() {
                    out.push(link(name));
                } else {
                    out.push(code(name.clone()));
                }
            }
            Value::Mapping(map) => {
                if let Some(inner) = map.get("list") {
                    out.push(text("list of "));
                    self.type_into(inner, true, out);
                    if map.get("unique").and_then(Value::as_bool) == Some(true) {
                        out.push(text(", no duplicates"));
                    }
                } else if let Some(inner) = map.get("optional") {
                    out.push(text("optional "));
                    self.type_into(inner, true, out);
                } else if let Some(inner) = map.get("map") {
                    out.push(text("map of "));
                    self.type_into(&inner["key"], true, out);
                    out.push(text(" → "));
                    self.type_into(&inner["value"], true, out);
                } else if let Some(values) = map.get("enum") {
                    let alternatives: Vec<Vec<Inline>> = values
                        .as_sequence()
                        .into_iter()
                        .flatten()
                        .map(|value| vec![code(scalar(value))])
                        .collect();
                    one_of(alternatives, nested, out);
                } else if let Some(alternatives) = map.get("one_of") {
                    let mut rendered = Vec::new();
                    for alternative in alternatives.as_sequence().into_iter().flatten() {
                        if let Some(values) = alternative.get("enum").and_then(Value::as_sequence) {
                            rendered.extend(values.iter().map(|value| vec![code(scalar(value))]));
                        } else {
                            let mut inlines = Vec::new();
                            self.type_into(alternative, true, &mut inlines);
                            rendered.push(inlines);
                        }
                    }
                    one_of(rendered, nested, out);
                } else if let Some(fields) = map.get("record") {
                    out.push(text("record { "));
                    for (i, (field, inner)) in mapping(fields).enumerate() {
                        if i > 0 {
                            out.push(text(", "));
                        }
                        out.push(code(field));
                        out.push(text(": "));
                        self.type_into(inner, true, out);
                    }
                    out.push(text(" }"));
                } else if let Some(kind) = map.get("ref").and_then(Value::as_str) {
                    if let Some(construct) = self.schema.ref_construct(kind) {
                        out.push(text(format!("name of {} ", article(construct.name))));
                        out.push(link(construct.name));
                    } else {
                        out.push(text("name of an ESS "));
                        out.push(code(kind));
                    }
                } else if let Some(value) = map.get("const") {
                    out.push(text("exactly "));
                    out.push(code(flow(value)));
                } else {
                    out.push(code(flow(ty)));
                }
            }
            other => out.push(code(flow(other))),
        }
    }
}

/// A value as inline YAML; nothing when it is absent.
fn value_code(value: &Value) -> Inline {
    if value.is_null() {
        text("")
    } else {
        code(flow(value))
    }
}

fn link(name: &str) -> Inline {
    Inline::Link {
        text: name.to_owned(),
        anchor: slug(name),
    }
}

fn one_of(alternatives: Vec<Vec<Inline>>, nested: bool, out: &mut Vec<Inline>) {
    if nested {
        out.push(text("("));
    }
    out.push(text("one of: "));
    for (i, alternative) in alternatives.into_iter().enumerate() {
        if i > 0 {
            out.push(text(" | "));
        }
        out.extend(alternative);
    }
    if nested {
        out.push(text(")"));
    }
}

pub(crate) fn article(name: &str) -> &'static str {
    if name.starts_with(|c: char| "aeiouAEIOU".contains(c)) {
        "an"
    } else {
        "a"
    }
}

/// A scalar's text without quotes; anything else in flow style.
fn scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => flow(other),
    }
}

fn mapping(value: &Value) -> impl Iterator<Item = (&str, &Value)> {
    value
        .as_mapping()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| key.as_str().map(|key| (key, value)))
}

fn without(value: &Value, keys: &[&str]) -> Value {
    let mut map = value.as_mapping().cloned().unwrap_or_default();
    for key in keys {
        map.remove(*key);
    }
    Value::Mapping(map)
}

/// `spec[key]` as prose, followed by every other key of `spec` as `key: value`.
/// `spec[key]` as prose, then every other key of `spec` but `skip` as `key: value`.
fn described(spec: &Value, key: &str, skip: &[&str]) -> Vec<Inline> {
    let mut out = Vec::new();
    if let Some(value) = spec.get(key) {
        out.push(prose_of(value));
    }
    for (other, value) in mapping(spec) {
        if other == key || skip.contains(&other) {
            continue;
        }
        if !out.is_empty() {
            out.push(text("; "));
        }
        out.push(text(format!("{}: ", other.replace('_', " "))));
        out.push(code(flow(value)));
    }
    out
}

fn codes(value: &Value) -> Vec<Inline> {
    match value.as_sequence() {
        Some(items) if items.is_empty() => vec![text("nothing")],
        Some(items) => join_codes(items.iter().map(scalar)),
        None => vec![code(scalar(value))],
    }
}

fn join_codes(items: impl Iterator<Item = String>) -> Vec<Inline> {
    let mut out = Vec::new();
    for (i, item) in items.enumerate() {
        if i > 0 {
            out.push(text(", "));
        }
        out.push(code(item));
    }
    out
}

/// A schema key as a label: `exactly_one_of` becomes "Exactly one of".
fn heading_of(key: &str) -> String {
    let words = key.replace('_', " ");
    let mut chars = words.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

/// A check's `subject` as a list of paths.
fn subjects(subject: &Value) -> Vec<String> {
    match subject {
        Value::Sequence(items) => items.iter().map(scalar).collect(),
        other => vec![scalar(other)],
    }
}

/// A check as `id | subject | rule | severity`: the rule is every key but those three.
fn check_row(check: &Value) -> Vec<Vec<Inline>> {
    vec![
        vec![code(scalar(&check["id"]))],
        codes(&check["subject"]),
        vec![code(flow(&without(check, &["id", "subject", "severity"])))],
        vec![text(scalar(&check["severity"]))],
    ]
}
