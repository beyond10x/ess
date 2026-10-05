//! Family F part C at the source (beyond10x/ess#237): `distinct: {in, as, by}`.
//!
//! From `ess/22` a predicate may require that no two elements of a `List` share a key: the element
//! itself where it resolves to one admitted scalar, or the one scalar member `by` names under the
//! binder. The admitted key domains are `Boolean`, `Integer`, `Decimal`, `String`, `Uuid`,
//! `Timestamp` and enums, through newtypes and `Optional`. The source leaves the key kind out; the
//! resolver reads it off the declarations and the canonical form carries it, and a kind the source
//! supplies must agree. A struct, list, map, union, `Json`, `Binary64`, `Duration` or `Bytes` key is
//! refused, so is a `Map` or a scalar to walk, and below `ess/22` the construct is refused naming the
//! format. `docs/design/expression-family-source22.md`, section `distinct`, is the design.

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::{Distinct, DistinctKeyKind, Predicate, Quantified};

/// One command whose `refused` branch is guarded by `when`, over lists of every key domain, and an
/// entity whose invariant is `invariant`, read off a view whose filter is `filter`.
fn model(format: u32, when: &str, invariant: &str, filter: &str) -> String {
    format!(
        r"format: ess/{format}
system: pool
version: v1
domain: pool.files
types:
  - {{name: pool.files.Tag, kind: newtype, of: String}}
  - {{name: pool.files.Pin, kind: newtype, of: Integer}}
  - name: pool.files.Color
    kind: enum
    variants: [Red, Green]
  - {{name: pool.files.Shade, kind: newtype, of: pool.files.Color}}
  - name: pool.files.Meta
    kind: struct
    fields:
      - {{name: path, type: String}}
  - name: pool.files.File
    kind: struct
    fields:
      - {{name: path, type: String}}
      - {{name: size, type: Integer}}
      - {{name: label, type: Optional<String>}}
      - {{name: meta, type: pool.files.Meta}}
      - {{name: seen, type: Timestamp}}
  - name: pool.files.Group
    kind: struct
    fields:
      - {{name: members, type: List<pool.files.File>}}
entities:
  - name: pool.files.Bundle
    identity: {{name: bundle_id, type: Uuid}}
    fields:
      - {{name: files, type: List<pool.files.File>}}
      - {{name: tags, type: List<String>}}
    invariants:
      - {invariant}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - name: pool.files.Refused
    summary: The bundle is refused.
events:
  - name: pool.files.Opened
    fields:
      - {{name: bundle_id, type: Uuid}}
commands:
  - name: pool.files.Open
    input:
      - {{name: files, type: List<pool.files.File>}}
      - {{name: tags, type: List<String>}}
      - {{name: named, type: List<pool.files.Tag>}}
      - {{name: ids, type: List<Integer>}}
      - {{name: pins, type: List<pool.files.Pin>}}
      - {{name: amounts, type: List<Decimal>}}
      - {{name: refs, type: List<Uuid>}}
      - {{name: at, type: List<Timestamp>}}
      - {{name: flags, type: List<Boolean>}}
      - {{name: colors, type: List<pool.files.Color>}}
      - {{name: shades, type: List<pool.files.Shade>}}
      - {{name: labels, type: List<Optional<String>>}}
      - {{name: maybe, type: Optional<List<String>>}}
      - {{name: ratios, type: List<Binary64>}}
      - {{name: spans, type: List<Duration>}}
      - {{name: blobs, type: List<Bytes>}}
      - {{name: docs, type: List<Json>}}
      - {{name: nested, type: List<List<String>>}}
      - {{name: table, type: 'Map<String, String>'}}
      - {{name: label, type: String}}
      - {{name: groups, type: List<pool.files.Group>}}
    outcomes:
      - name: refused
        when: {when}
        error: pool.files.Refused
      - name: opened
        creates: pool.files.Bundle
        instance: bundle_id
        emits: [pool.files.Opened]
        payload:
          pool.files.Opened: {{bundle_id: {{generated: true}}}}
        sets: {{files: input.files, tags: input.tags}}
views:
  - name: pool.files.Bundles
    source: pool.files.Bundle
    consistency: read_your_writes
    filter: {filter}
    fields:
      - {{name: bundle_id, type: Uuid}}
      - {{name: files, type: List<pool.files.File>}}
      - {{name: tags, type: List<String>}}
"
    )
}

const GUARD: &str = "label == x";
const PLAIN: &str = "defined(bundle_id)";

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("admitted:\n{errors}\n---\n{text}"))
}

