//! The suite as a TypeScript test package.
//!
//! `ess verify conform run` drives a target written in Rust, which is every implementation this
//! workspace can reach and none of the implementations adopters have. This module is the sibling
//! of [`go`](crate::go) and rests on the same argument: a suite that is regenerated on every model
//! change and that nothing can execute is not a weak suite, it is no suite.
//!
//! TypeScript is the case where that costs the most. A system whose server is Go and whose client
//! SDK is TypeScript can hold the server to its specification and not the client, and the client
//! is the half facing outward.
//!
//! # What is generated, and what is not
//!
//! The suite is data and the runner is not. `suite.json` is the same canonical document
//! `--target ir` writes, and the `.ts` files beside it are byte-identical for every specification
//! — they are checked into this repository as `.ts` and copied out, so they are read and reviewed
//! as TypeScript rather than as strings inside a Rust emitter.
//!
//! That split is the whole design. A generator that built the runner line by line would put the
//! semantics of three-valued evaluation, ordering and scenario isolation into `format!` calls,
//! where nothing type-checks them and no TypeScript tool ever looks at them.
//!
//! # One file each, where the Go target concatenates
//!
//! [`go::emit`](crate::go::emit) joins four checked-in `.go` files into one `runtime.go`, because
//! three of them carry no `package` clause and are fragments rather than files. TypeScript has
//! real modules, so each source is emitted as itself and they import one another as
//! `./runtime.js`, the specifier `tsc` resolves under `moduleResolution: NodeNext`. Concatenating
//! them would mean merging their `import` statements here, which is the line-by-line generation
//! the split exists to rule out.
//!
//! # Read rather than inlined
//!
//! `suite.json` is a file `src/runtime.ts` reads, not a string constant, for the reason the Go
//! target embeds rather than inlines: a generated purpose reads `` `acd.backend.ConnectAgent`
//! answers `requested` `` — backticks — so a TypeScript template literal cannot hold it unescaped,
//! and an interpreted literal would need the whole document escaped into one unreadable line. As a
//! file it stays diffable, and `git diff` on a model change shows which scenarios moved.
//!
//! # A package, not a directory of sources
//!
//! The Go target writes no `go.mod`: a module path is the adopter's identity and the adopter owns
//! it. `package.json` is not the same thing. A directory of `.ts` files without one is consumable
//! by nothing, and an adopter writing the manifest by hand is the one choosing `"type"`,
//! `moduleResolution` and the test runner — three ways to make a correct runtime fail to run, none
//! of which look any different from a runtime that is wrong.

use crate::ConformanceSuite;

/// One file of the emitted package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TsArtifact {
    /// Where it goes, relative to the output directory.
    pub path: String,
    /// What it holds.
    pub contents: String,
}

/// The package directory every artifact is written under.
///
/// The same name the Go target uses and for the same reason: named for what it is rather than for
/// the system, so an adopter's import path does not change when the specification's name does.
pub const PACKAGE: &str = "essconform";

/// The suite as a TypeScript test package: the runner, the evaluator, the suite it runs, and the
/// manifest that makes the three of them something `npm test` can execute.
///
/// Deterministic: the same suite produces the same bytes, because every `.ts` file is a constant
/// and the only file that moves is the suite's own canonical JSON.
pub fn emit(suite: &ConformanceSuite) -> Result<Vec<TsArtifact>, crate::admission::AdmissionError> {
    let json = suite.to_canonical_json()?;
    let mut files = sources(RUNTIME_TS.to_owned());
    files.push(file("suite.json", json));
    files.push(file("package.json", manifest()));
    files.push(file("tsconfig.json", tsconfig()));
    files.push(file("README.md", readme(suite)));
    Ok(files)
}

/// [`emit`], plus the model-based explorer: `src/explore.ts`, the compact model it interprets as
/// `ir.json`, and the entry point and README that name it.
///
/// `ir` must be the model `suite` was synthesized from. The explorer refuses a package whose
/// `ir.json` does not hash to the suite's `spec_digest`, so a mismatch here is an unusable
/// package rather than a wrong one.
pub fn emit_with_model(
    suite: &ConformanceSuite,
    ir: &ess_compiler::EssIr,
) -> Result<Vec<TsArtifact>, crate::admission::AdmissionError> {
    Ok(with_model(emit(suite)?, ir))
}

