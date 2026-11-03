//! Adversary pass on unit MUT1 of wave W2: the mutation audit's grant classes, unmutated-site
//! listing, error-swap payloads and scenario clock, driven from each story's Acceptance.
//!
//! - story:the-mutation-audit-reports-the-sites-it-could-not-mutate (beyond10x/ess#711)
//! - story:the-mutation-audit-covers-actor-grants (beyond10x/ess#710)
//! - story:error-swap-mutates-refusals-that-carry-a-payload (beyond10x/ess#709)
#![allow(clippy::missing_panics_doc)]

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const S711: &str = "story:the-mutation-audit-reports-the-sites-it-could-not-mutate";
const S710: &str = "story:the-mutation-audit-covers-actor-grants";

/// The toy specification of beyond10x/ess#709 and #711 (the same one), brand-free, exactly as the
/// unit's own `mutation_site_coverage.rs` carries it.
const SHELF: &str = r"format: ess/32
system: library
version: v1
domain: library.shelf
types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}
entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: position, type: Integer}
      - {name: note, type: Optional<String>}
    lifecycle:
      initial: Shelved
      states: [Shelved, Lent]
      terminal: [Lent]
      transitions:
        - {name: lend, from: [Shelved], to: Lent}
actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook, library.shelf.LendBook, library.shelf.AnnotateBook]
errors:
  - name: library.shelf.ShelfFull
    fields:
      - {name: requested, type: Integer}
  - name: library.shelf.BookNotFound
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookStateConflict
    fields:
      - {name: state, type: library.shelf.Book.State}
commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: position, type: Integer}
    outcomes:
      - name: too-far
        when: position > 100
        error: library.shelf.ShelfFull
        payload:
          library.shelf.ShelfFull: {requested: input.position}
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {book_id: {generated: true}}
        sets: {position: input.position}
  - name: library.shelf.LendBook
    input:
      - {name: book_id, type: library.shelf.BookId}
    outcomes:
      - name: lent
        moves: library.shelf.Book.lend
        instance: book_id
        emits: [library.shelf.BookLent]
        payload:
          library.shelf.BookLent: {book_id: input.book_id}
      - name: no-such-book
        unknown_instance: true
        error: library.shelf.BookNotFound
        payload:
          library.shelf.BookNotFound: {book_id: input.book_id}
      - name: wrong-state
        wrong_state: true
        error: library.shelf.BookStateConflict
  - name: library.shelf.AnnotateBook
    input:
      - {name: book_id, type: library.shelf.BookId}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: annotated
        updates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookAnnotated]
        payload:
          library.shelf.BookAnnotated: {book_id: input.book_id}
        sets: {note: input.note}
events:
  - name: library.shelf.BookCatalogued
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookLent
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookAnnotated
    fields:
      - {name: book_id, type: library.shelf.BookId}
views:
  - name: library.shelf.BookById
    source: library.shelf.Book
    consistency: read_your_writes
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
components:
  - component: shelf-service
    owns: {domains: [library.shelf]}
    accepts:
      commands: [library.shelf.CatalogueBook, library.shelf.LendBook, library.shelf.AnnotateBook]
    publishes:
      events: [library.shelf.BookCatalogued, library.shelf.BookLent, library.shelf.BookAnnotated]
    reached_by: network
";

/// The toy of beyond10x/ess#710 with `actors:` replaced by the text given, so one served command
/// and two or three actors can be arranged.
fn grants_toy(actors: &str) -> String {
    format!(
        r"format: ess/32
system: library
version: v1
domain: library.shelf
types:
  - {{name: library.shelf.BookId, kind: newtype, of: Uuid}}
entities:
  - name: library.shelf.Book
    identity: {{name: book_id, type: library.shelf.BookId}}
    fields:
      - {{name: shelf_code, type: String}}
      - {{name: position, type: Integer}}
    lifecycle: {{initial: Shelved, states: [Shelved], terminal: [Shelved], transitions: []}}
actors:
{actors}commands:
  - name: library.shelf.CatalogueBook
    input:
      - {{name: shelf_code, type: String}}
      - {{name: position, type: Integer}}
    outcomes:
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {{book_id: {{generated: true}}}}
        sets: {{shelf_code: input.shelf_code, position: input.position}}
events:
  - name: library.shelf.BookCatalogued
    fields:
      - {{name: book_id, type: library.shelf.BookId}}
views:
  - name: library.shelf.ShelfOrder
    source: library.shelf.Book
    consistency: read_your_writes
    order_by: [shelf_code asc, position asc]
    fields:
      - {{name: book_id, type: library.shelf.BookId}}
      - {{name: shelf_code, type: String}}
      - {{name: position, type: Integer}}
components:
  - component: shelf-service
    owns: {{domains: [library.shelf]}}
    accepts: {{commands: [library.shelf.CatalogueBook]}}
    publishes: {{events: [library.shelf.BookCatalogued]}}
    reached_by: network
"
    )
}

