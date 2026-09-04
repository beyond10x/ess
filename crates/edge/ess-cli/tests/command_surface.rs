//! The first level of `ess`, and the flat spelling of every verb under it.
//!
//! Two things a caller can observe and the derive cannot promise on its own. `ess --help` offers
//! the four areas and nothing else, and a verb spelled flat — `ess validate --path …`, which is
//! what every pinned caller and every published example says — prints the same bytes, on both
//! streams, with the same status, as the area path that replaced it. Byte-identical is the whole
//! claim: an alias that printed a deprecation line would be a different output and would break a
//! caller reading stdout.

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

/// The command names `--help` offers, without the wrapped remainder of their descriptions.
fn offered(help: &str) -> Vec<String> {
    help.split_once("Commands:\n")
        .expect("the help lists commands")
        .1
        .lines()
        .take_while(|line| !line.trim().is_empty())
        .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
        .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
        .collect()
}

#[test]
fn the_help_offers_exactly_the_four_areas() {
    let output = ess(&["--help"]);
    assert!(output.status.success(), "`ess --help` failed");
    let help = String::from_utf8(output.stdout).expect("the help is UTF-8");
    assert_eq!(
        offered(&help),
        ["specify", "generate", "verify", "infra"],
        "{help}"
    );
}

/// A flat spelling and the area path that replaced it, for verbs with fixtures in this tree.
///
/// One per area at least, and both verbs whose name is also an area name — `generate` and
/// `infra`, the two the grouping could not simply move.
const SPELLINGS: &[(&[&str], &[&str])] = &[
    (
        &["validate", "--path", "examples/billing"],
        &["specify", "validate", "--path", "examples/billing"],
    ),
    // A refusal, so the pair is held to its stderr and its exit status and not only to a report
    // that happens to be written on stdout.
    (
        &["validate", "--path", "examples/no-such-system"],
        &["specify", "validate", "--path", "examples/no-such-system"],
    ),
    (
        &["compile", "--path", "examples/billing", "--format", "json"],
        &[
            "specify",
            "compile",
            "--path",
            "examples/billing",
            "--format",
            "json",
        ],
    ),
    (
        &["graph", "--path", "examples/billing", "--format", "dot"],
        &[
            "specify",
            "graph",
            "--path",
            "examples/billing",
            "--format",
            "dot",
        ],
    ),
    (
        &["generate", "--path", "examples/billing", "--kind", "docs"],
        &[
            "generate",
            "generate",
            "--path",
            "examples/billing",
            "--kind",
            "docs",
        ],
    ),
    (
        &["conform", "synthesize", "--path", "examples/billing"],
        &[
            "verify",
            "conform",
            "synthesize",
            "--path",
            "examples/billing",
        ],
    ),
    (
        &[
            "schema",
            "validate",
            "crates/edge/ess-cli/tests/fixtures/schema-contract/instances",
            "--schemas",
            "crates/edge/ess-cli/tests/fixtures/schema-contract/registry",
        ],
        &[
            "generate",
            "schema",
            "validate",
            "crates/edge/ess-cli/tests/fixtures/schema-contract/instances",
            "--schemas",
            "crates/edge/ess-cli/tests/fixtures/schema-contract/registry",
        ],
    ),
    (
        &[
            "diff",
            "--from",
            "examples/billing",
            "--to",
            "examples/gatepass",
        ],
        &[
            "verify",
            "diff",
            "--from",
            "examples/billing",
            "--to",
            "examples/gatepass",
        ],
    ),
    (
        &[
            "infra",
            "diagnose",
            "--path",
            "examples/k3d-dev-cluster/observation.json",
        ],
        &[
            "infra",
            "infra",
            "diagnose",
            "--path",
            "examples/k3d-dev-cluster/observation.json",
        ],
    ),
    (
        &[
            "import",
            "openapi",
            "--path",
            "generated/openapi/invoice-service.yaml",
            "--format",
            "json",
        ],
        &[
            "infra",
            "import",
            "openapi",
            "--path",
            "generated/openapi/invoice-service.yaml",
            "--format",
            "json",
        ],
    ),
];

#[test]
fn a_flat_spelling_prints_what_its_area_path_prints() {
    for (flat, area) in SPELLINGS {
        let flat_run = ess(flat);
        let area_run = ess(area);
        let flat_spelling = flat.join(" ");
        let area_spelling = area.join(" ");
        assert_eq!(
            String::from_utf8_lossy(&flat_run.stdout),
            String::from_utf8_lossy(&area_run.stdout),
            "`ess {flat_spelling}` and `ess {area_spelling}` differ on stdout"
        );
        assert_eq!(
            String::from_utf8_lossy(&flat_run.stderr),
            String::from_utf8_lossy(&area_run.stderr),
            "`ess {flat_spelling}` and `ess {area_spelling}` differ on stderr"
        );
        assert_eq!(
            flat_run.status.code(),
            area_run.status.code(),
            "`ess {flat_spelling}` and `ess {area_spelling}` differ on exit status"
        );
        assert!(
            flat_run.stdout == area_run.stdout && flat_run.stderr == area_run.stderr,
            "`ess {flat_spelling}` and `ess {area_spelling}` differ in bytes that render the same"
        );
    }
}

