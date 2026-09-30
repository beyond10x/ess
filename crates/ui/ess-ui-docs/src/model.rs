//! The schema read as the reference needs it, refusing anything the reference could not document
//! completely: a construct without a summary, doc or example, or a type naming nothing.

use std::collections::BTreeSet;

use serde_yaml::{Mapping, Value};

/// One chapter.
pub(crate) struct Group<'a> {
    pub(crate) name: &'a str,
    pub(crate) order: u64,
    pub(crate) title: &'a str,
    pub(crate) summary: &'a str,
}

/// One property of a construct.
pub(crate) struct Field<'a> {
    pub(crate) name: &'a str,
    pub(crate) ty: &'a Value,
    pub(crate) required: bool,
    pub(crate) default: Option<&'a Value>,
    pub(crate) note: &'a str,
    /// Keys beside `type`, `required`, `default` and `note`, in schema order.
    pub(crate) extras: Vec<(&'a str, &'a Value)>,
}

/// One construct.
pub(crate) struct Construct<'a> {
    pub(crate) name: &'a str,
    pub(crate) group: &'a str,
    pub(crate) order: u64,
    pub(crate) summary: &'a str,
    pub(crate) doc: &'a str,
    pub(crate) fields: Vec<Field<'a>>,
    pub(crate) shorthands: Vec<&'a Value>,
    pub(crate) example: &'a Value,
    /// Every other key the construct declares, in schema order.
    pub(crate) extras: Vec<(&'a str, &'a Value)>,
}

/// The schema.
pub(crate) struct Schema<'a> {
    pub(crate) root: &'a Value,
    pub(crate) format: &'a str,
    /// In chapter order.
    pub(crate) groups: Vec<Group<'a>>,
    /// In schema order.
    pub(crate) constructs: Vec<Construct<'a>>,
    primitives: BTreeSet<&'a str>,
    constructors: Mapping,
    ref_kinds: BTreeSet<&'a str>,
}

const CONSTRUCT_KEYS: &[&str] = &[
    "group",
    "order",
    "summary",
    "doc",
    "fields",
    "shorthand",
    "example",
];
const FIELD_KEYS: &[&str] = &["type", "required", "default", "note"];

impl<'a> Schema<'a> {
    /// Reads `root`, or every reason it cannot be documented.
    pub(crate) fn read(root: &'a Value) -> Result<Self, Vec<String>> {
        let missing: Vec<&str> = SHAPE
            .iter()
            .copied()
            .filter(|key| root.get(*key).is_none())
            .collect();
        if !missing.is_empty() {
            return Err(vec![format!(
                "this is not an ess-ui schema: missing `{}`",
                missing.join("`, `")
            )]);
        }
        let mut problems = Vec::new();
        split_text(root, "", &mut problems);
        let format = text_of(&root["format"]).unwrap_or_else(|| {
            problems.push("the schema has no `format`".to_owned());
            ""
        });
        required_texts(root, &mut problems);
        let primitives: BTreeSet<&str> = root["type_rule"]["primitives"].as_mapping().map_or_else(
            || {
                problems.push("the schema has no `type_rule.primitives`".to_owned());
                BTreeSet::new()
            },
            |map| map.keys().filter_map(Value::as_str).collect(),
        );
        let constructors = root["type_rule"]["constructors"]
            .as_mapping()
            .cloned()
            .unwrap_or_else(|| {
                problems.push("the schema has no `type_rule.constructors`".to_owned());
                Mapping::new()
            });
        let ref_kinds: BTreeSet<&str> = root["type_rule"]["constructors"]["ref"]["kinds"]
            .as_sequence()
            .map(|kinds| kinds.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();

        let groups = read_groups(root, &mut problems);
        let mut constructs = Vec::new();
        let Some(declared) = root["constructs"].as_mapping() else {
            problems.push("the schema declares no `constructs`".to_owned());
            return Err(problems);
        };
        for (name, construct) in declared {
            let Some(name) = text_of(name) else {
                problems.push(format!(
                    "a construct is named {}; a name is text",
                    crate::yaml::flow(name)
                ));
                continue;
            };
            constructs.push(read_construct(name, construct, &groups, &mut problems));
        }

        let schema = Schema {
            root,
            format,
            groups,
            constructs,
            primitives,
            constructors,
            ref_kinds,
        };
        schema.check_constructs(&mut problems);
        if problems.is_empty() {
            Ok(schema)
        } else {
            Err(problems)
        }
    }

    /// Every field type and every shorthand of every construct.
    fn check_constructs(&self, problems: &mut Vec<String>) {
        for construct in &self.constructs {
            for field in &construct.fields {
                self.check_type(
                    field.ty,
                    &format!("{}.{}", construct.name, field.name),
                    problems,
                );
            }
            for shorthand in &construct.shorthands {
                for key in ["accepts", "expands_to"] {
                    if shorthand.get(key).is_none() {
                        problems.push(format!(
                            "{} shorthand at `{}` has no `{key}`",
                            construct.name,
                            shorthand["at"].as_str().unwrap_or("the value itself")
                        ));
                    }
                }
                let accepts = &shorthand["accepts"];
                if accepts.as_str() != Some("absent") && !accepts.is_null() {
                    self.check_type(accepts, &format!("{} shorthand", construct.name), problems);
                }
            }
        }
    }

    /// The construct named `name`.
    pub(crate) fn construct(&self, name: &str) -> Option<&Construct<'a>> {
        self.constructs
            .iter()
            .find(|construct| construct.name == name)
    }

