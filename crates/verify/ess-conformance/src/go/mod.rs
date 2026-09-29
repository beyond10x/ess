//! The suite as a Go test package.
//!
//! `ess conform run` drives a target written in Rust, which is every implementation this workspace
//! can reach and none of the implementations adopters have. So an adopter's suite is regenerated on
//! every model change and has never executed — which is the "generated tests are green" failure one
//! step further back: not a suite that checks less than it claims, a suite that checks nothing
//! because nothing can run it.
//!
//! # What is generated, and what is not
//!
//! The suite is data and the runner is not. `suite.json` is the same canonical document
//! `--target ir` writes, and the three `.go` files beside it are byte-identical for every
//! specification — they are checked into this repository as `.go` and copied out, so they are read
//! and reviewed as Go rather than as strings inside a Rust emitter.
//!
//! That split is the whole design. A generator that built the runner line by line would put the
//! semantics of three-valued evaluation, ordering and scenario isolation into `format!` calls,
//! where nothing type-checks them and no Go tool ever looks at them.
//!
//! # Embedded rather than inlined
//!
//! `suite.json` is a file the package embeds with `go:embed`, not a string constant. A generated
//! purpose reads `` `acd.backend.ConnectAgent` answers `requested` `` — backticks — so a Go raw
//! string literal cannot hold it, and an interpreted literal would need the whole document escaped
//! into one unreadable line. As a file it stays diffable, and `git diff` on a model change shows
//! which scenarios moved.

use crate::ConformanceSuite;

/// One file of the emitted package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GoArtifact {
    /// Where it goes, relative to the output directory.
    pub path: String,
    /// What it holds.
    pub contents: String,
}

/// The package directory every artifact is written under.
///
/// Named for what it is rather than for the system, so an adopter's import path does not change
/// when the specification's name does.
pub const PACKAGE: &str = "essconform";

/// The suite as a Go test package: the runner, the evaluator, and the suite it runs.
///
/// Deterministic: the same suite produces the same bytes, because the three Go files are constants
/// and the fourth is the suite's own canonical JSON.
pub fn emit(suite: &ConformanceSuite) -> Result<Vec<GoArtifact>, crate::admission::AdmissionError> {
    crate::direct_response::refuse_generation(suite, "Go")?;
    crate::delivery_context::refuse_generation(suite, "Go")?;
    refuse_unadmitted(suite, "Go")?;
    let json = suite.to_canonical_json()?;
    let file = |name: &str, contents: String| GoArtifact {
        path: format!("{PACKAGE}/{name}"),
        contents,
    };

    Ok(vec![
        file("runtime.go", runtime()),
        file("predicate.go", include_str!("predicate.go").to_owned()),
        file("suite.go", SUITE_GO.to_owned()),
        file("suite.json", json),
        file("README.md", readme(suite)),
    ])
}

/// Emit an immutable suite/5 input with all exact original ancestors.
pub fn emit_input(
    input: &crate::coverage::AdmittedInput,
) -> Result<Vec<GoArtifact>, crate::admission::AdmissionError> {
    let suite = input.selected();
    crate::direct_response::refuse_generation(suite.suite(), "Go")?;
    crate::delivery_context::refuse_generation(suite.suite(), "Go")?;
    refuse_unadmitted(suite.suite(), "Go")?;
    let file = |name: &str, contents: String| GoArtifact {
        path: format!("{PACKAGE}/{name}"),
        contents,
    };
    let embed = SUITE_GO.replace("go:embed suite.json", "go:embed input.json");
    Ok(vec![
        file("runtime.go", runtime()),
        file("predicate.go", include_str!("predicate.go").into()),
        file("suite.go", embed),
        file("suite.json", suite.original_json().into()),
        file("input.json", input.document().to_canonical_json()?),
        file(
            "README.md",
            format!(
                "{}\nCoverage input requires explicit ESS_REPORT_FORMAT=2. The embedded input retains the selected original bytes and every parent.\n",
                readme(suite.suite())
            ),
        ),
    ])
}

