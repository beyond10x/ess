//! A finite recursive fixture (beyond10x/ess#416) executed against generated Rust and Go targets.
//!
//! The issue's model, served by one network component, is emitted, built as its generated entry
//! point with no hand-written code, and driven over a real socket by the suite the same model
//! synthesizes. The provider supplies a finite tree for the typed fixture; the generated server
//! decodes it, and its event echoes it back, so the suite's fixture equality holds only if the
//! generated codecs carried every level of the recursive value.
//!
//! Two departures from the issue's text, each needed to serve it: the command declares no typed
//! `response:` (a command with one is not generated, `determined.rs`), and the event carries the
//! input value, so what was decoded is observable. The list boundary and the optional twin are both
//! run; the optional one is the boxed `Optional<Self>` layout of beyond10x/ess#400.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use std::collections::BTreeMap;

use ess_conformance::report::Status;
use ess_primitives::node::Node;
use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const SCENARIO: &str = "fixtureprobe.recursive.Submit/outcome/accepted";
const DOMAIN: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/recursive-fixtures/recursive.yaml"
);

/// The issue's two files as one served document, the value echoed and the typed response dropped.
fn model(boundary: &str) -> ess_compiler::ir::EssIr {
    let mut text = format!("format: ess/14\nsystem: fixtureprobe\nversion: v1\n{DOMAIN}");
    for (from, to) in [
        (
            "      - name: accepted\n        type: Boolean\ncommands:",
            "      - name: accepted\n        type: Boolean\n      - name: value\n        type: fixtureprobe.recursive.Value\ncommands:",
        ),
        (
            "    response:\n      - name: accepted\n        type: Boolean\n",
            "",
        ),
        (
            "            accepted: true\n",
            "            accepted: true\n            value: input.value\n",
        ),
    ] {
        assert!(text.contains(from), "the issue's model carries `{from}`");
        text = text.replace(from, to);
    }
    if boundary == "optional" {
        text = text.replace(
            "      - name: children\n        type: List<fixtureprobe.recursive.Value>\n",
            "      - name: next\n        type: Optional<fixtureprobe.recursive.Value>\n",
        );
    }
    served::model(&served::served(&text, "probe"))
}

fn node(json: &str) -> Node {
    serde_json::from_str(json).unwrap_or_else(|error| panic!("{error}: {json}"))
}

/// One case: the boundary, a conforming tree, and a tree with a wrong leaf deep inside it.
const CASES: [(&str, &str, &str); 2] = [
    (
        "list",
        r#"{"text":"root","children":[{"text":"a","children":[]},{"text":"b","children":[{"text":"c","children":[]}]}]}"#,
        r#"{"text":"root","children":[{"text":"a","children":[]},{"text":"b","children":[{"text":5,"children":[]}]}]}"#,
    ),
    (
        "optional",
        r#"{"text":"root","next":{"text":"a","next":{"text":"b"}}}"#,
        r#"{"text":"root","next":{"text":"a","next":{"text":false}}}"#,
    ),
];

fn served_fixture(target: Target) {
    for (boundary, finite, malformed) in CASES {
        let ir = model(boundary);
        let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
        assert_eq!(
            synthesis.plan.disposition_of(
                CapabilityKind::CommandBehavior,
                "fixtureprobe.recursive.Submit"
            ),
            Some(&SynthesisDisposition::Generated),
            "{target:?} {boundary}:\n{}",
            synthesis.plan.to_markdown()
        );
        // The echoed tree is held to the supplied fixture by equality, not only by its shape.
        let suite = serde_json::to_value(ess_conformance::synthesize(&ir).suite).unwrap();
        let steps = suite["scenarios"][SCENARIO]["steps"].as_array().unwrap();
        assert!(
            steps
                .iter()
                .any(|step| step["step"] == "expect_event_values"
                    && step["payload"]["value"]
                        == serde_json::json!({"kind": "fixture", "fixture": "finite-value"})),
            "{boundary}: {steps:#?}"
        );
        let (root, _) = served::emit(&ir, target, &format!("recursive-fixture-{boundary}"));
        let binary = served::build(&root, target, "fixtureprobe", "probe", false);
        let supplied = |json: &str| Some(BTreeMap::from([("finite-value".to_owned(), node(json))]));

        let statuses = served::run_with_fixtures(&ir, &binary, supplied(finite));
        served::assert_passes(
            &format!("{target:?} {boundary}"),
            &statuses,
            &["Submit/outcome/accepted"],
        );
        let statuses = served::run_with_fixtures(&ir, &binary, supplied(malformed));
        assert_eq!(
            statuses[SCENARIO],
            Status::Error,
            "{target:?} {boundary}: a malformed tree is refused before the target: {statuses:#?}"
        );
        let statuses = served::run_with_fixtures(&ir, &binary, None);
        assert_eq!(
            statuses[SCENARIO],
            Status::Unsupported,
            "{target:?} {boundary}: no provider leaves the obligation unmet: {statuses:#?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[test]
fn a_finite_recursive_fixture_executes_against_the_generated_rust_target() {
    served_fixture(Target::Rust);
}

#[test]
fn a_finite_recursive_fixture_executes_against_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    served_fixture(Target::Go);
}
