//! A calendar window added, removed or edited is a behaviour change, rendered as the window it is
//! (beyond10x/ess#244 part b, `docs/design/calendar-window-guards.md`): every field of it — the
//! instant, a day, `from`, `to`, the offset — moves the outcome's condition.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const RELEASES: &str = include_str!("../../ess-conformance/tests/fixtures/calendar-windows.yaml");

const WINDOW: &str = r#"window: {at: starts_at, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#;

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("releases.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn diff_window_changes_are_behaviour() {
    assert!(RELEASES.contains(WINDOW));
    let before = ir(RELEASES);
    for (written, rendered) in [
        (
            WINDOW.replace("[mon, tue, wed, thu]", "[mon, tue, wed]"),
            "when window(at starts_at, mon tue wed, 08:00-16:00, +01:00)",
        ),
        (
            WINDOW.replace(r#"from: "08:00""#, r#"from: "08:30""#),
            "when window(at starts_at, mon tue wed thu, 08:30-16:00, +01:00)",
        ),
        (
            WINDOW.replace(r#"to: "16:00""#, r#"to: "16:01""#),
            "when window(at starts_at, mon tue wed thu, 08:00-16:01, +01:00)",
        ),
        (
            WINDOW.replace(r#""+01:00""#, r#""+02:00""#),
            "when window(at starts_at, mon tue wed thu, 08:00-16:00, +02:00)",
        ),
        (
            WINDOW.replace("at: starts_at", "at: now"),
            "when window(at now, mon tue wed thu, 08:00-16:00, +01:00)",
        ),
    ] {
        let delta = diff(&before, &ir(&RELEASES.replacen(WINDOW, &written, 1))).unwrap();
        let json = delta.to_canonical_json();
        assert!(
            json.contains("outcome-condition-changed"),
            "{written}: {json}"
        );
        assert!(
            json.contains(
                r#""before": "when window(at starts_at, mon tue wed thu, 08:00-16:00, +01:00)""#
            ) && json.contains(&format!(r#""after": "{rendered}""#)),
            "{written}: {json}"
        );
    }
    // The same window written another way — its days in another order — is no change.
    let reordered = RELEASES.replacen(
        WINDOW,
        r#"window: {at: starts_at, days: [thu, wed, tue, mon], from: "08:00", to: "16:00", offset: "+01:00"}"#,
        1,
    );
    let json = diff(&before, &ir(&reordered)).unwrap().to_canonical_json();
    assert!(!json.contains("condition-changed"), "{json}");
}