fn refused(text: &str) -> String {
    match assemble(text) {
        Ok(_) => panic!("refused, not admitted:\n{text}"),
        Err(errors) => errors,
    }
}

fn guard(spec: &Specification) -> Predicate {
    let command = &spec.commands()[&"pool.files.Open".parse().unwrap()];
    match &command.outcomes[0].condition {
        OutcomeCondition::When(predicate) => predicate.clone(),
        other => panic!("a plain when, not {other:?}"),
    }
}

fn invariant(spec: &Specification) -> Predicate {
    spec.entities()[&"pool.files.Bundle".parse().unwrap()].invariants[0]
        .predicate
        .clone()
}

fn filter(spec: &Specification) -> Predicate {
    spec.views()[&"pool.files.Bundles".parse().unwrap()]
        .filter
        .clone()
        .expect("a filter")
}

fn path(text: &str) -> FactPath {
    text.parse().expect("a path")
}

fn distinct(over: &str, bind: &str, key: Option<&str>, kind: DistinctKeyKind) -> Predicate {
    Predicate::Distinct(Box::new(Distinct {
        over: path(over),
        bind: bind.to_owned(),
        key: key.map(path),
        key_kind: Some(kind),
    }))
}

#[test]
fn distinct_every_admitted_key_domain_resolves_its_kind() {
    for (list, kind) in [
        ("tags", DistinctKeyKind::String),
        ("named", DistinctKeyKind::String),
        ("ids", DistinctKeyKind::Integer),
        ("pins", DistinctKeyKind::Integer),
        ("amounts", DistinctKeyKind::Decimal),
        ("refs", DistinctKeyKind::Uuid),
        ("at", DistinctKeyKind::Timestamp),
        ("flags", DistinctKeyKind::Boolean),
        ("colors", DistinctKeyKind::Enum),
        ("shades", DistinctKeyKind::Enum),
        ("labels", DistinctKeyKind::String),
        ("maybe", DistinctKeyKind::String),
    ] {
        let when = format!("{{distinct: {{in: {list}, as: item}}}}");
        let spec = admitted(&model(22, &when, PLAIN, PLAIN));
        assert_eq!(guard(&spec), distinct(list, "item", None, kind), "{when}");
    }
}

#[test]
fn distinct_a_struct_list_is_keyed_by_one_member() {
    for (by, kind) in [
        ("file.path", DistinctKeyKind::String),
        ("file.meta.path", DistinctKeyKind::String),
        ("file.size", DistinctKeyKind::Integer),
        ("file.label", DistinctKeyKind::String),
        ("file.seen", DistinctKeyKind::Timestamp),
    ] {
        let when = format!("{{distinct: {{in: files, as: file, by: {by}}}}}");
        let spec = admitted(&model(22, &when, PLAIN, PLAIN));
        assert_eq!(
            guard(&spec),
            distinct("files", "file", Some(by), kind),
            "{when}"
        );
    }
    let spec = admitted(&model(
        22,
        "{distinct: {in: files, as: file, by: file.path}}",
        PLAIN,
        PLAIN,
    ));
    assert_eq!(
        serde_json::to_string(&guard(&spec)).expect("serialises"),
        r#"{"distinct":{"as":"file","by":"file.path","in":"files","kind":"string"}}"#,
        "the canonical form carries the resolved kind"
    );
}

#[test]
fn distinct_reads_an_outer_binder_and_composes_with_other_leaves() {
    let when = "{forall: {in: groups, as: group, that: {distinct: {in: group.members, as: member, by: member.path}}}}";
    let spec = admitted(&model(22, when, PLAIN, PLAIN));
    assert_eq!(
        guard(&spec),
        Predicate::Forall(Box::new(Quantified {
            over: path("groups"),
            bind: "group".to_owned(),
            body: distinct(
                "group.members",
                "member",
                Some("member.path"),
                DistinctKeyKind::String
            ),
        }))
    );
    let when =
        "{all: [{distinct: {in: tags, as: tag}}, {not: {distinct: {in: input.ids, as: id}}}]}";
    let spec = admitted(&model(22, when, PLAIN, PLAIN));
    assert_eq!(
        guard(&spec),
        Predicate::All(vec![
            distinct("tags", "tag", None, DistinctKeyKind::String),
            Predicate::Not(Box::new(distinct(
                "ids",
                "id",
                None,
                DistinctKeyKind::Integer
            ))),
        ]),
        "the input namespace is read as the input in a plain guard"
    );
}