/// [`emit`], plus the model-based explorer: `explore.go`, the compact model it interprets as
/// `ir.json`, and the README section that names it.
///
/// `ir` must be the model `suite` was synthesized from. The explorer refuses a package whose
/// `ir.json` does not hash to the suite's `spec_digest`, so a mismatch here is an unusable
/// package rather than a wrong one.
pub fn emit_with_model(
    suite: &ConformanceSuite,
    ir: &ess_compiler::EssIr,
) -> Result<Vec<GoArtifact>, crate::admission::AdmissionError> {
    Ok(with_model(emit(suite)?, ir))
}

/// [`emit_input`], plus the model-based explorer, as [`emit_with_model`] adds it.
pub fn emit_input_with_model(
    input: &crate::coverage::AdmittedInput,
    ir: &ess_compiler::EssIr,
) -> Result<Vec<GoArtifact>, crate::admission::AdmissionError> {
    Ok(with_model(emit_input(input)?, ir))
}

/// Adds the explorer to an emitted package: `explore.go` after `suite.go`, `ir.json` after
/// `suite.json`, and one more section in `README.md`. `explore.go` embeds `suite.json` itself, so
/// it reads the suite's digest from the same file in both package shapes.
fn with_model(files: Vec<GoArtifact>, ir: &ess_compiler::EssIr) -> Vec<GoArtifact> {
    let file = |name: &str, contents: String| GoArtifact {
        path: format!("{PACKAGE}/{name}"),
        contents,
    };
    let mut out = Vec::with_capacity(files.len() + 2);
    for mut artifact in files {
        let name = artifact
            .path
            .strip_prefix(&format!("{PACKAGE}/"))
            .unwrap_or_default()
            .to_owned();
        match name.as_str() {
            "suite.go" => {
                out.push(artifact);
                out.push(file("explore.go", EXPLORE_GO.to_owned()));
            }
            "suite.json" => {
                out.push(artifact);
                out.push(file("ir.json", format!("{}\n", ir.to_compact_json())));
            }
            "README.md" => {
                artifact.contents.push_str(EXPLORE_README);
                out.push(artifact);
            }
            _ => out.push(artifact),
        }
    }
    out
}

/// The explorer, as written.
const EXPLORE_GO: &str = include_str!("explore.go");

/// The README section a package that carries the explorer gains.
const EXPLORE_README: &str = r#"
## Random command sequences

`Explore` runs seeded random sequences of commands against fresh targets built by the same
factory `Run` takes, and checks every step against a reference model interpreted from `ir.json`:
the outcome, the error, the direct events and their determined payload fields, every view without
parameters (polling an `eventual` one), identity uniqueness and every invariant. A disagreement is
shrunk to a shorter trace that still fails the same way.

```go
func TestExplore(t *testing.T) {
    result, err := essconform.Explore(func() essconform.Target { return newTarget() },
        essconform.ExploreOptions{Seeds: 200, Steps: 60})
    if err != nil {
        t.Fatal(err)
    }
    essconform.AssertExplored(t, result, essconform.AssertOptions{})
}
```

`Seeds` sequences are seeded `1…Seeds`; `Seed` runs exactly one of them, which is how a reported
failure is replayed. `AssertExplored` fails on a disagreement, on a declared outcome no sequence
reached, and on the outcomes of a command the explorer left out (`Excluded` says why) unless
`AssertOptions{AllowExcluded: true}` accepts them. A view field is compared with the entity field
of the same name. `ir.json` must hash to `suite.json`'s `spec_digest`; regenerate both together.