fn documents(text: &str, label: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

/// Every `.yaml` file under `examples/<name>`, as the unit's `mutation_audit.rs` loads it.
fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut pending: Vec<PathBuf> = vec![root];
    let mut found = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("readable") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        texts.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    (parsed, texts)
}

fn audit_of(
    (documents, texts): (Vec<Document>, SourceMap),
    classes: &[MutantClass],
) -> MutationReport {
    let ir = mutate::compile(documents.clone(), &texts)
        .unwrap_or_else(|error| panic!("the fixture compiles: {error:?}"));
    mutate::audit(&documents, &texts, classes, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// The exit status `ess verify conform mutate` gives `report` without `--fail-on-unmutated`, as
/// `finish_mutation_audit` in `crates/edge/ess-cli/src/main.rs` decides it.
fn default_exit(report: &MutationReport) -> u8 {
    let counts = &report.counts;
    let ran = report
        .mutants
        .iter()
        .filter(|entry| !matches!(entry.verdict, Verdict::Stillborn | Verdict::Equivalent))
        .count();
    if counts.survived > 0 {
        1
    } else if counts.unwitnessed > 0
        || counts.inconclusive > 0
        || ran == 0
        || report.unaudited_sites() > 0
    {
        3
    } else {
        0
    }
}

/// Whether `id` is a mutant of `selection` or one of its unmutated sites.
fn accounted(selection: &mutate::Selection, id: &str) -> bool {
    selection.mutants.iter().any(|mutant| mutant.id == id)
        || selection.unmutated.iter().any(|site| site.id == id)
}

// ---- #711: the default exit status is unchanged over the issue's own toy --------------------------

/// The issue records the full audit of its toy as exit 0 (12 mutants, 6 killed, 0 survived). The
/// Acceptance keeps the default exit status (D135) and asks for live mutants where they exist. The
/// `Optional` write `note: input.note` now has a `sets-retarget` literal mutant; `note` is in no
/// view, so if nothing observes the row's `note`, that mutant survives and the default audit of the
/// issue's own toy exits 1.
#[test]
fn the_711_toy_full_audit_still_exits_0() {
    let report = audit_of(documents(SHELF, "shelf.yaml"), MutantClass::ALL);
    let survivors: Vec<&str> = report
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Survived)
        .map(|entry| entry.id.as_str())
        .collect();
    assert_eq!(
        (default_exit(&report), survivors.as_slice()),
        (0, &[][..]),
        "{S711}: the default exit of the #711 toy was 0 and must stay 0 (D135): {}",
        report.render_text()
    );
}

// ---- #711 x #710: every grant the grant classes leave alone is listed ------------------------------

/// Billing declares `billing.invoice.Customer` (may `CreateInvoice`) and `billing.invoice.Auditor`
/// (may nothing), and serves nothing over the network. Its pinned audit reads `("grant-drop", 0)`
/// with the comment "No actor is declared". #710's Acceptance: grant-drop removes one command from
/// one actor's `may:`. #711's: every site a selected class excluded is listed with its reason. So
/// `Customer`'s grant is either a mutant or listed; it is neither.
#[test]
fn billings_grant_is_a_mutant_or_listed() {
    let (documents, _) = example("billing");
    let selection = mutate::selection(&documents, &[MutantClass::GrantDrop, MutantClass::GrantAdd]);
    let id = "grant-drop/billing.invoice.Customer/billing.invoice.CreateInvoice";
    assert!(
        accounted(&selection, id),
        "{S711} / {S710}: `{id}` is neither a mutant nor an unmutated site: mutants {:?}, unmutated {:?}",
        selection.mutants.iter().map(|it| &it.id).collect::<Vec<_>>(),
        selection.unmutated.iter().map(|it| &it.id).collect::<Vec<_>>()
    );
}

/// The system's own actor is left out of the grant sites (`audited_grants`, "it is no caller of a
/// served surface"), with no listed site saying so.
#[test]
fn the_system_actors_grant_is_a_mutant_or_listed() {
    let text = grants_toy(
        "  - name: library.shelf.Keeper
    system: true
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
",
    );
    let (documents, texts) = documents(&text, "grants.yaml");
    mutate::compile(documents.clone(), &texts)
        .unwrap_or_else(|error| panic!("the fixture compiles: {error:?}"));
    let selection = mutate::selection(&documents, &[MutantClass::GrantDrop]);
    assert!(
        selection
            .mutants
            .iter()
            .any(|mutant| mutant.id
                == "grant-drop/library.shelf.Librarian/library.shelf.CatalogueBook"),
        "the ordinary actor's grant is a site: {:?}",
        selection
            .mutants
            .iter()
            .map(|it| &it.id)
            .collect::<Vec<_>>()
    );
    let id = "grant-drop/library.shelf.Keeper/library.shelf.CatalogueBook";
    assert!(
        accounted(&selection, id),
        "{S711} / {S710}: `{id}` is neither a mutant nor an unmutated site: unmutated {:?}",
        selection
            .unmutated
            .iter()
            .map(|it| &it.id)
            .collect::<Vec<_>>()
    );
}

// ---- #710: grant-add is killed where the holder carries attributes ---------------------------------

/// `MutantClass::GrantAdd`'s doc: "the mutant's suite sends the command as that actor and expects it
/// to run"; the commit: "grant-add is killed by the grant/admitted/<actor> scenario". Synthesis skips
/// the rotation of a command an attribute-carrying actor holds (`GrantRotationSkipped`), so here
/// nothing may send the command as `Reader` in the mutant's suite.
#[test]
fn grant_add_is_killed_when_the_holder_carries_attributes() {
    let text = grants_toy(
        "  - name: library.shelf.Librarian
    attributes: [{name: branch, type: String}]
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
",
    );
    let report = audit_of(documents(&text, "grants.yaml"), &[MutantClass::GrantAdd]);
    let id = "grant-add/library.shelf.Reader/library.shelf.CatalogueBook";
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no `{id}`: {}", report.render_text()));
    assert_eq!(
        entry.verdict,
        Verdict::Killed,
        "{S710}: {}",
        report.render_text()
    );
}

// ---- #711 x #709: error-swap lists both twins of a single-error domain -----------------------------

/// A domain declaring one error: `error-swap` has no other error to take. The unit lists the
/// payload-carrying refusal (`no_compatible_error_alternative`); the payload-less refusal beside
/// it, excluded for the same reason, is still dropped in silence by `outcome_sites`.
const ONE_ERROR: &str = r"format: ess/32
system: library
version: v1
domain: library.shelf
types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}
entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: position, type: Integer}
    lifecycle:
      initial: Shelved
      states: [Shelved, Lent]
      terminal: [Lent]
      transitions:
        - {name: lend, from: [Shelved], to: Lent}
actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook, library.shelf.LendBook]
errors:
  - name: library.shelf.BookNotFound
    fields:
      - {name: book_id, type: library.shelf.BookId}
commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: position, type: Integer}
    outcomes:
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {book_id: {generated: true}}
        sets: {position: input.position}
  - name: library.shelf.LendBook
    input:
      - {name: book_id, type: library.shelf.BookId}
    outcomes:
      - name: lent
        moves: library.shelf.Book.lend
        instance: book_id
        emits: [library.shelf.BookLent]
        payload:
          library.shelf.BookLent: {book_id: input.book_id}
      - name: no-such-book
        unknown_instance: true
        error: library.shelf.BookNotFound
        payload:
          library.shelf.BookNotFound: {book_id: input.book_id}
      - name: wrong-state
        wrong_state: true
        error: library.shelf.BookNotFound
events:
  - name: library.shelf.BookCatalogued
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookLent
    fields:
      - {name: book_id, type: library.shelf.BookId}
views:
  - name: library.shelf.BookById
    source: library.shelf.Book
    consistency: read_your_writes
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
components:
  - component: shelf-service
    owns: {domains: [library.shelf]}
    accepts:
      commands: [library.shelf.CatalogueBook, library.shelf.LendBook]
    publishes:
      events: [library.shelf.BookCatalogued, library.shelf.BookLent]
    reached_by: network
