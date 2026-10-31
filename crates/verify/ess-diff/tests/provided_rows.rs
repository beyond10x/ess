//! Adding or removing an entity's `provided: true` (ess/33,
//! story:a-fixture-may-be-a-row-the-deployment-already-holds) is a contract change: it decides
//! whether conformance takes a row of the entity from the deployment. No typed kind names it, so
//! the residual reports it as `system/<system>/unclassified-changed`, unknown on every axis, and
//! `--fail-on breaking-or-unknown` fails. A dedicated kind would need a new `ess-diff` delta; the
//! residual already reports the change, and keeps every earlier delta's vocabulary.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, diff, Compatibility, FailOn, Gate};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const UNCLASSIFIED: &str = "system/linking/unclassified-changed";

fn model(provided: bool) -> String {
    let marker = if provided { "    provided: true\n" } else { "" };
    format!(
        "format: ess/33
system: linking
version: v1
domain: linking.links
types:
  - {{name: linking.links.AppId, kind: newtype, of: Uuid}}
entities:
  - name: linking.links.App
{marker}    identity: {{name: app_id, type: linking.links.AppId}}
    fields: [{{name: title, type: String}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
"
    )
}

fn ir(provided: bool) -> EssIr {
    let text = model(provided);
    let spec = Specification::assemble([(
        Source::new("links.yaml"),
        RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn ids(old: &EssIr, new: &EssIr) -> Vec<String> {
    classified(old, new)
        .unwrap()
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

#[test]
fn adding_or_removing_the_marker_is_an_unknown_change_the_gate_fails() {
    for (old, new) in [(ir(false), ir(true)), (ir(true), ir(false))] {
        let delta = classified(&old, &new).unwrap();
        let json = delta.to_canonical_json();
        assert_eq!(ids(&old, &new), [UNCLASSIFIED], "{json}");
        let change = delta
            .changes()
            .iter()
            .find(|change| change.id().to_string() == UNCLASSIFIED)
            .expect("the residual names it");
        let compatibility = delta.compatibility_of(&change.id()).expect("classified");
        let unknown = Compatibility::Unknown;
        assert_eq!(
            (
                compatibility.callers(),
                compatibility.readers(),
                compatibility.history()
            ),
            (unknown, unknown, unknown),
            "{json}"
        );
        let outcome = Gate::new(FailOn::BreakingOrUnknown)
            .judge(&delta, None)
            .unwrap();
        assert!(!outcome.passed(), "{json}");
    }
}

#[test]
fn an_unchanged_marker_is_no_change() {
    for provided in [false, true] {
        let delta = diff(&ir(provided), &ir(provided)).unwrap();
        assert!(delta.changes().is_empty(), "{}", delta.to_canonical_json());
    }
}
