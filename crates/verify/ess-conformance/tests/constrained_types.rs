//! An alphabet-only newtype is refused wherever an invariant-carrying one is: the three
//! conformance observers that cannot execute a declared constraint (selection, response, retained
//! replay) ask `ResolvedBody::is_constrained`, not "has invariants".
//!
//! `docs/design/string-alphabet-and-length.md`, section 6, "constraint gates". One case per site,
//! each run twice — with an invariant and with only an alphabet — asserting the same refusal.

use ess_compiler::{
    ir::{EssIr, ResolvedMappingValue},
    resolve::compile,
    source::SourceMap,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model carries {from:?}");
    model.replacen(from, to, 1)
}

/// The two ways a `String` newtype is constrained, as the lines that declare it.
const CONSTRAINTS: [(&str, &str); 2] = [
    ("an invariant", "    invariants: ['value != \"\"']\n"),
    ("an alphabet", "    alphabet: \"abc\"\n"),
];

#[test]
fn a_selection_over_an_item_with_a_constrained_text_is_refused_either_way() {
    for (how, constraint) in CONSTRAINTS {
        let model = with(
            &with(
                &with(
                    include_str!("fixtures/binding-selection.yaml"),
                    "format: ess/3",
                    "format: ess/11",
                ),
                "types:\n",
                &format!("types:\n  - name: selection.core.From\n    kind: newtype\n    of: String\n{constraint}"),
            ),
            "      - name: from\n        type: String",
            "      - name: from\n        type: selection.core.From",
        );
        let ir = ir(&model);
        let binding = ir.bindings().values().next().unwrap();
        let mapped = &binding.mapping[0];
        let ResolvedMappingValue::Selection {
            selector,
            projection,
            ..
        } = &mapped.value
        else {
            panic!("the fixture maps a selection")
        };
        let refusal = ess_conformance::selection::Observation::of(
            &ir,
            binding,
            *selector,
            projection,
            &mapped.target_type,
        )
        .expect_err(how);
        assert!(refusal.contains("selection-constraint"), "{how}: {refusal}");
    }
}

#[test]
fn a_response_carrying_a_constrained_text_is_refused_either_way() {
    for (how, constraint) in CONSTRAINTS {
        let model = with(
            &with(
                &with(
                    include_str!("fixtures/response-payload.yaml"),
                    "format: ess/4",
                    "format: ess/11",
                ),
                "types:\n",
                &format!("types:\n  - name: demo.api.CallType\n    kind: newtype\n    of: String\n{constraint}"),
            ),
            "{name: call_type, type: String}",
            "{name: call_type, type: demo.api.CallType}",
        );
        let ir = ir(&model);
        let command = &ir.commands()[&"demo.api.Cancel".parse().unwrap()];
        let refusal =
            ess_conformance::response::Observation::of(&ir, command, &command.outcomes[0])
                .expect_err(how);
        assert!(
            refusal.contains("response constrained type"),
            "{how}: {refusal}"
        );
    }
}

#[test]
fn a_retained_response_carrying_a_constrained_text_is_refused_either_way() {
    for (how, constraint) in CONSTRAINTS {
        let model = with(
            &with(
                &with(
                    include_str!("fixtures/retained-replay.yaml"),
                    "format: ess/7",
                    "format: ess/11",
                ),
                "domain: retained.core\n",
                &format!("domain: retained.core\ntypes:\n  - name: retained.core.Note\n    kind: newtype\n    of: String\n{constraint}"),
            ),
            "{name: optional, type: Optional<String>}",
            "{name: optional, type: Optional<retained.core.Note>}",
        );
        let ir = ir(&model);
        let command = &ir.commands()[&"retained.core.Seed".parse().unwrap()];
        let replayed = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == "replayed")
            .unwrap();
        let refusal = ess_conformance::replay::Observation::of(
            &ir,
            command,
            replayed,
            ess_conformance::InstanceName::new("subject").unwrap(),
        )
        .expect_err(how);
        assert!(
            refusal.contains("replay response invariant/reading observer is unsupported"),
            "{how}: {refusal}"
        );
    }
}
