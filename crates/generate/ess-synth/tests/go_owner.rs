//! Valid system-level types receive a checked Go prerequisite refusal before layout allocation.
use ess_compiler::{source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_synth::{SynthesisPlan, Target, TargetFailure, TargetFailureCode};

fn compile(system: &str, core: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let parsed = [("system.yaml", system), ("core.yaml", core)]
        .into_iter()
        .map(|(label, text)| {
            sources.insert(label.to_owned(), text.to_owned());
            (
                Source::new(label),
                RawSpecFile::parse(text).expect("fixture parses"),
            )
        })
        .collect::<Vec<_>>();
    let spec = Specification::assemble(parsed).expect("fixture is valid authored ESS");
    ess_compiler::compile(&spec, &sources).expect("fixture resolves")
}
fn body(family: &str) -> &'static str {
    match family {
        "newtype" => "    kind: newtype\n    of: String\n",
        "struct" => "    kind: struct\n    fields:\n      - {name: value, type: String}\n",
        "enum" => "    kind: enum\n    variants: [One, Two]\n",
        "union" => "    kind: union\n    tag: kind\n    variants: {text: String, count: Integer}\n",
        _ => panic!("test family"),
    }
}
fn missing(family: &str, referenced: bool) -> EssIr {
    compile(&format!("format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\ntypes:\n  - name: demo.Value\n{}",body(family)),
        if referenced {"domain: demo.core\ntypes:\n  - name: demo.core.Holder\n    kind: struct\n    fields:\n      - {name: value, type: demo.Value}\n"}else{"domain: demo.core\n"})
}
fn check(failure: &TargetFailure, plan: &SynthesisPlan, names: &[&str]) {
    assert_eq!(failure.target(), "go");
    assert_eq!(failure.plan(), plan);
    let json: serde_json::Value = serde_json::from_str(&failure.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-target-failure/2");
    assert_eq!(failure.causes().len(), names.len());
    for (cause, name) in failure.causes().iter().zip(names) {
        assert_eq!(cause.code(), TargetFailureCode::MissingTypeOwner);
        assert_eq!(cause.sources(), [*name]);
        assert_eq!(
            cause.detail(),
            format!("Go cannot assign a package to type {name}: no domain owns it")
        );
    }
}
fn family(family: &str, referenced: bool) {
    let ir = missing(family, referenced);
    let plan = SynthesisPlan::of(&ir);
    let facade = ess_synth::synthesize_for(&ir, Target::Go)
        .err()
        .expect("whole Go workspace refuses");
    let direct = ess_synth::go::workspace(&ir, &plan)
        .err()
        .expect("direct Go workspace refuses");
    check(&facade, &plan, &["demo.Value"]);
    check(&direct, &plan, &["demo.Value"]);
    assert_eq!(facade, direct);
}
#[test]
fn unreferenced_newtype() {
    family("newtype", false);
}
#[test]
fn referenced_newtype() {
    family("newtype", true);
}
#[test]
fn unreferenced_struct() {
    family("struct", false);
}
#[test]
fn referenced_struct() {
    family("struct", true);
}
#[test]
fn unreferenced_enum() {
    family("enum", false);
}
#[test]
fn referenced_enum() {
    family("enum", true);
}
#[test]
fn unreferenced_union() {
    family("union", false);
}
#[test]
fn referenced_union() {
    family("union", true);
}
#[test]
fn direct_emitter_checks_before_layout() {
    let ir = missing("newtype", false);
    let plan = SynthesisPlan::of(&ir);
    let failure = ess_synth::go::workspace(&ir, &plan)
        .err()
        .expect("direct refusal");
    check(&failure, &plan, &["demo.Value"]);
}
#[test]
fn all_missing_names_are_sorted_once_independent_of_source_order() {
    let a = "  - {name: demo.Zed, kind: enum, variants: [A, B]}\n";
    let b = "  - {name: demo.Alpha, kind: newtype, of: String}\n";
    let mut failures = Vec::new();
    for declarations in [format!("{a}{b}"), format!("{b}{a}")] {
        let ir=compile(&format!("format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\ntypes:\n{declarations}"),"domain: demo.core\n");
        let plan = SynthesisPlan::of(&ir);
        let failure = ess_synth::synthesize_for(&ir, Target::Go)
            .err()
            .expect("refusal");
        check(&failure, &plan, &["demo.Alpha", "demo.Zed"]);
        assert_eq!(failure, ess_synth::go::workspace(&ir, &plan).err().unwrap());
        failures.push(failure);
    }
    assert_eq!(failures[0], failures[1]);
}
#[test]
fn ownerless_binary64_keeps_the_earlier_refusal() {
    let ir=compile("format: ess/2\nsystem: demo\nversion: v1\ndomains: [demo.core]\ntypes:\n  - {name: demo.Ratio, kind: newtype, of: Binary64}\n","domain: demo.core\n");
    let plan = SynthesisPlan::of(&ir);
    let facade = ess_synth::synthesize_for(&ir, Target::Go)
        .err()
        .expect("codec refusal");
    let direct = ess_synth::go::workspace(&ir, &plan)
        .err()
        .expect("codec refusal");
    assert_eq!(facade, direct);
    assert_eq!(facade.plan(), &plan);
    assert_eq!(facade.causes().len(), 1);
    assert_eq!(
        facade.causes()[0].code(),
        TargetFailureCode::MissingRepresentation
    );
    assert_eq!(
        facade.causes()[0].detail(),
        "this target has no qualified finite Binary64 codec"
    );
}
