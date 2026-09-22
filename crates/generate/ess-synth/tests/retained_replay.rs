//! Both native languages carry actual original and replay responses without event mappings.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/retained-replay.yaml");

#[test]
fn native_origin_and_replay_keep_the_complete_actual_response() {
    let spec = Specification::assemble([(
        Source::new("replay.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    for target in [Target::Rust, Target::Go] {
        let result = synthesize_for(&ir, target).unwrap();
        let text = result
            .artifacts
            .values()
            .map(|a| a.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        match target {
            Target::Rust => {
                assert!(
                    text.contains("Replayed {\n"),
                    "replay must carry response, not a unit variant"
                );
                assert_eq!(text.matches("response: SeedResponse").count(), 2);
            }
            Target::Go => assert_eq!(text.matches("Response SeedResponse").count(), 2),
            _ => unreachable!(),
        }
        assert!(
            !text.contains("response_payload_matches") && !text.contains("ResponsePayloadMatches"),
            "no invented event mapping"
        );
        if let Some(root) = std::env::var_os("ESS_REPLAY_NATIVE_OUT") {
            let root = std::path::PathBuf::from(root).join(target.name());
            for (relative, artifact) in result.artifacts {
                let path = root.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut contents = artifact.contents;
                if target == Target::Rust && contents.contains("pub struct SeedResponse") {
                    contents.push_str(include_str!("fixtures/retained-replay-native.rs"));
                }
                if target == Target::Go && contents.contains("type SeedResponse struct") {
                    let package = contents
                        .lines()
                        .find(|l| l.starts_with("package "))
                        .unwrap();
                    std::fs::write(
                        path.with_file_name("retained_replay_native_test.go"),
                        format!(
                            "{package}\n{}",
                            include_str!("fixtures/retained-replay-native.go")
                        ),
                    )
                    .unwrap();
                }
                std::fs::write(path, contents).unwrap();
            }
        }
    }
}
