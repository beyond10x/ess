//! The emitted Go and TypeScript runtimes admit suite/42 and /43 synthesis-seed provenance, refuse
//! every forged or dangling record and every major they do not implement, and the Go runtime
//! executes the seeded issue suite — exact `i64::MAX` and `MAX − 1` setup rows, inputs and
//! observations — with the reference runner's verdict (beyond10x/ess#413,
//! `docs/design/synthesis-seeds.md`).
mod support_go;
mod support_seeds;

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::coverage::{Origins, Scope};
use support_seeds::*;

/// The valid documents every runtime must admit, by file name.
fn valid(ir: &ess_compiler::ir::EssIr) -> Vec<(String, String)> {
    let (good, with_authored, _) = documents(ir);
    let seeds = admitted(ir, &issue_selections());
    let coverage = ess_conformance::coverage_build::build_with_seeds(
        ir,
        &[],
        Scope::System,
        Origins::Generated,
        &seeds,
    )
    .unwrap();
    assert_eq!(
        coverage
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/43"
    );
    vec![
        ("valid-42.json".to_owned(), text(&good)),
        ("valid-42-authored.json".to_owned(), text(&with_authored)),
        (
            "valid-43.json".to_owned(),
            coverage.selected().original_json().to_owned(),
        ),
    ]
}

/// Every forged document, by file name, with the refusal the runtimes must give.
fn forged(ir: &ess_compiler::ir::EssIr) -> Vec<(String, String, &'static str)> {
    let (good, with_authored, authored_id) = documents(ir);
    forgeries(&good, &with_authored, &authored_id)
        .into_iter()
        .map(|forgery| {
            (
                format!("forged-{}.json", forgery.label),
                text(&forgery.document),
                forgery.runtimes,
            )
        })
        .collect()
}

#[test]
fn go_admits_seeded_suites_and_refuses_every_forgery() {
    let ir = ir();
    let directory = support_go::package(
        "seed-admission",
        &seeded(&ir).suite,
        &[(
            "admission_test.go",
            include_str!("fixtures/suite-admission-go.go"),
        )],
    );
    let folder = directory.join("documents");
    std::fs::create_dir_all(&folder).unwrap();
    let valid = valid(&ir);
    let forged = forged(&ir);
    for (name, contents) in &valid {
        std::fs::write(folder.join(name), contents).unwrap();
    }
    for (name, contents, _) in &forged {
        std::fs::write(folder.join(name), contents).unwrap();
    }
    let run = support_go::go_test(
        &directory,
        "TestSuiteAdmission",
        &[("ESS_ADMISSION_DIR", folder.to_str().unwrap())],
    );
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(run.success, "{}", run.log);
    for (name, _) in &valid {
        assert!(
            run.log.contains(&format!("ADMITTED {name}\n")),
            "Go refused {name}: {}",
            run.log
        );
    }
    let mut wrong = Vec::new();
    for (name, _, reason) in &forged {
        let line = run
            .log
            .lines()
            .find(|line| line.contains(&format!(" {name}")))
            .unwrap_or_default();
        if !(line.starts_with("REFUSED ") && line.contains(reason)) {
            wrong.push(format!("{name}: expected `{reason}`, got `{line}`"));
        }
    }
    assert_eq!(wrong.len(), 0, "{}", wrong.join("\n"));
}

#[test]
fn go_executes_the_seeded_suite_with_the_reference_verdicts() {
    let ir = ir();
    let suite = seeded(&ir).suite;
    let healthy =
        support_go::assert_parity("seeds-healthy", &suite, Counters::new(&ir, Fault::None));
    assert_eq!(
        support_go::not_passed(&healthy).len(),
        0,
        "the healthy counter fails under Go: {healthy:#?}"
    );
    assert!(healthy.contains_key(EXHAUSTED) && healthy.contains_key(AUTHORIZED));
    for (fault, scenario) in [
        (Fault::GuardOmitted, EXHAUSTED),
        (Fault::ThresholdEarly, AUTHORIZED),
        (Fault::CasIgnored, STALE),
        (Fault::AcknowledgedOnly, EXHAUSTED),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("seeds-{fault:?}").to_lowercase(),
            &suite,
            Counters::new(&ir, fault),
        );
        assert_ne!(
            verdicts.get(scenario).map(String::as_str),
            Some("passed"),
            "{fault:?} survived {scenario} under Go: {verdicts:#?}"
        );
    }
}

/// The emitted TypeScript package for the seeded suite, compiled, under this test binary's scratch
/// directory.
fn typescript_package() -> PathBuf {
    let ir = ir();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("synthesis-seeds-typescript-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(&seeded(&ir).suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = root.join(ess_conformance::ts::PACKAGE);
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(Path::new(&modules).join("@types"));
    }
    let output = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    package
}

#[test]
fn typescript_admits_seeded_suites_and_refuses_every_forgery() {
    let ir = ir();
    let package = typescript_package();
    let folder = package.join("documents");
    std::fs::create_dir_all(&folder).unwrap();
    let valid = valid(&ir);
    let forged = forged(&ir);
    for (name, contents) in &valid {
        std::fs::write(folder.join(name), contents).unwrap();
    }
    for (name, contents, _) in &forged {
        std::fs::write(folder.join(name), contents).unwrap();
    }
    std::fs::write(
        package.join("admission.mjs"),
        r"
import {readFileSync, readdirSync} from 'node:fs';
import {admitSuite} from './dist/runtime.js';
for (const name of readdirSync('documents').sort()) {
  try {
    admitSuite(readFileSync(`documents/${name}`, 'utf8'));
    process.stdout.write(`ADMITTED ${name}\n`);
  } catch (error) {
    process.stdout.write(`REFUSED ${name}: ${String(error?.message ?? error)}\n`);
  }
}
",
    )
    .unwrap();
    let output = Command::new("node")
        .arg("admission.mjs")
        .current_dir(&package)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(package.parent().unwrap()).unwrap();
    assert!(output.status.success(), "{log}");
    for (name, _) in &valid {
        assert!(
            log.contains(&format!("ADMITTED {name}\n")),
            "TypeScript refused {name}: {log}"
        );
    }
    let mut wrong = Vec::new();
    for (name, _, reason) in &forged {
        let line = log
            .lines()
            .find(|line| line.contains(&format!(" {name}")))
            .unwrap_or_default();
        if !(line.starts_with("REFUSED ") && line.contains(reason)) {
            wrong.push(format!("{name}: expected `{reason}`, got `{line}`"));
        }
    }
    assert_eq!(wrong.len(), 0, "{}", wrong.join("\n"));
}

#[test]
fn both_emitters_write_packages_for_the_seed_pair() {
    let ir = ir();
    let suite = seeded(&ir).suite;
    ess_conformance::go::emit(&suite).expect("Go emits suite/42");
    ess_conformance::ts::emit(&suite).expect("TypeScript emits suite/42");
    let seeds = admitted(&ir, &issue_selections());
    let input = ess_conformance::coverage_build::build_with_seeds(
        &ir,
        &[],
        Scope::System,
        Origins::Generated,
        &seeds,
    )
    .unwrap();
    ess_conformance::go::emit_input(&input).expect("Go emits suite/43");
    ess_conformance::ts::emit_input(&input).expect("TypeScript emits suite/43");
}
