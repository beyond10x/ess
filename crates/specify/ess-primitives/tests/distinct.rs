//! Family F part C (beyond10x/ess#237): distinct list members, `distinct: {in, as, by}`.
//!
//! A list holds no two elements whose key is equal — the element itself, or one scalar member of
//! it named by `by`. Equality is the declared key kind's: exact numbers, instants for a
//! `Timestamp`, exact text otherwise. An empty or one-element list holds; two or more hold when
//! every key is known and pairwise unequal, are `False` as soon as two known keys are equal, and are
//! otherwise `Unknown`. An absent list is `Unknown`, not empty. The canonical form names the key
//! kind; the source form leaves it for the domain's resolver. The shared vectors
//! `tests/vectors/distinct.json` are answered here, by the Go reader and by the TypeScript one.
//! `docs/design/expression-family-source22.md`, section `distinct`, is the design.

use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{
    reading_source22_operands, Distinct, DistinctKeyKind, Predicate, Truth,
};

const VECTORS: &str = include_str!("vectors/distinct.json");

fn path(text: &str) -> FactPath {
    text.parse().expect("a fact path")
}

/// Flattens one JSON row the way every suite lane does: a scalar per dotted path, a list's
/// elements at `<path>.<index>` beside its `<path>.count`, and `null` unbound.
fn bind(at: &FactPath, value: &serde_json::Value, store: &mut FactStore) {
    match value {
        serde_json::Value::Null => {}
        serde_json::Value::Array(items) => {
            store.mark_present(at.clone());
            store.set(at.child("count"), FactValue::count(items.len()));
            for (index, item) in items.iter().enumerate() {
                bind(&at.child(&index.to_string()), item, store);
            }
        }
        serde_json::Value::Object(entries) => {
            store.mark_present(at.clone());
            for (key, entry) in entries {
                bind(&at.child(key), entry, store);
            }
        }
        scalar => {
            let node: Node = serde_json::from_value(scalar.clone()).expect("a scalar node");
            let value = match node {
                Node::Bool(flag) => FactValue::Bool(flag),
                Node::Number(number) => FactValue::Number(number),
                Node::Text(text) => FactValue::Text(text),
                other => panic!("not a scalar: {other}"),
            };
            store.set(at.clone(), value);
        }
    }
}

fn row(value: &serde_json::Value) -> FactStore {
    let mut store = FactStore::new();
    for (field, entry) in value.as_object().expect("a row is an object") {
        bind(&path(field), entry, &mut store);
    }
    store
}

fn vectors() -> serde_json::Value {
    serde_json::from_str(VECTORS).expect("the vectors are JSON")
}

fn read(value: &serde_json::Value) -> Result<Predicate, String> {
    let node: Node = serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    Predicate::from_node(&node).map_err(|error| error.to_string())
}

fn truth(text: &str) -> Truth {
    match text {
        "true" => Truth::True,
        "false" => Truth::False,
        "unknown" => Truth::Unknown,
        other => panic!("no truth {other}"),
    }
}

fn distinct(over: &str, bind: &str, key: Option<&str>, kind: Option<DistinctKeyKind>) -> Predicate {
    Predicate::Distinct(Box::new(Distinct {
        over: path(over),
        bind: bind.to_owned(),
        key: key.map(path),
        key_kind: kind,
    }))
}

#[test]
fn distinct_every_shared_vector_is_answered() {
    let vectors = vectors();
    let mut answered = 0;
    for vector in vectors["evaluate"].as_array().expect("evaluate") {
        let name = vector["name"].as_str().expect("a name");
        let predicate =
            read(&vector["predicate"]).unwrap_or_else(|error| panic!("{name}: {error}"));
        let facts = row(&vector["row"]);
        assert_eq!(
            predicate.evaluate(&facts),
            truth(vector["truth"].as_str().expect("a truth")),
            "{name}: {predicate}"
        );
        assert_eq!(
            Predicate::from_node(&predicate.to_node()).expect("reads back"),
            predicate,
            "{name}: the canonical form round-trips"
        );
        answered += 1;
    }
    for vector in vectors["refused"].as_array().expect("refused") {
        let name = vector["name"].as_str().expect("a name");
        assert!(
            read(&vector["predicate"]).is_err(),
            "{name}: refused, not read"
        );
        answered += 1;
    }
    for vector in vectors["unkinded"].as_array().expect("unkinded") {
        let name = vector["name"].as_str().expect("a name");
        let predicate =
            read(&vector["predicate"]).unwrap_or_else(|error| panic!("{name}: {error}"));
        let Predicate::Distinct(read) = &predicate else {
            panic!("{name}: read as {predicate:?}");
        };
        assert_eq!(read.key_kind, None, "{name}: the source form names no kind");
        assert_eq!(
            predicate.evaluate(&FactStore::new()),
            Truth::Unknown,
            "{name}: an unresolved key is never compared"
        );
        answered += 1;
    }
    assert!(answered >= 50, "only {answered} vectors were answered");
}