/// [`emit_input`], plus the model-based explorer, as [`emit_with_model`] adds it.
pub fn emit_input_with_model(
    input: &crate::coverage::AdmittedInput,
    ir: &ess_compiler::EssIr,
) -> Result<Vec<TsArtifact>, crate::admission::AdmissionError> {
    Ok(with_model(emit_input(input)?, ir))
}

/// Adds the explorer to an emitted package: its source after the other sources, `ir.json` after
/// `suite.json`, one more export in `src/index.ts` and one more section in `README.md`.
fn with_model(files: Vec<TsArtifact>, ir: &ess_compiler::EssIr) -> Vec<TsArtifact> {
    let mut out = Vec::with_capacity(files.len() + 2);
    for mut artifact in files {
        let name = artifact
            .path
            .strip_prefix(&format!("{PACKAGE}/"))
            .unwrap_or_default()
            .to_owned();
        match name.as_str() {
            "src/index.ts" => {
                artifact.contents.push_str(EXPLORE_EXPORT);
                out.push(artifact);
            }
            "README.md" => {
                artifact.contents.push_str(EXPLORE_README);
                out.push(artifact);
            }
            "src/coordinate.ts" => {
                out.push(artifact);
                out.push(file("src/explore.ts", EXPLORE_TS.to_owned()));
            }
            "suite.json" => {
                out.push(artifact);
                out.push(file("ir.json", format!("{}\n", ir.to_compact_json())));
            }
            _ => out.push(artifact),
        }
    }
    out
}

/// The explorer, as written.
const EXPLORE_TS: &str = include_str!("explore.ts");

/// The line `src/index.ts` gains in a package that carries the explorer.
const EXPLORE_EXPORT: &str = "export * from './explore.js';\n";

/// The README section a package that carries the explorer gains.
const EXPLORE_README: &str = r#"
## Random command sequences

`explore` runs seeded random sequences of commands against fresh targets built by the same
factory `run` takes, and checks every step against a reference model interpreted from `ir.json`:
the outcome, the error, the direct events and their determined payload fields, every view without
parameters (polling an `eventual` one), identity uniqueness and every invariant. A disagreement is
shrunk to a shorter trace that still fails the same way.

```ts
// src/explore.test.ts
import { test } from "node:test";
import { assertExplored, explore } from "./index.js";
import { newTarget } from "./my-implementation.js";

await test("random command sequences", async () => {
  const result = await explore(() => newTarget(), { seeds: 200, steps: 60 });
  assertExplored(result);
});
```

`seeds` sequences are seeded `1…seeds`; `seed` runs exactly one of them, which is how a reported
failure is replayed. `assertExplored` fails on a disagreement, on a declared outcome no sequence
reached, and on the outcomes of a command the explorer left out (`excluded` says why) unless
`{ allowExcluded: true }` accepts them. A view field is compared with the entity field of the same
name. `ir.json` must hash to `suite.json`'s `spec_digest`; regenerate both together.
"#;

/// Emit an immutable suite/5 input with all exact original ancestors.
pub fn emit_input(
    input: &crate::coverage::AdmittedInput,
) -> Result<Vec<TsArtifact>, crate::admission::AdmissionError> {
    let suite = input.selected();
    let mut files = sources(RUNTIME_TS.replace(SUITE_DOCUMENT, INPUT_DOCUMENT));
    files.push(file("suite.json", suite.original_json().into()));
    files.push(file("input.json", input.document().to_canonical_json()?));
    files.push(file("package.json", manifest()));
    files.push(file("tsconfig.json", tsconfig()));
    files.push(file(
        "README.md",
        format!(
            "{}\nCoverage input requires explicit ESS_REPORT_FORMAT=2. The embedded input retains the selected original bytes and every parent.\n",
            readme(suite.suite())
        ),
    ));
    Ok(files)
}

/// One artifact of the package, at its path under [`PACKAGE`].
fn file(name: &str, contents: String) -> TsArtifact {
    TsArtifact {
        path: format!("{PACKAGE}/{name}"),
        contents,
    }
}

