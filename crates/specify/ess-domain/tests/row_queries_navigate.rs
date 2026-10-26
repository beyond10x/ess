//! What validation decides about reading through a declared reference (`ess/28`,
//! beyond10x/ess#557, beyond10x/ess#564, `docs/design/generic-language-features.md` § G2,
//! navigation): `{related: {via, field: state}}` as a value, a view field
//! `field: {related: {via, field}}`, and a filter path `<via>.<field>`, one hop.

const DESK: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/navigate-a-reference.yaml");

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// `DESK` with `from` replaced by `to`, after checking `from` is there.
fn with(from: &str, to: &str) -> String {
    assert!(DESK.contains(from), "the fixture holds `{from}`");
    DESK.replacen(from, to, 1)
}

/// `DESK` at another source format.
fn at(format: &str) -> String {
    with("format: ess/28", &format!("format: {format}"))
}

const VIEWS: &str = "views:\n";

/// `DESK` with one more view declared first.
fn with_view(view: &str) -> String {
    with(VIEWS, &format!("{VIEWS}{view}"))
}

#[test]
fn the_fixture_validates_at_ess_28() {
    spec(DESK).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn every_navigation_is_refused_below_ess_28_naming_it() {
    let errors = refused(&at("ess/27"));
    assert!(errors.contains("unsupported_format_version"), "{errors}");
    // The payload's state read, the view field and the filter path, each refused by name.
    for named in [
        "field: state}}` reads the lifecycle state",
        "`desk_state` reads the row a reference names",
        "`desk_id.state` reads the row a reference names",
    ] {
        assert!(errors.contains(named), "`{named}` is refused: {errors}");
    }
    assert!(errors.matches("ess/28").count() >= 3, "{errors}");
}

#[test]
fn a_related_state_is_typed_as_its_entitys_state() {
    let errors = refused(&with(
        "      - {name: current, type: relstate.desk.Desk.State}",
        "      - {name: current, type: String}",
    ));
    assert!(
        errors.contains("type_mismatch") && errors.contains("relstate.desk.Desk.State"),
        "{errors}"
    );
}

#[test]
fn a_view_field_read_through_a_reference_is_optional() {
    let errors = refused(&with(
        "        type: Optional<relstate.desk.Desk.State>",
        "        type: relstate.desk.Desk.State",
    ));
    assert!(
        errors.contains("type_mismatch")
            && errors.contains("absent where the reference is or names no row")
            && errors.contains("declare `desk_state` as `Optional<relstate.desk.Desk.State>`"),
        "{errors}"
    );
}

#[test]
fn a_view_field_reads_a_field_of_the_referenced_row() {
    let errors = refused(&with(
        "field: {related: {via: desk_id, field: label}}",
        "field: {related: {via: desk_id, field: colour}}",
    ));
    assert!(
        errors.contains("undeclared_reference")
            && errors.contains("`colour` is not a field of `relstate.desk.Desk`")
            && errors.contains("`desk_id`, `label`, `state`"),
        "{errors}"
    );
}

#[test]
fn a_view_field_reads_through_a_reference_only() {
    let errors = refused(&with(
        "field: {related: {via: desk_id, field: label}}",
        "field: {related: {via: ticket_id, field: label}}",
    ));
    assert!(
        errors.contains("type_mismatch") && errors.contains("names no one entity's row"),
        "{errors}"
    );
    let errors = refused(&with(
        "field: {related: {via: desk_id, field: label}}",
        "field: {related: {via: desk, field: label}}",
    ));
    assert!(
        errors.contains("undeclared_reference")
            && errors.contains("`desk`, which is not a field of `relstate.desk.Ticket`"),
        "{errors}"
    );
}

#[test]
fn a_view_field_read_through_a_reference_has_a_name_of_its_own() {
    let errors = refused(&with(
        "      - name: desk_label\n        type: Optional<String>",
        "      - name: state\n        type: Optional<String>",
    ));
    assert!(
        errors.contains("conflicting_declaration")
            && errors.contains("`state` is a field of `relstate.desk.Ticket`"),
        "{errors}"
    );
}

#[test]
fn a_filter_path_through_a_reference_reads_a_field_of_the_referenced_row() {
    let errors = refused(&with(
        "    filter: desk_id.state == Open",
        "    filter: desk_id.colour == Open",
    ));
    assert!(errors.contains("desk_id.colour"), "{errors}");
    let errors = refused(&with(
        "    filter: desk_id.state == Open",
        "    filter: desk_id.state == Shut",
    ));
    assert!(
        errors.contains("Shut"),
        "an unknown state is refused: {errors}"
    );
}

#[test]
fn navigation_beside_an_aggregate_paging_absent_or_an_order_is_refused_by_name() {
    let aggregate = with_view(
        "  - name: relstate.desk.TicketsByDesk
    source: relstate.desk.Ticket
    consistency: read_your_writes
    fields:
      - {name: tickets, type: Integer, aggregate: {count: {}}}
    filter: desk_id.state == Open
",
    );
    let errors = refused(&aggregate);
    assert!(
        errors.contains("unsupported_construct") && errors.contains("an aggregate view"),
        "{errors}"
    );
    let ordered = with(
        "        field: {related: {via: desk_id, field: label}}\n",
        "        field: {related: {via: desk_id, field: label}}\n    order_by: [desk_label]\n",
    );
    let errors = refused(&ordered);
    assert!(
        errors.contains("unsupported_construct") && errors.contains("an order key"),
        "{errors}"
    );
}

/// A view over two entities, the shape beyond10x/ess#564 wrote.
const TWO_SOURCES: &str = "  - name: relstate.desk.TicketDesks
    source: [relstate.desk.Ticket, relstate.desk.Desk]
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: relstate.desk.TicketId}
      - {name: label, type: String}