An outcome declared `external:` is a choice the explorer may take: where it is eligible, a seeded
draw picks it or the ordinary branch, `ConfigureExternalOutcome` arranges it, and the step expects
it. Once a command has had a branch arranged in a sequence, the explorer stops expecting that
command's ordinary branch where an external one is eligible, so an arrangement may hold until the
scenario ends. `External` reports each external branch as `reached`, `unreached`, `excluded` or
`unarrangeable` (the target returned `ErrUnsupported` when asked to arrange it). An unarrangeable
branch is not a disagreement and not `Unreached`; `AssertExplored` fails on it unless
`AllowExcluded` is set. `External` is absent when the specification declares no external branch.

## Concurrent histories

`ExploreConcurrent` drives fresh targets from two to four clients at once and writes each run as
an `ess-history/1` document, one per seed, into `Out`. It does not judge them itself: it runs
`ess verify conform check-history --path <Path>` on each and reports the verdict, and for a
violation the checker's report with its shrunk history. `ess` must be on `PATH`; without it the
call fails with `ErrNoEss`, and nothing is skipped.

```go
func TestConcurrent(t *testing.T) {
    result, err := essconform.ExploreConcurrent(func() essconform.Target { return newTarget() },
        essconform.ConcurrentOptions{Path: "../spec", Out: t.TempDir(), Seeds: 200})
    if err != nil {
        t.Fatal(err)
    }
    essconform.AssertConcurrent(t, result)
}
```

Concurrency is simulated: a seed draws the workload and picks, at every tick of a logical clock,
which client invokes its next call or receives its answer, so one seed writes one history, byte
for byte, in Go and in TypeScript alike. A target that implements `InterleavedTarget` does part of
a call's work at its invoke and the rest at its return; any other target does all of it at the
return. A call that returns `ErrIndeterminate` (or wraps it, or `context.DeadlineExceeded`) is written
`Indeterminate` and counted; any other error fails the exploration. A command answered with
`ErrUnsupported` is left out, and `AssertConcurrent` fails on it unless
`ConcurrentOptions{AllowExcluded: true}` accepts it. `Clients` outside 2 to 4 is refused. `Unknown`
fails `AssertConcurrent` as a violation does.

A client's call may also be a read of one of the specification's views, drawn from the seed beside
the commands. A read of a `read_your_writes` view demands the token the client's own last answered
command returned (`ViewRequest.AtLeast`); a read of an `eventual` view demands none. The view is
asked at the read's return instant, and the identity of each row it answered is written as `rows`,
so `ess` holds the target to each view's declared consistency. A view answered with
`ErrUnsupported` is left out as a command is.

`ConcurrentOptions{Inject: true}` injects every fault the specification declares, and no other: a
second delivery (`RedeliverEvent`) of an event only `delivery: at_least_once` bindings react to; the
same request sent again, written with `retry_of` naming the first, for a command declaring
`replays:`; and an answer delayed past the client's wait or never arriving, written `Indeterminate`,
for a command declaring another `external:` branch. Each is a move the seed schedules, and
`ConcurrentResult.Injected` counts them by what declared them and names the branches they reached.
Nothing is injected into the prefix, and a target restart never is: nothing declares one. Each
call carries its own correlation, which only its retry repeats, so a target tells a retry from a new
call of the same input by it.
"#;

/// The one file that exists only to embed the other one.
const SUITE_GO: &str = r#"// The suite this package runs.
//
// Generated by `ess verify conform synthesize --target go`. Do not edit: change the specification
// and regenerate.
package essconform

import _ "embed"

// suiteJSON is the canonical suite document, embedded so the package needs no working directory
// and no build tag to find it.
//
//go:embed suite.json
var suiteJSON string
"#;

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
Generated by `ess verify conform synthesize --target go`. Nothing in this directory is written by
hand, and re-running the command after a change to the specification is the only way to update it.

## Wiring it up

Implement `Target` — nine methods, each answering a question some construct in the specification
asks — and hand it over from one test. `example.com/yourservice` stands for your own module path,
whatever `go.mod` names it, and `yourservice_test` for the package the test sits in:

```go
package yourservice_test

import (
    "testing"

    "example.com/yourservice/{PACKAGE}"
)

func TestConformance(t *testing.T) {{
    {PACKAGE}.Run(t, func() {PACKAGE}.Target {{ return newTarget() }})
}}
```

