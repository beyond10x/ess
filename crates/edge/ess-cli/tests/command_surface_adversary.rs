//! Adversarial cases against the four-area command surface.
//!
//! Three claims this tree makes about itself, driven against the binary it ships:
//!
//! 1. `crates/edge/ess-cli/src/main.rs` mounts the `generate` verb's arguments on the `generate`
//!    *area*, which also carries seven sibling subcommands. Nothing refuses the combination, so an
//!    argument a caller wrote is accepted and then discarded.
//! 2. `CHANGELOG.md` and `website/docs/reference/cli.md` both say a flat spelling and its area path
//!    print "the same bytes on stdout and stderr, the same exit status" and that "a caller reading
//!    the output cannot tell which one it ran".
//! 3. `ess generate --path …` is the spelling `README.md`, the CLI reference page, `ess-xtask`,
//!    agentide's gate and `ess_gen::provenance::REGENERATE` all use, while the binary's own usage
//!    line for that command name says it takes no options.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The workspace root, found by walking up rather than by counting `..`.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

/// One run of the built binary, from the workspace root so the fixture paths resolve.
fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs")
}

/// An accepted argument has an effect, or the invocation is refused.
///
/// `ess generate` carries `--kind`, `--out`, `--path`, `--include` and `--format` so that
/// `ess generate --path … --kind docs --out …` parses without a subcommand. It also carries seven
/// sibling subcommands, and nothing sets `args_conflicts_with_subcommands`, so both may be written
/// in one invocation. `run` then takes the subcommand branch and drops the arguments on the floor:
/// the command reports success while the `--out` directory the caller named is never touched.
#[test]
fn the_generate_area_honours_the_arguments_it_accepts_or_refuses_them() {
    let out = workspace_root()
        .join("target")
        .join(format!("adversary-generate-out-{}", std::process::id()));
    assert!(
        !out.exists(),
        "the case picks a path that does not exist yet"
    );
    let out_argument = out.to_str().expect("the scratch path is UTF-8").to_owned();

    let output = ess(&[
        "generate",
        "--kind",
        "docs",
        "--out",
        &out_argument,
        "schema",
        "validate",
        "crates/edge/ess-cli/tests/fixtures/schema-contract/instances",
        "--schemas",
        "crates/edge/ess-cli/tests/fixtures/schema-contract/registry",
    ]);

    let honoured = out.exists();
    let _ = std::fs::remove_dir_all(&out);
    assert!(
        !output.status.success() || honoured,
        "`ess generate --kind docs --out {out_argument} schema validate …` exited {:?} without \
         creating {out_argument}: the arguments were accepted and discarded\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// A `--path` written once is the path the command reads.
///
/// `ess generate --path examples/billing synthesize` names a specification and a verb that takes
/// one. Either the invocation means `ess generate synthesize --path examples/billing` or it is
/// refused; what it must not do is run `synthesize` against the default `.` as though the caller
/// had written no path at all.
#[test]
fn the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given() {
    let ambiguous = ess(&["generate", "--path", "examples/billing", "synthesize"]);
    let refused = ambiguous.status.code() == Some(2);
    if refused {
        return;
    }

    let explicit = ess(&["generate", "synthesize", "--path", "examples/billing"]);
    assert_eq!(
        String::from_utf8_lossy(&ambiguous.stdout),
        String::from_utf8_lossy(&explicit.stdout),
        "`ess generate --path examples/billing synthesize` exited {:?} and is neither refused nor \
         the same run as `ess generate synthesize --path examples/billing`\nstderr: {}",
        ambiguous.status.code(),
        String::from_utf8_lossy(&ambiguous.stderr),
    );
    assert_eq!(
        ambiguous.status.code(),
        explicit.status.code(),
        "`ess generate --path examples/billing synthesize` and \
         `ess generate synthesize --path examples/billing` differ on exit status"
    );
}

/// The byte-identity the CHANGELOG and the CLI reference page claim, on a refusal clap writes.
///
/// `crates/edge/ess-cli/tests/command_surface.rs` holds ten pairs to their stderr, but the one
/// refusal among them — a specification directory that does not exist — is an application refusal
/// that never mentions how the command was spelled. A refusal clap itself writes does mention it,
/// and a caller reading stderr can therefore tell exactly which spelling it ran.
#[test]
fn a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses() {
    for (flat, area) in [
        (["compose"].as_slice(), ["specify", "compose"].as_slice()),
        (
            ["runtime", "compile", "--path", "ess/runtime.yaml"].as_slice(),
            [
                "specify",
                "runtime",
                "compile",
                "--path",
                "ess/runtime.yaml",
            ]
            .as_slice(),
        ),
    ] {
        let flat_run = ess(flat);
        let area_run = ess(area);
        // The accepted contract (story revision 5): when clap itself refuses, only the `Usage:`
        // line may differ, because it names the path the caller typed. Everything else on stderr,
        // and the exit status, must be the same through either spelling.
        let without_usage = |bytes: &[u8]| -> String {
            String::from_utf8_lossy(bytes)
                .lines()
                .filter(|line| !line.starts_with("Usage: "))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(
            without_usage(&flat_run.stderr),
            without_usage(&area_run.stderr),
            "`ess {}` and `ess {}` differ on stderr beyond the `Usage:` line",
            flat.join(" "),
            area.join(" "),
        );
        assert_eq!(
            flat_run.status.code(),
            area_run.status.code(),
            "`ess {}` and `ess {}` exit differently",
            flat.join(" "),
            area.join(" "),
        );
    }
}

/// The usage line does not deny arguments the command accepts.
///
/// `ess generate --path examples/billing --kind docs` is what `README.md`, the CLI reference page,
/// `ess-xtask`'s drift check, agentide's gate and `ess_gen::provenance::REGENERATE` all spell, and
/// it parses. Hiding those arguments to keep `ess generate --help` short also removes them from
/// the usage line clap prints when the caller mistypes one, so the binary answers a typo in
/// `--kind` with a usage line saying `ess generate` takes no options at all.
#[test]
fn the_generate_usage_line_admits_the_arguments_the_command_takes() {
    assert!(
        ess(&["generate", "--path", "examples/billing", "--kind", "docs"])
            .status
            .success(),
        "`ess generate --path examples/billing --kind docs` is the documented spelling and runs"
    );

    let refusal = ess(&["generate", "--kidn", "docs"]);
    let stderr = String::from_utf8_lossy(&refusal.stderr).into_owned();
    let usage = stderr
        .lines()
        .find(|line| line.starts_with("Usage: ess generate"))
        .unwrap_or_else(|| panic!("the refusal carries a usage line: {stderr}"))
        .to_owned();
    assert!(
        usage.contains("[OPTIONS]"),
        "`ess generate` accepts `--path` and `--kind` and its usage line is `{usage}`"
    );
}
