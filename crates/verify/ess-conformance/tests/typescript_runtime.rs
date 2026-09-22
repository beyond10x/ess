//! The TypeScript runtime's own cases, executed.
//!
//! `src/ts/mod.rs` embeds the five sources and not the `*.test.ts` files beside them, which is
//! right — the Go target ships no `*_test.go` either, and an adopter running `npm test` is asking
//! about their implementation rather than about ours. The consequence, until this file existed,
//! was that **nothing anywhere ran them**: `task check` is Rust, the emitted package does not
//! carry them, and a green gate said nothing about any line of the runtime.
//!
//! That is the failure this repository already knows by another name. A lane that selects nothing
//! exits zero, and an absent lane is indistinguishable from a passing one, so the checks go on
//! being written and go on never running.
//!
//! # Why Rust, and not a Taskfile step
//!
//! `AGENTS.md`: "Anything executable is Rust. Do not add Python or shell checkers." It is also the
//! only shape that can read the **count**. `node --test 'glob'` over a pattern that matches
//! nothing exits 0, so an exit status is not evidence the cases ran; what this asserts is that
//! every `*.test.ts` in the source directory was reported, by name, and that none failed.
//!
//! # Why the sources are compiled rather than stripped
//!
//! The sources import one another as `./runtime.js` while being `.ts` on disk — the specifier
//! `tsc` resolves under `moduleResolution: NodeNext`, and the one Node's own type stripping does
//! not: it looks for a literal `runtime.js` and fails with `ERR_MODULE_NOT_FOUND`. Running them
//! therefore means compiling them, and `tsc --noCheck` does that with no `@types/node` present,
//! which matters because the gate is offline and this repository installs nothing.
//!
//! Type *checking* is a separate case here, and it is separate on purpose. `--noCheck` cannot see
//! a type error at all, and one was there: the emitted `tsconfig.json` named no `types`, so
//! `process`, `Buffer`, `TextEncoder` and `URL` were unresolvable and an adopter running `tsc` in
//! the output directory met fourteen errors while every check in this repository was green. The
//! lesson is not about that defect, it is about the shape — a transpile standing in for a check
//! is a lane that has entered the directory without reading anything in it.
//!
//! So `the_typescript_sources_typecheck_against_the_node_declarations` runs the real compile, and
//! it needs `@types/node`, which this offline repository installs nothing to obtain. Where it is
//! absent that case prints what it needed and returns rather than passing quietly. A gate that
//! wants it to bind installs `@types/node` under the repository root or names a `node_modules` in
//! `ESS_TYPES_NODE`.
//!
//! # The output goes inside the repository
//!
//! Not into a temporary directory. `predicate.test.ts` reads
//! `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` by walking up from its
//! own location, so a build written anywhere else fails at the corpus rather than at a predicate.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// Where the hand-written TypeScript is.
fn sources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ts")
}

/// A tool on `PATH`, or `None` when this machine has none.
///
/// Skipped rather than failed, and said out loud, for the reason `tests/go_conformance.rs` skips
/// where the machine has no Go: a machine without the toolchain cannot answer this question, and
/// a test that silently passed there would report the runtime as checked. A machine that *has*
/// the toolchain and runs nothing is the case the counts below exist for.
fn tool(name: &str, version: &[&str]) -> Option<PathBuf> {
    let output = Command::new(name).args(version).output().ok()?;
    output.status.success().then(|| PathBuf::from(name))
}

/// Every `*.test.ts` in the source directory, by stem.
///
/// Read off the tree rather than listed here, so a new case file is required to run by the fact
/// of existing. A hand-maintained list is the defect that lets the next one be forgotten.
fn case_files() -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(sources())
        .expect("the TypeScript source directory exists")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            name.ends_with(".test.ts").then_some(name)
        })
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "no *.test.ts under {} — this lane would run nothing and exit zero",
        sources().display()
    );
    found
}

