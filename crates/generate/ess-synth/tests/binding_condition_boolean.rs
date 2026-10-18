//! A binding condition comparing a Boolean leaf with `true` or `false` (ess/27, downstream report,
//! 2026-10-18) in the generated Rust and Go dispatch: each generated condition answers every
//! payload exactly as the interpreter's reading (`ConditionPlan::evaluate`) does, over a required
//! flag, an Optional flag and an enum beside them, under `not` and `all`.
//!
//! The generated condition functions are lifted out of the generated system and compiled against
//! the generated types alone, as `adversary_binding_condition_kleene.rs` does for text leaves.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::predicate::Truth;
use ess_synth::{synthesize_for, Target};

/// Each binding's condition, by binding name.
const CONDITIONS: &[(&str, &str)] = &[
    ("b1", "event.flag == true"),
    ("b2", "event.flag != true"),
    ("b3", "[event.kind == ship, event.held == false]"),
    ("b4", "{not: event.held != false}"),
    ("b5", "{any: [event.held == true, event.flag == false]}"),
];

fn model() -> String {
    let mut out = String::from(
        "format: ess/27
system: demo
version: v1
domain: demo.probe
summary: Boolean conditions over every payload shape.
types:
  - name: demo.probe.Kind
    kind: enum
    variants: [note, ship]
events:
  - name: demo.probe.Changed
    fields:
      - {name: kind, type: demo.probe.Kind}
      - {name: flag, type: Boolean}
      - {name: held, type: Optional<Boolean>}
  - name: demo.probe.Noted
    fields:
      - {name: kind, type: demo.probe.Kind}
commands:
  - name: demo.probe.Change
    input:
      - {name: kind, type: demo.probe.Kind}
      - {name: flag, type: Boolean}
      - {name: held, type: Optional<Boolean>}
    outcomes:
      - name: changed
        emits: [demo.probe.Changed]
        payload:
          demo.probe.Changed: {kind: input.kind, flag: input.flag, held: input.held}
  - name: demo.probe.Note
    input:
      - {name: kind, type: demo.probe.Kind}
    outcomes:
      - name: noted
        emits: [demo.probe.Noted]
        payload:
          demo.probe.Noted: {kind: input.kind}
bindings:
",
    );
    for (name, condition) in CONDITIONS {
        let _ = write!(
            out,
            "  - id: {name}
    when:
      event: demo.probe.Changed
      where: {condition}
    invoke: {{command: demo.probe.Note}}
    mapping:
      kind: event.kind
    delivery: at_least_once
    on_failure: drop
"
        );
    }
    out.push_str(
        "components:
  - component: probe-service
    summary: Changes and notes.
    owns:
      domains: [demo.probe]
    accepts:
      commands: [demo.probe.Change, demo.probe.Note]
    publishes:
      events: [demo.probe.Changed, demo.probe.Noted]
    reached_by: network
",
    );
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("probe.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[derive(Debug, Clone, Copy)]
struct Case {
    kind: &'static str,
    flag: bool,
    held: Option<bool>,
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for kind in ["note", "ship"] {
        for flag in [false, true] {
            for held in [None, Some(false), Some(true)] {
                out.push(Case { kind, flag, held });
            }
        }
    }
    out
}

fn payload(case: Case) -> BTreeMap<String, Node> {
    let mut out = BTreeMap::from([
        ("kind".to_owned(), Node::Text(case.kind.into())),
        ("flag".to_owned(), Node::Bool(case.flag)),
    ]);
    if let Some(held) = case.held {
        out.insert("held".into(), Node::Bool(held));
    }
    out
}

fn pascal(word: &str) -> String {
    let mut out = word[..1].to_uppercase();
    out.push_str(&word[1..]);
    out
}

fn word(truth: Truth) -> &'static str {
    match truth {
        Truth::True => "T",
        Truth::False => "F",
        Truth::Unknown => "U",
    }
}

/// What the interpreter answers, one line per case, one letter per condition in table order.
fn expected(ir: &EssIr, cases: &[Case], order: &[&str]) -> Vec<String> {
    cases
        .iter()
        .map(|case| {
            order
                .iter()
                .map(|name| {
                    let binding = ir
                        .bindings()
                        .values()
                        .find(|binding| binding.name.as_str() == *name)
                        .expect("binding");
                    let plan = &binding.condition.as_ref().expect("condition").plan;
                    word(plan.evaluate(&payload(*case)).expect("well-formed payload"))
                })
                .collect::<String>()
        })
        .collect()
}

fn written(root: &Path, synthesis: &ess_synth::Synthesis) {
    for (relative, artifact) in &synthesis.artifacts {
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
}

/// The `pub mod conditions { … }` block of the generated system crate, verbatim.
fn conditions_module(lib: &str) -> String {
    let start = lib
        .find("pub mod conditions {")
        .unwrap_or_else(|| panic!("no conditions module in the generated system:\n{lib}"));
    let mut depth = 0usize;
    for (offset, character) in lib[start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return lib[start..=start + offset].to_owned();
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced conditions module")
}

#[test]
fn the_ir_reads_a_boolean_leaf_as_boolean_from_ess_27_only() {
    let at = |text: &str| {
        let ir = ir(text);
        let binding = ir.bindings().values().next().expect("a binding");
        serde_json::to_value(&binding.condition.as_ref().expect("condition").plan.reads)
            .expect("serialises")["event.flag"]["leaf"]["kind"]
            .clone()
    };
    // One binding that only tests the flag's presence, which every format admits.
    let full = model();
    let defined = format!(
        "{}bindings:
  - id: present
    when:
      event: demo.probe.Changed
      where: defined(event.flag)
    invoke: {{command: demo.probe.Note}}
    mapping:
      kind: event.kind
    delivery: at_least_once
    on_failure: drop
",
        &full[..full.find("bindings:\n").expect("a bindings section")]
    );
    assert_eq!(at(&defined), serde_json::json!("boolean"));
    assert_eq!(
        at(&defined.replace("format: ess/27", "format: ess/26")),
        serde_json::json!("other"),
        "below ess/27 a Boolean read by `defined` keeps its bytes"
    );
}

#[test]
fn generated_rust_boolean_conditions_answer_as_the_interpreter() {
    let ir = ir(&model());
    let cases = cases();
    let order: Vec<&str> = CONDITIONS.iter().map(|(name, _)| *name).collect();
    let expected = expected(&ir, &cases, &order);

    let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("boolean-condition-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("demo");
    written(&tree, &synthesis);
    let lib = std::fs::read_to_string(tree.join("crates/demo-system/src/lib.rs")).unwrap();
    let module = conditions_module(&lib);

    let mut main = format!("#![allow(warnings)]\n{module}\n\nfn main() {{\n");
    for case in &cases {
        let held = case
            .held
            .map_or("None".to_owned(), |held| format!("Some({held})"));
        let _ = writeln!(
            main,
            "    {{\n        let event = demo_types::probe::Changed {{ kind: \
             demo_types::probe::Kind::{}, flag: {}, held: {held} }};",
            pascal(case.kind),
            case.flag
        );
        main.push_str("        let mut line = String::new();\n");
        for name in &order {
            let _ = writeln!(
                main,
                "        line.push_str(match conditions::{name}(&event) {{ Some(true) => \"T\", \
                 Some(false) => \"F\", None => \"U\" }});"
            );
        }
        main.push_str("        println!(\"{line}\");\n    }\n");
    }
    main.push_str("}\n");
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(harness.join("src/main.rs"), &main).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"boolean-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish \
         = false\n\n[workspace]\n\n[dependencies]\ndemo-types = { path = \
         \"../demo/crates/demo-types\" }\n",
    )
    .unwrap();
    let target = root.join("target");
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(["run", "--offline", "--quiet", "--target-dir"])
        .arg(&target)
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", "")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        output.status.success(),
        "the lifted conditions module builds against the generated types: {}\n----\n{module}",
        String::from_utf8_lossy(&output.stderr),
    );
    let actual: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(actual, expected, "generated Rust and the interpreter agree");
}

/// Every `func …Condition(event …) int8 { … }` the generated Go system package holds, verbatim.
fn go_conditions(system: &str) -> (String, Vec<String>) {
    let mut out = String::new();
    let mut names = Vec::new();
    let mut rest = system;
    while let Some(at) = rest.find("\nfunc ") {
        let tail = &rest[at + 1..];
        let end = tail.find("\n}\n").map_or(tail.len(), |end| end + 3);
        let function = &tail[..end];
        let head = function.lines().next().unwrap_or_default();
        if head.contains("Condition(event ") && head.ends_with(") int8 {") {
            names.push(head["func ".len()..head.find('(').unwrap()].to_owned());
            out.push_str(function);
            out.push('\n');
        }
        rest = &tail[end.min(tail.len())..];
    }
    (out, names)
}

#[test]
fn generated_go_boolean_conditions_answer_as_the_interpreter() {
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(found, "this probe needs `go`");
    let ir = ir(&model());
    let cases = cases();
    let synthesis = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("boolean-condition-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("go-demo");
    written(&tree, &synthesis);
    let system = std::fs::read_to_string(tree.join("system/system.go")).unwrap();
    let (functions, names) = go_conditions(&system);
    assert_eq!(names.len(), CONDITIONS.len(), "{names:?}\n{functions}");
    // The functions are in layout order: each is named after its binding.
    let order: Vec<&str> = names
        .iter()
        .map(|function| {
            CONDITIONS
                .iter()
                .map(|(name, _)| *name)
                .find(|name| function.to_lowercase().contains(name))
                .unwrap_or_else(|| panic!("`{function}` names a binding"))
        })
        .collect();
    let expected = expected(&ir, &cases, &order);
    let mut main = format!(
        "package main\n\nimport (\n\t\"fmt\"\n\n\t\"example.invalid/demo/types/probe\"\n)\n\nfunc \
         flag(value bool) *bool {{ return &value }}\n\nfunc word(answer int8) string {{\n\tswitch \
         answer {{\n\tcase 1:\n\t\treturn \"T\"\n\tcase 0:\n\t\treturn \"F\"\n\tcase -1:\n\t\treturn \
         \"U\"\n\t}}\n\treturn \"?\"\n}}\n\n{functions}\nfunc main() {{\n"
    );
    for case in &cases {
        let held = case
            .held
            .map_or("nil".to_owned(), |held| format!("flag({held})"));
        let _ = writeln!(
            main,
            "\t{{\n\t\tevent := probe.Changed{{Kind: probe.Kind{}{{}}, Flag: {}, Held: {held}}}",
            pascal(case.kind),
            case.flag
        );
        let calls = names
            .iter()
            .map(|name| format!("word({name}(event))"))
            .collect::<Vec<_>>()
            .join(" + ");
        let _ = writeln!(main, "\t\tfmt.Println({calls})\n\t}}");
    }
    main.push_str("}\n");
    let harness = root.join("go-harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(harness.join("main.go"), &main).unwrap();
    std::fs::write(
        harness.join("go.mod"),
        "module booleanharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../go-demo\n",
    )
    .unwrap();
    let output = Command::new("go")
        .args(["run", "."])
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        output.status.success(),
        "the lifted Go condition functions build against the generated types: {}\n----\n{functions}",
        String::from_utf8_lossy(&output.stderr),
    );
    let actual: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(actual, expected, "generated Go and the interpreter agree");
}
