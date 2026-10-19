//! A quantifier over the rows of an entity (beyond10x/ess#551, `ess/27`):
//! `exists: {in: {entity: E}, as: b, that: P}`.
//!
//! `in:` takes a fact path, as it always did, or a mapping with exactly the key `entity`. Over a
//! rows collection the body is evaluated once per row the fact source hands out for that entity:
//! `b.<rest>` reads the row and every other path the enclosing source. A source that knows no rows
//! answers `None`, and the quantifier is then `Unknown`, never the vacuous answer of an empty
//! collection. `docs/design/related-rows-quantifier.md` is the design.

use std::collections::BTreeMap;

use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Collection, Predicate, Truth};

fn node(json: &str) -> Node {
    serde_json::from_str(json).expect("the test node is JSON")
}

fn read(json: &str) -> Result<Predicate, String> {
    Predicate::from_node(&node(json)).map_err(|error| error.to_string())
}

fn parsed(json: &str) -> Predicate {
    read(json).expect("the predicate parses")
}

/// An enclosing source with rows per entity.
#[derive(Default)]
struct Rows {
    enclosing: FactStore,
    rows: BTreeMap<String, Vec<FactStore>>,
}

impl Rows {
    fn row(mut self, entity: &str, fields: &[(&str, FactValue)]) -> Self {
        let mut row = FactStore::new();
        for (field, value) in fields {
            row.set_path(field, value.clone());
        }
        self.rows.entry(entity.to_owned()).or_default().push(row);
        self
    }

    fn none_of(mut self, entity: &str) -> Self {
        self.rows.entry(entity.to_owned()).or_default();
        self
    }
}

impl FactSource for Rows {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.enclosing.fact(path)
    }

    fn rows(&self, entity: &str) -> Option<Vec<Box<dyn FactSource + '_>>> {
        self.rows.get(entity).map(|rows| {
            rows.iter()
                .map(|row| Box::new(row.clone()) as Box<dyn FactSource>)
                .collect()
        })
    }
}

const MEMBERSHIP: &str = "demo.slots.Membership";

fn text(value: &str) -> FactValue {
    FactValue::Text(value.to_owned())
}

/// `exists` over the memberships, tied to the candidate's `slot_id`, on the `Left` side.
const LEFT: &str = r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
    "that": {"all": [{"member.slot_id": {"eq": {"fact": "slot_id"}}}, "member.side == Left"]}}}"#;

#[test]
fn in_entity_parses_and_prints_its_canonical_form() {
    let predicate = parsed(LEFT);
    let Predicate::Exists(quantified) = &predicate else {
        panic!("an exists: {predicate:?}");
    };
    assert_eq!(
        quantified.over,
        Collection::Rows {
            entity: MEMBERSHIP.to_owned()
        }
    );
    assert_eq!(quantified.bind, "member");
    assert_eq!(
        predicate.to_node(),
        node(
            r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
            "that": {"all": [{"member.slot_id": {"eq": {"fact": "slot_id"}}}, "member.side == Left"]}}}"#
        )
    );
    assert_eq!(
        parsed(&serde_json::to_string(&predicate).unwrap()),
        predicate
    );
    assert_eq!(
        predicate.to_string(),
        "exists member in {entity: demo.slots.Membership}: \
         ((member.slot_id == {fact: slot_id} and member.side == Left))"
    );

    // A fact path keeps its collection, its bytes and its text.
    let path = parsed(r#"{"forall": {"in": "slots", "as": "slot", "that": "slot.open == true"}}"#);
    let Predicate::Forall(quantified) = &path else {
        panic!("a forall: {path:?}");
    };
    assert_eq!(quantified.over, Collection::Fact("slots".parse().unwrap()));
    assert_eq!(
        path.to_node(),
        node(r#"{"forall": {"in": "slots", "as": "slot", "that": "slot.open == true"}}"#)
    );
    assert_eq!(
        path.to_string(),
        "forall slot in slots: (slot.open == true)"
    );
}

#[test]
fn in_takes_only_a_path_or_one_entity_key() {
    for collection in [
        r#"{"entity": "demo.slots.Membership", "where": "side == Left"}"#,
        r#"{"table": "demo.slots.Membership"}"#,
        r"{}",
        r#"{"entity": 3}"#,
        r#"{"entity": {"name": "demo.slots.Membership"}}"#,
        r#"{"entity": ""}"#,
        r#"["slots"]"#,
        r"7",
    ] {
        let json = format!(
            r#"{{"exists": {{"in": {collection}, "as": "member", "that": "member.side == Left"}}}}"#
        );
        let refused = read(&json).expect_err(&format!("`in: {collection}` is refused"));
        assert!(
            refused.contains("a fact path or `{entity: <Entity>}`"),
            "`in: {collection}` names both shapes: {refused}"
        );
    }
}

#[test]
fn a_rows_test_without_a_rows_source_is_unknown() {
    let mut store = FactStore::new();
    store.set_path("slot_id", text("s1"));
    assert_eq!(parsed(LEFT).evaluate(&store), Truth::Unknown);
    // Even a body no row could satisfy: nobody looked.
    let never = parsed(
        r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
        "that": false}}"#,
    );
    assert_eq!(never.evaluate(&store), Truth::Unknown);
    // And a source with rows of another entity only.
    let other = Rows::default().row("demo.slots.Other", &[("slot_id", text("s1"))]);
    assert_eq!(parsed(LEFT).evaluate(&other), Truth::Unknown);
}