/// The `# key N` summary lines Node's test runner prints.
fn summary(printed: &str, key: &str) -> Option<u32> {
    printed
        .lines()
        .find_map(|line| line.trim().strip_prefix(&format!("# {key} ")))
        .and_then(|value| value.trim().parse().ok())
}

/// Compiles the sources into a directory of this run's own and returns it.
fn build(tsc: &Path) -> PathBuf {
    let out = root()
        .join("target/typescript-runtime")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("a build directory");

    let declarations = node_declarations();
    let mut command = Command::new(tsc);
    command
        .arg("--project")
        .arg(sources().join("tsconfig.json"))
        // Transpiling, not checking, and deliberately so: this exists to make the cases
        // *runnable* where no `@types/node` is installed. It is not a typecheck and must not be
        // read as one — `the_typescript_sources_typecheck_against_the_node_declarations` below is
        // the typecheck, and it says out loud when it could not run. A `--noCheck` standing in
        // for a check is how the `types` defect this package shipped with went unseen.
        .arg("--noCheck")
        .arg("--outDir")
        .arg(&out);
    if let Some(modules) = &declarations {
        command.arg("--typeRoots").arg(modules.join("@types"));
    }
    let compiled = command.output().expect("tsc runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );

    if declarations.is_some() {
        assert!(
            compiled.status.success(),
            "tsc refused the sources:\n{printed}"
        );
    } else {
        // `types: ["node"]` is in the project file because the emitted package needs it, and on a
        // machine with no declarations `tsc` answers TS2688 for it even under `--noCheck` — while
        // still emitting. That one diagnostic is a fact about this machine. Every other one is a
        // fact about the sources and still fails here.
        for line in printed.lines().filter(|line| line.contains("error TS")) {
            assert!(
                line.contains("TS2688"),
                "tsc refused the sources for a reason other than the absent node \
                 declarations:\n{printed}"
            );
        }
    }

    // Emission is the thing this function is for, and a `tsc` that reported nothing and wrote
    // nothing would otherwise reach the runner as "no cases", not as "no build".
    for file in case_files() {
        let compiled = out.join(file.replace(".test.ts", ".test.js"));
        assert!(
            compiled.is_file(),
            "{} was not emitted by tsc:\n{printed}",
            compiled.display()
        );
    }

    // The emitted `.js` is ESM because the sources are, and Node decides that from the nearest
    // `package.json` rather than from the bytes. Without this the first import is
    // "exports is not defined in ES module scope".
    std::fs::write(out.join("package.json"), "{\"type\":\"module\"}\n")
        .expect("the module marker writes");
    out
}

/// Every case the TypeScript runtime carries runs, and passes.
#[test]
fn every_case_the_typescript_runtime_carries_is_executed_and_passes() {
    let Some(tsc) = tool("tsc", &["--version"]) else {
        println!("skipped: no `tsc` on PATH, so the TypeScript runtime was not compiled or run");
        return;
    };
    let Some(node) = tool("node", &["--version"]) else {
        println!("skipped: no `node` on PATH, so the TypeScript runtime was not run");
        return;
    };
    let expected = case_files();
    let out = build(&tsc);

    // One invocation per file, rather than one over all of them. Node's reporter labels the
    // top-level entries of a multi-file run by test name and not by path, so a file that
    // contributed nothing is invisible in the combined output — which is the whole thing this
    // lane exists to notice. Per file, "it ran" is a count of its own.
    let mut total = 0;
    for file in &expected {
        let compiled = out.join(file.replace(".test.ts", ".test.js"));
        let run = Command::new(&node)
            .arg("--test")
            .arg(&compiled)
            .current_dir(root())
            .output()
            .expect("node --test runs");
        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let passed = summary(&printed, "pass").unwrap_or_else(|| {
            panic!("{file} produced no `# pass` line, so nothing here is a count\n{printed}")
        });
        let failed = summary(&printed, "fail")
            .unwrap_or_else(|| panic!("{file} produced no `# fail` line\n{printed}"));
        assert_eq!(failed, 0, "{file} has failing cases\n{printed}");
        assert!(
            passed > 0,
            "{file} was selected and reported no passing case, so it ran nothing\n{printed}"
        );
        assert!(
            run.status.success(),
            "{file} failed while reporting no failing case\n{printed}"
        );
        println!("typescript: {file} — {passed} passing case(s)");
        total += passed;
    }

    // A floor, not an expectation. The per-file checks above catch a file that stopped being
    // collected or stopped contributing; this catches the other shape — every file still loading,
    // and most of its cases gone. Measured at 201 on 2026-09-19; the floor is deliberately below
    // that, so adding and removing a case is not a gate change, and deliberately far above zero.
    assert!(
        total >= 150,
        "the runtime reported only {total} passing cases across {} file(s), fewer than it has \
         ever carried",
        expected.len()
    );
    println!(
        "typescript: {total} passing case(s) across {} file(s)",
        expected.len()
    );
}

/// Where Node's type declarations are, or `None` when this machine has none.
///
/// Three places, in order: what `ESS_TYPES_NODE` names, a `node_modules` beside the sources, and
/// one at the repository root. Any of them makes the typecheck below real; none of them makes it
/// say so rather than pass.
fn node_declarations() -> Option<PathBuf> {
    let candidates = [
        std::env::var_os("ESS_TYPES_NODE").map(PathBuf::from),
        Some(sources().join("node_modules")),
        Some(root().join("node_modules")),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|modules| modules.join("@types/node/package.json").is_file())
}

/// `tsc --noEmit` over `project`, with the declarations an adopter compiles against.
///
/// `--typeRoots`, not an environment variable: `tsc` finds `@types` by walking up from the
/// project file, and the declarations this repository can reach are not necessarily on that path.
/// Naming the root is the one way that does not depend on where they were installed.
fn typecheck(tsc: &Path, project: &Path, modules: &Path) -> (bool, String) {
    let checked = Command::new(tsc)
        .arg("--noEmit")
        .arg("--project")
        .arg(project)
        .arg("--typeRoots")
        .arg(modules.join("@types"))
        .current_dir(project.parent().expect("a project file has a directory"))
        .output()
        .expect("tsc runs");
    (
        checked.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        ),
    )
}