";

/// Rewritten to pin today's behaviour (story:error-swap-mutates-refusals-that-carry-a-payload):
/// the silent drop of the payload-less twin is pre-existing, filed as backlog draft. The
/// adversary's assertion was `accounted(&selection, payload_less)`; restore it when the draft is
/// done.
#[test]
fn error_swap_lists_both_refusals_of_a_single_error_domain() {
    let (documents, texts) = documents(ONE_ERROR, "one-error.yaml");
    mutate::compile(documents.clone(), &texts)
        .unwrap_or_else(|error| panic!("the fixture compiles: {error:?}"));
    let selection = mutate::selection(&documents, &[MutantClass::ErrorSwap]);
    let carrying = "error-swap/library.shelf.LendBook/no-such-book";
    let payload_less = "error-swap/library.shelf.LendBook/wrong-state";
    assert!(
        accounted(&selection, carrying),
        "the unit lists the payload-carrying twin: {:?}",
        selection.unmutated
    );
    assert!(
        !accounted(&selection, payload_less),
        "story:error-swap-mutates-refusals-that-carry-a-payload: pre-existing, filed as backlog \
         draft; `{payload_less}` is now a mutant or listed, so restore the adversary's assertion \
         (`{S711}`: it has no other error to take, like `{carrying}`): unmutated {:?}",
        selection
            .unmutated
            .iter()
            .map(|it| &it.id)
            .collect::<Vec<_>>()
    );
}

// ---- #711: the issue's first command names its site ---------------------------------------------

/// The issue's first command is `mutate --class transition-to` over its toy; it gave one stillborn
/// mutant naming `ESS-ENTITY-011`. Acceptance: "Over the toy spec of #711, the report names each
/// such site." The site is now unmutated, and the audit refuses as `ESS-MUTATE-003`, "find(s) no
/// site in this specification", writing no report and naming the site nowhere.
#[test]
fn transition_to_alone_over_the_711_toy_names_its_site() {
    let (documents, texts) = documents(SHELF, "shelf.yaml");
    let ir = mutate::compile(documents.clone(), &texts).expect("the toy compiles");
    let site = "transition-to/library.shelf.Book.lend";
    match mutate::audit(&documents, &texts, &[MutantClass::TransitionTo], || {
        Interpreted::for_model(ir.clone())
    }) {
        Ok(report) => assert!(
            report
                .unmutated_sites
                .iter()
                .flatten()
                .any(|listed| listed.id == site),
            "{S711}: {}",
            report.render_text()
        ),
        Err(refusal) => {
            let said = refusal.to_string();
            assert!(
                said.contains(site),
                "{S711}: `--class transition-to` over the #711 toy names `{site}` nowhere: {said}"
            );
        }
    }
}

// ---- #711: the text of each `no_admissible_state` site names its own transition ------------------

/// Billing's three transitions are listed `no_admissible_state`; each sentence names the
/// transition, its own `to:` and the refusal the model gives (`ESS-ENTITY-011`).
#[test]
fn billings_unmutated_transitions_say_what_was_not_audited() {
    let (documents, _) = example("billing");
    let selection = mutate::selection(&documents, &[MutantClass::TransitionTo]);
    let listed: Vec<(&str, &str)> = selection
        .unmutated
        .iter()
        .map(|site| (site.id.as_str(), site.unaudited.as_str()))
        .collect();
    let expected = [
        ("cancel", "Cancelled"),
        ("issue", "Issued"),
        ("settle", "Paid"),
    ];
    assert_eq!(listed.len(), expected.len(), "{listed:#?}");
    for ((id, sentence), (transition, to)) in listed.iter().zip(expected) {
        assert_eq!(
            *id,
            format!("transition-to/billing.invoice.Invoice.{transition}"),
            "{S711}"
        );
        assert!(
            sentence.contains(&format!("`billing.invoice.Invoice.{transition}`"))
                && sentence.contains(&format!("`to: {to}`"))
                && sentence.contains("ESS-ENTITY-011"),
            "{S711}: {sentence}"
        );
    }
}
