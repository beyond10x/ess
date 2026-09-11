//! Native outcomes retain actual typed responses and executable response-payload comparisons.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/response-payload.yaml");
fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("response.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
#[test]
fn native_response_results_have_executable_typed_comparisons() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let result = synthesize_for(&ir, target).unwrap();
        let text = result
            .artifacts
            .values()
            .map(|a| a.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("CancelResponse"));
        assert!(
            text.contains("response_payload_matches") || text.contains("ResponsePayloadMatches")
        );
        if let Some(root) = std::env::var_os("ESS_RESPONSE_NATIVE_OUT") {
            let root = std::path::PathBuf::from(root).join(target.name());
            for (relative, artifact) in result.artifacts {
                let path = root.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut contents = artifact.contents;
                if target == Target::Rust && contents.contains("pub struct CancelResponse") {
                    contents.push_str(include_str!("fixtures/response-native.rs"));
                }
                if target == Target::Go && contents.contains("type CancelResponse struct") {
                    let package = contents
                        .lines()
                        .find(|l| l.starts_with("package "))
                        .unwrap();
                    std::fs::write(
                        path.with_file_name("response_native_test.go"),
                        format!("{package}\n{}", include_str!("fixtures/response-native.go")),
                    )
                    .unwrap();
                }
                std::fs::write(path, contents).unwrap();
            }
        }
    }
}
#[test]
fn response_type_names_participate_in_native_collision_admission() {
    let collision = MODEL.replace(
        "types:\n",
        "types:\n  - name: demo.api.CancelResponse\n    kind: newtype\n    of: String\n",
    );
    let model = ir(&collision);
    assert!(synthesize_for(&model, Target::Rust).is_err());
    let go = synthesize_for(&model, Target::Go).unwrap();
    let text = go
        .artifacts
        .values()
        .map(|a| a.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(text.matches("type CancelResponse struct").count(), 1);
    assert!(text.contains("ResponsePayloadMatches"));
}
