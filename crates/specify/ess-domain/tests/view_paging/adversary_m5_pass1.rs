//! Adversary pass 1 for the absent-row view (`absent:`, ess/25, beyond10x/ess#533): literals a
//! field's type does not admit, field shapes no literal spells, and filters that look like an
//! identity read and are not one.

const FLAGS: &str =
    include_str!("../../../../verify/ess-conformance/tests/fixtures/view-absent-row.yaml");
const ROW: &str = "    absent:\n      row: {paused: false, level: 0, state: Set}\n";
const FLAG_ID: &str = "  - {name: demo.status.FlagId, kind: newtype, of: Uuid}\n";
const ENTITY_NOTE: &str = "      - {name: note, type: 'Optional<String>'}\n    lifecycle:";
const VIEW_NOTE: &str = "      - {name: note, type: 'Optional<String>'}\n    absent:";

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

/// The diagnostic lines of a document, empty where it is admitted.
fn refusals(text: &str) -> Vec<String> {
    match spec(text) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .lines()
            .filter(|line| line.starts_with('['))
            .map(str::to_owned)
            .collect(),
    }
}

/// `FLAGS` with one more `Optional` field `name: type` on the entity and the view, and the default
/// row `row`.
fn with_field(declared: &str, field: &str, row: &str) -> String {
    let field = format!("      - {{name: {field}}}\n");
    let typed = if declared.is_empty() {
        FLAGS.to_owned()
    } else {
        replaced(FLAGS, FLAG_ID, &format!("{FLAG_ID}{declared}"))
    };
    replaced(
        &replaced(
            &replaced(
                &typed,
                ENTITY_NOTE,
                &ENTITY_NOTE.replace("    lifecycle:", &format!("{field}    lifecycle:")),
            ),
            VIEW_NOTE,
            &VIEW_NOTE.replace("    absent:", &format!("{field}    absent:")),
        ),
        ROW,
        &format!("    absent:\n      row: {row}\n"),
    )
}

/// `level` is a newtype of `Integer` whose values are at least 1; the default row's `0` is not one.
#[test]
fn a_literal_outside_a_newtype_invariant_is_refused() {
    let text = replaced(
        &replaced(
            FLAGS,
            FLAG_ID,
            &format!(
                "{FLAG_ID}  - {{name: demo.status.Level, kind: newtype, of: Integer, invariants: \
                 [\"value >= 1\"]}}\n"
            ),
        ),
        "{name: level, type: Integer}",
        "{name: level, type: demo.status.Level}",
    );
    let lines = refusals(&text);
    assert!(
        lines
            .iter()
            .any(|line| line
                .starts_with("[type_mismatch] view.demo.status.FlagById.absent.row.level")),
        "`level: 0` breaks `value >= 1`: {lines:?}"
    );
}

/// `code` is a newtype of `String` drawn from `ab`; the default row's `xyz` is not.
#[test]
fn a_literal_outside_a_declared_alphabet_is_refused() {
    let text = with_field(
        "  - {name: demo.status.Code, kind: newtype, of: String, alphabet: \"ab\"}\n",
        "code, type: 'Optional<demo.status.Code>'",
        "{paused: false, level: 0, state: Set, code: xyz}",
    );
    let lines = refusals(&text);
    assert!(
        lines
            .iter()
            .any(|line| line
                .starts_with("[type_mismatch] view.demo.status.FlagById.absent.row.code")),
        "`xyz` is outside the alphabet `ab`: {lines:?}"
    );
}

/// A `List` field has no literal spelling: naming it in `row:` is refused at the key.
#[test]
fn a_list_field_in_the_default_row_is_refused() {
    let text = with_field(
        "",
        "tags, type: 'Optional<List<String>>'",
        "{paused: false, level: 0, state: Set, tags: red}",
    );
    let lines = refusals(&text);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("view.demo.status.FlagById.absent.row.tags")),
        "`tags: red` is no `List<String>`: {lines:?}"
    );
}

/// The identity compared with the parameter by anything but equality reads no single row.
#[test]
fn an_identity_compared_by_inequality_is_refused() {
    let text = replaced(FLAGS, "filter: id == param.id", "filter: id != param.id");
    let lines = refusals(&text);
    assert!(
        lines
            .iter()
            .any(|line| line
                .starts_with("[conflicting_declaration] view.demo.status.FlagById.absent:")),
        "{lines:?}"
    );
}

/// The probe beside the two above: the same out-of-alphabet literal written in a command's `sets:`,
/// the grammar `absent:` says it reuses. Tells whether the gap is the default row's or the shared
/// literal check's.
#[test]
fn the_same_out_of_alphabet_literal_in_sets_is_refused() {
    let text = replaced(
        &with_field(
            "  - {name: demo.status.Code, kind: newtype, of: String, alphabet: \"ab\"}\n",
            "code, type: 'Optional<demo.status.Code>'",
            "{paused: false, level: 0, state: Set, note: x}",
        ),
        "        sets: {paused: true}\n",
        "        sets: {paused: true, code: xyz}\n",
    );
    let lines = refusals(&text);
    assert!(
        lines.iter().any(|line| line.contains("code")),
        "`code: xyz` in `sets:` is outside the alphabet `ab`: {lines:?}"
    );
}
