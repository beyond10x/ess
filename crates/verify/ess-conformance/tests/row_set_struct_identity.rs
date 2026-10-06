//! A row-set selector over the members of a struct identity (ess/23, beyond10x/ess#463).
//!
//! The shelf: `demo.vault.Shelf` is identified by `at: demo.vault.Place {region, shelf}`, and
//! `Store {place}` is refused as `sealed` where a shelf whose identity is the place sent is sealed:
//! `where: {all: [at.region == input.place.region, at.shelf == input.place.shelf, mode == sealed]}`.
//! From `ess/23` an equality between a `String` member of the identity and the input scopes the
//! selector, the row's literal identity is read member by member, its decoys are arranged under
//! other identities, and no arrangement creates an identity a row already carries. `ess/22` keeps
//! its suite and refusals byte for byte, and `at == input.place` stays a type mismatch.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::resolve::diagnose;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::{ScenarioStep, ScenarioValue};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;
use ess_primitives::node::Node;

const SHELVES: &str = include_str!("fixtures/row-set-struct-identity.yaml");

const SEALED: &str = "demo.vault.Store/outcome/sealed";
const STORED: &str = "demo.vault.Store/outcome/stored";

const REGION: &str = "at.region == input.place.region";
const SHELF: &str = "at.shelf == input.place.shelf";
const SELECTOR: &str =
    "{all: [at.region == input.place.region, at.shelf == input.place.shelf, mode == sealed]}";

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` is in the model");
    text.replace(from, to)
}

/// The first place `from` appears: the creating command's input, ahead of the view's fields.
fn replaced_once(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` is in the model");
    text.replacen(from, to, 1)
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("shelves.yaml"), raw)])
}

fn ir_of(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesized(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

/// The suite of `text`, which must synthesize without a refusal, and which the interpreter — the
/// honest target — passes whole.
fn passing_suite(text: &str) -> ConformanceSuite {
    let synthesis = synthesized(text);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new(), "no refusal");
    let verdicts =
        support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(ir_of(text)));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
    synthesis.suite
}

/// The scenarios of `suite` a target running `mutant` does not pass.
fn failed_by(suite: &ConformanceSuite, mutant: &str) -> Vec<String> {
    let verdicts = support_go::rust_outcomes(suite, &Interpreted::for_model(ir_of(mutant)));
    support_go::not_passed(&verdicts)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn steps_of(suite: &ConformanceSuite, id: &str) -> Vec<ScenarioStep> {
    suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {id}: {:#?}",
                suite.scenarios.keys().collect::<Vec<_>>()
            )
        })
        .1
        .steps
        .clone()
}

/// Every send of `command` in `steps`, by its input.
fn sends(steps: &[ScenarioStep], command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

fn literal<'a>(input: &'a BTreeMap<String, ScenarioValue>, field: &str) -> &'a Node {
    input
        .get(field)
        .and_then(ScenarioValue::as_literal)
        .unwrap_or_else(|| panic!("`{field}` is sent as a literal: {input:#?}"))
}

fn member<'a>(value: &'a Node, name: &str) -> &'a Node {
    match value {
        Node::Map(members) => &members[name],
        other => panic!("a struct: {other:?}"),
    }
}

/// The shelves a scenario opens before it sends `Store`, as `(at, mode)`, and the place it sends.
fn opened_and_sent(steps: &[ScenarioStep]) -> (Vec<(Node, Node)>, Node) {
    let opened = sends(steps, "demo.vault.OpenShelf")
        .iter()
        .map(|input| (literal(input, "at").clone(), literal(input, "mode").clone()))
        .collect();
    let stores = sends(steps, "demo.vault.Store");
    assert_eq!(stores.len(), 1, "{steps:#?}");
    (opened, literal(&stores[0], "place").clone())
}