";

#[test]
fn a_view_source_list_is_refused_by_name_at_every_format() {
    for format in ["ess/16", "ess/27", "ess/28", "ess/29", "ess/30"] {
        let document =
            with_view(TWO_SOURCES).replacen("format: ess/28", &format!("format: {format}"), 1);
        let errors = refused(&document);
        assert!(
            errors.contains(
                "[unsupported_construct] view.relstate.desk.TicketDesks.source: \
                 `relstate.desk.TicketDesks` names 2 sources"
            ) && errors.contains("a view has one source"),
            "{format}: {errors}"
        );
        assert!(
            errors.contains("{related: {via, field}}"),
            "{format}: the hint names navigation: {errors}"
        );
        assert!(
            !errors.contains("invalid type: sequence"),
            "{format}: no parse error: {errors}"
        );
    }
}

/// Adopter report N22 (beyond10x/ess#675): a shelf view reading the cover of the book its shelf
/// holds, written `from: book.cover`. `book` is a `references` relation of the source, carried by
/// `book_id`.
const SHELF: &str = "format: ess/31
system: fit
version: v1
domain: fit.shelf
summary: A shelf view shows the cover of the book it holds.
types:
  - {name: fit.shelf.ShelfId, kind: newtype, of: Uuid}
  - {name: fit.shelf.BookId, kind: newtype, of: Uuid}