/// `tsc`, and the node declarations, or a printed reason there is no answer here.
///
/// Both are things the offline gate installs nothing to obtain, so on a machine without them this
/// says what it needed rather than passing quietly. A gate that wants these checks to bind
/// installs `@types/node` under the repository root, or names a `node_modules` in
/// `ESS_TYPES_NODE`.
fn typechecker(what: &str) -> Option<(PathBuf, PathBuf)> {
    let Some(tsc) = tool("tsc", &["--version"]) else {
        println!("skipped: no `tsc` on PATH, so {what} was not typechecked");
        return None;
    };
    let Some(modules) = node_declarations() else {
        println!(
            "skipped: no `@types/node` found, so {what} was not typechecked. This check needs it \
             at `<repo>/node_modules/@types/node`, beside the sources, or under a `node_modules` \
             named by ESS_TYPES_NODE. Without it the TypeScript is transpiled and run but never \
             typechecked, and a missing declaration reaches an adopter first."
        );
        return None;
    };
    Some((tsc, modules))
}

/// The package an adopter is handed typechecks, compiled the way an adopter compiles it.
///
/// This is the claim the emitter is responsible for, and the one that was false. The lane above
/// transpiles with `--noCheck`, which cannot see a type error at all — and there was one: the
/// emitted `tsconfig.json` named no `types`, so `process`, `Buffer`, `TextEncoder` and `URL` were
/// unresolvable, `tsc --noEmit` in the output directory answered fourteen errors, and every check
/// in this repository was green. An adopter would have been the first reader of it.
///
/// So this emits a package into a directory of its own and compiles **that** — the emitted
/// project file, the emitted layout, the real declarations, nothing relaxed. Checking the
/// repository's flat source directory instead would not have caught it, because the defect was in
/// the settings the emitter writes rather than in the sources it copies.
#[test]
fn the_emitted_package_typechecks_as_an_adopter_compiles_it() {
    let Some((tsc, modules)) = typechecker("the emitted package") else {
        return;
    };
    let out = root()
        .join("target/typescript-emitted")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&out);
    for artifact in ess_conformance::ts::emit(&suite()).expect("the suite emits") {
        let path = out.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("every artifact has a directory"))
            .expect("the package directory");
        std::fs::write(&path, &artifact.contents).expect("the artifact writes");
    }

    let package = out.join(ess_conformance::ts::PACKAGE);
    std::fs::write(
        package.join("src/fixture-provider.ts"),
        "import type { FixtureContract, Target } from './index.js';\n\
         export const provider: NonNullable<Target['fixtureValues']> = (_context, contract: FixtureContract) => {\n\
           const values: Record<string, unknown> = {};\n\
           for (const field of contract.fields) values[field.name] = 'independently provisioned';\n\
           return values;\n\
         };\n",
    )
    .expect("an adopter can name the fixture provider contract through the package entry");
    let (ok, printed) = typecheck(&tsc, &package.join("tsconfig.json"), &modules);
    assert!(
        ok,
        "the emitted package does not typecheck, which is what an adopter meets first:\n{printed}"
    );
    println!(
        "typecheck: the emitted package compiles clean against {}",
        modules.display()
    );
}