    /// The construct a `{ref: kind}` names, if the kind resolves in this schema rather than in the
    /// ESS model: the construct whose name equals the kind without underscores and case, or, for a
    /// `<x>_kind`, the construct named `<x>`.
    pub(crate) fn ref_construct(&self, kind: &str) -> Option<&Construct<'a>> {
        let squash = |name: &str| name.replace('_', "").to_ascii_lowercase();
        let find = |kind: &str| {
            self.constructs
                .iter()
                .find(|construct| squash(construct.name) == squash(kind))
        };
        find(kind).or_else(|| kind.strip_suffix("_kind").and_then(find))
    }

    fn check_type(&self, ty: &Value, at: &str, problems: &mut Vec<String>) {
        match ty {
            Value::String(name) => {
                if self.primitives.contains(name.as_str()) || self.construct(name).is_some() {
                    return;
                }
                if name.starts_with(|c: char| c.is_uppercase()) {
                    problems.push(format!("{at}: the type `{name}` names no construct"));
                } else {
                    problems.push(format!(
                        "{at}: the type `{name}` is neither a primitive nor a construct"
                    ));
                }
            }
            Value::Mapping(map) => self.check_constructor(map, at, problems),
            other => problems.push(format!("{at}: {} is not a type", crate::yaml::flow(other))),
        }
    }

    fn check_constructor(&self, map: &Mapping, at: &str, problems: &mut Vec<String>) {
        let keys: Vec<&str> = map.keys().filter_map(Value::as_str).collect();
        let named: Vec<&str> = keys
            .iter()
            .copied()
            .filter(|key| self.constructors.contains_key(*key))
            .collect();
        let [constructor] = named.as_slice() else {
            let unknown: Vec<&str> = keys
                .iter()
                .copied()
                .filter(|key| !self.constructors.contains_key(*key))
                .collect();
            problems.push(format!(
                "{at}: a type map names exactly one constructor; it names [{}] and `{}` is not a type constructor",
                named.join(", "),
                unknown.join("`, `")
            ));
            return;
        };
        let options = &self.constructors[*constructor]["options"];
        for key in &keys {
            if key != constructor && options.get(*key).is_none() {
                problems.push(format!(
                    "{at}: `{key}` is not a type constructor nor an option of `{constructor}`"
                ));
            }
        }
        let inner = &map[*constructor];
        match *constructor {
            "list" | "optional" => self.check_type(inner, at, problems),
            "map" => {
                self.check_type(&inner["key"], at, problems);
                self.check_type(&inner["value"], at, problems);
            }
            "one_of" => match inner.as_sequence() {
                Some(alternatives) if !alternatives.is_empty() => {
                    for alternative in alternatives {
                        self.check_type(alternative, at, problems);
                    }
                }
                _ => problems.push(format!(
                    "{at}: `one_of` takes a non-empty list of types; a type with no alternative admits nothing"
                )),
            },
            "record" => match inner.as_mapping() {
                Some(fields) if !fields.is_empty() => {
                    for (field, ty) in fields {
                        let field = field.as_str().unwrap_or("?");
                        self.check_type(ty, &format!("{at}.{field}"), problems);
                    }
                }
                _ => problems.push(format!(
                    "{at}: `record` takes a non-empty map of fields"
                )),
            },
            "enum" if inner.as_sequence().is_none_or(Vec::is_empty) => {
                problems.push(format!(
                    "{at}: `enum` takes a non-empty list of values; an enum with no value admits nothing"
                ));
            }
            "ref" => match inner.as_str() {
                Some(kind) if self.ref_kinds.contains(kind) => {}
                Some(kind) => problems.push(format!(
                    "{at}: `{{ref: {kind}}}` names the kind `{kind}`, which `type_rule.constructors.ref.kinds` does not declare"
                )),
                None => problems.push(format!("{at}: `ref` takes a kind name")),
            },
            _ => {}
        }
    }

    /// The constructs a check's subject path resolves to through the schema's own field types.
    ///
    /// A subject `a.b[].c` (or `**.a.b`) starts at every construct with a field `a` and follows
    /// each segment through `optional`, `list`, `map`, `one_of`, `record` and construct names;
    /// `*` steps into a map's values and `[]` into a list's elements. The check applies to the
    /// construct the final type names, or else to the construct that owns the last field. A
    /// subject that resolves nowhere (`**`, prose) applies to no single construct.
    pub(crate) fn check_owners(&self, subject: &str) -> Vec<&'a str> {
        let path = subject.split(' ').next().unwrap_or_default();
        let segments: Vec<&str> = path
            .strip_prefix("**.")
            .unwrap_or(path)
            .split('.')
            .collect();
        let Some((first, rest)) = segments.split_first() else {
            return Vec::new();
        };
        let (first_name, first_list) = split_list(first);
        let mut states: Vec<(&'a str, &'a Value)> = Vec::new();
        for construct in &self.constructs {
            if let Some(field) = construct.fields.iter().find(|f| f.name == first_name) {
                let ty = if first_list {
                    list_element(field.ty)
                } else {
                    Some(field.ty)
                };
                states.extend(ty.map(|ty| (construct.name, ty)));
            }
        }
        for segment in rest {
            let mut next = Vec::new();
            for (owner, ty) in states {
                next.extend(self.step(owner, ty, segment));
            }
            states = next;
        }
        let mut owners: Vec<&'a str> = Vec::new();
        for (owner, ty) in states {
            let named = match unwrap(ty) {
                Value::String(name) => self.construct(name).map(|c| c.name),
                _ => None,
            };
            let owner = named.unwrap_or(owner);
            if !owners.contains(&owner) {
                owners.push(owner);
            }
        }
        owners
    }

    fn step(&self, owner: &'a str, ty: &'a Value, segment: &str) -> Vec<(&'a str, &'a Value)> {
        let (name, list) = split_list(segment);
        let found: Vec<(&'a str, &'a Value)> = if name == "*" {
            map_value(ty)
                .map(|value| (owner, value))
                .into_iter()
                .collect()
        } else {
            self.field_of(owner, ty, name)
        };
        if list {
            found
                .into_iter()
                .filter_map(|(owner, ty)| list_element(ty).map(|ty| (owner, ty)))
                .collect()
        } else {
            found
        }
    }

    fn field_of(&self, owner: &'a str, ty: &'a Value, name: &str) -> Vec<(&'a str, &'a Value)> {
        match ty {
            Value::String(construct) => self
                .constructs
                .iter()
                .find(|c| c.name == construct)
                .and_then(|c| {
                    c.fields
                        .iter()
                        .find(|f| f.name == name)
                        .map(|f| vec![(c.name, f.ty)])
                })
                .unwrap_or_default(),
            Value::Mapping(map) => {
                if let Some(inner) = map.get("optional") {
                    return self.field_of(owner, inner, name);
                }
                if let Some(record) = map.get("record") {
                    return record
                        .get(name)
                        .map(|inner| vec![(owner, inner)])
                        .unwrap_or_default();
                }
                map.get("one_of")
                    .and_then(Value::as_sequence)
                    .into_iter()
                    .flatten()
                    .flat_map(|alternative| self.field_of(owner, alternative, name))
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}

/// Every key without a value, except `const`.
///
/// In a flow mapping an unquoted comma ends the value: `{note: a, b}` is `{note: a, b: null}`,
/// and the reference would print half a sentence. `{const: null}` is the one place the schema
/// writes a null on purpose.
fn split_text(value: &Value, at: &str, problems: &mut Vec<String>) {
    match value {
        Value::Mapping(map) => {
            for (key, inner) in map {
                let key = match key {
                    Value::String(key) => key.clone(),
                    other => crate::yaml::flow(other),
                };
                let path = if at.is_empty() {
                    key.clone()
                } else {
                    format!("{at}.{key}")
                };
                if inner.is_null() && key != "const" {
                    problems.push(format!(
                        "{at}: `{key}` has no value; an unquoted comma in a flow mapping split the \
                         text before it (quote that text)"
                    ));
                }
                split_text(inner, &path, problems);
            }
        }
        Value::Sequence(items) => {
            for (i, item) in items.iter().enumerate() {
                split_text(item, &format!("{at}[{i}]"), problems);
            }
        }
        _ => {}
    }
}

fn split_list(segment: &str) -> (&str, bool) {
    match segment.strip_suffix("[]") {
        Some(name) => (name, true),
        None => (segment, false),
    }
}

/// `ty` without any `optional` around it.
fn unwrap(ty: &Value) -> &Value {
    match ty.get("optional") {
        Some(inner) => unwrap(inner),
        None => ty,
    }
}

fn list_element(ty: &Value) -> Option<&Value> {
    unwrap(ty).get("list")
}

fn map_value(ty: &Value) -> Option<&Value> {
    unwrap(ty).get("map").and_then(|map| map.get("value"))
}

fn read_construct<'a>(
    name: &'a str,
    construct: &'a Value,
    groups: &[Group<'a>],
    problems: &mut Vec<String>,
) -> Construct<'a> {
    let text = |key: &str, problems: &mut Vec<String>| {
        text_of(&construct[key]).unwrap_or_else(|| {
            problems.push(format!("{name} has no `{key}`"));
            ""
        })
    };
    let summary = text("summary", problems);
    let doc = text("doc", problems);
    let group = text_of(&construct["group"]).unwrap_or_default();
    if !groups.iter().any(|g| g.name == group) {
        problems.push(format!("{name} names no declared group (`{group}`)"));
    }
    let order = construct["order"].as_u64().unwrap_or_else(|| {
        problems.push(format!("{name} has no `order`"));
        0
    });
    let example = construct
        .get("example")
        .filter(|example| !is_empty(example))
        .unwrap_or_else(|| {
            problems.push(format!("{name} has no `example`"));
            &Value::Null
        });
    let mut fields = Vec::new();
    match construct["fields"].as_mapping() {
        Some(declared) => {
            for (field, spec) in declared {
                let field = field.as_str().unwrap_or("?");
                let Some(ty) = spec.get("type") else {
                    problems.push(format!("{name}.{field} has no `type`"));
                    continue;
                };
                let note = text_of(&spec["note"]).unwrap_or_else(|| {
                    problems.push(format!("{name}.{field} has no `note`"));
                    ""
                });
                fields.push(Field {
                    name: field,
                    ty,
                    required: spec["required"].as_bool().unwrap_or(false),
                    default: spec.get("default"),
                    note,
                    extras: extras(spec, FIELD_KEYS),
                });
            }
        }
        None => problems.push(format!("{name} has no `fields`")),
    }
    let shorthands = construct["shorthand"]
        .as_sequence()
        .map(|entries| entries.iter().collect())
        .unwrap_or_default();
    Construct {
        name,
        group,
        order,
        summary,
        doc,
        fields,
        shorthands,
        example,
        extras: extras(construct, CONSTRUCT_KEYS),
    }
}

fn extras<'a>(value: &'a Value, known: &[&str]) -> Vec<(&'a str, &'a Value)> {
    value
        .as_mapping()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| key.as_str().map(|key| (key, value)))
        .filter(|(key, _)| !known.contains(key))
        .collect()
}

