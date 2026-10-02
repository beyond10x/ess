//! A quantifier binder on the right of a comparison means the binder (beyond10x/ess#289).
//!
//! `forall a in tags: forall b in tags: a != b` validated and compared `a` with the text `"b"`: the
//! right-hand side was read without the binders in scope, so a bare word without a dot was always
//! a literal. A bare word naming a binder in scope now reads the binder, in the compact and the
//! operator spelling; a bare word naming no field, input or binder stays the text the reference
//! documents; and the spellings that can only be a literal — the equality shorthand and a quoted
//! word — are refused when the literal names a binder in scope, as they are for a declared field.

use ess_compiler::{compile, ir::EssIr, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    facts::{FactPath, FactStore, FactValue},
    predicate::{Operand, Predicate, Truth},
};

/// The reproduction from the issue, with the invariant left as a placeholder.
const MODEL: &str = r"
format: ess/18
system: demo
version: v1
domain: demo.bundle
summary: A bundle of files whose paths must be distinct; at most 3 open bundles per owner.
types:
  - {name: demo.bundle.BundleId, kind: newtype, of: Uuid}
  - name: demo.bundle.File
    kind: struct
    fields:
      - {name: path, type: String}
      - {name: size, type: Integer}
entities:
  - name: demo.bundle.Bundle
    identity: {name: bundle_id, type: demo.bundle.BundleId}
    fields:
      - {name: owner, type: String}
      - {name: files, type: List<demo.bundle.File>}
      - {name: tags, type: List<String>}
      - {name: banned, type: List<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
    invariants:
      - INVARIANT
errors:
  - {name: demo.bundle.TooMany, summary: The owner has too many open bundles., fields: []}
events:
  - name: demo.bundle.Created
    fields:
      - {name: bundle_id, type: demo.bundle.BundleId}
actors:
  - name: demo.bundle.Operator
    may: [demo.bundle.CreateBundle]
commands:
  - name: demo.bundle.CreateBundle
    input:
      - {name: owner, type: String}
      - {name: files, type: List<demo.bundle.File>}
      - {name: tags, type: List<String>}
      - {name: banned, type: List<String>}
    outcomes:
      - name: created
        creates: demo.bundle.Bundle
        instance: bundle_id
        emits: [demo.bundle.Created]
        payload:
          demo.bundle.Created: {bundle_id: {generated: true}}
        sets: {owner: input.owner, files: input.files, tags: input.tags, banned: input.banned}
views:
  - name: demo.bundle.Bundles
    source: demo.bundle.Bundle
    consistency: read_your_writes
    fields:
      - {name: bundle_id, type: demo.bundle.BundleId}
      - {name: owner, type: String}
";

/// The issue's own invariant, written exactly as reported.
const DISTINCT: &str = "forall: {in: tags, as: a, that: {forall: {in: tags, as: b, that: a != b}}}";

fn model(invariant: &str) -> String {
    MODEL.replace("INVARIANT", invariant)
}

fn compiled(invariant: &str) -> Result<EssIr, String> {
    let text = model(invariant);
    let raw = RawSpecFile::parse(&text).map_err(|error| error.to_string())?;
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .map_err(|errors| format!("{errors}"))?;
    compile(&spec, &SourceMap::new()).map_err(|error| format!("{error:?}"))
}

fn invariant(invariant: &str) -> Predicate {
    let ir = compiled(invariant).unwrap_or_else(|error| panic!("{error}\n{invariant}"));
    let entity = ir.entities().values().next().expect("one entity");
    entity.invariants[0].predicate.clone()
}

/// The comparison at the bottom of a chain of quantifiers.
fn innermost(predicate: &Predicate) -> &Predicate {
    match predicate {
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            innermost(&quantified.body)
        }
        other => other,
    }
}

fn right(predicate: &Predicate) -> &Operand {
    match innermost(predicate) {
        Predicate::Compare { right, .. } => right,
        other => panic!("expected a comparison, found `{other}`"),
    }
}

fn binder(name: &str) -> Operand {
    Operand::Fact(FactPath::new(name).expect("path"))
}

fn list(store: &mut FactStore, name: &str, items: &[&str]) {
    store.set_path(&format!("{name}.count"), FactValue::count(items.len()));
    for (index, item) in items.iter().enumerate() {
        store.set_path(&format!("{name}.{index}"), FactValue::text(*item));
    }
}

