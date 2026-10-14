//! Adversary pass 1 for the absent-row view (`absent:`, ess/25, beyond10x/ess#533): a default
//! row respelled, not changed, is no change a reader meets.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::classified;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const FLAGS: &str = include_str!("../../../ess-conformance/tests/fixtures/view-absent-row.yaml");
const ROW: &str = "    absent:\n      row: {paused: false, level: 0, state: Set}\n";
const FIELD: &str = "      - {name: amount, type: 'Optional<Decimal>'}\n";

/// The fixture with one more `Optional` field, `amount`, given `row` as the default row; `None`
/// where the document is refused.
fn ir(row: &str) -> Option<EssIr> {
    let text = FLAGS
        .replace(
            "      - {name: note, type: 'Optional<String>'}\n    lifecycle:",
            &format!("      - {{name: note, type: 'Optional<String>'}}\n{FIELD}    lifecycle:"),
        )
        .replace(
            "      - {name: note, type: 'Optional<String>'}\n    absent:",
            &format!("      - {{name: note, type: 'Optional<String>'}}\n{FIELD}    absent:"),
        )
        .replace(ROW, &format!("    absent:\n      row: {row}\n"));
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("flags.yaml"), raw)]).ok()?;
    Some(compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}")))
}

/// Two spellings of one default row, each admitted, are no `absent-changed`: `1.5` and `'1.50'`
/// are one `Decimal`, and `0` and `'0'` one `Integer`.
#[test]
fn a_respelled_literal_is_no_change() {
    let base = "{paused: false, level: 0, state: Set, amount: 1.5}";
    let before = ir(base).expect("the base default row is admitted");
    let mut compared = 0;
    for respelled in [
        "{paused: false, level: 0, state: Set, amount: '1.50'}",
        "{paused: false, level: '0', state: Set, amount: 1.5}",
    ] {
        let Some(after) = ir(respelled) else {
            continue;
        };
        compared += 1;
        let delta = classified(&before, &after).unwrap();
        assert!(
            delta.changes().is_empty(),
            "`{respelled}` respells `{base}` and is reported as {}",
            delta.to_canonical_json()
        );
    }
    assert!(
        compared > 0,
        "no respelling is admitted, so nothing was compared"
    );
}
