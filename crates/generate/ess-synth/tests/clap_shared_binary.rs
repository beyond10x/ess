//! Components that declare one `cli.binary` generate one clap crate whose tree defines every
//! group of every such component, and whose `main.rs` dispatches exactly those groups
//! (story:components-sharing-a-cli-binary-generate-one-tree-with-every-group).
//!
//! The model is `tests/fixtures/clap-shared-binary/`: `audit-service` and `desk-service` both
//! declare `binary: desk`, with the groups `audit` and `visits`. Its files are `.yaml.in` so no
//! walker of the repository's models reads them.
//!
//! The binary-run case drives the executable `task clap-shared-binary` generates from that model
//! with `ess generate synthesize --target clap` and builds, named by `ESS_CLAP_SHARED_BINARY`: a
//! test here spawns no cargo (`crates/edge/ess-xtask/tests/no_test_builds.rs`).

use std::path::PathBuf;
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};

const STORY: &str = "story:components-sharing-a-cli-binary-generate-one-tree-with-every-group";

/// The variable naming the binary `task clap-shared-binary` built.
const BINARY: &str = "ESS_CLAP_SHARED_BINARY";

const SYSTEM: &str = include_str!("fixtures/clap-shared-binary/system.yaml.in");
const PASS: &str = include_str!("fixtures/clap-shared-binary/domains/pass.yaml.in");
const AUDIT: &str = include_str!("fixtures/clap-shared-binary/domains/audit.yaml.in");
const COMPONENTS: &str = include_str!("fixtures/clap-shared-binary/components.yaml.in");

fn fixture(documents: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("{STORY}: `{label}` is well formed: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("{STORY}: the fixture validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("{STORY}: the fixture resolves:\n{diagnostics}"))
}

fn emitted() -> Synthesis {
    let ir = fixture(&[
        ("system.yaml", SYSTEM),
        ("domains/pass.yaml", PASS),
        ("domains/audit.yaml", AUDIT),
        ("components.yaml", COMPONENTS),
    ]);
    synthesize_for(&ir, Target::Clap)
        .unwrap_or_else(|failure| panic!("{STORY}: clap emits: {}", failure.to_canonical_json()))
}

fn source(synthesis: &Synthesis, path: &str) -> String {
    synthesis
        .artifacts
        .get(path)
        .unwrap_or_else(|| {
            panic!(
                "{STORY}: `{path}` is emitted; the tree holds {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
        .clone()
}

/// The words `main.rs` dispatches at one level: every `Some(("<word>", …))` arm.
fn dispatched(main: &str, indent: &str) -> Vec<String> {
    let prefix = format!("{indent}Some((\"");
    main.lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .filter_map(|rest| rest.split_once('"').map(|(word, _)| word.to_owned()))
        .filter(|word| word != "completions")
        .collect()
}

#[test]
fn one_crate_carries_every_group_of_every_component_sharing_the_binary() {
    let synthesis = emitted();
    let manifests: Vec<&String> = synthesis
        .artifacts
        .keys()
        .filter(|path| path.ends_with("Cargo.toml"))
        .collect();
    assert_eq!(
        manifests,
        ["crates/desk-cli/Cargo.toml"],
        "{STORY}: one crate"
    );
    let tree = source(&synthesis, "crates/desk-cli/src/tree.rs");
    for word in ["desk", "audit", "record", "visits", "register", "expected"] {
        assert!(
            tree.contains(&format!("::clap::Command::new({word:?})")),
            "{STORY}: the tree defines `{word}`:\n{tree}"
        );
    }
    assert_eq!(
        tree.matches("::clap::Command::new(\"desk\")").count(),
        1,
        "{STORY}: one root for the one binary:\n{tree}"
    );
}

#[test]
fn main_dispatches_exactly_the_groups_the_tree_defines() {
    let synthesis = emitted();
    let tree = source(&synthesis, "crates/desk-cli/src/tree.rs");
    let main = source(&synthesis, "crates/desk-cli/src/main.rs");
    let groups = dispatched(&main, "        ");
    assert_eq!(
        groups,
        ["audit", "visits"],
        "{STORY}: the top-level arms:\n{main}"
    );
    for group in &groups {
        assert!(
            tree.contains(&format!(
                "            ::clap::Command::new({group:?})\n                \
                 .subcommand_required(true)"
            )),
            "{STORY}: `main.rs` dispatches `{group}` and the tree defines it as a group:\n{tree}"
        );
    }
}

/// The binary `task clap-shared-binary` built, or a failure naming the task.
fn binary() -> PathBuf {
    let path = std::env::var_os(BINARY).unwrap_or_else(|| {
        panic!(
            "{STORY}: {BINARY} is not set: run `task clap-shared-binary` and set it to the binary \
             that task writes"
        )
    });
    let path = PathBuf::from(path);
    assert!(
        path.is_file(),
        "{STORY}: {BINARY}={} names no binary: run `task clap-shared-binary`",
        path.display()
    );
    // CI hands the binary to the test shards in an artifact, which keeps no mode bits.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = std::fs::metadata(&path)
            .unwrap_or_else(|error| panic!("{STORY}: {BINARY}={}: {error}", path.display()))
            .permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions)
                .unwrap_or_else(|error| panic!("{STORY}: the binary is made executable: {error}"));
        }
    }
    path
}

/// Runs the generated binary; its exit code and its standard error.
fn run(arguments: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(binary())
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{STORY}: the generated binary runs: {error}"));
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn each_groups_command_runs_through_the_generated_binary() {
    // With nothing implemented, a dispatched word reaches its handler and is refused by name; a
    // word the tree lacks is a usage error (exit 2) before any handler runs.
    for (arguments, owed) in [
        (
            &["audit", "record", "--note", "arrived"][..],
            "desk.audit.RecordEntry",
        ),
        (
            &["visits", "register", "--building", "North"][..],
            "desk.pass.RegisterVisit",
        ),
        (&["visits", "expected"][..], "desk.pass.ExpectedVisits"),
    ] {
        let (code, stderr) = run(arguments);
        assert_eq!(
            code,
            Some(1),
            "{STORY}: `desk {}` reaches its handler: {stderr}",
            arguments.join(" ")
        );
        assert!(
            stderr.contains(&format!(
                "`{owed}` is an obligation nothing has implemented"
            )),
            "{STORY}: `desk {}` is dispatched to `{owed}`: {stderr}",
            arguments.join(" ")
        );
    }
    let (code, stderr) = run(&["lobby"]);
    assert_eq!(
        code,
        Some(2),
        "{STORY}: an undeclared group is refused: {stderr}"
    );
}