/// No two creations of one scenario name one identity.
fn no_identity_created_twice(suite: &ConformanceSuite) {
    for (id, scenario) in &suite.scenarios {
        let created: Vec<Node> = sends(&scenario.steps, "demo.vault.OpenShelf")
            .iter()
            .filter_map(|input| input.get("at").and_then(ScenarioValue::as_literal).cloned())
            .collect();
        for (at, identity) in created.iter().enumerate() {
            assert!(
                !created[..at].contains(identity),
                "{id} creates {identity:?} twice: {created:#?}"
            );
        }
    }
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

// ---- the selector over the members --------------------------------------------------------------

#[test]
fn struct_identity_member_selector_witnesses_both_branches() {
    let suite = passing_suite(SHELVES);
    for id in [SEALED, STORED] {
        steps_of(&suite, id);
    }
}

#[test]
fn struct_identity_decoys_carry_other_identities() {
    let suite = passing_suite(SHELVES);
    no_identity_created_twice(&suite);
    for id in [SEALED, STORED] {
        let (opened, place) = opened_and_sent(&steps_of(&suite, id));
        let (region, shelf) = (member(&place, "region"), member(&place, "shelf"));
        // A decoy refuting the region alone, and one refuting the shelf alone: each under an
        // identity other than the one sent. Where a shelf is selected, each again after it.
        let region_decoys: Vec<usize> = opened
            .iter()
            .enumerate()
            .filter(|(_, (at, mode))| {
                member(at, "region") != region
                    && member(at, "shelf") == shelf
                    && *mode == text("sealed")
            })
            .map(|(at, _)| at)
            .collect();
        let shelf_decoys: Vec<usize> = opened
            .iter()
            .enumerate()
            .filter(|(_, (at, mode))| {
                member(at, "region") == region
                    && member(at, "shelf") != shelf
                    && *mode == text("sealed")
            })
            .map(|(at, _)| at)
            .collect();
        let each = if id == SEALED { 2 } else { 1 };
        assert_eq!(region_decoys.len(), each, "{id}: {opened:#?} for {place:?}");
        assert_eq!(shelf_decoys.len(), each, "{id}: {opened:#?} for {place:?}");
        let selected: Vec<usize> = opened
            .iter()
            .enumerate()
            .filter(|(_, (at, mode))| *at == place && *mode == text("sealed"))
            .map(|(at, _)| at)
            .collect();
        assert_eq!(
            selected.len(),
            usize::from(id == SEALED),
            "{id}: {opened:#?} for {place:?}"
        );
        for chosen in &selected {
            for decoys in [&region_decoys, &shelf_decoys] {
                assert!(
                    decoys.first() < Some(chosen) && decoys.last() > Some(chosen),
                    "{id}: the selected shelf lies between its decoys: {opened:#?}"
                );
            }
        }
    }
}

#[test]
fn pinned_identity_nonkey_decoy_on_empty_branch() {
    let suite = passing_suite(SHELVES);
    let (opened, place) = opened_and_sent(&steps_of(&suite, STORED));
    assert!(
        opened
            .iter()
            .any(|(at, mode)| *at == place && *mode != text("sealed")),
        "`stored` arranges the sent shelf, not sealed: {opened:#?} for {place:?}"
    );
    let ignores_mode = replaced(
        SHELVES,
        SELECTOR,
        "{all: [at.region == input.place.region, at.shelf == input.place.shelf]}",
    );
    let failed = failed_by(&suite, &ignores_mode);
    assert!(
        failed.iter().any(|id| id == STORED),
        "a target ignoring `mode == sealed` fails `stored`: {failed:#?}"
    );
}

#[test]
fn ignored_identity_member_mutant_fails() {
    let suite = passing_suite(SHELVES);
    for (dropped, kept) in [(REGION, SHELF), (SHELF, REGION)] {
        let mutant = replaced(
            SHELVES,
            SELECTOR,
            &format!("{{all: [{kept}, mode == sealed]}}"),
        );
        let failed = failed_by(&suite, &mutant);
        assert!(
            !failed.is_empty(),
            "a target dropping `{dropped}` fails a scenario"
        );
    }
}

#[test]
fn partial_member_selector_selects_many() {
    let crowded = replaced(
        &replaced(
            SHELVES,
            SELECTOR,
            "{all: [at.region == input.place.region, mode == sealed]}",
        ),
        "          exists: true\n",
        "          count: {gte: 2}\n",
    );
    let suite = passing_suite(&crowded);
    no_identity_created_twice(&suite);
    let (opened, place) = opened_and_sent(&steps_of(&suite, SEALED));
    let selected: Vec<&Node> = opened
        .iter()
        .filter(|(at, mode)| {
            member(at, "region") == member(&place, "region") && *mode == text("sealed")
        })
        .map(|(at, _)| at)
        .collect();
    assert_eq!(selected.len(), 2, "{opened:#?} for {place:?}");
    assert_ne!(selected[0], selected[1], "two distinct identities");
}

/// Two records under one fully pinned key are no arrangement at all: the branch needing them is
/// refused, never witnessed by creating the one identity twice, which a target refuses whatever the
/// creator declares.
#[test]
fn pinned_identity_is_never_created_twice() {
    let twice = replaced(
        SHELVES,
        "          exists: true\n",
        "          count: {gte: 2}\n",
    );
    let synthesis = synthesized(&twice);
    no_identity_created_twice(&synthesis.suite);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.contains("demo.vault.Store/sealed")),
        "{refusals:#?}"
    );
    steps_of(&synthesis.suite, STORED);
}

