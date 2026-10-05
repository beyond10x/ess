//! Adversary pass 1 against #268 slice 2: the generated Rust `conditions` module answers every
//! condition exactly as the interpreter's reading (`ConditionPlan::evaluate`, Kleene logic) does,
//! over every payload of a small model whose conditions nest `not`, `any` and `all` around
//! Unknown, compare enum and newtype-over-String leaves with `==` and `!=`, cross an Optional
//! struct, and read a root member whose name is a Rust keyword (`ref`).
//!
//! The `conditions` module the generator wrote is lifted verbatim out of the generated system
//! crate and compiled against the generated types crate alone, with a `main` that builds each
//! payload as a typed event and prints each condition's answer. The interpreter reads the same
//! payloads as nodes.

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
    ("k1", "{not: event.ref.tier == gold}"),
    ("k2", "{any: [event.ref.tier == gold, event.memo == x]}"),
    ("k3", "[event.ref.tier != basic, defined(event.memo)]"),
    ("k4", "{not: {any: [event.kind == note, event.ref.id == a]}}"),
    ("k5", "{any: [defined(event.ref.tier), {not: event.memo != x}]}"),
    ("k6", "event.kind != hold"),
    (
        "k7",
        "{all: [{any: [event.ref.id != b, event.kind == ship]}, {not: {all: [defined(event.ref), event.memo == y]}}]}",
    ),
];

