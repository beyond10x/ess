//! Adversary pass 1 on story:an-each-write-may-take-repeated-identities-in-list-order (ess/35,
//! beyond10x/ess#748, wave-2026-11-06-ess U3).
//!
//! The format's own record (`FORMAT_HISTORY` ess/35, the design note, the guide and the CHANGELOG)
//! says of `repeated: last_wins`: "its elements apply one after another in list order, each to the
//! row as the elements before it left it, so the last element naming a row decides what it holds".
//! The interpreter walks the `affects:` entries in the order written and, inside each entry, the
//! elements its `where:` selects. Where one entity is written by more than one `each:` entry over
//! one list (the ess/27 shape: one entry per verdict), the two orders differ for a repeated
//! identity, and the interpreter applies a later element before an earlier one.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Step, Store,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture holds:\n{from}");
    text.replacen(from, to, 1)
}

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

// ---- three moving entries over one list (the ess/27 `each-moves.yaml` fixture) -------------------

mod moves {
    use super::*;

    const EACH: &str = include_str!("fixtures/each-moves.yaml");
    const DUPLICATED: &str = "      - name: duplicated\n        when: {not: {distinct: {in: answers, as: a, by: a.proposal_id}}}\n        error: perrow.batch.Duplicate\n";

    /// The fixture at ess/35, its `distinct:` refusal dropped and `repeated: last_wins` on each of
    /// its three entries.
    fn repeated() -> String {
        let mut text = edit(EACH, "format: ess/27", "format: ess/35");
        text = edit(&text, DUPLICATED, "");
        for verdict in ["applied", "refused", "invalid"] {
            text = edit(
                &text,
                &format!("where: a.verdict == {verdict}}}"),
                &format!("where: a.verdict == {verdict}, repeated: last_wins}}"),
            );
        }
        text
    }

    fn id(n: u8) -> Node {
        text(&format!("00000000-0000-4000-8000-0000000000{n:02}"))
    }

    fn created(
        ir: &EssIr,
        store: &mut Store,
        (command, event, field): (&str, &str, &str),
        at: Node,
        input: BTreeMap<String, Node>,
    ) {
        let mut steps = execute_generating(
            ir,
            store,
            &format!("perrow.batch.{command}").parse().unwrap(),
            &input,
            &Externals::Withheld,
            &Generated::Given(BTreeMap::from([(
                GeneratedSlot::new(format!("perrow.batch.{event}").parse().unwrap(), field),
                at,
            )])),
        )
        .unwrap_or_else(|error| panic!("{command}: {error:?}"));
        *store = steps.remove(0).next;
    }

    fn answer(n: u8, verdict: &str) -> Node {
        Node::Map(BTreeMap::from([
            ("proposal_id".to_owned(), id(n)),
            ("verdict".to_owned(), text(verdict)),
        ]))
    }

    fn state(store: &Store, n: u8) -> Option<String> {
        store
            .instances()
            .find(|(entity, identity, _)| {
                entity.to_string() == "perrow.batch.Proposal" && **identity == id(n)
            })
            .map(|(_, _, row)| row.state.to_string())
    }

    /// A list naming proposal 11 twice, `refused` first and `applied` second. In list order the
    /// first element refuses the row, and the second, its row now `Refused` and outside `apply`'s
    /// `from`, is skipped: the row ends `Refused`. Walking the entries first, `apply` (written
    /// first) takes the second element before `refuse` sees the first, and the row ends `Applied`.
    #[test]
    fn a_repeated_identity_across_two_moving_entries_applies_in_list_order() {
        let ir = model(&repeated());
        let mut store = Store::default();
        created(
            &ir,
            &mut store,
            ("Open", "BatchOpened", "batch_id"),
            id(1),
            BTreeMap::from([("label".to_owned(), text("b"))]),
        );
        created(
            &ir,
            &mut store,
            ("Propose", "Proposed", "proposal_id"),
            id(11),
            BTreeMap::from([("batch_id".to_owned(), id(1))]),
        );
        let mut steps = execute(
            &ir,
            &store,
            &"perrow.batch.Apply".parse().unwrap(),
            &BTreeMap::from([
                ("batch_id".to_owned(), id(1)),
                (
                    "answers".to_owned(),
                    Node::Seq(vec![answer(11, "refused"), answer(11, "applied")]),
                ),
            ]),
            &Externals::Withheld,
        )
        .unwrap_or_else(|error| panic!("{error:?}"));
        let step: Step = steps.remove(0);
        assert_eq!(
            step.outcome.as_ref().map(ToString::to_string).as_deref(),
            Some("perrow.batch.Apply/applied")
        );
        assert_eq!(
            state(&step.next, 11).as_deref(),
            Some("Refused"),
            "defect: the interpreter applies a repeated identity in entry order, not in list order \
             as ess/35 `repeated: last_wins` documents (FORMAT_HISTORY, design note, guide): the \
             second element (`applied`) moved the row before the first (`refused`) was read"
        );
    }
}

