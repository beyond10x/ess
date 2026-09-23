//! Control coverage that excludes an unused conversion beside a selected preparation conversion.

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
    let mut paths = std::fs::read_dir(base)
        .expect("the fixture directory is readable")
        .map(|entry| entry.expect("a fixture entry").path())
        .collect::<Vec<_>>();
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

fn capability(source: &str) -> Capability {
    Capability {
        kind: CapabilityKind::Conversion,
        source: source.to_owned(),
    }
}

#[test]
fn selected_preparation_conversion_excludes_an_unused_competing_conversion() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/selection-conversion-control-fixtures");
    let ir = compile_directory(&path);
    let plan = SynthesisPlan::of(&ir);
    let used = capability("String -> List<selection.control.Leg>");
    let unused = capability("String -> List<selection.control.UnusedLeg>");
    assert!(plan
        .capabilities
        .iter()
        .any(|entry| entry.capability == used));
    assert!(plan
        .capabilities
        .iter()
        .any(|entry| entry.capability == unused));

    let selected = extract(
        &ir,
        &plan,
        &ComponentName::new("selection-control-service").expect("valid component name"),
    )
    .expect("the exact plan and component are admitted");
    let conversions = selected
        .capabilities()
        .filter(|entry| entry.capability.kind == CapabilityKind::Conversion)
        .map(|entry| entry.capability.clone())
        .collect::<Vec<_>>();
    assert_eq!(conversions, [used]);
    assert!(!conversions.contains(&unused));
}
