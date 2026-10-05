//! A `distinct` whose list, key or kind moves is a behaviour change, and the diff renders it as the
//! predicate it is (`docs/design/expression-family-source22.md`, `distinct`, final review decision 7).

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(guard: &str) -> String {
    format!(
        r"format: ess/22
system: pool
version: v1
domain: pool.files
types:
  - name: pool.files.File
    kind: struct
    fields:
      - {{name: path, type: String}}
      - {{name: name, type: String}}
      - {{name: seen, type: Timestamp}}
errors:
  - name: pool.files.Refused
    summary: The bundle is refused.
events:
  - name: pool.files.Opened
    fields: []
commands:
  - name: pool.files.Open
    input:
      - {{name: files, type: List<pool.files.File>}}
      - {{name: others, type: List<pool.files.File>}}
    outcomes:
      - name: refused
        when: {guard}
        error: pool.files.Refused
      - name: opened
        emits: [pool.files.Opened]
"
    )
}

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("pool.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

const GUARD: &str = "{not: {distinct: {in: files, as: file, by: file.path}}}";

#[test]
fn diff_distinct_changes_are_behaviour() {
    let before = ir(&model(GUARD));
    for (guard, rendered) in [
        (
            "{not: {distinct: {in: files, as: file, by: file.name}}}",
            "when not (distinct file in files by file.name as string)",
        ),
        (
            "{not: {distinct: {in: files, as: file, by: file.seen}}}",
            "when not (distinct file in files by file.seen as timestamp)",
        ),
        (
            "{not: {distinct: {in: others, as: file, by: file.path}}}",
            "when not (distinct file in others by file.path as string)",
        ),
        (
            "{distinct: {in: files, as: file, by: file.path}}",
            "when distinct file in files by file.path as string",
        ),
    ] {
        let delta = diff(&before, &ir(&model(guard))).unwrap();
        let json = delta.to_canonical_json();
        assert!(
            json.contains("outcome-condition-changed")
                && json.contains(
                    r#""before": "when not (distinct file in files by file.path as string)""#
                )
                && json.contains(&format!(r#""after": "{rendered}""#)),
            "{guard}: {json}"
        );
    }
}

#[test]
fn an_unchanged_distinct_is_no_change() {
    let before = ir(&model(GUARD));
    let json = diff(&before, &ir(&model(GUARD)))
        .unwrap()
        .to_canonical_json();
    assert!(!json.contains("condition-changed"), "{json}");
}
