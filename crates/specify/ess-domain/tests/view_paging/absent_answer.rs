//! A view read by identity declares its answer for an identity no row carries (`absent:`, ess/25,
//! beyond10x/ess#533, `docs/design/read-api-view-idioms.md`, "An identity no row carries"): where
//! the key validates and where it is refused.

const FLAGS: &str =
    include_str!("../../../../verify/ess-conformance/tests/fixtures/view-absent-row.yaml");

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("flags.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

/// The diagnostic lines of a refused document.
fn refusals(text: &str) -> Vec<String> {
    spec(text)
        .map(|_| ())
        .expect_err("the document is refused")
        .lines()
        .filter(|line| line.starts_with('['))
        .map(str::to_owned)
        .collect()
}

const ROW: &str = "    absent:\n      row: {paused: false, level: 0, state: Set}\n";

/// The fixture validates at ess/25, and the view carries the default row as written.
#[test]
fn absent_default_row_validates_on_an_identity_selecting_view_at_ess_25() {
    let spec = spec(FLAGS).unwrap_or_else(|errors| panic!("{errors}"));
    let view = spec
        .views()
        .values()
        .find(|view| view.name.to_string() == "demo.status.FlagById")
        .expect("the view is declared");
    let absent = view.absent.as_ref().expect("the view declares `absent:`");
    let row: Vec<(&str, &str)> = absent
        .row
        .iter()
        .map(|(field, literal)| (field.as_str(), literal.value.as_str()))
        .collect();
    assert_eq!(
        row,
        [("level", "0"), ("paused", "false"), ("state", "Set")],
        "the default row, in field-name order"
    );
    // The filter written the other way round selects by identity too.
    let flipped = replaced(FLAGS, "filter: id == param.id", "filter: param.id == id");
    if let Err(errors) = self::spec(&flipped) {
        panic!("{errors}");
    }
}

/// A filter that does not select the source entity by its identity, alone, is refused, naming the
/// entity and its identity field: another field, a further conjunct, and no filter at all.
#[test]
fn absent_is_refused_on_a_view_not_selecting_by_identity() {
    let by_level = replaced(
        &replaced(
            FLAGS,
            "filter: id == param.id",
            "filter: level == param.level",
        ),
        "params: [{name: id, type: demo.status.FlagId}]",
        "params: [{name: level, type: Integer}]",
    );
    assert_eq!(
        refusals(&by_level),
        [
            "[conflicting_declaration] view.demo.status.FlagById.absent: `demo.status.FlagById` \
          declares `absent:`, the answer for an identity no row carries, and its filter does not \
          select `demo.status.Flag` by its identity field `id`: the filter must be `id == \
          param.<name>` and nothing else (hint: write `filter: id == param.<name>`, or drop \
          `absent:`)"
        ]
    );
    let conjoined = replaced(
        FLAGS,
        "filter: id == param.id",
        "filter: [id == param.id, paused == false]",
    );
    assert_eq!(refusals(&conjoined).len(), 1, "{:?}", refusals(&conjoined));
    let unfiltered = replaced(
        &replaced(FLAGS, "    filter: id == param.id\n", ""),
        "    params: [{name: id, type: demo.status.FlagId}]\n",
        "",
    );
    assert_eq!(
        refusals(&unfiltered).len(),
        1,
        "{:?}",
        refusals(&unfiltered)
    );
}

/// Below ess/25 the key is refused once, naming ess/25.
#[test]
fn absent_is_refused_below_ess_25() {
    assert_eq!(
        refusals(&replaced(FLAGS, "format: ess/25", "format: ess/24")),
        ["[unsupported_format_version] view.demo.status.FlagById.absent: a view's answer for an \
          identity no row carries (`absent:`) requires specification format ess/25 (hint: \
          declare `format: ess/25`, or drop `absent:`)"]
    );
}

/// What the default row may hold: projected fields only, never the identity, every required field
/// given, each a literal of its field's type.
#[test]
fn the_default_row_is_checked_field_by_field() {
    let unknown = replaced(
        FLAGS,
        ROW,
        "    absent:\n      row: {paused: false, level: 0, state: Set, colour: red}\n",
    );
    assert_eq!(
        refusals(&unknown),
        [
            "[undeclared_reference] view.demo.status.FlagById.absent.row.colour: \
          `demo.status.FlagById` does not project `colour`, so its default row cannot hold it \
          (hint: projected fields: `id`, `paused`, `level`, `state`, `note`)"
        ]
    );
    let identity = replaced(
        FLAGS,
        ROW,
        "    absent:\n      row: {id: 00000000-0000-4000-8000-000000000001, paused: false, level: 0, state: Set}\n",
    );
    assert_eq!(
        refusals(&identity),
        ["[conflicting_declaration] view.demo.status.FlagById.absent.row.id: `id` is the identity \
          `demo.status.FlagById` is read by, and the default row holds the requested value there \
          (hint: drop `id` from `row:`)"]
    );
    let missing = replaced(
        FLAGS,
        ROW,
        "    absent:\n      row: {paused: false, state: Set}\n",
    );
    assert_eq!(
        refusals(&missing),
        [
            "[missing_declaration] view.demo.status.FlagById.absent.row: the default row of \
          `demo.status.FlagById` gives no value for `level`, which is not `Optional` (hint: add \
          `level: <literal>` to `row:`)"
        ]
    );
    let mistyped = replaced(
        FLAGS,
        ROW,
        "    absent:\n      row: {paused: 'no', level: 0, state: Set}\n",
    );
    assert_eq!(refusals(&mistyped).len(), 1, "{:?}", refusals(&mistyped));
    assert!(
        refusals(&mistyped)[0]
            .starts_with("[type_mismatch] view.demo.status.FlagById.absent.row.paused: "),
        "{:?}",
        refusals(&mistyped)
    );
    let variant = replaced(
        FLAGS,
        ROW,
        "    absent:\n      row: {paused: false, level: 0, state: Gone}\n",
    );
    assert_eq!(refusals(&variant).len(), 1, "{:?}", refusals(&variant));
    // A value no literal spells has no input to come from on a view: the hint names the view's
    // default row, where an `Optional` field is left out instead.
    let unspelled = replaced(
        &FLAGS.replace("'Optional<String>'", "'Optional<Uuid>'"),
        ROW,
        "    absent:\n      row: {paused: false, level: 0, state: Set, note: none}\n",
    );
    assert_eq!(
        refusals(&unspelled),
        [
            "[type_mismatch] view.demo.status.FlagById.absent.row.note: \
          `demo.status.FlagById.note` is `Uuid` underneath, and a literal in a view's default row \
          (`absent: {row: …}`) is text, `true` or `false`, a whole number or a decimal (hint: a \
          literal may be text, a variant of an enum, `true` or `false`, a whole number or a \
          decimal; anything else has no literal in the default row of `demo.status.FlagById`: \
          leave an `Optional` field out of `row:`, and it is absent there)"
        ]
    );
    // An `Optional` field is given a value or left out; both validate.
    let noted = replaced(
        FLAGS,
        ROW,
        "    absent:\n      row: {paused: false, level: 0, state: Set, note: none yet}\n",
    );
    if let Err(errors) = spec(&noted) {
        panic!("{errors}");
    }
}

/// Not beside an aggregate or paging, and `row:` is the one key.
#[test]
fn absent_is_refused_beside_paging_and_unknown_keys() {
    let paged = replaced(
        FLAGS,
        "params: [{name: id, type: demo.status.FlagId}]",
        "params: [{name: id, type: demo.status.FlagId}, {name: page, type: Integer}, {name: \
         size, type: Integer}]\n    paging: {page: page, size: size}\n    order_by: [id asc]",
    );
    let lines = refusals(&paged);
    assert!(
        lines.iter().any(|line| line
            .starts_with("[conflicting_declaration] view.demo.status.FlagById.absent: ")
            && line.contains("`paging:`")),
        "{lines:?}"
    );
    let error = replaced(FLAGS, ROW, "    absent:\n      error: demo.status.NotSet\n");
    let refused = spec(&error)
        .map(|_| ())
        .expect_err("`error:` is not in this cut");
    assert!(refused.contains("unknown field `error`"), "{refused}");
}