/// Every hand-written source, in the order a reader meets them, with `src/runtime.ts` as given.
///
/// `coordinate.ts` is the one the Go target reaches outside its own directory, at
/// `crates/specify/ess-domain/src/reading/coordinate.go` beside the Rust that reads the same
/// thing (`src/go/mod.rs:202`). The TypeScript port of it is in this directory instead, so a
/// reader of `src/ts/` sees the whole emitted runtime in one place.
///
/// `*.test.ts` beside these is deliberately not emitted, the same way the Go target emits no
/// `*_test.go`: those are this repository's checks on its own runtime, and an adopter running
/// `npm test` is asking about their implementation, not about ours.
fn sources(runtime: String) -> Vec<TsArtifact> {
    vec![
        file("src/runtime.ts", runtime),
        file("src/predicate.ts", include_str!("predicate.ts").to_owned()),
        file("src/response.ts", include_str!("response.ts").to_owned()),
        file("src/fixtures.ts", include_str!("fixtures.ts").to_owned()),
        file("src/reading.ts", include_str!("reading.ts").to_owned()),
        file(
            "src/coordinate.ts",
            include_str!("coordinate.ts").to_owned(),
        ),
        file("src/index.ts", INDEX_TS.to_owned()),
    ]
}

/// The runtime as written, before any coverage rewrite.
const RUNTIME_TS: &str = include_str!("runtime.ts");

/// How `runtime.ts` reads the suite, and the one thing [`emit_input`] rewrites.
///
/// The Go target does the same by replacing `go:embed suite.json` in the file that carries the
/// directive, and it has to be the file that really reads the document rather than a shim beside
/// it. An earlier draft of this module emitted its own `src/suite.ts` exporting a `suiteJSON`
/// constant, rewrote *that* for the coverage carrier, and shipped a package whose runner went on
/// reading `suite.json` — the selection's parent — while the rewritten file was imported by
/// nothing. The counts it reported would have been of the wrong suite, and no verdict in it would
/// have been wrong on its own terms.
///
/// `../` reaches the package root from `src/` before compilation and from `dist/` after it, so one
/// source works whether the package is read where it was written or where it was built.
const SUITE_DOCUMENT: &str = "new URL('../suite.json', import.meta.url)";

/// What the coverage carrier's runtime reads instead.
const INPUT_DOCUMENT: &str = "new URL('../input.json', import.meta.url)";

/// What `import "essconform"` reaches.
///
/// Four modules, not one. `runtime.ts` re-exports none of the other three, so an entry point of
/// `export * from "./runtime.js"` alone leaves an adopter unable to name `ClockReadingTarget` —
/// the interface they have to implement for every clock-reading scenario — or
/// `ClockReadingEvidence`, which is what that method returns. Measured as `tsc` TS2724 against
/// this tree before the other three lines were added.
///
/// The Go target has no equivalent gap because it is one package and one namespace; a TypeScript
/// package has to say which modules its surface is.
///
/// `predicate.js` is deliberately absent: nothing an adopter writes names a predicate.
const INDEX_TS: &str = r"// The package entry point.
//
// Generated by `ess verify conform synthesize --target typescript`. Do not edit: change the
// specification and regenerate.
export * from './runtime.js';
export * from './coordinate.js';
export * from './reading.js';
export * from './response.js';
export type { FixtureContract } from './fixtures.js';
";

/// A JSON document written the way the other emitted manifests in this workspace are written.
fn pretty(value: &serde_json::Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(value).expect("typed metadata serializes")
    )
}

/// The npm manifest: ESM, no runtime dependencies, and `npm test` reaching `node --test`.
///
/// The compiler range is the one `website/package.json` already pins, so this repository has one
/// answer to which TypeScript its own emitted sources are written against.
fn manifest() -> String {
    pretty(&serde_json::json!({
        "name": PACKAGE,
        "version": "0.0.0",
        "private": true,
        "type": "module",
        "exports": {".": {"types": "./dist/index.d.ts", "import": "./dist/index.js"}},
        "types": "./dist/index.d.ts",
        // `node --test dist` is wrong and measured wrong: Node 22 reads a bare directory as a
        // module path and answers `Cannot find module`, having run nothing. The glob is what
        // selects the compiled cases.
        "scripts": {
            "build": "tsc --project tsconfig.json",
            "typecheck": "tsc --project tsconfig.json --noEmit",
            "test": "tsc --project tsconfig.json && node --test \"dist/*.test.js\""
        },
        "devDependencies": {"@types/node": "^22.0.0", "typescript": "~6.0.2"},
        "engines": {"node": ">=20.0"}
    }))
}