`Run` builds one target per scenario, because scenario isolation is the suite's requirement and a
shared target would make it your discipline instead.

## What to return when you cannot answer

`ErrUnsupported`, not an error. A scenario whose semantic the implementation does not expose is
reported as skipped, which is a different fact from a failed one — `ObserveInvocations` is the
method most often in that position, and the specification explicitly does not require it.

Every method may say it, `ExecuteCommand` included: a command whose actor is the implementation
itself has no caller a target can be, and answering for it would be the target deciding its own
verdict. Wrapping the sentinel (`fmt.Errorf("...: %w", essconform.ErrUnsupported)`) is read the
same way, so the reason can carry which command it was.

## What this does not check

Whatever the synthesis refused. Read the refusal list `ess verify conform synthesize` prints: a
suite that quietly holds fewer checks than the specification requires is the failure this whole
thing exists to rule out, and unlike a refusal, nothing about it is visible in a passing run.

## The report

Set `ESS_REPORT_OUT` to a file path and `Run` writes an `ess-conformance-report/1` there when the
last scenario has finished — the same closed document the Rust runner writes, so
`aep plan artifact evidence --from <that file>` records the run against the specification artifact
without anybody typing a count. `passed` means every scenario passed; a skipped scenario makes the
run `inconclusive`, because a target that could not answer a question has not shown the answer.

```console
ESS_REPORT_OUT=$PWD/report.json go test ./...
```
"#,
        count = suite.len(),
        system = provenance.system,
        version = provenance.specification_version,
        digest = provenance.spec_digest,
    );
    if requires_report_format_2(provenance.suite_version) {
        readme
            .replace(
                "## What to return when you cannot answer",
                &format!(
                    "## Running it\n\n{}## What to return when you cannot answer",
                    report_format_requirement(provenance.suite_version, "Run", &[], "go test ./...")
                ),
            )
            .replace(
                "Set `ESS_REPORT_OUT` to a file path and `Run` writes an `ess-conformance-report/1` there when the",
                "Set `ESS_REPORT_OUT` to a file path and `Run` writes an `ess-conformance-report/2` there when the",
            )
            .replace(
                "ESS_REPORT_OUT=$PWD/report.json go test ./...",
                "ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=$PWD/report.json go test ./...",
            )
    } else {
        readme
    }
}

/// The newest suite major the generated Go and TypeScript runners admit.
///
/// The Rust side of `newestSuiteMajor` in `runtime.go` and of the `SUITE_MAJORS` table in
/// `runtime.ts`. `/28` and `/29` carry direct-return observations neither runner executes, so a
/// package for them would be refused by its own runner at admission; [`refuse_unadmitted`] refuses
/// it at generation instead, and `tests/generated_docs.rs` runs both emitted runners over every
/// major and fails when either disagrees (beyond10x/ess#186).
pub(crate) const NEWEST_ADMITTED_SUITE_MAJOR: u32 = 27;

/// The oldest suite major the generated runners execute only under an explicit
/// `ESS_REPORT_FORMAT=2`: `/5` through `/7` and `/8` onwards, the two gates in `Run` / `runWith`.
pub(crate) const REPORT_FORMAT_2_FROM_SUITE_MAJOR: u32 = 5;

/// `true` when the generated runners stop before the first scenario without `ESS_REPORT_FORMAT=2`.
///
/// The one answer the Go and TypeScript READMEs both state, and the one
/// `tests/generated_docs.rs` holds against both runners' observed behaviour.
pub(crate) fn requires_report_format_2(version: crate::scenario::SuiteFormat) -> bool {
    (REPORT_FORMAT_2_FROM_SUITE_MAJOR..=NEWEST_ADMITTED_SUITE_MAJOR).contains(&version.major())
}

