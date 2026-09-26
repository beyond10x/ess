//! Native list selection over `starts_with`, `ends_with` and `contains` (beyond10x/ess#95): both
//! emitters write the operator, and the Go one writes its literal byte for byte.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

const MODEL: &str = include_str!("fixtures/binding-selection.yaml");

fn compile(guard: &str) -> EssIr {
    let text = MODEL.replace("format: ess/3", "format: ess/8").replace(
        "          where: 'item.id != \"\"'",
        &format!("          where: {guard}"),
    );
    let raw = RawSpecFile::parse(&text).unwrap();
    let spec = Specification::assemble([(Source::new("selection.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", text.as_str());
    compile_locating(&spec, &sources, &["selection.yaml"]).unwrap()
}

fn code(ir: &EssIr, target: Target) -> String {
    synthesize_for(ir, target)
        .unwrap()
        .artifacts
        .values()
        .map(|artifact| artifact.contents.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn each_operator_is_written_by_both_emitters() {
    for (operator, rust, go) in [
        ("starts_with", ".starts_with(\"leg-\")", "\"starts_with\""),
        ("ends_with", ".ends_with(\"leg-\")", "\"ends_with\""),
        ("contains", ".contains(\"leg-\")", "\"contains\""),
    ] {
        let ir = compile(&format!("{{item.id: {{{operator}: \"leg-\"}}}}"));
        let rust_code = code(&ir, Target::Rust);
        assert!(rust_code.contains(rust), "{operator}: Rust lacks {rust}");
        let go_code = code(&ir, Target::Go);
        assert!(
            go_code.contains("selectionText(read")
                && go_code.contains(&format!(", \"leg-\", {go})")),
            "{operator}: Go lacks the selectionText call"
        );
        assert!(go_code.contains("func selectionText("), "{operator}");
        assert!(
            go_code.contains("\t\"strings\"\n"),
            "{operator}: Go imports strings"
        );
    }
}

#[test]
fn a_go_literal_is_written_byte_for_byte_and_never_as_a_rust_escape() {
    let ir = compile("{item.id: {starts_with: \"e\u{301}\\\"\\\\\"}}");
    let go_code = code(&ir, Target::Go);
    assert!(
        go_code.contains(r#""e\xcc\x81\"\\""#),
        "the literal is Go syntax byte for byte"
    );
    assert!(
        !go_code.contains(r"\u{301}"),
        "no Rust escape reaches Go source"
    );
}