/// The compiler settings the emitted sources are written against.
///
/// `NodeNext` is not a preference. The sources import one another as `./runtime.js` while being
/// `.ts` on disk, and that specifier resolves under no other module resolution.
///
/// `erasableSyntaxOnly` is the one that is not about correctness but about where the complaint
/// arrives. Node's type stripping refuses syntax it cannot erase — an enum, a namespace, a
/// constructor parameter property — with `ERR_UNSUPPORTED_TYPESCRIPT_SYNTAX` at run time, naming
/// no fix. Under this flag the compiler says it instead, at the line.
///
/// Every option here except `rootDir` and `outDir` is the same set that
/// `crates/verify/ess-conformance/src/ts/tsconfig.json` checks this repository's own copies of
/// these sources under, and a test below holds the two together. Sources checked under settings
/// looser than the adopter's compile under ours and not theirs.
fn tsconfig() -> String {
    pretty(&serde_json::json!({
        "compilerOptions": shared_compiler_options("src", Some("dist")),
        "include": ["src/**/*.ts"]
    }))
}

/// The compiler options the emitted package and this repository's own check share.
///
/// `types` is named, and has to be. An earlier version of this function left it out, arguing that
/// naming it only *restricts* the compiler and that `@types/node` would be discovered anyway.
/// That argument is wrong, and it was wrong in the direction that reaches an adopter: measured
/// against an emitted package with TypeScript 6.0.3 and `@types/node` installed beside it,
/// `tsc --noEmit` answers **14 errors** without this line — `TS2591 Cannot find name 'process'`
/// and `'Buffer'`, `TS2304 Cannot find name 'TextEncoder'` and `'URL'`, `TS2591` on the
/// `node:crypto` and `node:fs` imports — and **zero** with it. `lib` is `["ES2022"]` with no DOM,
/// so `TextEncoder` and `URL` have nowhere else to come from, and the runtime reads `process.env`,
/// writes a file and hashes the suite.
///
/// `skipLibCheck` is the other half of the same measurement. With `types` named and this absent,
/// the same package answers five `TS2307 Cannot find module 'undici-types'` from inside
/// `@types/node`'s own declarations — errors in a dependency's `.d.ts` that an adopter can do
/// nothing about and that say nothing about the emitted sources.
fn shared_compiler_options(root: &str, out: Option<&str>) -> serde_json::Value {
    let mut options = serde_json::json!({
        "target": "ES2022",
        "lib": ["ES2022"],
        "types": ["node"],
        "module": "NodeNext",
        "moduleResolution": "NodeNext",
        "strict": true,
        "noUncheckedIndexedAccess": true,
        "exactOptionalPropertyTypes": true,
        "erasableSyntaxOnly": true,
        "declaration": true,
        "skipLibCheck": true,
        "rootDir": root
    });
    if let Some(out) = out {
        options["outDir"] = serde_json::Value::String(out.to_owned());
    }
    options
}

/// How to wire the package up, written against this suite's own numbers.
fn readme(suite: &ConformanceSuite) -> String {
    let provenance = &suite.provenance;
    let scope = match &provenance.component {
        None => String::new(),
        Some(component) => format!(
            "\nScoped to the component `{component}`: only the scenarios whose every command, event \
             and view it accepts, publishes or owns. The scenarios the specification obliges of \
             another component are listed by `ess verify conform synthesize --component {component}` \
             as `outside:`, and belong in that component's suite.\n"
        ),
    };
    let readme = format!(
        r#"# `{PACKAGE}`

{count} scenario(s) synthesized from `{system} {version}`, spec digest `{digest}`.
{scope}
Generated by `ess verify conform synthesize --target typescript`. Nothing in this directory is
written by hand, and re-running the command after a change to the specification is the only way to
update it.

## Running it

This directory is a complete package. It has no runtime dependencies; its two development
dependencies are the TypeScript compiler and Node's own type declarations.

```console
npm install
npm test
```

`npm test` compiles `src/` into `dist/` and runs Node's own runner over `dist/*.test.js`, so a
file you add as `src/<name>.test.ts` is compiled and executed against the runtime.
`npm run typecheck` checks without emitting.

## Wiring it up

Implement `Target` — nine methods, each answering a question some construct in the specification
asks — and hand it over from one test file:

```ts
// src/conformance.test.ts
import {{ test }} from "node:test";
import {{ run }} from "./index.js";
import type {{ Target }} from "./index.js";
import {{ newTarget }} from "./my-implementation.js";

await test("conformance", async (t) => {{
  await run(t, (): Target => newTarget());
}});
```

`run` takes the test scope because it opens one subtest per scenario, and it builds one target per
scenario — scenario isolation is the suite's requirement, and a shared target would make it your
discipline instead. `runWith` is the same against a suite document you hand over rather than the
one beside the package.

A specification that declares a clock reading also asks for `ClockReadingTarget`, a second
interface beside `Target` whose method answers with `ClockReadingEvidence`: the facts your adapter
observed for one occurrence, which the runtime compares rather than recomputes. Both are reachable
from this package's entry point; implement it where the specification declares one.

## What to throw when you cannot answer

`ErrUnsupported`, not an ordinary error. A scenario whose semantic the implementation does not
expose is reported as skipped, which is a different fact from a failed one — `observeInvocations`
is the method most often in that position, and the specification explicitly does not require it.

Every method may say it, `executeCommand` included: a command whose actor is the implementation
itself has no caller a target can be, and answering for it would be the target deciding its own
verdict. Wrapping the sentinel (`new Error("...", {{ cause: ErrUnsupported }})`) is read the same
way, so the reason can carry which command it was.

## What this does not check

Whatever the synthesis refused. Read the refusal list `ess verify conform synthesize` prints: a
suite that quietly holds fewer checks than the specification requires is the failure this whole
thing exists to rule out, and unlike a refusal, nothing about it is visible in a passing run.

## The report

Set `ESS_REPORT_OUT` to a file path and `run` writes an `ess-conformance-report/1` there when the
last scenario has finished — the same closed document the Rust runner writes, so
`aep plan artifact evidence --from <that file>` records the run against the specification artifact
without anybody typing a count. `passed` means every scenario passed; a skipped scenario makes the
run `inconclusive`, because a target that could not answer a question has not shown the answer.

```console
ESS_REPORT_OUT=$PWD/report.json npm test
```
"#,
        count = suite.len(),
        system = provenance.system,
        version = provenance.specification_version,
        digest = provenance.spec_digest,
    );
    if provenance.suite_version.major() >= 5 {
        readme.replace(
            "Set `ESS_REPORT_OUT` to a file path and `run` writes an `ess-conformance-report/1` there when the",
            "Select `ESS_REPORT_FORMAT=2` explicitly before execution. Set `ESS_REPORT_OUT` to a file path\nand `run` writes an `ess-conformance-report/2` there when the",
        ).replace(
            "ESS_REPORT_OUT=$PWD/report.json npm test",
            "ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=$PWD/report.json npm test",
        )
    } else {
        readme
    }
}