#[test]
fn distinct_the_canonical_form_reads_and_writes_back_byte_for_byte() {
    let text = r#"{"distinct":{"as":"file","by":"file.path","in":"files","kind":"string"}}"#;
    let parsed: Predicate = serde_json::from_str(text).expect("reads");
    assert_eq!(
        parsed,
        distinct(
            "files",
            "file",
            Some("file.path"),
            Some(DistinctKeyKind::String)
        )
    );
    assert_eq!(serde_json::to_string(&parsed).expect("writes"), text);
    for kind in DistinctKeyKind::ALL {
        let text = format!(
            r#"{{"distinct":{{"as":"x","in":"xs","kind":"{}"}}}}"#,
            kind.keyword()
        );
        let parsed: Predicate = serde_json::from_str(&text).expect("reads");
        assert_eq!(parsed, distinct("xs", "x", None, Some(kind)));
        assert_eq!(serde_json::to_string(&parsed).expect("writes"), text);
        assert_eq!(DistinctKeyKind::from_keyword(kind.keyword()), Some(kind));
    }
    let source = r#"{"distinct":{"as":"tag","in":"tags"}}"#;
    let parsed: Predicate = serde_json::from_str(source).expect("the source form reads");
    assert_eq!(parsed, distinct("tags", "tag", None, None));
    assert_eq!(
        serde_json::to_string(&parsed).expect("writes"),
        source,
        "an unresolved kind is not invented"
    );
}

#[test]
fn distinct_is_rendered_apart_from_every_other_predicate() {
    let by = distinct(
        "files",
        "file",
        Some("file.path"),
        Some(DistinctKeyKind::Timestamp),
    );
    assert_eq!(
        by.to_string(),
        "distinct file in files by file.path as timestamp"
    );
    assert_eq!(
        distinct("tags", "tag", None, None).to_string(),
        "distinct tag in tags"
    );
    assert!(by.reads_distinct());
    let nested: Predicate = serde_json::from_str(
        r#"{"not": {"forall": {"in": "groups", "as": "g", "that": {"distinct": {"in": "g.members", "as": "m", "kind": "string"}}}}}"#,
    )
    .expect("reads");
    assert!(nested.reads_distinct(), "inside a negated quantifier body");
    assert!(!Predicate::parse_expression("a == b")
        .expect("parses")
        .reads_distinct());
    assert_eq!(
        by.fact_paths()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["files"],
        "the list is read, the key is bound"
    );
    assert_eq!(
        by.quantified_collections()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["files"],
        "the list is walked"
    );
    assert_eq!(
        nested
            .quantified_collections()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["groups"],
        "a list under an outer binder is the outer element's"
    );
    for predicate in [&by, &nested] {
        assert!(!predicate.reads_offset());
        assert!(!predicate.reads_root_fact_operand());
        assert!(!predicate.compares_instants());
        assert!(!predicate.uses_text_match());
        assert!(!predicate.uses_case_fold());
    }
}

