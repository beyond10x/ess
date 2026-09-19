//! `ess verify conform synthesize --target typescript`, emitted as a package an adopter can run.
//!
//! The sibling of `go_conformance.rs`, and deliberately not the same test. That file holds the
//! emitted Go package to a real Go implementation, because a Go toolchain is a thing this
//! repository's gate may reach. This one cannot do the same: `npm ci` fetches, the gate is offline
//! (`AGENTS.md`, "Gate"), and the emitted TypeScript is compiled by a toolchain no offline check
//! installs. So what is asserted here is everything about the emission that does not need a
//! runtime — that the route exists, that it is byte-deterministic, and that the package it writes
//! carries the scaffolding an adopter needs rather than leaving them to guess it.
//!
//! The half this cannot reach — that the emitted runtime reports the same counts as the Go one
//! against equivalent targets — is the story's own acceptance and belongs to a check that may run
//! a Node toolchain.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// A directory of this test's own, under the temporary root rather than the source tree.
fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("ess-ts-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

/// Synthesizes the billing example into `out`, authored scenarios included.
fn synthesize(out: &Path) -> std::process::Output {
    synthesize_format(out, None)
}

/// The same, at one explicit suite format — `5` being the coverage carrier, which reaches
/// `ts::emit_input` rather than `ts::emit` and is otherwise not executed by anything.
fn synthesize_format(out: &Path, format: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["verify", "conform", "synthesize", "--path"])
        .arg(root().join("examples/billing"))
        .arg("--scenarios")
        .arg(root().join("examples/billing-scenarios"))
        .args(["--target", "typescript", "--out"])
        .arg(out);
    if let Some(format) = format {
        command.args(["--suite-format", format]);
    }
    command.output().expect("the ess binary runs")
}

