//! Adversary pass 1, unit d (ESS 0.70.0): `interpret::values::mint` and
//! `interpret::values::order_view_rows` held to what their rustdoc promises.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::values::{mint, order_view_rows};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::{
    ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest, ViewRow,
};
use ess_domain::{name::QualifiedName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, facts::Number, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/16
system: probe
version: v1
domain: probe.rank
types:
  - {name: probe.rank.Level, kind: enum, variants: [Medium, High, Low]}
entities:
  - name: probe.rank.Item
    identity: {name: id, type: Uuid}
    fields:
      - {name: score, type: Optional<Integer>}
      - {name: amount, type: Decimal}
      - {name: level, type: probe.rank.Level}
      - {name: seen, type: Timestamp}
    lifecycle: {initial: Held, states: [Held], terminal: [Held], transitions: []}
events:
  - name: probe.rank.Added
    fields: [{name: id, type: Uuid}]
commands:
  - name: probe.rank.Add
    input:
      - {name: score, type: Optional<Integer>}
      - {name: amount, type: Decimal}
      - {name: level, type: probe.rank.Level}
      - {name: seen, type: Timestamp}
    outcomes:
      - name: added
        creates: probe.rank.Item
        instance: id
        sets: {score: input.score, amount: input.amount, level: input.level, seen: input.seen}
        emits: [probe.rank.Added]
        payload: {probe.rank.Added: {id: {generated: true}}}
views:
  - name: probe.rank.ByScore
    source: probe.rank.Item
    consistency: read_your_writes
    order_by: [score desc, id]
    fields: [{name: id, type: Uuid}, {name: score, type: Optional<Integer>}]
  - name: probe.rank.ByLevel
    source: probe.rank.Item
    consistency: read_your_writes
    order_by: [level, amount desc]
    fields:
      - {name: id, type: Uuid}
      - {name: level, type: probe.rank.Level}
      - {name: amount, type: Decimal}
";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("probe.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn uuid(tick: u64) -> Node {
    Node::Text(format!("00000000-0000-4000-8000-{tick:012}"))
}

fn decimal(value: f64) -> Node {
    Node::Number(Number::new(value).unwrap())
}

fn read(target: &Interpreted, view: &str) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: view.parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: CorrelationId::new("probe").unwrap(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}

fn ids(rows: &[ViewRow]) -> Vec<Node> {
    rows.iter().map(|row| row["id"].clone()).collect()
}

/// Absent `Optional` keys (carried as `null` by the interpreter, or omitted by a consumer), the
/// identity tie-break, an enum key and a `Decimal` key: the interpreter's answer is the order the
/// rustdoc of `order_view_rows` describes, and the public function reproduces it from any shuffle.
#[test]
fn adv_d_absent_identity_enum_and_decimal_keys_rank_as_documented() {
    let ir = ir();
    let target = Interpreted::for_model(ir.clone());
    let items: [(Option<i64>, f64, &str); 5] = [
        (Some(5), 10.5, "Low"),
        (None, 9.25, "High"),
        (Some(7), 100.0, "Low"),
        (Some(5), 0.5, "Medium"),
        (None, 10.5, "High"),
    ];
    for (index, (score, amount, level)) in items.iter().enumerate() {
        let mut input = BTreeMap::from([
            ("amount".to_owned(), decimal(*amount)),
            ("level".to_owned(), Node::Text((*level).to_owned())),
            (
                "seen".to_owned(),
                Node::Text(format!("2026-01-0{}T00:00:00Z", index + 1)),
            ),
        ]);
        if let Some(score) = score {
            input.insert("score".to_owned(), Node::Number((*score).into()));
        }
        let result = target
            .execute_command(SemanticCommandRequest {
                command: "probe.rank.Add".parse().unwrap(),
                actor: None,
                caller: None,
                input,
                correlation: CorrelationId::new("probe").unwrap(),
            })
            .unwrap();
        assert_eq!(
            result.direct_events[0].payload["id"],
            uuid(u64::try_from(index + 1).unwrap()),
            "item {index} is minted on tick {}",
            index + 1
        );
    }
    // `score desc`: 7, 5, 5, then the two without a score; ties broken by the identity ascending.
    let by_score = read(&target, "probe.rank.ByScore");
    assert_eq!(
        ids(&by_score),
        [3, 1, 4, 2, 5].map(uuid),
        "ByScore: {by_score:#?}"
    );
    // `level` by its node (text) order, High < Low < Medium, not declaration order; `amount desc`.
    let by_level = read(&target, "probe.rank.ByLevel");
    assert_eq!(
        ids(&by_level),
        [5, 2, 3, 1, 4].map(uuid),
        "ByLevel: {by_level:#?}"
    );
    for (name, answered) in [
        ("probe.rank.ByScore", &by_score),
        ("probe.rank.ByLevel", &by_level),
    ] {
        let view = &ir.views()[&name.parse::<QualifiedName>().unwrap()];
        for shift in 0..answered.len() {
            let mut shuffled = answered.clone();
            shuffled.reverse();
            shuffled.rotate_left(shift);
            order_view_rows(&ir, view, &mut shuffled);
            assert_eq!(&shuffled, answered, "{name} rotated by {shift}");
        }
    }
    // A consumer row that omits the key "ranks below every value": last under `score desc`.
    let view = &ir.views()[&"probe.rank.ByScore".parse::<QualifiedName>().unwrap()];
    let omitted = ViewRow::from([("id".to_owned(), uuid(0))]);
    let mut held = vec![omitted.clone()];
    held.extend(by_score.iter().cloned());
    order_view_rows(&ir, view, &mut held);
    assert_eq!(held.last(), Some(&omitted), "{held:#?}");
}

/// `order_view_rows` promises a stable sort. Up to 20 rows `sort_unstable_by` keeps tied rows in
/// place too, so the six-row tie case cannot tell the two apart; 64 rows in three classes can.
#[test]
fn adv_d_order_view_rows_is_stable_past_the_small_sort_threshold() {
    let ir = ir();
    let view = &ir.views()[&"probe.rank.ByLevel".parse::<QualifiedName>().unwrap()];
    let row = |index: usize| {
        ViewRow::from([
            ("id".to_owned(), Node::Text(format!("row-{index:03}"))),
            ("level".to_owned(), Node::Text("High".to_owned())),
            (
                "amount".to_owned(),
                Node::Number(i64::try_from(index % 3).unwrap().into()),
            ),
        ])
    };
    let mut rows: Vec<ViewRow> = (0..64).map(row).collect();
    // `amount desc`; within a class the caller's order, written out rather than sorted.
    let expected: Vec<ViewRow> = [2, 1, 0]
        .into_iter()
        .flat_map(|class| (0..64).filter(move |index| index % 3 == class))
        .map(row)
        .collect();
    order_view_rows(&ir, view, &mut rows);
    assert_eq!(ids(&rows), ids(&expected));
}

/// The rustdoc: "A `Uuid` ... is `00000000-0000-4000-8000-<tick, zero-padded to 12 digits>`", and
/// `tick` is any `u64`. From tick 10^12 the last group would have 13 or more digits, which is not
/// a UUID: every tick mints a UUID or is refused, and the refusal starts exactly past
/// `MAX_UUID_TICK`.
#[test]
fn adv_d_a_uuid_minted_on_any_tick_is_a_uuid() {
    let ir = ir();
    let identity = &ir
        .entities()
        .values()
        .next()
        .expect("one entity")
        .identity
        .type_ref;
    let mut bad = Vec::new();
    for tick in [1, 999_999_999_999, 1_000_000_000_000, u64::MAX] {
        let value = match mint(&ir, identity, tick) {
            Err(ess_conformance::interpret::execute::Undetermined::NoValue { .. })
                if tick > ess_conformance::interpret::values::MAX_UUID_TICK =>
            {
                continue;
            }
            minted => minted
                .unwrap_or_else(|why| panic!("tick {tick} is refused: {why:?}"))
                .expect("required"),
        };
        let text = value.as_text().expect("text").to_owned();
        let groups: Vec<usize> = text.split('-').map(str::len).collect();
        if groups != [8, 4, 4, 4, 12] || !text.chars().all(|c| c == '-' || c.is_ascii_hexdigit()) {
            bad.push((tick, text));
        }
    }
    assert!(bad.is_empty(), "not a UUID: {bad:?}");
}

/// The rustdoc's repeats, each with the neighbouring tick that does not repeat: a `Boolean`
/// alternates (1 and 3 alike, 1 and 2 not), a `Timestamp` repeats every 28 ticks (1 and 29 alike,
/// 1 and 28 not), and an `Integer` or `Decimal` saturates at `u32::MAX` (it and every later tick
/// alike, the tick before it not).
#[test]
fn adv_d_minted_values_repeat_where_the_rustdoc_says_and_not_before() {
    use ess_compiler::ir::ResolvedTypeRef;
    use ess_domain::types::Primitive;
    let ir = ir();
    let saturated = u64::from(u32::MAX);
    let mut wrong = Vec::new();
    for (primitive, first, same, different) in [
        (Primitive::Boolean, 1_u64, 3_u64, 2_u64),
        (Primitive::Timestamp, 1, 29, 28),
        (Primitive::Integer, saturated, saturated + 1, saturated - 1),
        (Primitive::Integer, saturated, 1 << 40, saturated - 1),
        (Primitive::Decimal, saturated, saturated + 1, saturated - 1),
    ] {
        let ty = ResolvedTypeRef::Primitive { name: primitive };
        let at = |tick| {
            mint(&ir, &ty, tick)
                .unwrap_or_else(|why| panic!("{primitive:?} tick {tick}: {why:?}"))
                .expect("required")
        };
        if at(first) != at(same) {
            wrong.push(format!(
                "{primitive:?}: ticks {first} and {same} differ: {:?}, {:?}",
                at(first),
                at(same)
            ));
        }
        if at(first) == at(different) {
            wrong.push(format!(
                "{primitive:?}: ticks {first} and {different} both mint {:?}",
                at(first)
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
