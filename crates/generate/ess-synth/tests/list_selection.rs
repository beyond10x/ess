//! Native list selection is emitted as typed executable loops rather than a mapping obligation.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
const MODEL: &str = include_str!("fixtures/binding-selection.yaml");
fn compile(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("selection.yaml"), raw)]).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", text);
    compile_locating(&spec, &sources, &["selection.yaml"]).unwrap()
}
#[test]
fn direct_declared_lists_emit_native_selection_and_typed_preflight() {
    let ir = compile(MODEL);
    for target in [Target::Rust, Target::Go] {
        let output = synthesize_for(&ir, target).unwrap();
        let code = output
            .artifacts
            .values()
            .map(|artifact| artifact.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            code.contains("selected[0]"),
            "first-occurrence selection must execute"
        );
        assert!(
            code.contains("SelectionFailure"),
            "failure must precede invocation"
        );
        assert!(
            code.contains("selected[1]"),
            "exclusion and fallback retain prior occurrence indices"
        );
        if let Some(directory) = std::env::var_os("ESS_SELECTION_NATIVE_OUT") {
            let base = std::path::PathBuf::from(directory).join(target.name());
            for (relative, artifact) in output.artifacts {
                let path = base.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, artifact.contents).unwrap();
            }
        }
    }
}
#[test]
fn undeclared_preparation_never_becomes_an_invented_event_member() {
    let text = MODEL.replace("from: event.data", "from: event.legs");
    let raw = RawSpecFile::parse(&text).unwrap();
    assert!(Specification::assemble([(Source::new("selection.yaml"), raw)]).is_err());
}

#[test]
fn declared_host_preparation_retains_an_executable_typed_selection_helper() {
    let model = MODEL.replacen("type: List<Optional<selection.core.Leg>>", "type: String", 1)
        + "\nconversions:\n  - from: String\n    to: List<Optional<selection.core.Leg>>\n    because: host decodes an ordered occurrence collection\n";
    let ir = compile(&model);
    for target in [Target::Rust, Target::Go] {
        let output = synthesize_for(&ir, target).unwrap();
        let code = output
            .artifacts
            .values()
            .map(|artifact| artifact.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(code.contains(if target == Target::Rust {
            "_from_prepared("
        } else {
            "FromPrepared("
        }));
        assert!(code.contains("selected[0]"));
        assert!(code.contains("prepared_0"));
        assert!(serde_json::to_string(&output.plan)
            .unwrap()
            .contains("selection-input-conversion"));
        if let Some(directory) = std::env::var_os("ESS_SELECTION_NATIVE_OUT") {
            let base = std::path::PathBuf::from(directory)
                .join("prepared")
                .join(target.name());
            for (relative, artifact) in output.artifacts {
                let path = base.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, artifact.contents).unwrap();
            }
        }
    }
}

#[test]
fn direct_and_prepared_helpers_refuse_reachable_invariants_before_emission() {
    let constrained = MODEL.replace(
        "events:\n",
        "    invariants: ['from != \"blocked\"']\nevents:\n",
    );
    let prepared = constrained.replacen("type: List<Optional<selection.core.Leg>>", "type: String", 1)
        + "\nconversions:\n  - from: String\n    to: List<Optional<selection.core.Leg>>\n    because: host decodes ordered records\n";
    let nested = MODEL.replace("      - name: from\n        type: String", "      - name: from\n        type: selection.core.Label")
        .replace("events:\n", "  - name: selection.core.Label\n    kind: newtype\n    of: String\n    invariants: ['value != \"blocked\"']\nevents:\n");
    for model in [constrained, prepared, nested] {
        let ir = compile(&model);
        for target in [Target::Rust, Target::Go] {
            let failure = synthesize_for(&ir, target)
                .err()
                .expect("unsupported constraint must refuse");
            let json = serde_json::to_string(&failure).unwrap();
            assert!(json.contains("selection-constraint"), "{json}");
            assert!(json.contains("ess-target-failure/3"), "{json}");
        }
    }
}

#[test]
fn selection_support_and_prepared_names_cannot_collide_with_binding_functions() {
    let rust = compile(&MODEL.replace("id: choose", "id: selection-all"));
    let failure = synthesize_for(&rust, Target::Rust)
        .err()
        .expect("symbol must refuse");
    assert!(serde_json::to_string(&failure)
        .unwrap()
        .contains("symbol-collision"));
    let go = compile(&MODEL.replace("id: choose", "id: selection-failure"));
    let output = synthesize_for(&go, Target::Go).unwrap();
    let code = output
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(code.contains("func SelectionFailure_("));
}