entities:
  - name: fit.shelf.Book
    identity: {name: book_id, type: fit.shelf.BookId}
    fields:
      - {name: cover, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
  - name: fit.shelf.Shelf
    identity: {name: shelf_id, type: fit.shelf.ShelfId}
    fields:
      - {name: book_id, type: fit.shelf.BookId}
    relations:
      - {name: book, kind: references, target: fit.shelf.Book, cardinality: one, via: book_id}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
events:
  - name: fit.shelf.BookAdded
    fields: [{name: book_id, type: fit.shelf.BookId}]
  - name: fit.shelf.ShelfAdded
    fields: [{name: shelf_id, type: fit.shelf.ShelfId}]
actors:
  - name: fit.shelf.Keeper
    may: [fit.shelf.AddBook, fit.shelf.AddShelf]
commands:
  - name: fit.shelf.AddBook
    input:
      - {name: cover, type: Optional<String>}
    outcomes:
      - name: added
        creates: fit.shelf.Book
        instance: book_id
        emits: [fit.shelf.BookAdded]
        payload: {fit.shelf.BookAdded: {book_id: {generated: true}}}
        sets: {cover: input.cover}
  - name: fit.shelf.AddShelf
    input:
      - {name: book_id, type: fit.shelf.BookId}
    outcomes:
      - name: unknown-book
        when_related: {via: input.book_id, exists: false}
        error: fit.shelf.BookNotFound
      - name: added
        creates: fit.shelf.Shelf
        instance: shelf_id
        emits: [fit.shelf.ShelfAdded]
        payload: {fit.shelf.ShelfAdded: {shelf_id: {generated: true}}}
        sets: {book_id: input.book_id}
errors:
  - {name: fit.shelf.BookNotFound, summary: No book carries this identity., fields: []}
views:
  - name: fit.shelf.Shelves
    source: fit.shelf.Shelf
    consistency: read_your_writes
    fields:
      - {name: shelf_id, type: fit.shelf.ShelfId}
      - {name: cover, type: Optional<String>, from: book.cover}
";

const FROM: &str = "from: book.cover";
const FROM_AT: &str = "view.fit.shelf.Shelves.fields[1].from";

/// Every refusal of `body`, which must not validate.
fn refusals(body: &str) -> Vec<ess_primitives::error::ValidationError> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    match ess_domain::Specification::assemble([(
        ess_domain::system::Source::new("shelf.yaml"),
        raw,
    )]) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors.as_slice().to_vec(),
    }
}

/// The one refusal of `body`, which is the refusal of its view field's `from:`.
fn from_refusal(body: &str) -> ess_primitives::error::ValidationError {
    let found = refusals(body);
    let listed: Vec<String> = found.iter().map(ToString::to_string).collect();
    assert_eq!(
        found.len(),
        1,
        "the `from:` alone is refused, and nothing it would read is refused again: {listed:#?}"
    );
    let refused = found.into_iter().next().expect("one refusal");
    assert_eq!(
        refused.code,
        ess_primitives::error::ValidationCode::UnsupportedConstruct,
        "{refused}"
    );
    assert_eq!(refused.location, FROM_AT, "{refused}");
    refused
}

#[test]
fn a_view_field_from_names_the_related_spelling() {
    let idiom = "field: {related: {via: book_id, field: cover}}";
    // The premise: the spelling the hint names is the one that validates.
    let written = SHELF.replacen(FROM, idiom, 1);
    spec(&written).unwrap_or_else(|errors| panic!("the idiom validates: {errors}"));

    // `book` is the relation, and the rewrite reads through the field that carries it.
    for from in [FROM, "from: book_id.cover"] {
        let refused = from_refusal(&SHELF.replacen(FROM, from, 1));
        let hint = refused.hint.clone().unwrap_or_default();
        assert!(
            hint.contains(idiom),
            "`{from}`: the hint is the rewrite: {refused}"
        );
        assert!(
            refused.message.contains(&format!("`{from}`")),
            "`{from}`: the refusal quotes what was written: {refused}"
        );
        assert!(
            !refused.to_string().contains("unknown field"),
            "`{from}`: {refused}"
        );
    }
}

#[test]
fn a_view_field_from_that_names_no_reference_names_both_spellings() {
    for from in [
        "from: cover",
        "from: shelf.cover",
        "from: shelf_id.cover",
        "from: book.edition.cover",
        "from: {entity: fit.shelf.Book, field: cover}",
    ] {
        let refused = from_refusal(&SHELF.replacen(FROM, from, 1));
        let hint = refused.hint.clone().unwrap_or_default();
        assert!(
            hint.contains("field: {related: {via: <reference field>, field: <field>}}")
                && hint.contains("ess/28"),
            "`{from}`: the hint names the related spelling: {refused}"
        );
        assert!(
            hint.contains("expr:") && hint.contains("ess/27"),
            "`{from}`: the hint names the computed spelling: {refused}"
        );
        assert!(
            !hint.contains("via: book_id"),
            "`{from}`: no rewrite is guessed for what names no reference: {refused}"
        );
    }
}