// ---- a scalar identity beside another conjunct --------------------------------------------------

fn scalar(format: &str) -> String {
    let mut model = replaced(SHELVES, "format: ess/23", format);
    model = replaced(
        &model,
        "types:\n  - name: demo.vault.Place\n    kind: struct\n    fields:\n      - {name: region, type: String}\n      - {name: shelf, type: String}\n",
        "",
    );
    model = replaced(&model, "type: demo.vault.Place}", "type: String}");
    replaced(
        &model,
        SELECTOR,
        "{all: [at == input.place, mode == sealed]}",
    )
}

/// Both branches are witnessed, and on decoys that decide: a row under the identity sent that is
/// not sealed on the empty branch, so a target ignoring either conjunct fails a scenario. At the
/// unit's base the branches synthesized on rows refuting neither conjunct alone (`at-1`/`mode-1`,
/// `at-2`/`mode-2`): a stored identity the predicate does not read left `mode == sealed` unknown.
#[test]
fn scalar_identity_with_extra_conjunct_synthesizes() {
    let model = scalar("format: ess/23");
    let suite = passing_suite(&model);
    no_identity_created_twice(&suite);
    steps_of(&suite, SEALED);
    let (opened, place) = opened_and_sent(&steps_of(&suite, STORED));
    assert!(
        opened
            .iter()
            .any(|(at, mode)| *at == place && *mode != text("sealed")),
        "`stored` arranges the sent shelf, not sealed: {opened:#?} for {place:?}"
    );
    for mutant in ["{all: [at == input.place]}", "{all: [mode == sealed]}"] {
        let failed = failed_by(
            &suite,
            &replaced(&model, "{all: [at == input.place, mode == sealed]}", mutant),
        );
        assert!(
            !failed.is_empty(),
            "a target selecting `{mutant}` fails a scenario"
        );
    }
}

// ---- an identity held only as an observation ----------------------------------------------------

#[test]
fn instance_struct_identity_member_refused_by_name() {
    let observed = replaced(
        &replaced_once(
            SHELVES,
            "      - {name: at, type: demo.vault.Place}\n      - {name: mode, type: String}\n",
            "      - {name: region, type: String}\n      - {name: shelf, type: String}\n      - {name: mode, type: String}\n",
        ),
        "demo.vault.ShelfOpened: {at: input.at}",
        "demo.vault.ShelfOpened: {at: {region: input.region, shelf: input.shelf}}",
    );
    let synthesis = synthesized(&observed);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    for branch in ["demo.vault.Store/sealed", "demo.vault.Store/stored"] {
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.contains(branch) && refusal.contains("member projection")),
            "{branch} is refused naming the missing member projection: {refusals:#?}"
        );
    }
}

// ---- the older format, and the whole-struct equality ---------------------------------------------

/// The canonical suite and refusals of `text`, as `adversary_w3_266_267_dump.rs` writes them.
fn dumped(text: &str) -> String {
    use std::fmt::Write as _;
    let synthesis = synthesized(text);
    let mut out = synthesis.suite.to_canonical_json().unwrap();
    out.push_str("\n---- refusals\n");
    for refusal in &synthesis.refusals {
        writeln!(out, "{refusal}").unwrap();
    }
    out
}

fn sha256(text: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        })
}

/// The digests were read at the unit's base `09ec1dc33`, before any production change; the
/// refusal lines are ess 0.53.0's own output for the same two models.
#[test]
fn ess22_struct_identity_selector_keeps_bytes() {
    let members = replaced(SHELVES, "format: ess/23", "format: ess/22");
    let out = dumped(&members);
    for branch in ["sealed", "stored"] {
        let line = format!(
            "no witness: `demo.vault.Store/{branch}` is `row set`, which selects rows by no \
             equality between a String or Uuid field and the input or the addressed subject, so \
             the rows it counts are not only the scenario's own"
        );
        assert!(out.contains(&line), "{line}\n{out}");
    }
    assert_eq!(
        sha256(&out),
        "0d1e58cd635a14790c94fb1d738055c052d0b07b031915e64f6ec20a3f85fcd1",
        "the ess/22 suite moved:\n{out}"
    );
    let scalar = dumped(&scalar("format: ess/22"));
    assert!(
        scalar.contains(
            "no witness: `demo.vault.Store/sealed: its own row set is False` is `row set`, which \
             has no arrangement of rows the declared commands produce on which the rows its \
             selector selects take this branch"
        ),
        "{scalar}"
    );
    assert_eq!(
        sha256(&scalar),
        "73e7ff8e63ade3db600d7c419dd1d13ce14ab87711b12cb2209bf94e2f3a4f5a",
        "the ess/22 suite moved:\n{scalar}"
    );
}