/// A flat spelling and its area path, for a refusal **clap** writes rather than the program.
///
/// One per area, and one for the area whose name is also a verb. Each is a required argument left
/// out, which is the refusal a caller is most likely to see.
const REFUSALS: &[(&[&str], &[&str])] = &[
    (&["compose"], &["specify", "compose"]),
    (&["stack", "resolve"], &["generate", "stack", "resolve"]),
    (&["diff"], &["verify", "diff"]),
    (&["import", "openapi"], &["infra", "import", "openapi"]),
    (&["infra", "diff"], &["infra", "infra", "diff"]),
];

/// The `Usage:` line of a refusal, and everything else in it, separately.
fn usage_and_rest(stderr: &str) -> (String, String) {
    let usage = stderr
        .lines()
        .find(|line| line.starts_with("Usage: "))
        .unwrap_or_default()
        .to_owned();
    let rest = stderr
        .lines()
        .filter(|line| !line.starts_with("Usage: "))
        .collect::<Vec<_>>()
        .join("\n");
    (usage, rest)
}

/// What a clap-written refusal keeps between the two spellings, and the one thing it does not.
///
/// Byte-identity is a claim about a command that **runs**. clap writes the usage line out of the
/// path it was invoked by, so a refusal it writes names the spelling the caller typed — which is
/// what the flat spelling did before the areas existed, and what it should keep doing. Everything
/// else in the refusal, and the exit status, is the same for both.
#[test]
fn a_clap_refusal_differs_only_in_its_usage_line() {
    for (flat, area) in REFUSALS {
        let flat_run = ess(flat);
        let area_run = ess(area);
        let flat_spelling = flat.join(" ");
        let area_spelling = area.join(" ");
        let (flat_usage, flat_rest) = usage_and_rest(&String::from_utf8_lossy(&flat_run.stderr));
        let (area_usage, area_rest) = usage_and_rest(&String::from_utf8_lossy(&area_run.stderr));

        assert_eq!(
            flat_run.status.code(),
            Some(2),
            "`ess {flat_spelling}` is a clap refusal and exits 2"
        );
        assert_eq!(
            flat_run.status.code(),
            area_run.status.code(),
            "`ess {flat_spelling}` and `ess {area_spelling}` differ on exit status"
        );
        assert_eq!(
            flat_rest, area_rest,
            "`ess {flat_spelling}` and `ess {area_spelling}` differ on stderr outside the usage line"
        );
        assert_eq!(
            String::from_utf8_lossy(&flat_run.stdout),
            String::from_utf8_lossy(&area_run.stdout),
            "`ess {flat_spelling}` and `ess {area_spelling}` differ on stdout"
        );
        assert!(
            flat_usage.starts_with(&format!("Usage: ess {flat_spelling}")),
            "the usage line names the path that was typed, and reads `{flat_usage}`"
        );
        assert!(
            area_usage.starts_with(&format!("Usage: ess {area_spelling}")),
            "the usage line names the path that was typed, and reads `{area_usage}`"
        );
        assert_ne!(
            flat_usage, area_usage,
            "the two usage lines are the one documented difference; if they are equal the \
             reference page and the CHANGELOG are wrong about it"
        );
    }
}

/// An argument the `generate` area accepts is one it acts on, or the invocation is refused.
///
/// The area carries the `generate` verb's arguments so `ess generate --path …` keeps parsing, and
/// it carries seven sibling verbs. Writing both means two things at once, and the base refused it
/// because `--path` was not an argument of `ess generate synthesize`'s parent. It still refuses.
#[test]
fn the_generate_area_refuses_its_arguments_beside_a_sibling_verb() {
    for arguments in [
        ["generate", "--path", "examples/billing", "synthesize"].as_slice(),
        ["generate", "--kind", "docs", "schema", "validate"].as_slice(),
    ] {
        let refusal = ess(arguments);
        assert_eq!(
            refusal.status.code(),
            Some(2),
            "`ess {}` says two things at once and is refused\nstdout: {}\nstderr: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&refusal.stdout),
            String::from_utf8_lossy(&refusal.stderr),
        );
    }

    assert!(
        ess(&["generate", "--path", "examples/billing", "--kind", "docs"])
            .status
            .success(),
        "the documented flat spelling still runs"
    );
    assert!(
        ess(&["generate", "synthesize", "--path", "examples/billing"])
            .status
            .success(),
        "the area path of a sibling verb still runs"
    );
}

/// `ess generate --help` offers the verb's options beside the area's subcommands.
///
/// Both are reachable there — `ess generate --path …` is the verb and `ess generate schema …` is a
/// sibling — so a help that lists only one half describes a command that does not exist.
#[test]
fn the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands() {
    let output = ess(&["generate", "--help"]);
    assert!(output.status.success(), "`ess generate --help` failed");
    let help = String::from_utf8(output.stdout).expect("the help is UTF-8");

    for option in ["--path", "--kind", "--include", "--out", "--format"] {
        assert!(
            help.contains(option),
            "`ess generate --help` does not offer `{option}`, which it accepts:\n{help}"
        );
    }
    assert_eq!(
        offered(&help),
        [
            "generate",
            "synthesize",
            "project",
            "schema",
            "build",
            "release",
            "stack",
            "deployment"
        ],
        "{help}"
    );
    assert!(
        help.lines()
            .any(|line| line.starts_with("Usage: ess generate") && line.contains("[OPTIONS]")),
        "the usage line denies the options the command takes:\n{help}"
    );
}