/// A required text: a string with something in it. A blank one documents nothing, so it is
/// refused exactly as a missing one is.
fn text_of(value: &Value) -> Option<&str> {
    value.as_str().filter(|text| !text.trim().is_empty())
}

/// The chapters, in order; a group lacking any of its texts is refused.
fn read_groups<'a>(root: &'a Value, problems: &mut Vec<String>) -> Vec<Group<'a>> {
    let mut groups = Vec::new();
    for group in root["groups"].as_sequence().into_iter().flatten() {
        let text = |key: &str| text_of(&group[key]);
        match (
            text("name"),
            group["order"].as_u64(),
            text("title"),
            text("summary"),
        ) {
            (Some(name), Some(order), Some(title), Some(summary)) => groups.push(Group {
                name,
                order,
                title,
                summary,
            }),
            _ => problems.push(format!(
                "the group {} lacks a name, order, title or summary",
                crate::yaml::flow(group)
            )),
        }
    }
    if groups.is_empty() {
        problems.push("the schema declares no `groups`".to_owned());
    }
    groups.sort_by_key(|group| group.order);
    groups
}

/// The top-level keys every schema has; a file without them is some other document.
const SHAPE: &[&str] = &["format", "type_rule", "groups", "constructs"];

/// Every text the page prints outside constructs, required wherever its section is present, so
/// the page never shows `null` for text the schema did not write.
fn required_texts(root: &Value, problems: &mut Vec<String>) {
    fn need(value: &Value, at: &str, problems: &mut Vec<String>) {
        if text_of(value).is_none() {
            problems.push(format!("{at} has no text"));
        }
    }
    need(&root["type_rule"]["summary"], "type_rule.summary", problems);
    need(&root["type_rule"]["doc"], "type_rule.doc", problems);
    let expressions = &root["expressions"];
    if !expressions.is_null() {
        need(&expressions["summary"], "expressions.summary", problems);
        need(&expressions["doc"], "expressions.doc", problems);
        for (i, form) in items(&expressions["forms"]) {
            need(
                &form["form"],
                &format!("expressions.forms[{i}].form"),
                problems,
            );
            need(
                &form["note"],
                &format!("expressions.forms[{i}].note"),
                problems,
            );
        }
    }
    if !root["shorthands"].is_null() {
        need(
            &root["shorthands"]["summary"],
            "shorthands.summary",
            problems,
        );
    }
    for (i, layer) in items(&root["layers"]) {
        need(&layer["name"], &format!("layers[{i}].name"), problems);
        need(&layer["note"], &format!("layers[{i}].note"), problems);
    }
    if !root["unmapped_marker"].is_null() {
        need(
            &root["unmapped_marker"]["summary"],
            "unmapped_marker.summary",
            problems,
        );
    }
    for (i, entry) in items(&root["lowering"]) {
        need(
            &entry["construct"],
            &format!("lowering[{i}].construct"),
            problems,
        );
        need(
            &entry["lowers_to"],
            &format!("lowering[{i}].lowers_to"),
            problems,
        );
    }
    for (i, check) in items(&root["checks"]["list"]) {
        need(&check["id"], &format!("checks.list[{i}].id"), problems);
        need(
            &check["severity"],
            &format!("checks.list[{i}].severity"),
            problems,
        );
        if is_empty(&check["subject"]) {
            problems.push(format!("checks.list[{i}].subject has no text"));
        }
    }
}

fn items(value: &Value) -> impl Iterator<Item = (usize, &Value)> {
    value.as_sequence().into_iter().flatten().enumerate()
}

/// Whether a required value shows nothing: null, a blank string, or an empty map or list.
fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Mapping(map) => map.is_empty(),
        Value::Sequence(items) => items.is_empty(),
        _ => false,
    }
}