#[test]
fn distinct_an_entity_invariant_and_a_view_filter_persist_the_kind() {
    let spec = admitted(&model(
        22,
        GUARD,
        "{distinct: {in: files, as: file, by: file.path}}",
        "{distinct: {in: tags, as: tag}}",
    ));
    assert_eq!(
        invariant(&spec),
        distinct("files", "file", Some("file.path"), DistinctKeyKind::String)
    );
    assert_eq!(
        filter(&spec),
        distinct("tags", "tag", None, DistinctKeyKind::String)
    );
}

#[test]
fn distinct_a_supplied_kind_must_agree_with_the_declarations() {
    let spec = admitted(&model(
        22,
        "{distinct: {in: at, as: t, kind: timestamp}}",
        PLAIN,
        PLAIN,
    ));
    assert_eq!(
        guard(&spec),
        distinct("at", "t", None, DistinctKeyKind::Timestamp)
    );
    let errors = refused(&model(
        22,
        "{distinct: {in: at, as: t, kind: string}}",
        PLAIN,
        PLAIN,
    ));
    assert!(
        errors.contains("type_mismatch")
            && errors.contains("`string`")
            && errors.contains("timestamp"),
        "{errors}"
    );
}

#[test]
fn distinct_key_domains_without_equality_are_refused_by_name() {
    for (when, wanted) in [
        ("{distinct: {in: files, as: file}}", "by"),
        (
            "{distinct: {in: files, as: file, by: file.meta}}",
            "pool.files.Meta",
        ),
        ("{distinct: {in: ratios, as: r}}", "Binary64"),
        ("{distinct: {in: spans, as: s}}", "Duration"),
        ("{distinct: {in: blobs, as: b}}", "Bytes"),
        ("{distinct: {in: docs, as: d}}", "Json"),
        ("{distinct: {in: nested, as: n}}", "List<String>"),
    ] {
        let errors = refused(&model(22, when, PLAIN, PLAIN));
        assert!(
            errors.contains("type_mismatch")
                && errors.contains("no key equality")
                && errors.contains(wanted),
            "{when}: {errors}"
        );
    }
}

#[test]
fn distinct_walks_a_list_and_nothing_else() {
    for (when, wanted) in [
        ("{distinct: {in: table, as: entry}}", "Map"),
        ("{distinct: {in: label, as: letter}}", "String"),
    ] {
        let errors = refused(&model(22, when, PLAIN, PLAIN));
        assert!(
            errors.contains("type_mismatch")
                && errors.contains("reads a List")
                && errors.contains(wanted),
            "{when}: {errors}"
        );
    }
    for when in [
        "{distinct: {in: missing, as: item}}",
        "{distinct: {in: files, as: file, by: file.missing}}",
    ] {
        let errors = refused(&model(22, when, PLAIN, PLAIN));
        assert!(errors.contains("unobservable_fact"), "{when}: {errors}");
    }
}

#[test]
fn distinct_below_ess22_is_refused_naming_the_format() {
    for format in [16, 21] {
        let errors = refused(&model(
            format,
            "{distinct: {in: tags, as: tag}}",
            PLAIN,
            PLAIN,
        ));
        assert!(errors.contains("ess/22"), "ess/{format}: {errors}");
    }
}

#[test]
fn distinct_a_direct_assembly_without_a_kind_is_refused_and_one_with_it_is_checked() {
    // Deserialised directly, a source keeps no spelling to resolve: the key kind is what it wrote.
    let direct = |when: &str| -> Result<Specification, String> {
        let raw: RawSpecFile = serde_yaml::from_str(&model(22, when, PLAIN, PLAIN))
            .map_err(|error| format!("parse: {error}"))?;
        Specification::assemble([(Source::new("model.yaml"), raw)])
            .map_err(|errors| errors.to_string())
    };
    let errors = direct("{distinct: {in: tags, as: tag}}").expect_err("no kind is refused");
    assert!(errors.contains("key kind"), "{errors}");
    let spec = direct("{distinct: {in: tags, as: tag, kind: string}}").expect("a kind is admitted");
    assert_eq!(
        guard(&spec),
        distinct("tags", "tag", None, DistinctKeyKind::String)
    );
    let errors = direct("{distinct: {in: tags, as: tag, kind: integer}}")
        .expect_err("a wrong kind is refused");
    assert!(errors.contains("type_mismatch"), "{errors}");
}

#[test]
fn distinct_a_field_named_distinct_keeps_its_meaning_in_every_format() {
    for format in [21, 22] {
        let spec = admitted(&model(format, "{label: {eq: distinct}}", PLAIN, PLAIN));
        assert!(!guard(&spec).reads_distinct(), "ess/{format}");
    }
}