#[test]
fn distinct_below_source22_is_refused_naming_the_format() {
    let node: Node =
        serde_json::from_str(r#"{"distinct": {"in": "tags", "as": "tag"}}"#).expect("json");
    let refused = reading_source22_operands(false, || Predicate::from_node(&node))
        .expect_err("refused below ess/22");
    assert!(refused.to_string().contains("ess/22"), "{refused}");
    assert!(reading_source22_operands(true, || Predicate::from_node(&node)).is_ok());
}

#[test]
fn distinct_a_fact_named_distinct_keeps_its_meaning() {
    for (text, expected) in [
        (r#"{"distinct": {"in": ["a", "b"]}}"#, "distinct in [a, b]"),
        (r#"{"distinct": {"eq": 5}}"#, "distinct == 5"),
        (r#"{"distinct": true}"#, "distinct == true"),
        (r#""distinct == 5""#, "distinct == 5"),
    ] {
        let node: Node = serde_json::from_str(text).expect("json");
        for admitted in [false, true] {
            let parsed = reading_source22_operands(admitted, || Predicate::from_node(&node))
                .unwrap_or_else(|error| panic!("{text}: {error}"));
            assert_eq!(parsed.to_string(), expected, "{text}");
            assert!(!parsed.reads_distinct(), "{text}");
        }
    }
}

#[test]
fn distinct_rebinds_through_an_outer_binder() {
    let predicate: Predicate = serde_json::from_str(
        r#"{"forall": {"in": "groups", "as": "group", "that": {"distinct": {"in": "group.tags", "as": "tag", "kind": "string"}}}}"#,
    )
    .expect("reads");
    let healthy = row(&serde_json::json!({"groups": [{"tags": ["a", "b"]}, {"tags": ["a"]}]}));
    assert_eq!(predicate.evaluate(&healthy), Truth::True);
    let broken = row(&serde_json::json!({"groups": [{"tags": ["a", "b"]}, {"tags": ["b", "b"]}]}));
    assert_eq!(predicate.evaluate(&broken), Truth::False);
}

#[test]
fn distinct_a_failing_leaf_reports_the_list_it_read() {
    let predicate = distinct("tags", "tag", None, Some(DistinctKeyKind::String));
    let outcome = predicate.outcome(&row(&serde_json::json!({"tags": ["a", "a"]})));
    assert_eq!(outcome.truth, Truth::False);
    assert_eq!(outcome.causes.len(), 1);
    assert_eq!(
        outcome.causes[0].expression,
        "distinct tag in tags as string"
    );
}

// ---- paired faults ------------------------------------------------------------------------------

/// How one lane might decide a top-level `distinct` row: the keys as the row holds them (`None`
/// where absent), the key kind, and the whole elements for a lane that forgets `by`.
type Judge = fn(&[Option<serde_json::Value>], &[serde_json::Value], &str) -> &'static str;

fn number_of(value: &serde_json::Value) -> Option<ess_primitives::facts::Number> {
    let node: Node = serde_json::from_value(value.clone()).ok()?;
    match node {
        Node::Number(number) => Some(number),
        _ => None,
    }
}

/// The key a value is under `kind`, spelled so equal keys spell alike; `None` outside the kind.
fn keyed(value: &serde_json::Value, kind: &str) -> Option<String> {
    match (kind, value) {
        ("boolean", serde_json::Value::Bool(flag)) => Some(flag.to_string()),
        ("integer", serde_json::Value::Number(_)) => number_of(value)
            .filter(|number| number.is_integral())
            .map(|number| number.to_string()),
        ("decimal", serde_json::Value::Number(_)) => {
            number_of(value).map(|number| number.to_string())
        }
        ("string" | "uuid" | "enum", serde_json::Value::String(text)) => Some(text.clone()),
        ("timestamp", serde_json::Value::String(text)) => {
            ess_primitives::time::Rfc3339Instant::parse_rfc3339(text)
                .map(|instant| format!("{instant:?}"))
        }
        _ => None,
    }
}

/// Every pair, Kleene: a known duplicate is false, an unknown key otherwise unknown.
fn pairwise(keys: &[Option<String>], adjacent_only: bool) -> &'static str {
    if keys.len() < 2 {
        return "true";
    }
    let mut unknown = false;
    for (index, left) in keys.iter().enumerate() {
        for (offset, right) in keys[index + 1..].iter().enumerate() {
            if adjacent_only && offset > 0 {
                break;
            }
            match (left, right) {
                (Some(left), Some(right)) if left == right => return "false",
                (Some(_), Some(_)) => {}
                _ => unknown = true,
            }
        }
    }
    if unknown {
        "unknown"
    } else {
        "true"
    }
}

fn reference(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = keys
        .iter()
        .map(|key| key.as_ref().and_then(|value| keyed(value, kind)))
        .collect();
    pairwise(&keys, false)
}

fn adjacent_only(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = keys
        .iter()
        .map(|key| key.as_ref().and_then(|value| keyed(value, kind)))
        .collect();
    pairwise(&keys, true)
}

fn ignores_by(
    _: &[Option<serde_json::Value>],
    elements: &[serde_json::Value],
    _: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = elements
        .iter()
        .map(|element| (!element.is_null()).then(|| element.to_string()))
        .collect();
    pairwise(&keys, false)
}

fn compares_spellings(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let kind = if kind == "timestamp" { "string" } else { kind };
    reference(keys, &[], kind)
}

fn instants_for_text(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = keys
        .iter()
        .map(|key| {
            let value = key.as_ref()?;
            if kind == "string" {
                if let Some(instant) = keyed(value, "timestamp") {
                    return Some(instant);
                }
            }
            keyed(value, kind)
        })
        .collect();
    pairwise(&keys, false)
}

fn unknown_as_distinct(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let known: Vec<Option<String>> = keys
        .iter()
        .filter_map(|key| key.as_ref().and_then(|value| keyed(value, kind)))
        .map(Some)
        .collect();
    pairwise(&known, false)
}

fn shared_null(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = keys
        .iter()
        .map(|key| match key {
            None => Some("null".to_owned()),
            Some(value) => keyed(value, kind),
        })
        .collect();
    pairwise(&keys, false)
}

fn binary64(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = keys
        .iter()
        .map(|key| match key.as_ref()? {
            serde_json::Value::Number(number) if kind == "integer" || kind == "decimal" => {
                number.as_f64().map(|value| value.to_string())
            }
            value => keyed(value, kind),
        })
        .collect();
    pairwise(&keys, false)
}

fn lexical_decimals(
    keys: &[Option<serde_json::Value>],
    _: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    let keys: Vec<Option<String>> = keys
        .iter()
        .map(|key| match key.as_ref()? {
            serde_json::Value::Number(number) if kind == "decimal" => Some(number.to_string()),
            value => keyed(value, kind),
        })
        .collect();
    pairwise(&keys, false)
}

fn any_two(
    keys: &[Option<serde_json::Value>],
    elements: &[serde_json::Value],
    kind: &str,
) -> &'static str {
    if keys.len() == 2 {
        return "true";
    }
    reference(keys, elements, kind)
}

/// One top-level `distinct` vector: its name, the keys and elements its row holds, its key kind and
/// its truth.
type Decided = (
    String,
    Vec<Option<serde_json::Value>>,
    Vec<serde_json::Value>,
    String,
    String,
);

/// Every top-level `distinct` vector, as the keys and elements its row holds.
fn decided() -> Vec<Decided> {
    let vectors = vectors();
    let mut rows = Vec::new();
    for vector in vectors["evaluate"].as_array().expect("evaluate") {
        let Some(fields) = vector["predicate"]["distinct"].as_object() else {
            continue;
        };
        let over = fields["in"].as_str().expect("a list");
        let Some(elements) = vector["row"]
            .get(over)
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        let bind = fields["as"].as_str().expect("a binder");
        let keys = elements
            .iter()
            .map(|element| {
                let mut at = element;
                if let Some(by) = fields.get("by").and_then(serde_json::Value::as_str) {
                    for segment in by.split('.').skip(1) {
                        at = at.get(segment)?;
                    }
                }
                let _ = bind;
                (!at.is_null()).then(|| at.clone())
            })
            .collect();
        rows.push((
            vector["name"].as_str().expect("a name").to_owned(),
            keys,
            elements.clone(),
            fields["kind"].as_str().expect("a kind").to_owned(),
            vector["truth"].as_str().expect("a truth").to_owned(),
        ));
    }
    rows
}

#[test]
fn distinct_every_paired_fault_disagrees_with_a_vector() {
    let rows = decided();
    assert!(rows.len() >= 30, "{} rows", rows.len());
    for (name, keys, elements, kind, truth) in &rows {
        assert_eq!(
            reference(keys, elements, kind),
            truth,
            "the reference answers {name}"
        );
    }
    for (fault, judge) in [
        ("adjacent pairs only", adjacent_only as Judge),
        ("`by` ignored", ignores_by),
        ("spellings compared", compares_spellings),
        ("instants for text", instants_for_text),
        ("Unknown read as distinct", unknown_as_distinct),
        ("one shared null", shared_null),
        ("binary64 numbers", binary64),
        ("lexical decimals", lexical_decimals),
        ("any two accepted", any_two),
    ] {
        assert!(
            rows.iter()
                .any(|(_, keys, elements, kind, truth)| judge(keys, elements, kind) != truth),
            "no vector tells the {fault} fault apart"
        );
    }
}
