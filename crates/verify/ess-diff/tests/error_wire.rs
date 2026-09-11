//! Error wire changes retain semantic identity and version their new delta vocabulary.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{
    change::{ErrorChange, SemanticChange},
    diff, EssDelta, RawEssDelta,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(naming: &str) -> EssIr {
    let text = format!("format: ess/4\nsystem: desk\nversion: v1\ndomains: [desk.api]\ndomain: desk.api\nerrors:\n  - name: desk.api.Invalid\n    summary: Legacy summary\n{naming}");
    let spec = Specification::assemble([(
        Source::new("error.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("error.yaml", text);
    compile(&spec, &sources).unwrap()
}

#[test]
fn wire_changes_use_qualified_fallback_and_require_new_delta_vocabulary() {
    let plain = model("");
    let named = model("    naming: {wire: bad_request}\n");
    for (before, after, expected) in [
        (&plain, &named, ("desk.api.Invalid", "bad_request")),
        (&named, &plain, ("bad_request", "desk.api.Invalid")),
    ] {
        let delta = diff(before, after).unwrap();
        assert_eq!(delta.changes().len(), 1);
        assert!(
            matches!(&delta.changes()[0], SemanticChange::Error { subject, changed: ErrorChange::WireNameChanged {before, after}} if subject.to_string() == "desk.api.Invalid" && before == expected.0 && after == expected.1)
        );
        assert_eq!(delta.format.major(), 4);
        let text = delta.to_canonical_json();
        let raw: RawEssDelta = serde_json::from_str(&text).unwrap();
        EssDelta::try_from(raw).unwrap();
        for old in 1..=3 {
            let raw: RawEssDelta =
                serde_json::from_str(&text.replace("ess-diff/4", &format!("ess-diff/{old}")))
                    .unwrap();
            assert!(EssDelta::try_from(raw).is_err());
        }
    }
}

#[test]
fn presentation_metadata_is_not_lost_or_confused_with_legacy_summary() {
    let delta = diff(
        &model(""),
        &model("    naming: {display: Rejected, summary: Naming summary}\n"),
    )
    .unwrap();
    assert_eq!(delta.changes().len(), 2);
    assert!(delta.changes().iter().any(|c| matches!(
        c,
        SemanticChange::Error {
            changed: ErrorChange::DisplayNameChanged { .. },
            ..
        }
    )));
    assert!(delta.changes().iter().any(|c| matches!(
        c,
        SemanticChange::Error {
            changed: ErrorChange::NamingSummaryChanged { .. },
            ..
        }
    )));
    assert!(!delta.changes().iter().any(|c| matches!(
        c,
        SemanticChange::Error {
            changed: ErrorChange::SummaryChanged { .. },
            ..
        }
    )));
}

#[test]
fn an_explicit_legacy_code_has_no_wire_delta() {
    assert!(
        diff(&model(""), &model("    naming: {wire: desk.api.Invalid}\n"))
            .unwrap()
            .changes()
            .is_empty()
    );
}