#[test]
fn a_view_field_from_below_ess_28_names_ess_28() {
    for format in ["ess/27", "ess/16"] {
        let body = SHELF.replacen("format: ess/31", &format!("format: {format}"), 1);
        let found = refusals(&body);
        let listed: Vec<String> = found.iter().map(ToString::to_string).collect();
        let at: Vec<_> = found
            .iter()
            .filter(|refused| refused.location == FROM_AT)
            .collect();
        assert_eq!(at.len(), 1, "{format}: {listed:#?}");
        let refused = at[0];
        assert_eq!(
            refused.code,
            ess_primitives::error::ValidationCode::UnsupportedConstruct,
            "{format}: `from:` is never admitted, at any format: {refused}"
        );
        let hint = refused.hint.clone().unwrap_or_default();
        assert!(
            hint.contains("format: ess/28")
                && hint.contains("field: {related: {via: book_id, field: cover}}"),
            "{format}: the hint names the format that reads the rewrite: {refused}"
        );
    }
    // From `ess/28` on the hint names no format to declare.
    for format in ["ess/28", "ess/31"] {
        let body = SHELF.replacen("format: ess/31", &format!("format: {format}"), 1);
        let refused = from_refusal(&body);
        assert!(
            !refused.hint.clone().unwrap_or_default().contains("format:"),
            "{format}: {refused}"
        );
    }
}

#[test]
fn a_view_source_string_is_unchanged() {
    spec(DESK).unwrap_or_else(|errors| panic!("{errors}"));
    let raw = ess_domain::spec::RawSpecFile::parse(DESK).expect("the fixture parses");
    let written = serde_json::to_value(&raw.views[0]).expect("the view serializes");
    assert_eq!(
        written["source"],
        serde_json::json!("relstate.desk.Ticket"),
        "a one-entity source is written back as the string it was"
    );
}

/// Adversary pass 1 on unit h: a view withheld for its `from:` is still the view an actor's grant
/// names (`ess/22`), so the grant is not refused a second time.
#[test]
fn adversary_h_a_granted_view_with_a_field_from_is_refused_once() {
    let body = SHELF.replacen(
        "may: [fit.shelf.AddBook, fit.shelf.AddShelf]",
        "may: [fit.shelf.AddBook, fit.shelf.AddShelf, fit.shelf.Shelves]",
        1,
    );
    assert_ne!(body, SHELF, "the fixture holds the grant");
    let written = body.replacen(FROM, "field: {related: {via: book_id, field: cover}}", 1);
    spec(&written).unwrap_or_else(|errors| panic!("the premise validates: {errors}"));
    let refused = from_refusal(&body);
    assert!(
        refused
            .hint
            .clone()
            .unwrap_or_default()
            .contains("field: {related: {via: book_id, field: cover}}"),
        "{refused}"
    );
}