fn holds(predicate: &Predicate, tags: &[&str], banned: &[&str]) -> Truth {
    let mut store = FactStore::new();
    list(&mut store, "tags", tags);
    list(&mut store, "banned", banned);
    predicate.evaluate(&store)
}

fn refused(invariant: &str, needle: &str) {
    let errors = compiled(invariant).expect_err("refused");
    assert!(
        errors.contains(needle),
        "expected a refusal naming {needle:?} for {invariant}, got:\n{errors}"
    );
}

#[test]
fn issue_289_the_outer_binder_on_the_right_is_the_binder_not_the_text() {
    let predicate = invariant(DISTINCT);
    assert_eq!(
        right(&predicate),
        &binder("b"),
        "`a != b` must compare two binders, not `a` with the text \"b\": {predicate:?}"
    );
}

#[test]
fn issue_289_the_operator_spelling_reads_the_binder_too() {
    let predicate = invariant(
        "forall: {in: tags, as: a, that: {forall: {in: tags, as: b, that: {a: {ne: b}}}}}",
    );
    assert_eq!(right(&predicate), &binder("b"), "{predicate:?}");
}

#[test]
fn issue_289_an_outer_binder_read_from_an_inner_body_is_the_binder() {
    let predicate = invariant(
        "forall: {in: tags, as: outer, that: {forall: {in: banned, as: inner, that: inner != outer}}}",
    );
    assert_eq!(right(&predicate), &binder("outer"), "{predicate:?}");
}

#[test]
fn issue_289_evaluation_refuses_a_shared_item_and_accepts_disjoint_lists() {
    let disjoint = invariant(
        "forall: {in: tags, as: tag, that: {forall: {in: banned, as: b, that: tag != b}}}",
    );
    assert_eq!(
        holds(&disjoint, &["x", "y"], &["z"]),
        Truth::True,
        "distinct items are accepted"
    );
    assert_eq!(
        holds(&disjoint, &["x", "b"], &["b"]),
        Truth::False,
        "an item in both lists is refused"
    );
    assert_eq!(
        holds(&disjoint, &["x"], &["x"]),
        Truth::False,
        "an item in both lists is refused even when it is not the binder's own name"
    );
}

#[test]
fn issue_289_evaluation_of_the_reported_invariant_refuses_a_duplicate() {
    let distinct = invariant(DISTINCT);
    assert_eq!(
        holds(&distinct, &["x", "x"], &[]),
        Truth::False,
        "a list with duplicates is refused"
    );
    assert_eq!(
        holds(&distinct, &[], &[]),
        Truth::True,
        "an empty list holds"
    );
}

#[test]
fn issue_289_a_bare_word_naming_nothing_in_scope_stays_text() {
    let predicate = invariant("forall: {in: tags, as: a, that: a != vip}");
    assert_eq!(
        right(&predicate),
        &Operand::Literal(FactValue::text("vip")),
        "{predicate:?}"
    );
    // A binder out of scope is no binder: the sibling quantifier's `b` ended with its body.
    let predicate = invariant(
        "all: [{forall: {in: banned, as: b, that: b != x}}, {forall: {in: tags, as: a, that: a != b}}]",
    );
    let Predicate::All(children) = &predicate else {
        panic!("expected a conjunction, found {predicate:?}");
    };
    assert_eq!(
        right(&children[1]),
        &Operand::Literal(FactValue::text("b")),
        "{predicate:?}"
    );
}

#[test]
fn issue_289_a_literal_naming_a_binder_in_scope_is_refused_in_the_literal_only_spellings() {
    // The equality shorthand is always a literal, and a quoted word is text: either one naming a
    // binder in scope is the same misread, so it is refused as a literal naming a field is.
    refused(
        "forall: {in: tags, as: a, that: {forall: {in: tags, as: b, that: {a: b}}}}",
        "not the binder `b`",
    );
    refused(
        "forall: {in: tags, as: a, that: {forall: {in: tags, as: b, that: 'a != \"b\"'}}}",
        "not the binder `b`",
    );
    // With no binder `b` in scope the shorthand is the text it always was.
    let predicate = invariant("forall: {in: tags, as: a, that: {a: b}}");
    assert_eq!(
        right(&predicate),
        &Operand::Literal(FactValue::text("b")),
        "{predicate:?}"
    );
}
