//! Adversary pass 1 on beyond10x/ess#195: a delivery-context change as `ess-diff` reports it.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_diff::SemanticChange;
use ess_domain::{system::Source, RawSpecFile, Specification};

const INBOX: &str = include_str!("../../ess-conformance/tests/fixtures/delivery-context.yaml");

fn model(text: &str) -> EssIr {
    let specification =
        Specification::assemble([(Source::new("inbox.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("inbox.yaml", text);
    compile_locating(&specification, &sources, &["inbox.yaml"]).unwrap()
}

fn changed_authority() -> String {
    INBOX.replacen(
        "context_authority: account-messages",
        "context_authority: tenant-messages",
        1,
    )
}

/// A delta carrying the new `external` cause cannot be written for a format whose readers only
/// know `event` and `periodic`: `ess-diff/3` … `ess-diff/9` readers were released before the
/// variant existed and would read `{"external": …}` as an unknown variant, not as an unsupported
/// format. The writer must refuse the downgrade, as it does for every other change a format cannot
/// represent.
#[test]
fn adv1_an_external_cause_is_not_writable_for_a_format_that_predates_it() {
    let delta = ess_diff::diff(&model(INBOX), &model(&changed_authority())).unwrap();
    let json = delta.to_canonical_json();
    assert!(json.contains("cause-changed"), "{json}");
    assert!(json.contains("\"external\""), "{json}");
    for older in 3..=9 {
        let format = ess_diff::DeltaFormat::parse(&format!("ess-diff/{older}")).unwrap();
        assert!(
            delta.to_canonical_json_for(format).is_err(),
            "a delta holding an `external` cause is written as ess-diff/{older}, whose readers \
             do not know the variant; current format {}",
            delta.format
        );
    }
}

/// A change to the delivery context alone renders as a cause change whose two sides read the
/// same, because `BindingCause::External` displays only its event.
#[test]
fn adv1_a_context_only_change_describes_what_changed() {
    let delta = ess_diff::diff(&model(INBOX), &model(&changed_authority())).unwrap();
    let described: Vec<String> = delta
        .changes()
        .iter()
        .filter_map(|change| match change {
            SemanticChange::Binding { changed, .. } => Some(changed.describe()),
            _ => None,
        })
        .collect();
    assert!(!described.is_empty(), "a binding change is reported");
    for text in &described {
        let Some((_, sides)) = text.split_once(": ") else {
            continue;
        };
        let Some((before, after)) = sides.split_once(" → ") else {
            continue;
        };
        assert_ne!(
            before, after,
            "the rendered change does not say what changed: {text}"
        );
    }
}