/// Refuses a package whose suite version its own generated runner would refuse at admission.
///
/// A README is a promise that the package runs; writing one, with run instructions, for a suite
/// the runner answers with `unsupported suite version` is the drift beyond10x/ess#186 closes.
pub(crate) fn refuse_unadmitted(
    suite: &ConformanceSuite,
    target: &str,
) -> Result<(), crate::admission::AdmissionError> {
    let version = suite.provenance.suite_version;
    if version.major() > NEWEST_ADMITTED_SUITE_MAJOR {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedTarget",
            "$.provenance.suite_version",
            format!(
                "the generated {target} runner admits suite versions up to \
                 `ess-conformance/{NEWEST_ADMITTED_SUITE_MAJOR}` and would refuse `{version}`; \
                 use the Rust runner"
            ),
        ));
    }
    Ok(())
}

/// The README's statement that this suite runs only under `ESS_REPORT_FORMAT=2`, with the
/// commands that run it.
///
/// Part of the run instructions rather than of "The report", because it is not a report option:
/// without it the run stops before the first scenario whether or not a report was asked for, and
/// two of two agents in a trial round read it as one (beyond10x/ess#186).
pub(crate) fn report_format_requirement(
    version: crate::scenario::SuiteFormat,
    runner: &str,
    setup: &[&str],
    command: &str,
) -> String {
    let commands = setup
        .iter()
        .map(|line| format!("{line}\n"))
        .chain(std::iter::once(format!("ESS_REPORT_FORMAT=2 {command}\n")))
        .collect::<String>();
    format!(
        "This suite is `{version}`, and the runner executes suite/{REPORT_FORMAT_2_FROM_SUITE_MAJOR} \
         and later only with\n\
         `ESS_REPORT_FORMAT=2` set in the environment. Without it `{runner}` stops before the first\n\
         scenario (`… require explicit ESS_REPORT_FORMAT=2 before execution`), whatever the target\n\
         does, so set it for every run, not only when you want a report:\n\
         \n\
         ```console\n\
         {commands}\
         ```\n\
         \n"
    )
}

fn runtime() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        include_str!("runtime.go"),
        include_str!("reading.go"),
        include_str!("response.go"),
        include_str!("replay.go"),
        include_str!("fixtures.go"),
        include_str!("../../../../specify/ess-domain/src/reading/coordinate.go")
    )
}

#[cfg(test)]
mod tests {
    use ess_primitives::evidence::SpecDigest;

    use super::*;
    use crate::scenario::{SuiteFormat, SuiteProvenance};

    /// An empty suite, which is all these two tests need: what they check is that the runner does
    /// not move when the model does.
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

    /// The emitted package is the same bytes twice, and the Go files do not depend on the model.
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
                "essconform/runtime.go",
                "essconform/predicate.go",
                "essconform/suite.go",
                "essconform/suite.json",
                "essconform/README.md",
            ]
        );
    }

    /// The wiring example names no project of its own: the generator cannot know the adopter's
    /// module path, so it writes a placeholder that reads as one (beyond10x/ess#76).
    #[test]
    fn the_readme_wires_the_package_into_a_placeholder_module() {
        let readme = emit(&suite())
            .unwrap()
            .into_iter()
            .find(|file| file.path == "essconform/README.md")
            .expect("a README");
        assert!(
            readme.contents.contains("package yourservice_test"),
            "{}",
            readme.contents
        );
        assert!(
            readme
                .contents
                .contains("\"example.com/yourservice/essconform\""),
            "{}",
            readme.contents
        );
        assert!(!readme.contents.contains("acd"), "{}", readme.contents);
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

    /// Every emitted Go file declares the package the README tells an adopter to import.
    #[test]
    fn every_go_file_is_in_the_package_the_readme_names() {
        for file in emit(&suite()).unwrap() {
            if std::path::Path::new(&file.path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("go"))
            {
                assert!(
                    file.contents.contains(&format!("package {PACKAGE}")),
                    "{} does not declare the package",
                    file.path
                );
            }
        }
    }
}