#[test]
fn whole_struct_equality_stays_refused() {
    let whole = replaced(
        SHELVES,
        SELECTOR,
        "{all: [at == input.place, mode == sealed]}",
    );
    let errors = assemble(&whole).expect_err("`at == input.place` is refused");
    let codes: Vec<String> = diagnose(&errors, &SourceMap::new())
        .as_slice()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect();
    assert!(
        codes.iter().any(|code| code == "ESS-COMMAND-002"),
        "{codes:?}: {errors}"
    );
}

// ---- an upsert whose creation an input guard selects (coordinator note, #462 adversary pass) -----

/// `Shelve` creates the book where `mode == new` and updates it otherwise, and is refused while
/// the library it names is closed: a row-set guard over another entity.
const UPSERT: &str = r"format: ess/23
system: demo
version: v1
domain: demo.lib
summary: Books shelved in a library, added or updated by one command.
entities:
  - name: demo.lib.Library
    identity: {name: library, type: String}
    fields:
      - {name: open, type: String}
    lifecycle: {initial: Running, states: [Running], terminal: [Running]}
  - name: demo.lib.Book
    identity: {name: book, type: String}
    fields:
      - {name: note, type: String}
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved]}
commands:
  - name: demo.lib.OpenLibrary
    input:
      - {name: library, type: String}
      - {name: open, type: String}
    outcomes:
      - name: opened
        creates: demo.lib.Library
        instance: library
        sets: {open: input.open}
        emits: [demo.lib.LibraryOpened]
        payload:
          demo.lib.LibraryOpened: {library: input.library}
  - name: demo.lib.Shelve
    input:
      - {name: book, type: String}
      - {name: mode, type: String}
      - {name: library, type: String}
      - {name: note, type: String}
    outcomes:
      - name: closed
        when_related:
          entity: demo.lib.Library
          where: {all: [library == input.library, open == closed]}
          exists: true
        error: demo.lib.Closed
      - name: added
        when: mode == new
        creates: demo.lib.Book
        instance: book
        sets: {note: input.note}
        emits: [demo.lib.BookAdded]
        payload:
          demo.lib.BookAdded: {book: input.book}
      - name: updated
        updates: demo.lib.Book
        instance: book
        sets: {note: input.note}
        emits: [demo.lib.BookUpdated]
        payload:
          demo.lib.BookUpdated: {book: input.book}
      - {name: missing, unknown_instance: true, error: demo.lib.NoSuchBook}
errors:
  - {name: demo.lib.Closed, summary: The library is closed.}
  - {name: demo.lib.NoSuchBook, summary: No book carries the identity.}
events:
  - name: demo.lib.LibraryOpened
    fields: [{name: library, type: String}]
  - name: demo.lib.BookAdded
    fields: [{name: book, type: String}]
  - name: demo.lib.BookUpdated
    fields: [{name: book, type: String}]
";

/// The creation's scenario never arranges the identity it expects to create: exactly one send
/// expects `added`, and no earlier step shelved the book it names. At the unit's base the scenario
/// shelved the book first, through the sibling `updated`'s addressed subject, then expected `added`
/// for it again. Whether the interpreter passes the suite needs its input-guarded creation in an
/// upsert (beyond10x/ess#462), so it is not asserted here.
#[test]
fn input_guarded_creation_never_arranges_its_own_identity() {
    let synthesis = synthesized(UPSERT);
    let (_, scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.lib.Shelve/outcome/added")
        .unwrap_or_else(|| panic!("`added` is witnessed: {:#?}", synthesis.refusals));
    let mut shelved: Vec<ScenarioValue> = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        let added = matches!(
            scenario.steps.get(at + 1),
            Some(ScenarioStep::ExpectOutcome { outcome }) if outcome.outcome.as_str() == "added"
        );
        if command.to_string() == "demo.lib.Shelve" && added {
            let book = input["book"].clone();
            assert!(
                !shelved.contains(&book),
                "`added` expects to create {book:?}, which an earlier step shelved: {:#?}",
                scenario.steps
            );
            shelved.push(book);
        }
    }
    assert_eq!(shelved.len(), 1, "{:#?}", scenario.steps);
}
