//! Adversarial coverage for conversions used while preparing a selected binding's inputs.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_service_contract::extract;
use ess_synth::{Capability, CapabilityKind, SynthesisPlan};

fn compile_directory(base: &Path) -> EssIr {
    let mut pending = vec![base.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the fixture directory is readable") {
            let path = entry.expect("a fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();

    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path
            .strip_prefix(base)
            .expect("the fixture is below its base")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("the fixture is readable");
        let raw = RawSpecFile::parse(&text).expect("the fixture is well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label.clone()), raw));
        labels.push(label);
    }

    let specification = Specification::assemble(parsed).expect("the fixture validates");
    compile_locating(&specification, &sources, &labels).expect("the fixture resolves")
}

fn fixture() -> EssIr {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/selection-conversion-fixtures");
    compile_directory(&path)
}

#[test]
fn selected_binding_retains_its_selection_input_preparation_conversion() {
    let ir = fixture();
    let plan = SynthesisPlan::of(&ir);
    let expected = Capability {
        kind: CapabilityKind::Conversion,
        source: "String -> List<selection.core.Leg>".to_owned(),
    };
    let planned = plan
        .capabilities
        .iter()
        .find(|entry| entry.capability == expected)
        .expect("the fixture plan contains the declared preparation conversion");
    let binding = ir
        .bindings()
        .values()
        .find(|binding| binding.name.to_string() == "choose")
        .expect("the selected operation is reached by the fixture binding");
    assert_eq!(
        binding
            .selection
            .as_ref()
            .expect("the binding retains its selection plan")
            .plan
            .inputs[0]
            .conversion
            .as_deref(),
        Some("the host decodes the ordered legs")
    );

    let selected = extract(
        &ir,
        &plan,
        &ComponentName::new("selection-service").expect("valid component name"),
    )
    .expect("the exact plan and component are admitted");
    assert!(selected.capabilities().any(|entry| {
        entry.capability.kind == CapabilityKind::BindingTransformation
            && entry.capability.source == "choose"
    }));
    assert_eq!(
        selected
            .capabilities()
            .find(|entry| entry.capability == expected),
        Some(planned),
        "a conversion used by an included binding must retain its original plan disposition"
    );
}