/// This repository's own copies typecheck too, `*.test.ts` included.
///
/// Broader than the case above and a different claim: the emitted package carries none of the
/// `*.test.ts`, so a type error in one of those reaches no adopter — but it does mean the cases
/// the lane executes were never checked, only transpiled, and `--noCheck` will run a file whose
/// types are wrong right up until the assertion that depends on them.
#[test]
fn the_repositorys_own_typescript_typechecks_including_its_cases() {
    let Some((tsc, modules)) = typechecker("this repository's TypeScript") else {
        return;
    };
    let (ok, printed) = typecheck(&tsc, &sources().join("tsconfig.json"), &modules);
    assert!(ok, "the TypeScript sources do not typecheck:\n{printed}");
}

/// An empty suite, which is all the emitted-package typecheck needs: what it compiles is the
/// runtime and the settings, and neither moves with the model.
#[cfg(test)]
fn suite() -> ess_conformance::ConformanceSuite {
    use ess_conformance::scenario::SuiteFormat;
    use ess_primitives::evidence::SpecDigest;

    let digest = |value: &str| SpecDigest::new(value).expect("a digest");
    ess_conformance::ConformanceSuite::new(ess_conformance::SuiteProvenance {
        suite_version: SuiteFormat::CURRENT,
        system: "billing".to_owned(),
        specification_version: "v3".to_owned(),
        spec_digest: digest("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        contract_digest: digest("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        component: None,
    })
}

/// The TypeScript is formatted, by one configuration that exists.
///
/// There was none: no `.prettierrc`, no eslint config, no `.editorconfig` and no gate step, so
/// four units wrote four house styles into one package and the next person to run a formatter
/// would have produced a diff across all of it. `src/ts/.prettierrc.json` is the decision;
/// this is the thing that makes it one.
#[test]
fn the_typescript_sources_are_formatted_by_the_configuration_beside_them() {
    let Some(prettier) = tool("prettier", &["--version"]) else {
        println!("skipped: no `prettier` on PATH, so the TypeScript formatting was not checked");
        return;
    };
    let checked = Command::new(prettier)
        .arg("--check")
        .arg(sources().join("*.ts"))
        .current_dir(sources())
        .output()
        .expect("prettier runs");
    assert!(
        checked.status.success(),
        "the TypeScript is not formatted. `prettier --write '{}'` is the fix:\n{}{}",
        sources().join("*.ts").display(),
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
}
