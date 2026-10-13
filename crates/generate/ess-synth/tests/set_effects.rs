//! Set effects over filtered instances (ess/16, beyond10x/ess#167, #175) in generated seams.
//!
//! From beyond10x/ess#547 (`docs/design/set-effect-identity-filters.md`, D4) no code target refuses
//! a workspace for `instances:` or `affects:`: the command is an owed behaviour, its obligation
//! names the construct that keeps it owed, and its contract names the rows each set effect changes.
//! The re-key refusal outside Rust (ess/23, beyond10x/ess#429) has a different reason and stays.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{
    synthesize_for, CapabilityKind, Synthesis, SynthesisDisposition, SynthesisPlan, Target,
    TargetFailureCode,
};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");
const DELETES: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-deletes.yaml");
const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
const REKEY: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/identity-changing-updates.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("set-effects.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// `text` with one network component owning its domains, accepting every command and publishing
/// every event, so that a code target emits the command seams and not only the types.
fn served(text: &str) -> EssIr {
    let bare = ir(text);
    let list = |names: Vec<String>| names.join(", ");
    let component = format!(
        "components:\n  - component: set-service\n    owns: {{domains: [{}]}}\n    accepts: \
         {{commands: [{}]}}\n    publishes: {{events: [{}]}}\n    reached_by: network\n",
        list(bare.domains().keys().map(ToString::to_string).collect()),
        list(bare.commands().keys().map(ToString::to_string).collect()),
        list(bare.events().keys().map(ToString::to_string).collect()),
    );
    assert!(!text.contains("\ncomponents:"), "the model declares none");
    ir(&format!("{text}{component}"))
}

/// `set-effects.yaml` with `Invite` answering an identity no row carries, so that the set effect,
/// not the unknown identity, is what keeps it owed.
fn answered() -> String {
    let last = "              on_hold: true\nviews:";
    assert!(MODEL.contains(last), "{last}");
    MODEL.replacen(
        last,
        "              on_hold: true\n      - {name: no-such-session, unknown_instance: true, \
         error: demo.desk.NotOpen}\nviews:",
        1,
    )
}

/// [`answered`] with `Invite`'s entry moving its rows (ess/22, beyond10x/ess#229).
fn moving() -> String {
    let from = "            where: team == subject.team\n";
    let model = answered();
    assert!(model.contains(from), "{from}");
    model
        .replacen("format: ess/16", "format: ess/22", 1)
        .replacen(
            from,
            "            where: team == subject.team\n            moves: demo.desk.Session.park\n",
            1,
        )
}

/// One command a set effect keeps owed: the construct its obligation names and the clause its
/// contract carries for the rows it changes.
struct Owed {
    command: &'static str,
    construct: &'static str,
    clause: &'static str,
}

fn models() -> Vec<(&'static str, String, Vec<Owed>)> {
    vec![
        (
            "set-effects",
            answered(),
            vec![
                Owed {
                    command: "demo.desk.EndTeam",
                    construct: "`instances:`",
                    clause: "changes every `demo.desk.Session` where `team == input.team`, \
                             moving it along `end` (a row resting outside its `from` states is \
                             left as it is), setting `note`",
                },
                Owed {
                    command: "demo.desk.NoteTeam",
                    construct: "`instances:`",
                    clause: "changes every `demo.desk.Session` where `team == input.team`, \
                             setting `note`",
                },
                Owed {
                    command: "demo.desk.Invite",
                    construct: "`affects:`",
                    clause: "changes, beside its subject, every `demo.desk.Session` where \
                             `team == subject.team`, setting `on_hold`",
                },
            ],
        ),
        (
            "set-effects-moving",
            moving(),
            vec![Owed {
                command: "demo.desk.Invite",
                construct: "`affects:`",
                clause: "changes, beside its subject, every `demo.desk.Session` where \
                         `team == subject.team`, moving it along `park` (a row resting outside \
                         its `from` states is left as it is), setting `on_hold`",
            }],
        ),
        (
            "set-deletes",
            DELETES.to_owned(),
            vec![
                Owed {
                    command: "demo.auth.RevokeTokens",
                    construct: "`instances:`",
                    clause: "removes every `demo.auth.Token` where `",
                },
                Owed {
                    command: "demo.auth.DeleteUser",
                    construct: "`affects:`",
                    clause: "removes, beside its subject, every `demo.auth.Token` where \
                             `user_id == subject.user_id`",
                },
            ],
        ),
        (
            "set-each",
            EACH.to_owned(),
            // An `each:` entry validates only beside a `distinct:` over its member, which no
            // generated behaviour compares and which the plan meets first.
            vec![Owed {
                command: "demo.feed.RunSource",
                construct: "distinct list members",
                clause: "writes, beside its subject, one `demo.feed.SeenDocument` per element \
                         of `input.applied`: the row `doc.document_id` names, updated if held and \
                         created if not",
            }],
        ),
    ]
}

/// The obligation the plan records for `command`, with its reason and contract.
fn obligation(plan: &SynthesisPlan, command: &str) -> (String, String) {
    match plan.disposition_of(CapabilityKind::CommandBehavior, command) {
        Some(SynthesisDisposition::Obligation(owed)) => {
            (owed.reason.describes(), owed.contract.clone())
        }
        other => panic!("`{command}` stays an obligation, not {other:?}"),
    }
}

fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("set-effects-{name}-{}", std::process::id()))
}