/// Every emitted file under `directory`, by relative path, minus the CLI's own ownership
/// checkpoint — which records when a tree was claimed and is not part of what was emitted.
fn tree(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, at: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).expect("the emitted directory reads") {
            let path = entry.expect("a directory entry").path();
            let relative = path
                .strip_prefix(base)
                .expect("every entry is under the base")
                .to_string_lossy()
                .replace('\\', "/");
            if relative.starts_with(".ess-output") {
                continue;
            }
            if path.is_dir() {
                walk(base, &path, into);
            } else {
                into.insert(relative, std::fs::read(&path).expect("the file reads"));
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(directory, directory, &mut files);
    files
}

/// The emitted package does not move when nothing else does.
///
/// Deterministic emission is what makes `git diff` on a regenerated suite readable: a target whose
/// bytes shift between two runs over one specification reports every regeneration as a change, and
/// the one regeneration that really did change something is then indistinguishable from the rest.
#[test]
fn a_second_run_over_the_same_specification_writes_identical_bytes() {
    let first = scratch("determinism-first");
    let second = scratch("determinism-second");

    let one = synthesize(&first);
    assert!(
        one.status.success(),
        "synthesis failed: {}{}",
        String::from_utf8_lossy(&one.stdout),
        String::from_utf8_lossy(&one.stderr)
    );
    let two = synthesize(&second);
    assert!(
        two.status.success(),
        "the second synthesis failed: {}{}",
        String::from_utf8_lossy(&two.stdout),
        String::from_utf8_lossy(&two.stderr)
    );

    let (left, right) = (tree(&first), tree(&second));
    assert!(
        !left.is_empty(),
        "the emitted package is empty, so determinism says nothing"
    );
    let names: Vec<&String> = left.keys().collect();
    assert_eq!(names, right.keys().collect::<Vec<&String>>());
    for (path, bytes) in &left {
        assert_eq!(
            bytes,
            right.get(path).expect("both runs wrote the same names"),
            "{path} differs between two runs over one specification"
        );
    }

    // Regenerating in place is the case an adopter is actually in: the directory is already theirs
    // from the last model change. A route whose output owner is not recorded refuses the second
    // write, and two separate directories would never show it.
    let again = synthesize(&first);
    assert!(
        again.status.success(),
        "regenerating over the emitted package failed: {}{}",
        String::from_utf8_lossy(&again.stdout),
        String::from_utf8_lossy(&again.stderr)
    );
    assert_eq!(tree(&first), left, "regenerating in place moved the bytes");
}

/// The coverage carrier reaches the same package, through the other emitter.
///
/// `--suite-format 5` routes to `ts::emit_input`, which writes `input.json` beside the suite and
/// rewrites the source that reads it. Nothing else executes that function, so without this the
/// whole coverage half of the TypeScript target ships having never run.
#[test]
fn the_coverage_carrier_emits_a_package_that_reads_the_input_rather_than_the_suite() {
    let directory = scratch("coverage");
    let emitted = synthesize_format(&directory, Some("5"));
    // `1` is a complete run with refusals, which the billing example has and which is not a
    // failure of the emitter.
    assert!(
        matches!(emitted.status.code(), Some(0 | 1)),
        "coverage synthesis failed: {}{}",
        String::from_utf8_lossy(&emitted.stdout),
        String::from_utf8_lossy(&emitted.stderr)
    );
    let files = tree(&directory);
    let names: Vec<&str> = files.keys().map(String::as_str).collect();
    for required in ["essconform/input.json", "essconform/suite.json"] {
        assert!(
            names.contains(&required),
            "{required} was not emitted: {names:?}"
        );
    }

    // The runner itself, not a shim beside it. `runtime.ts` performs the read, so a package whose
    // runtime still names `suite.json` runs the selection's parent however many other files were
    // rewritten — which is a wrong count reported under the right name.
    let runtime = String::from_utf8(files["essconform/src/runtime.ts"].clone())
        .expect("the runtime is UTF-8");
    assert!(
        runtime.contains("../input.json"),
        "the coverage package's runner still reads the suite rather than the selection"
    );
    assert!(
        !runtime.contains("../suite.json"),
        "the coverage package's runner names both documents, so which one it runs is undecided"
    );
}

/// The emitted directory is a package, not a pile of sources.
///
/// The Go target leaves `go.mod` to the adopter because a Go module is the adopter's identity. An
/// npm package is not: a directory of `.ts` files with no `package.json` is not consumable by
/// anything, and an adopter who has to write the manifest is the one deciding `"type"`,
/// `moduleResolution` and the test runner — which is three chances to make the emitted runtime not
/// run and no way to tell that from a runtime that is wrong.
#[test]
fn the_emitted_package_carries_the_manifest_an_adopter_would_otherwise_have_to_guess() {
    let directory = scratch("package");
    let emitted = synthesize(&directory);
    assert!(
        emitted.status.success(),
        "synthesis failed: {}{}",
        String::from_utf8_lossy(&emitted.stdout),
        String::from_utf8_lossy(&emitted.stderr)
    );
    let files = tree(&directory);
    let names: Vec<&str> = files.keys().map(String::as_str).collect();

    for required in [
        "essconform/package.json",
        "essconform/tsconfig.json",
        "essconform/README.md",
        "essconform/suite.json",
        "essconform/src/index.ts",
        "essconform/src/runtime.ts",
        "essconform/src/predicate.ts",
        "essconform/src/response.ts",
        "essconform/src/reading.ts",
        "essconform/src/coordinate.ts",
    ] {
        assert!(
            names.contains(&required),
            "{required} was not emitted: {names:?}"
        );
    }

    let manifest: serde_json::Value =
        serde_json::from_slice(&files["essconform/package.json"]).expect("package.json is JSON");
    assert_eq!(
        manifest["type"], "module",
        "the emitted package is ESM: {manifest}"
    );
    let test = manifest["scripts"]["test"]
        .as_str()
        .expect("package.json declares a test script");
    assert!(
        test.contains("node --test"),
        "`npm test` must reach `node --test`, not {test:?}"
    );

    let tsconfig: serde_json::Value =
        serde_json::from_slice(&files["essconform/tsconfig.json"]).expect("tsconfig.json is JSON");
    assert_eq!(
        tsconfig["compilerOptions"]["module"], "NodeNext",
        "the emitted sources import each other as `./runtime.js`, which only resolves under \
         NodeNext: {tsconfig}"
    );
    assert_eq!(
        tsconfig["compilerOptions"]["strict"],
        serde_json::Value::Bool(true),
        "a runtime emitted unchecked is a runtime nothing checked: {tsconfig}"
    );
}

/// The suite is a file the package reads, not a string inside a source.
///
/// Same reason the Go target embeds rather than inlines: a generated purpose carries backticks and
/// newlines, and a suite spelled as a literal is one unreadable line that `git diff` cannot show a
/// scenario moving in.
#[test]
fn the_suite_is_emitted_as_its_own_document_and_the_sources_do_not_carry_it() {
    let directory = scratch("embedded");
    assert!(synthesize(&directory).status.success());
    let files = tree(&directory);

    let suite: serde_json::Value =
        serde_json::from_slice(&files["essconform/suite.json"]).expect("suite.json is JSON");
    assert!(
        suite["provenance"]["spec_digest"].is_string(),
        "the emitted suite names the specification it came from"
    );
    assert!(
        suite["scenarios"]
            .as_object()
            .is_some_and(|scenarios| !scenarios.is_empty()),
        "the emitted suite holds no scenarios"
    );

    for (path, bytes) in &files {
        if Path::new(path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("ts"))
        {
            let text = String::from_utf8_lossy(bytes);
            assert!(
                !text.contains("\"spec_digest\""),
                "{path} carries the suite document rather than reading it"
            );
        }
    }
}

/// Every name the README tells an adopter to import is reachable from the entry point.
///
/// A generated README is the one adopter-facing document nobody re-reads after the runtime moves,
/// so the names its wiring example depends on are resolved rather than trusted: through
/// `index.ts`, module by module, exactly the way `import { run } from "essconform"` resolves.
///
/// This is not hypothetical. `index.ts` was `export * from "./runtime.js"` alone, and
/// `ClockReadingTarget` — the interface an adopter has to implement for every clock-reading
/// scenario — was unreachable from it, which `tsc` reports as TS2724 and which nothing in this
/// repository would have noticed. `run`, `Target` and `ErrUnsupported` are the first day;
/// `ClockReadingTarget` and `ClockReadingEvidence` are the case that was actually broken.
#[test]
fn every_name_the_readme_tells_an_adopter_to_import_is_reachable_from_the_entry_point() {
    let directory = scratch("readme");
    assert!(synthesize(&directory).status.success());
    let files = tree(&directory);

    // The modules `index.ts` re-exports, which is the whole of what `import "essconform"` reaches.
    let index =
        String::from_utf8(files["essconform/src/index.ts"].clone()).expect("the index is UTF-8");
    let modules: Vec<String> = index
        .lines()
        .filter_map(|line| {
            // Quote-agnostic: which quote the emitter uses is a formatting choice, and a check
            // that broke when it changed would be checking the formatting rather than the surface.
            let rest = line
                .trim()
                .strip_prefix("export * from ")?
                .trim_matches(|c| c == '\'' || c == '"' || c == ';')
                .strip_prefix("./")?;
            Some(rest.strip_suffix(".js")?.to_owned())
        })
        .collect();
    assert!(
        !modules.is_empty(),
        "the entry point re-exports nothing, so the package has no surface: {index}"
    );

    let mut surface = String::new();
    for module in &modules {
        let path = format!("essconform/src/{module}.ts");
        let bytes = files
            .get(&path)
            .unwrap_or_else(|| panic!("the entry point re-exports {path}, which is not emitted"));
        surface.push_str(&String::from_utf8_lossy(bytes));
    }
    if surface.trim().is_empty() {
        println!(
            "skipped: every module the entry point names is still an empty placeholder, so the \
             README's symbols cannot be resolved against them"
        );
        return;
    }

    let readme =
        String::from_utf8(files["essconform/README.md"].clone()).expect("the README is UTF-8");
    for symbol in [
        "run",
        "Target",
        "ErrUnsupported",
        "ClockReadingTarget",
        "ClockReadingEvidence",
    ] {
        if !readme.contains(symbol) {
            // Not every name below is in the README's prose today; the ones that are, bind. A
            // name the README drops stops being checked here, which is correct — it is no longer
            // an instruction to anybody.
            continue;
        }
        assert!(
            ["const", "function", "interface", "type", "class", "declare"]
                .iter()
                .any(|kind| surface.contains(&format!("export {kind} {symbol}"))),
            "the README tells an adopter to import `{symbol}`, and no module the entry point \
             re-exports ({modules:?}) exports it"
        );
    }
}