#[cfg(test)]
mod tests {
    use ess_primitives::evidence::SpecDigest;

    use super::*;
    use crate::scenario::{SuiteFormat, SuiteProvenance};

    /// An empty suite, which is all these tests need: what they check is that the package does not
    /// move when the model does.
    fn suite() -> ConformanceSuite {
        let digest = |value: &str| SpecDigest::new(value).expect("a digest");
        ConformanceSuite::new(SuiteProvenance {
            suite_version: SuiteFormat::CURRENT,
            system: "billing".to_owned(),
            specification_version: "v3".to_owned(),
            spec_digest: digest("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            contract_digest: digest(
                "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
            ),
            component: None,
        })
    }

    /// The emitted package is the same bytes twice, and the TypeScript does not depend on the
    /// model.
    #[test]
    fn the_runner_is_a_constant_and_only_the_suite_moves() {
        let suite = suite();
        let first = emit(&suite).unwrap();
        let second = emit(&suite).unwrap();
        assert_eq!(first, second);

        let paths: Vec<&str> = first.iter().map(|file| file.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "essconform/src/runtime.ts",
                "essconform/src/predicate.ts",
                "essconform/src/response.ts",
                "essconform/src/fixtures.ts",
                "essconform/src/reading.ts",
                "essconform/src/coordinate.ts",
                "essconform/src/index.ts",
                "essconform/suite.json",
                "essconform/package.json",
                "essconform/tsconfig.json",
                "essconform/README.md",
            ]
        );
    }

    /// Every file the package carries spells the grouped commands the CLI help and the agent
    /// skills spell, for the unscoped suite and for a component's (beyond10x/ess#77).
    #[test]
    fn every_emitted_file_spells_the_grouped_commands() {
        let mut scoped = suite();
        scoped.provenance.component = Some("billing-service".to_owned());
        for suite in [suite(), scoped] {
            for file in emit(&suite).unwrap() {
                for flat in ["ess conform ", "aep artifact "] {
                    assert!(
                        !file.contents.contains(flat),
                        "{} spells the flat `{flat}`",
                        file.path
                    );
                }
            }
            let readme = readme(&suite);
            assert!(
                readme.contains("`ess verify conform synthesize"),
                "{readme}"
            );
            assert!(readme.contains("`aep plan artifact evidence"), "{readme}");
        }
    }

    /// Every emitted source is inside the directory `tsconfig.json` tells the compiler to read.
    ///
    /// A `.ts` file outside `rootDir` is not a compile error an adopter sees as one: `tsc` simply
    /// does not emit it, and the failure arrives at run time as a module that cannot be resolved.
    #[test]
    fn every_typescript_source_is_under_the_root_the_compiler_is_given() {
        let config: serde_json::Value = serde_json::from_str(&tsconfig()).expect("a tsconfig");
        let root = config["compilerOptions"]["rootDir"]
            .as_str()
            .expect("the tsconfig names a root");
        for file in emit(&suite()).unwrap() {
            if std::path::Path::new(&file.path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("ts"))
            {
                assert!(
                    file.path.starts_with(&format!("{PACKAGE}/{root}/")),
                    "{} is outside the compiler's root `{root}`",
                    file.path
                );
            }
        }
    }

    /// The runner itself reads the coverage input, not a shim beside it.
    ///
    /// The whole of `emit_input` rests on one string appearing in `runtime.ts` exactly once, and
    /// on that occurrence being the read the runner actually performs. If the runtime renames it,
    /// re-quotes it, or grows a second reader, the rewrite silently stops applying and the
    /// coverage package runs the selection's parent while reporting the selection's name. This is
    /// the assertion that makes that a failure here rather than a wrong count there.
    #[test]
    fn the_coverage_rewrite_lands_on_the_one_read_the_runner_performs() {
        assert_eq!(
            RUNTIME_TS.matches(SUITE_DOCUMENT).count(),
            1,
            "`runtime.ts` no longer reads the suite as `{SUITE_DOCUMENT}` exactly once, so \
             `emit_input` is rewriting nothing or rewriting too much"
        );
        let rewritten = RUNTIME_TS.replace(SUITE_DOCUMENT, INPUT_DOCUMENT);
        assert!(!rewritten.contains(SUITE_DOCUMENT));
        assert!(rewritten.contains(INPUT_DOCUMENT));
    }

    /// No emitted source is empty.
    ///
    /// An empty `.ts` beside this file is a placeholder somebody left behind, and the way this
    /// repository finds out today is `clippy::manual_string_new` going red on the `""` that
    /// `include_str!` yields — a lint about a Rust idiom, which names no TypeScript file and does
    /// not say the runtime is missing. It fired during this port while one source was still a
    /// placeholder. Named here instead, so the next one is reported as what it is.
    #[test]
    fn no_emitted_source_is_a_placeholder() {
        for file in emit(&suite()).unwrap() {
            if std::path::Path::new(&file.path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("ts"))
            {
                assert!(
                    !file.contents.trim().is_empty(),
                    "{} is emitted empty, so the package ships a module with no runtime in it",
                    file.path
                );
            }
        }
    }

    /// The emitted settings are the settings these sources are checked under.
    ///
    /// This repository compiles `src/ts/*.ts` under its own `tsconfig.json`, and an adopter
    /// compiles the same bytes under the emitted one. Where those two disagree, every check this
    /// repository runs is a check of a different program from the one that shipped — and the
    /// direction that costs is the loose one, which nothing downstream complains about.
    ///
    /// `rootDir` and `outDir` are the permitted difference and the only one: the repository's copy
    /// is flat and the emitted package has a `src/`.
    #[test]
    fn the_emitted_settings_are_the_settings_the_sources_are_checked_under() {
        let mut emitted: serde_json::Value =
            serde_json::from_str(&tsconfig()).expect("the emitted tsconfig is JSON");
        let mut checked: serde_json::Value =
            serde_json::from_str(include_str!("tsconfig.json")).expect("the repository's tsconfig");

        for options in [&mut emitted, &mut checked] {
            let options = options["compilerOptions"]
                .as_object_mut()
                .expect("a tsconfig names its compiler options");
            options.remove("rootDir");
            options.remove("outDir");
        }
        assert_eq!(
            emitted["compilerOptions"], checked["compilerOptions"],
            "the emitted compiler settings and the ones this repository checks its own sources \
             under have diverged"
        );
    }

    /// The manifest is a manifest: ESM, and `npm test` reaching Node's own runner.
    #[test]
    fn the_manifest_declares_the_module_system_and_the_runner_the_readme_names() {
        let declared: serde_json::Value = serde_json::from_str(&manifest()).expect("a manifest");
        assert_eq!(declared["type"], "module");
        assert_eq!(declared["name"], PACKAGE);
        let test = declared["scripts"]["test"].as_str().expect("a test script");
        assert!(test.contains("node --test"), "{test}");
    }
}