/// Runs a Go tool in `directory` with nothing fetched from a network.
fn go_tool(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("CGO_ENABLED", "0")
        .output()
        .expect("the Go tool runs")
}

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Writes a synthesized tree under a fresh scratch directory.
fn written(label: &str, synthesis: &Synthesis) -> PathBuf {
    let tree = scratch(label);
    let _ = std::fs::remove_dir_all(&tree);
    for artifact in synthesis.artifacts.values() {
        let path = tree.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
    tree
}

/// Builds every package of a synthesized Go tree; `Err` carries the compiler's words.
fn go_builds(label: &str, synthesis: &Synthesis) -> Result<(), String> {
    let tree = written(&format!("go-{label}"), synthesis);
    let built = go_tool(&tree, &["build", "./..."]);
    let _ = std::fs::remove_dir_all(&tree);
    if built.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            String::from_utf8_lossy(&built.stdout),
            String::from_utf8_lossy(&built.stderr)
        ))
    }
}

/// Type-checks a synthesized Rust workspace offline; `Err` carries the compiler's words. The models
/// share one target directory under this test binary's own scratch, so their dependencies are
/// checked once.
fn rust_checks(label: &str, synthesis: &Synthesis) -> Result<(), String> {
    let tree = written(&format!("rust-{label}"), synthesis);
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(["check", "--offline", "--workspace", "--quiet"])
        .current_dir(&tree)
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("set-effects-target"),
        )
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("RUSTC_WRAPPER")
        .output()
        .expect("cargo runs");
    let _ = std::fs::remove_dir_all(&tree);
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[test]
fn every_code_target_emits_a_set_effect_as_an_owed_behaviour() {
    for (label, text, owed) in models() {
        let ir = served(&text);
        for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
            let synthesis = synthesize_for(&ir, target).unwrap_or_else(|failure| {
                panic!(
                    "{label}: {target:?} emits the workspace: {}",
                    failure.to_canonical_json()
                )
            });
            for case in &owed {
                let (why, contract) = obligation(&synthesis.plan, case.command);
                assert!(
                    why.contains(case.construct),
                    "{label}: {target:?}: `{}` names {} as what kept it owed, not: {why}",
                    case.command,
                    case.construct
                );
                assert!(
                    contract.contains(case.clause),
                    "{label}: {target:?}: `{}`'s contract names the rows it changes \
                     ({}), not: {contract}",
                    case.command,
                    case.clause
                );
                assert!(
                    synthesis.plan.to_markdown().contains(case.clause),
                    "{label}: {target:?}: the plan document carries the clause"
                );
                // The seam an implementor fills states the rows in its own source.
                let source = match target {
                    Target::Rust => Some(".rs"),
                    Target::Go => Some(".go"),
                    _ => None,
                };
                if let Some(extension) = source {
                    assert!(
                        synthesis.artifacts.iter().any(|(path, artifact)| {
                            path.ends_with(extension) && artifact.contents.contains(case.clause)
                        }),
                        "{label}: {target:?}: the seam of `{}` carries the clause",
                        case.command
                    );
                }
            }
            if target == Target::Rust {
                rust_checks(label, &synthesis).unwrap_or_else(|words| {
                    panic!("{label}: the Rust workspace type-checks:\n{words}")
                });
            }
            if target == Target::Go && go_available() {
                go_builds(label, &synthesis)
                    .unwrap_or_else(|words| panic!("{label}: the Go tree builds:\n{words}"));
            }
        }
        // Each target's own `workspace` emits too, so a caller that skips `synthesize_for` is not
        // refused either.
        let plan = SynthesisPlan::of(&ir);
        let emitted = [
            ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
            ("go", ess_synth::go::workspace(&ir, &plan).err()),
            ("web", ess_synth::web::workspace(&ir, &plan).err()),
            ("clap", ess_synth::clap::workspace(&ir, &plan).err()),
        ];
        for (target, failure) in emitted {
            if let Some(failure) = failure {
                panic!(
                    "{label}: {target} emits the workspace: {}",
                    failure.to_canonical_json()
                );
            }
        }
    }
}

/// A re-key (ess/23, beyond10x/ess#429) is refused outside Rust for its own reason, which the
/// change above leaves standing: the seam writes the row its request names in place.
#[test]
fn a_rekey_outside_rust_is_still_refused() {
    const WRITE: &str = "commands.demo.vault.RenameSecret.outcomes.renamed.sets.name";
    let ir = ir(REKEY);
    for target in [Target::Go, Target::Web, Target::Clap] {
        let Err(failure) = synthesize_for(&ir, target) else {
            panic!("{target:?} emits a re-key it cannot represent");
        };
        assert!(
            failure.causes().iter().any(|cause| {
                cause.code() == TargetFailureCode::MissingRepresentation
                    && cause.sources() == [WRITE]
                    && cause.detail().contains("identity")
            }),
            "{target:?}: {}",
            failure.to_canonical_json()
        );
    }
    synthesize_for(&ir, Target::Rust).expect("the Rust target emits the re-key");
}