fn model() -> String {
    let mut out = String::from(
        "format: ess/22
system: demo
version: v1
domain: demo.probe
summary: Conditions over every payload shape.
types:
  - name: demo.probe.Kind
    kind: enum
    variants: [note, ship, hold]
  - name: demo.probe.Tier
    kind: enum
    variants: [gold, basic]
  - name: demo.probe.Label
    kind: newtype
    of: String
  - name: demo.probe.Ref
    kind: struct
    fields:
      - {name: id, type: demo.probe.Label}
      - {name: tier, type: Optional<demo.probe.Tier>}
events:
  - name: demo.probe.Changed
    fields:
      - {name: kind, type: demo.probe.Kind}
      - {name: ref, type: Optional<demo.probe.Ref>}
      - {name: memo, type: Optional<String>}
  - name: demo.probe.Noted
    fields:
      - {name: memo, type: Optional<String>}
commands:
  - name: demo.probe.Change
    input:
      - {name: kind, type: demo.probe.Kind}
      - {name: ref, type: Optional<demo.probe.Ref>}
      - {name: memo, type: Optional<String>}
    outcomes:
      - name: changed
        emits: [demo.probe.Changed]
        payload:
          demo.probe.Changed: {kind: input.kind, ref: input.ref, memo: input.memo}
  - name: demo.probe.Note
    input:
      - {name: memo, type: Optional<String>}
    outcomes:
      - name: noted
        emits: [demo.probe.Noted]
        payload:
          demo.probe.Noted: {memo: input.memo}
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
      memo: event.memo
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

/// One payload: the kind, the reference (its id and Optional tier) if present, and the memo.
#[derive(Debug, Clone)]
struct Case {
    kind: &'static str,
    reference: Option<(&'static str, Option<&'static str>)>,
    memo: Option<&'static str>,
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for kind in ["note", "ship", "hold"] {
        let mut references = vec![None];
        for id in ["a", "b"] {
            for tier in [None, Some("gold"), Some("basic")] {
                references.push(Some((id, tier)));
            }
        }
        for reference in references {
            for memo in [None, Some("x"), Some("y")] {
                out.push(Case {
                    kind,
                    reference,
                    memo,
                });
            }
        }
    }
    out
}

fn payload(case: &Case) -> BTreeMap<String, Node> {
    let mut out = BTreeMap::from([("kind".to_owned(), Node::Text(case.kind.into()))]);
    if let Some((id, tier)) = case.reference {
        let mut fields = BTreeMap::from([("id".to_owned(), Node::Text(id.into()))]);
        if let Some(tier) = tier {
            fields.insert("tier".into(), Node::Text(tier.into()));
        }
        out.insert("ref".into(), Node::Map(fields));
    }
    if let Some(memo) = case.memo {
        out.insert("memo".into(), Node::Text(memo.into()));
    }
    out
}

fn pascal(word: &str) -> String {
    let mut out = word[..1].to_uppercase();
    out.push_str(&word[1..]);
    out
}

/// The typed event for `case`, as Rust source against the generated types crate.
fn rust_event(case: &Case) -> String {
    let reference = match case.reference {
        None => "None".to_owned(),
        Some((id, tier)) => format!(
            "Some(demo_types::probe::Ref {{ id: demo_types::probe::Label({id:?}.to_owned()), \
             tier: {} }}.into())",
            tier.map_or("None".to_owned(), |tier| format!(
                "Some(demo_types::probe::Tier::{})",
                pascal(tier)
            ))
        ),
    };
    format!(
        "demo_types::probe::Changed {{ kind: demo_types::probe::Kind::{}, r#ref: {reference}, \
         memo: {} }}",
        pascal(case.kind),
        case.memo.map_or("None".to_owned(), |memo| format!(
            "Some({memo:?}.to_owned())"
        ))
    )
}

fn word(truth: Truth) -> &'static str {
    match truth {
        Truth::True => "T",
        Truth::False => "F",
        Truth::Unknown => "U",
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
#[allow(clippy::too_many_lines)]
fn generated_rust_conditions_answer_as_the_interpreter_on_every_payload() {
    let text = model();
    let ir = ir(&text);
    let cases = cases();
    let expected: Vec<String> = cases
        .iter()
        .map(|case| {
            CONDITIONS
                .iter()
                .map(|(name, _)| {
                    let binding = ir
                        .bindings()
                        .values()
                        .find(|binding| binding.name.as_str() == *name)
                        .expect("binding");
                    let plan = &binding.condition.as_ref().expect("condition").plan;
                    word(plan.evaluate(&payload(case)).expect("well-formed payload"))
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .collect();

    let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-kleene-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("demo");
    for (relative, artifact) in &synthesis.artifacts {
        let destination = tree.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
    let lib = std::fs::read_to_string(tree.join("crates/demo-system/src/lib.rs")).unwrap();
    let module = conditions_module(&lib);

    let mut main = format!("#![allow(warnings)]\n{module}\n\nfn main() {{\n");
    for case in &cases {
        let _ = writeln!(main, "    {{\n        let event = {};", rust_event(case));
        main.push_str("        let mut line = String::new();\n");
        for (name, _) in CONDITIONS {
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
        "[package]\nname = \"kleene-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish \
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
    let _ = std::fs::remove_dir_all(&target);
    assert!(
        output.status.success(),
        "the lifted conditions module builds against the generated types: {}\n{}\n----\n{module}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let actual: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(actual.len(), cases.len(), "one line per payload");
    let unknowns = expected
        .iter()
        .map(|line| line.matches('U').count())
        .sum::<usize>();
    println!(
        "{} payloads x {} conditions compared, {unknowns} Unknown answers; first lines: {:?} / {:?}",
        cases.len(),
        CONDITIONS.len(),
        &actual[..3],
        &expected[..3]
    );
    let mut differences = Vec::new();
    for ((case, want), got) in cases.iter().zip(&expected).zip(&actual) {
        if want != got {
            differences.push(format!(
                "{case:?}: interpreter {want}, generated {got} ({})",
                CONDITIONS
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>()
                    .join("")
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        differences,
        Vec::<String>::new(),
        "generated Rust and the interpreter disagree"
    );
}

/// The typed event for `case`, as Go source against the generated types package.
fn go_event(case: &Case) -> String {
    let reference = match case.reference {
        None => "nil".to_owned(),
        Some((id, tier)) => format!(
            "&probe.Ref{{Id: probe.NewLabel({id:?}), Tier: {}}}",
            tier.map_or("nil".to_owned(), |tier| format!(
                "tier(probe.Tier{}{{}})",
                pascal(tier)
            ))
        ),
    };
    format!(
        "probe.Changed{{Kind: probe.Kind{}{{}}, Ref: {reference}, Memo: {}}}",
        pascal(case.kind),
        case.memo
            .map_or("nil".to_owned(), |memo| format!("text({memo:?})"))
    )
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
#[allow(clippy::too_many_lines)]
fn generated_go_conditions_answer_as_the_interpreter_on_every_payload() {
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(found, "this probe needs `go`");
    let text = model();
    let ir = ir(&text);
    let cases = cases();
    let synthesis = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-kleene-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("go-demo");
    for (relative, artifact) in &synthesis.artifacts {
        let destination = tree.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
    let system = std::fs::read_to_string(tree.join("system/system.go")).unwrap();
    let (functions, names) = go_conditions(&system);
    assert_eq!(names.len(), CONDITIONS.len(), "{names:?}\n{functions}");
    let mut main = format!(
        "package main\n\nimport (\n\t\"fmt\"\n\n\t\"example.invalid/demo/types/probe\"\n)\n\nfunc \
         tier(value probe.Tier) *probe.Tier {{ return &value }}\n\nfunc text(value string) \
         *string {{ return &value }}\n\nfunc word(answer int8) string {{\n\tswitch answer {{\n\tcase \
         1:\n\t\treturn \"T\"\n\tcase 0:\n\t\treturn \"F\"\n\tcase -1:\n\t\treturn \"U\"\n\t}}\n\treturn \
         \"?\"\n}}\n\n{functions}\nfunc main() {{\n"
    );
    for case in &cases {
        let _ = writeln!(main, "\t{{\n\t\tevent := {}", go_event(case));
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
        "module kleeneharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
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
    let types = std::fs::read_dir(tree.join("types/probe"))
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        output.status.success(),
        "the lifted Go condition functions build against the generated types: {}\n{}\n----\n{functions}\n----\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout),
        types.lines().filter(|line| line.starts_with("type ") || line.starts_with("func New") || line.starts_with('\t')).take(80).collect::<Vec<_>>().join("\n")
    );
    // The functions are in layout order, not the table's: answer per binding name.
    let order: Vec<usize> = names
        .iter()
        .map(|name| {
            CONDITIONS
                .iter()
                .position(|(binding, _)| name.to_lowercase().starts_with(binding))
                .unwrap_or_else(|| panic!("{name} names no binding"))
        })
        .collect();
    let mut differences = Vec::new();
    let mut unknowns = 0;
    for (case, line) in cases
        .iter()
        .zip(String::from_utf8_lossy(&output.stdout).lines())
    {
        for (column, answer) in line.chars().enumerate() {
            let (binding, _) = CONDITIONS[order[column]];
            let plan = &ir
                .bindings()
                .values()
                .find(|candidate| candidate.name.as_str() == binding)
                .expect("binding")
                .condition
                .as_ref()
                .expect("condition")
                .plan;
            let want = word(plan.evaluate(&payload(case)).expect("well-formed payload"));
            unknowns += usize::from(want == "U");
            if want != answer.to_string() {
                differences.push(format!(
                    "{case:?} {binding}: interpreter {want}, generated Go {answer}"
                ));
            }
        }
    }
    println!(
        "{} payloads x {} conditions compared in Go, {unknowns} Unknown answers",
        cases.len(),
        names.len()
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).lines().count(),
        cases.len()
    );
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        differences,
        Vec::<String>::new(),
        "generated Go and the interpreter disagree"
    );
}