// ---- two updating entries over one list ----------------------------------------------------------

mod upserts {
    use super::*;

    const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
    const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";
    const ENTRY: &str = "          - entity: demo.feed.SeenDocument\n            each: {in: input.applied, as: doc}\n            instance: doc.document_id\n            sets: {source_id: input.source_id, content_hash: doc.content_hash, revision: doc.revision}\n";

    /// `set-each.yaml` at ess/35 with a `kind` member on the element and its one entry split in
    /// two, one per kind, both `repeated: last_wins` and writing the same fields.
    fn repeated() -> String {
        let mut text = edit(EACH, "format: ess/23", "format: ess/35");
        text = edit(&text, DISTINCT, "");
        text = edit(
            &text,
            "  - name: demo.feed.AppliedDocument\n",
            "  - {name: demo.feed.Kind, kind: enum, variants: [early, late]}\n  - name: demo.feed.AppliedDocument\n",
        );
        text = edit(
            &text,
            "      - {name: revision, type: Integer}\nentities:",
            "      - {name: revision, type: Integer}\n      - {name: kind, type: demo.feed.Kind}\nentities:",
        );
        let entry = |kind: &str| {
            ENTRY.replace(
                "each: {in: input.applied, as: doc}",
                &format!("each: {{in: input.applied, as: doc, where: doc.kind == {kind}, repeated: last_wins}}"),
            )
        };
        edit(
            &text,
            ENTRY,
            &format!("{}{}", entry("early"), entry("late")),
        )
    }

    fn element(id: &str, kind: &str, hash: &str, revision: i64) -> Node {
        Node::Map(BTreeMap::from([
            ("document_id".to_owned(), text(id)),
            ("kind".to_owned(), text(kind)),
            ("content_hash".to_owned(), text(hash)),
            ("revision".to_owned(), Node::Number(Number::from(revision))),
        ]))
    }

    /// A list naming document d1 twice, a `late` element first and an `early` one last. The format
    /// says the last element naming a row decides what it holds: `last`, revision 2. Walking the
    /// entries first, the `early` entry (written first) writes the last element, and the `late`
    /// entry then writes the first element over it.
    #[test]
    fn the_last_element_naming_a_row_decides_it_across_two_entries() {
        let ir = model(&repeated());
        let mut steps = execute_generating(
            &ir,
            &Store::default(),
            &"demo.feed.AddSource".parse().unwrap(),
            &BTreeMap::from([("label".to_owned(), text("feed"))]),
            &Externals::Withheld,
            &Generated::Given(BTreeMap::from([(
                GeneratedSlot::new("demo.feed.SourceAdded".parse().unwrap(), "source_id"),
                text("s1"),
            )])),
        )
        .unwrap_or_else(|error| panic!("{error:?}"));
        let store = steps.remove(0).next;
        let mut steps = execute(
            &ir,
            &store,
            &"demo.feed.RunSource".parse().unwrap(),
            &BTreeMap::from([
                ("source_id".to_owned(), text("s1")),
                ("label".to_owned(), text("ran")),
                (
                    "applied".to_owned(),
                    Node::Seq(vec![
                        element("d1", "late", "first", 1),
                        element("d1", "early", "last", 2),
                    ]),
                ),
            ]),
            &Externals::Withheld,
        )
        .unwrap_or_else(|error| panic!("{error:?}"));
        let step = steps.remove(0);
        assert_eq!(
            step.outcome.as_ref().map(ToString::to_string).as_deref(),
            Some("demo.feed.RunSource/ran")
        );
        let seen = "demo.feed.SeenDocument".parse().unwrap();
        let d1 = step.next.instance(&seen, "d1").expect("d1 is held");
        assert_eq!(
            (
                d1.fields["content_hash"].clone(),
                d1.fields["revision"].clone()
            ),
            (text("last"), Node::Number(Number::from(2_i64))),
            "defect: under `repeated: last_wins` the first element naming the row decided it, not \
             the last: the interpreter applies entry by entry, so a later-written entry's earlier \
             element overwrites an earlier-written entry's later element: {d1:?}"
        );
    }
}
