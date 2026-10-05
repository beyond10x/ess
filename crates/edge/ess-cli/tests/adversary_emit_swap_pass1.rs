//! Adversary, pass 1, on `emit-swap` at the CLI (beyond10x/ess#295).
//!
//! The design's "Implementation decisions for #295": in an emission scoped to a component, an
//! unavailable site whose command that component does not handle is `outside_component` and does
//! not affect the exit status. The unit pins that reason in the library and never at the exit
//! status. Here the CLI emits for `owner`, a real runner (the native runner over the model
//! interpreter) answers every emitted suite, and `--collect --component owner` must exit 0; the
//! same reports collected from an unscoped emission exit 3, so the fixture is one the count
//! decides.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const MEMBERSHIP: &str = r"format: ess/16
system: mem
version: v1
domain: mem.order
commands:
  - name: mem.order.Place
    input:
      - {name: order_id, type: String}
    outcomes:
      - name: placed
        emits: [mem.events.Placed]
        payload:
          mem.events.Placed: {order_id: input.order_id}
components:
  - component: owner
    owns: {domains: [mem.events]}
    accepts: {commands: [mem.order.Place]}
";

const MEMBERSHIP_EVENTS: &str = r"domain: mem.events
events:
  - name: mem.events.Alpha
    fields:
      - {name: order_id, type: String}
  - name: mem.events.Placed
    fields:
      - {name: order_id, type: String}
";

const MEMBERSHIP_OTHER: &str = r"domain: mem.other
events:
  - name: mem.other.Also
    fields:
      - {name: ref, type: String}
  - name: mem.other.Done
    fields:
      - {name: ref, type: String}
commands:
  - name: mem.other.Orphan
    input:
      - {name: ref, type: String}
    outcomes:
      - name: done
        emits: [mem.other.Done]
        payload:
          mem.other.Done: {ref: input.ref}
";

const FILES: [(&str, &str); 3] = [
    ("system.yaml", MEMBERSHIP),
    ("membership-events.yaml", MEMBERSHIP_EVENTS),
    ("membership-other.yaml", MEMBERSHIP_OTHER),
];

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn s(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-emit-swap-cli")
        .join(format!("{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The specification, written as a directory, and its model interpreter.
fn specification(directory: &Path) -> (PathBuf, impl Fn() -> Interpreted) {
    let spec = directory.join("spec");
    std::fs::create_dir_all(&spec).unwrap();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for (name, text) in FILES {
        std::fs::write(spec.join(name), text).unwrap();
        texts.insert(name.to_owned(), text.to_owned());
        parsed.push((Source::new(name), RawSpecFile::parse(text).unwrap()));
    }
    let ir = mutate::compile(parsed, &texts).expect("the membership fixture compiles");
    (spec, move || Interpreted::for_model(ir.clone()))
}

/// Answers every suite of the emission at `emitted` with the native runner over `target`.
fn answer(emitted: &Path, target: &impl Fn() -> Interpreted) {
    let manifest = json(&emitted.join("manifest.json"));
    let mut dirs = vec![manifest["baseline"]["dir"].as_str().unwrap().to_owned()];
    dirs.extend(
        manifest["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|it| it["dir"].as_str().map(str::to_owned)),
    );
    for dir in dirs {
        let suite = std::fs::read_to_string(emitted.join(&dir).join("suite.json")).unwrap();
        let admitted = AdmittedSuite::from_json(&suite).expect("an emitted suite is admitted");
        let executed = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target());
        let report = CountReport::from_run(&executed, &admitted)
            .expect("a complete run")
            .to_canonical_json()
            .expect("serializes");
        std::fs::write(emitted.join(&dir).join("report.json"), report).unwrap();
    }
}

fn emit_and_collect(directory: &Path, component: Option<&str>) -> (Output, Value) {
    let (spec, target) = specification(directory);
    let emitted = directory.join("emitted");
    let mut args = vec![
        "verify",
        "conform",
        "mutate",
        "--path",
        s(&spec),
        "--class",
        "emit-swap",
        "--emit",
        s(&emitted),
    ];
    if let Some(component) = component {
        args.extend(["--component", component]);
    }
    let output = ess(&args);
    assert_eq!(output.status.code(), Some(0), "{}", shown(&output));
    answer(&emitted, &target);
    let out = directory.join("collected.json");
    let mut args = vec![
        "verify",
        "conform",
        "mutate",
        "--collect",
        s(&emitted),
        "--format",
        "json",
        "--report-out",
        s(&out),
    ];
    if let Some(component) = component {
        args.extend(["--component", component]);
    }
    let collected = ess(&args);
    let report = json(&out);
    (collected, report)
}

#[test]
fn an_outside_component_site_does_not_change_the_collect_exit_status() {
    let directory = scratch("scoped");
    let (collected, report) = emit_and_collect(&directory, Some("owner"));
    assert_eq!(report["format"], "ess-mutation-report/4");
    assert_eq!(report["counts"]["killed"], 1, "{report:#}");
    assert_eq!(report["counts"]["survived"], 0);
    assert_eq!(
        report["unavailable_sites"][0]["reason"], "outside_component",
        "{report:#}"
    );
    assert_eq!(
        collected.status.code(),
        Some(0),
        "an outside_component site changed the exit status\n{}",
        shown(&collected)
    );
}

/// The control: the same model and runner, unscoped, leave `Orphan` in scope and exit 3.
#[test]
fn the_same_site_unscoped_keeps_the_audit_at_exit_three() {
    let directory = scratch("unscoped");
    let (collected, report) = emit_and_collect(&directory, None);
    assert_eq!(report["counts"]["killed"], 1, "{report:#}");
    assert_eq!(
        report["unavailable_sites"][0]["reason"],
        "no_compatible_event_alternative"
    );
    assert_eq!(collected.status.code(), Some(3), "{}", shown(&collected));
}
