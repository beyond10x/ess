//! Values read through input paths in the published projections (source `ess/22`, Family F A4,
//! `docs/design/expression-family-source22.md`): the generated documentation names each path and
//! the fallback where a branch sets a field, while the command's request and the event's message
//! stay driven by declarations — a path adds no request field and no payload field.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/input-value-paths.yaml");

const COMPONENTS: &str = "
components:
  - component: lease-service
    owns: {domains: [leases.pool]}
    accepts: {commands: [leases.pool.Open]}
    publishes: {events: [leases.pool.Opened]}
";

fn ir() -> EssIr {
    let text = format!("{MODEL}{COMPONENTS}");
    let spec = Specification::assemble([(
        Source::new("leases.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifact(ir: &EssIr, part: &str) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| path.contains(part))
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a4_the_documentation_names_each_path_and_the_fallback() {
    let docs = artifact(&ir(), ".md");
    for said in [
        "`generation_id` from `input.opening.generation_id`",
        "`previous_generation` from `input.previous.generation_id`",
        "`label` from `input.previous.label, else input.settings.defaults.label`",
        "`sealed_label` from `input.sealed.label`",
    ] {
        assert!(docs.contains(said), "`{said}` missing:\n{docs}");
    }
}

#[test]
fn a4_a_path_adds_no_request_or_payload_field() {
    let ir = ir();
    for part in ["openapi", "asyncapi"] {
        let projected = artifact(&ir, part);
        assert_ne!(projected.len(), 0, "the {part} projection is generated");
        for path in [
            "opening.generation_id",
            "previous.generation_id",
            "previous.label",
            "settings.defaults.label",
            "sealed.label",
        ] {
            assert!(
                !projected.contains(path),
                "{part}: `{path}` is no wire field:\n{projected}"
            );
        }
    }
}