/// Adversary pass 1 on unit h: the source and its relation in one file, the view in another,
/// behind a view of the first file at the same position. The rewrite is resolved against the
/// assembled specification, and the position of the view that wrote `from:` is its own file's.
#[test]
fn adversary_h_a_view_field_from_resolves_a_source_declared_in_another_file() {
    let (model, views) = SHELF.split_once("views:\n").expect("the fixture has views");
    let first = format!(
        "{model}views:\n  - name: fit.shelf.Plain\n    source: fit.shelf.Shelf\n    consistency: \
         read_your_writes\n    fields:\n      - {{name: shelf_id, type: fit.shelf.ShelfId}}\n"
    );
    let second = format!("domain: fit.shelf\nviews:\n{views}");
    let texts = [first.as_str(), second.as_str()];
    let parsed: Vec<_> = ess_domain::spec::RawSpecFile::parse_all(&texts)
        .into_iter()
        .map(|parsed| parsed.unwrap_or_else(|error| panic!("each file parses: {error}")))
        .collect();
    let sources = ["model.yaml", "views.yaml"]
        .into_iter()
        .map(ess_domain::system::Source::new)
        .zip(parsed);
    let errors = match ess_domain::Specification::assemble(sources) {
        Ok(_) => panic!("the specification validates:\n{first}\n---\n{second}"),
        Err(errors) => errors.as_slice().to_vec(),
    };
    let listed: Vec<String> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(errors.len(), 1, "the `from:` alone is refused: {listed:#?}");
    assert_eq!(errors[0].location, FROM_AT, "{listed:#?}");
    assert!(
        errors[0]
            .hint
            .clone()
            .unwrap_or_default()
            .contains("field: {related: {via: book_id, field: cover}}"),
        "{listed:#?}"
    );
}

/// Adversary pass 1 on unit h: a `<x>.<field>` with an empty side names no field to read, and no
/// rewrite is guessed for it (the guard in `view::from::rewrite`).
#[test]
fn adversary_h_a_view_field_from_with_an_empty_segment_guesses_no_rewrite() {
    for from in ["from: book.", "from: .cover", "from: book_id."] {
        let refused = from_refusal(&SHELF.replacen(FROM, from, 1));
        let hint = refused.hint.clone().unwrap_or_default();
        assert!(
            !hint.contains("via: book_id"),
            "`{from}`: no rewrite: {refused}"
        );
        assert!(
            hint.contains("expr:"),
            "`{from}`: both spellings: {refused}"
        );
    }
}

/// Correction round 1 on unit h: each `from:` carries the line and column of its own key, whatever
/// else in the file writes `from:` — a lifecycle transition, a second field of the same view, a
/// block-style field — so no refusal depends on the key occurring once in the file.
#[test]
fn every_view_field_from_carries_the_position_of_its_own_key() {
    let body = SHELF
        .replacen(
            "    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: \
             []}\n  - name: fit.shelf.Shelf",
            "    lifecycle:\n      initial: Active\n      states: [Active, Retired]\n      \
             terminal: [Retired]\n      transitions:\n        - {name: retire, from: [Active], to: \
             Retired}\n  - name: fit.shelf.Shelf",
            1,
        )
        .replacen(
            "      - {name: shelf_id, type: fit.shelf.ShelfId}\n      - {name: cover",
            "      - name: shelf_id\n        type: fit.shelf.ShelfId\n        from: shelf.shelf_id\n      \
             - {name: cover",
            1,
        );
    assert_eq!(body.matches("from:").count(), 3, "{body}");
    let found = refusals(&body);
    let mut cited: Vec<(String, usize, usize)> = found
        .iter()
        .filter(|refused| refused.location.starts_with("view."))
        .map(|refused| {
            let span = refused
                .site()
                .and_then(|site| site.span.clone())
                .unwrap_or_else(|| panic!("a position: {refused}"));
            assert_eq!(span.source, "shelf.yaml", "{refused}");
            (refused.location.clone(), span.line, span.column)
        })
        .collect();
    cited.sort();
    let at = |needle: &str| {
        let (index, line) = body
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains(needle))
            .unwrap_or_else(|| panic!("`{needle}` is written"));
        (index + 1, line.find("from:").expect("a key") + 1)
    };
    let (shelf_line, shelf_column) = at("from: shelf.shelf_id");
    let (cover_line, cover_column) = at("from: book.cover");
    assert_eq!(
        cited,
        vec![
            (
                "view.fit.shelf.Shelves.fields[0].from".to_owned(),
                shelf_line,
                shelf_column
            ),
            (FROM_AT.to_owned(), cover_line, cover_column),
        ]
    );
}