#[test]
fn exists_over_rows_is_the_kleene_disjunction() {
    let with = |rows: Rows| {
        let mut rows = rows;
        rows.enclosing.set_path("slot_id", text("s1"));
        parsed(LEFT).evaluate(&rows)
    };
    let left = [("slot_id", text("s1")), ("side", text("Left"))];
    let right = [("slot_id", text("s1")), ("side", text("Right"))];
    let elsewhere = [("slot_id", text("s2")), ("side", text("Left"))];
    let undecided = [("slot_id", text("s1"))];

    // A `True` row.
    assert_eq!(
        with(
            Rows::default()
                .row(MEMBERSHIP, &right)
                .row(MEMBERSHIP, &left)
        ),
        Truth::True
    );
    // Every row `False`.
    assert_eq!(
        with(
            Rows::default()
                .row(MEMBERSHIP, &right)
                .row(MEMBERSHIP, &elsewhere)
        ),
        Truth::False
    );
    // No row.
    assert_eq!(with(Rows::default().none_of(MEMBERSHIP)), Truth::False);
    // One `Unknown` row beside `False` ones stays a possible witness.
    assert_eq!(
        with(
            Rows::default()
                .row(MEMBERSHIP, &right)
                .row(MEMBERSHIP, &undecided)
        ),
        Truth::Unknown
    );
    // And a `True` row decides over it.
    assert_eq!(
        with(
            Rows::default()
                .row(MEMBERSHIP, &undecided)
                .row(MEMBERSHIP, &left)
        ),
        Truth::True
    );
    // Negated, the three values swap as Kleene's do.
    let not = |rows: Rows| {
        let mut rows = rows;
        rows.enclosing.set_path("slot_id", text("s1"));
        Predicate::Not(Box::new(parsed(LEFT))).evaluate(&rows)
    };
    assert_eq!(not(Rows::default().none_of(MEMBERSHIP)), Truth::True);
    assert_eq!(not(Rows::default().row(MEMBERSHIP, &left)), Truth::False);
}

#[test]
fn the_binder_reads_the_row_and_bare_paths_the_enclosing_source() {
    // The row and the enclosing source both hold `slot_id` and `side`; the binder reads the row's,
    // a bare path the enclosing one's.
    let mut rows = Rows::default()
        .row(
            MEMBERSHIP,
            &[
                ("slot_id", text("s2")),
                ("side", text("Left")),
                ("tags.count", FactValue::count(2)),
                ("tags.0", text("vip")),
                ("tags.1", text("night")),
            ],
        )
        .row(
            MEMBERSHIP,
            &[("slot_id", text("s1")), ("side", text("Right"))],
        );
    rows.enclosing.set_path("slot_id", text("s1"));
    rows.enclosing.set_path("side", text("Left"));
    rows.enclosing.set_path("member_side", text("Right"));

    // Tied to the enclosing `s1`: only the `Right` row is related.
    assert_eq!(parsed(LEFT).evaluate(&rows), Truth::False);
    let right = parsed(
        r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
        "that": {"all": [{"member.slot_id": {"eq": {"fact": "slot_id"}}}, {"member.side": {"eq": {"fact": "member_side"}}}]}}}"#,
    );
    assert_eq!(right.evaluate(&rows), Truth::True);
    // A bare `side` is the enclosing source's, not the row's.
    let bare = parsed(
        r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
        "that": {"all": ["member.slot_id == s2", "side == Left"]}}}"#,
    );
    assert_eq!(bare.evaluate(&rows), Truth::True);
    // An ordinary quantifier over a list of the row reads the row's list.
    let tagged = parsed(
        r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
        "that": {"exists": {"in": "member.tags", "as": "tag", "that": "tag == night"}}}}"#,
    );
    assert_eq!(tagged.evaluate(&rows), Truth::True);
    // The binder alone is no value of the row.
    let whole = parsed(
        r#"{"exists": {"in": {"entity": "demo.slots.Membership"}, "as": "member",
        "that": "member == s1"}}"#,
    );
    assert_eq!(whole.evaluate(&rows), Truth::Unknown);
}
