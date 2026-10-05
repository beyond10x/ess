//! Adversary pass 1 against E-U9 (beyond10x/ess#200) on the generated lane.
//!
//! - A generated Rust view query binds each parameter to a local `param_<name>` before the body
//!   runs. A second parameter whose own name is `param_<first>` is then shadowed by the first's
//!   alias, and its string operator reads the first parameter's value.
//! - A byte-identity probe: every model in the repository, digested (IR, conformance suite and
//!   refusals, plans and artifacts for each target, `ess-gen` projections). It runs only when
//!   `E_U9_DIGEST_OUT` names a file to write; the adversary runs the same file at the base commit
//!   and at the unit's tree and compares the two outputs.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

const FIXTURE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/typed-text-operands.yaml");

fn compile_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    compile(&specification, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn adv_u9_a_parameter_named_like_another_parameters_alias_reads_its_own_value() {
    let model = FIXTURE.replace(
        "    params: [{name: q, type: String}]\n    filter: {name: {starts_with: {param: q}}}",
        "    params: [{name: q, type: String}, {name: param_q, type: String}]\n    filter: \
             {all: [{name: {starts_with: {param: q}}}, {note: {contains: {param: param_q}}}]}",
    );
    assert_ne!(model, FIXTURE, "the fixture's ByName view was rewritten");
    let ir = compile_text(&model);
    let rust = synthesize_for(&ir, Target::Rust).expect("Rust");
    assert_eq!(
        rust.plan.disposition_of(
            ess_synth::CapabilityKind::ViewQuery,
            "directory.people.ByName"
        ),
        Some(&ess_synth::SynthesisDisposition::Generated),
        "the two-parameter query is generated"
    );
    let behaviour = &rust.artifacts["crates/directory-types/src/behaviour.rs"].contents;
    let start = behaviour
        .find("fn by_name(&self")
        .unwrap_or_else(|| panic!("no by_name query:\n{behaviour}"));
    let body = &behaviour[start..];
    let body = &body[..body.find("\n    }\n").unwrap_or(body.len())];
    // Every `let <alias> = &<argument>;` must read an argument, never an alias bound above it.
    let mut aliases: Vec<String> = Vec::new();
    let mut shadowed = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        let Some(binding) = line.strip_prefix("let ") else {
            continue;
        };
        let Some((alias, read)) = binding.split_once(" = &") else {
            continue;
        };
        let read = read.trim_end_matches(';');
        if aliases.iter().any(|bound| bound == read) {
            shadowed.push(line.to_owned());
        }
        aliases.push(alias.to_owned());
    }
    assert_eq!(
        shadowed,
        Vec::<String>::new(),
        "a parameter alias reads an earlier alias, not its own argument:\n{body}"
    );
}

// ---- byte identity --------------------------------------------------------------------------

fn fnv(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}/{}", bytes.len())
}

fn skipped(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("target" | "node_modules" | ".git" | "website" | ".engineering")
        )
    })
}

fn yaml_under(directory: &Path, out: &mut Vec<PathBuf>, top: bool) {
    if skipped(directory) || (!top && directory.join("system.yaml").exists()) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            yaml_under(&path, out, false);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            out.push(path);
        }
    }
}

fn walk(directory: &Path, systems: &mut Vec<PathBuf>, singles: &mut Vec<PathBuf>, inside: bool) {
    if skipped(directory) {
        return;
    }
    let system = directory.join("system.yaml").exists();
    if system {
        systems.push(directory.to_path_buf());
    }
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk(&path, systems, singles, inside || system);
        } else if !inside
            && !system
            && path
                .extension()
                .is_some_and(|extension| extension == "yaml")
        {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text.lines().any(|line| line.starts_with("format: ess/"))
                && text.lines().any(|line| line.starts_with("system:"))
            {
                singles.push(path);
            }
        }
    }
}

fn assemble(root: &Path, files: &[PathBuf]) -> Result<EssIr, String> {
    let mut parsed = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(file).map_err(|error| error.to_string())?;
        let label = file
            .strip_prefix(root)
            .unwrap_or(file)
            .display()
            .to_string();
        let raw = RawSpecFile::parse(&text).map_err(|error| format!("parse {label}: {error}"))?;
        parsed.push((Source::new(label), raw));
    }
    let specification =
        Specification::assemble(parsed).map_err(|errors| format!("assemble: {errors}"))?;
    compile(&specification, &SourceMap::new()).map_err(|error| format!("compile: {error:?}"))
}

fn guarded<T>(step: impl FnOnce() -> T) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(step)).map_err(|panic| {
        panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
            .unwrap_or_else(|| "panic".to_owned())
    })
}

fn digest(ir: &EssIr) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert(
        "ir".to_owned(),
        match guarded(|| ir.to_canonical_json()) {
            Ok(json) => fnv(json.as_bytes()),
            Err(panic) => format!("PANIC {}", fnv(panic.as_bytes())),
        },
    );
    let conformance = guarded(|| {
        let synthesis = ess_conformance::synthesize::synthesize(ir);
        let mut text = synthesis
            .suite
            .to_canonical_json()
            .unwrap_or_else(|error| format!("ERR {error}"));
        for refusal in &synthesis.refusals {
            let _ = writeln!(text, "refusal {} {}", refusal.cause.code(), refusal.cause);
        }
        let _ = writeln!(text, "outside {}", synthesis.outside.len());
        text
    });
    out.insert(
        "suite".to_owned(),
        match conformance {
            Ok(text) => fnv(text.as_bytes()),
            Err(panic) => format!("PANIC {}", fnv(panic.as_bytes())),
        },
    );
    for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
        let generated = guarded(|| match synthesize_for(ir, target) {
            Ok(synthesis) => {
                let mut text = serde_json::to_string(&synthesis.plan).unwrap_or_default();
                text.push_str(&serde_json::to_string(&synthesis.target).unwrap_or_default());
                for (path, artifact) in &synthesis.artifacts {
                    let _ = writeln!(text, "\n== {path} {}", fnv(artifact.contents.as_bytes()));
                }
                text
            }
            Err(error) => format!("ERR {error:?}"),
        });
        out.insert(
            format!("synth-{}", target.name()),
            match generated {
                Ok(text) => fnv(text.as_bytes()),
                Err(panic) => format!("PANIC {}", fnv(panic.as_bytes())),
            },
        );
    }
    let projected = guarded(|| match ess_gen::generate_all(ir) {
        Ok(artifacts) => artifacts
            .iter()
            .fold(String::new(), |mut text, (path, artifact)| {
                let _ = writeln!(text, "{path} {}", fnv(artifact.contents.as_bytes()));
                text
            }),
        Err(error) => format!("ERR {error:?}"),
    });
    out.insert(
        "gen".to_owned(),
        match projected {
            Ok(text) => fnv(text.as_bytes()),
            Err(panic) => format!("PANIC {}", fnv(panic.as_bytes())),
        },
    );
    out
}

#[test]
fn adv_u9_byte_identity_digest() {
    let Some(output) = std::env::var_os("E_U9_DIGEST_OUT") else {
        eprintln!("E_U9_DIGEST_OUT unset: the byte-identity probe writes nothing");
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = root.canonicalize().expect("the repository root");
    let (mut systems, mut singles) = (Vec::new(), Vec::new());
    walk(&root, &mut systems, &mut singles, false);
    let mut lines = String::new();
    let mut models: Vec<(String, Vec<PathBuf>)> = Vec::new();
    for system in &systems {
        let mut files = Vec::new();
        yaml_under(system, &mut files, true);
        models.push((
            system
                .strip_prefix(&root)
                .unwrap_or(system)
                .display()
                .to_string(),
            files,
        ));
    }
    for single in &singles {
        models.push((
            single
                .strip_prefix(&root)
                .unwrap_or(single)
                .display()
                .to_string(),
            vec![single.clone()],
        ));
    }
    let only = std::env::var("E_U9_DIGEST_ONLY").ok();
    for (label, files) in &models {
        if only.as_deref().is_some_and(|only| !label.contains(only)) {
            continue;
        }
        match guarded(|| assemble(&root, files)) {
            Ok(Ok(ir)) => {
                for (kind, hash) in digest(&ir) {
                    let _ = writeln!(lines, "{label}\t{kind}\t{hash}");
                }
            }
            Ok(Err(error)) => {
                let _ = writeln!(lines, "{label}\tinvalid\t{}", fnv(error.as_bytes()));
            }
            Err(panic) => {
                let _ = writeln!(lines, "{label}\tpanic\t{}", fnv(panic.as_bytes()));
            }
        }
        eprintln!("digested {label}");
    }
    std::fs::write(&output, lines).expect("the digest writes");
}

/// The argument names of the generated query `signature_marker` opens, and its body.
fn query<'a>(source: &'a str, signature_marker: &str, end: &str) -> (Vec<String>, &'a str) {
    let start = source
        .find(signature_marker)
        .unwrap_or_else(|| panic!("no `{signature_marker}` in:\n{source}"));
    let text = &source[start..];
    let open = text.find('(').expect("a parameter list") + 1;
    let close = open + text[open..].find(')').expect("a closed parameter list");
    let arguments = text[open..close]
        .split(',')
        .filter_map(|argument| {
            let name = argument.trim().split([':', ' ']).next()?.trim();
            (!name.is_empty() && name != "&self").then(|| name.to_owned())
        })
        .collect();
    let body = &text[..text.find(end).map_or(text.len(), |at| at + end.len())];
    (arguments, body)
}

/// Every argument the body also calls as a function: `name(` after a non-identifier character.
fn called_arguments(arguments: &[String], body: &str) -> Vec<String> {
    arguments
        .iter()
        .filter(|argument| {
            body.match_indices(&format!("{argument}(")).any(|(at, _)| {
                body[..at].chars().last().is_none_or(|before| {
                    !(before.is_alphanumeric() || before == '_' || before == '.')
                })
            })
        })
        .cloned()
        .collect()
}

#[test]
fn adv_u9_a_parameter_named_like_a_guard_helper_does_not_shadow_it() {
    let mut shadowing = Vec::new();
    for (target, name, filter, marker, end) in [
        (
            Target::Rust,
            "all",
            "{all: [{name: {starts_with: {param: all}}}, {note: {contains: \"x\"}}]}",
            "fn by_name(&self",
            "\n    }\n",
        ),
        (
            Target::Rust,
            "any",
            "{any: [{name: {starts_with: {param: any}}}, {note: {contains: \"x\"}}]}",
            "fn by_name(&self",
            "\n    }\n",
        ),
        (
            Target::Go,
            "not",
            "{not: {name: {starts_with: {param: not}}}}",
            ") ByName(",
            "\n}\n",
        ),
    ] {
        let model = FIXTURE.replace(
            "    params: [{name: q, type: String}]\n    filter: {name: {starts_with: {param: q}}}",
            &format!("    params: [{{name: {name}, type: String}}]\n    filter: {filter}"),
        );
        assert_ne!(model, FIXTURE);
        let ir = compile_text(&model);
        let synthesis = synthesize_for(&ir, target).expect("synthesizes");
        assert_eq!(
            synthesis.plan.disposition_of(
                ess_synth::CapabilityKind::ViewQuery,
                "directory.people.ByName"
            ),
            Some(&ess_synth::SynthesisDisposition::Generated),
            "{target:?} {name}: the query is generated"
        );
        let path = match target {
            Target::Rust => "crates/directory-types/src/behaviour.rs",
            _ => "types/behaviour/behaviour.go",
        };
        let source = &synthesis.artifacts[path].contents;
        let (arguments, body) = query(source, marker, end);
        let called = called_arguments(&arguments, body);
        if !called.is_empty() {
            shadowing.push(format!(
                "{target:?}: argument(s) {called:?} shadow the helper the body calls:\n{body}"
            ));
        }
    }
    assert_eq!(shadowing, Vec::<String>::new());
}

/// The generated tree for `target` written under the build's temporary directory.
fn written(label: &str, synthesis: &ess_synth::Synthesis) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adv-e-u9-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in synthesis.artifacts.values() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
    root
}

fn renamed(name: &str, filter: &str) -> EssIr {
    let model = FIXTURE.replace(
        "    params: [{name: q, type: String}]\n    filter: {name: {starts_with: {param: q}}}",
        &format!("    params: [{{name: {name}, type: String}}]\n    filter: {filter}"),
    );
    assert_ne!(model, FIXTURE);
    compile_text(&model)
}

/// The generated Rust system compiles when a view parameter is named `all` and the filter is a
/// conjunction.
#[test]
fn adv_u9_generated_rust_with_a_parameter_named_all_compiles() {
    let ir = renamed(
        "all",
        "{all: [{name: {starts_with: {param: all}}}, {note: {contains: \"x\"}}]}",
    );
    let synthesis = synthesize_for(&ir, Target::Rust).expect("Rust");
    let tree = written("rust-all", &synthesis);
    let output = std::process::Command::new(
        std::env::var_os("CARGO").expect("Cargo supplies its executable"),
    )
    .args(["check", "--offline", "--workspace"])
    .current_dir(&tree)
    .env(
        "CARGO_TARGET_DIR",
        Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-e-u9-target"),
    )
    .env("CARGO_INCREMENTAL", "0")
    .env_remove("RUSTC_WRAPPER")
    .output()
    .expect("cargo runs");
    let log = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&tree);
    let errors: Vec<&str> = log
        .lines()
        .filter(|line| line.starts_with("error"))
        .collect();
    assert!(
        output.status.success(),
        "the generated Rust system does not compile: {errors:#?}\n{}",
        log.lines()
            .skip_while(|line| !line.starts_with("error"))
            .take(14)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The generated Go system builds when a view parameter is named `not` and the filter negates.
#[test]
fn adv_u9_generated_go_with_a_parameter_named_not_builds() {
    if std::process::Command::new("go")
        .arg("version")
        .output()
        .is_err()
    {
        eprintln!("no Go toolchain: unchecked");
        return;
    }
    let ir = renamed("not", "{not: {name: {starts_with: {param: not}}}}");
    let synthesis = synthesize_for(&ir, Target::Go).expect("Go");
    let tree = written("go-not", &synthesis);
    let output = std::process::Command::new("go")
        .args(["build", "./..."])
        .current_dir(&tree)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs");
    let log = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&tree);
    assert!(
        output.status.success(),
        "the generated Go system does not build:\n{log}"
    );
}

/// With parameters `q` and `param_q`, every argument of the generated Rust query is read: built
/// with warnings denied, an argument nothing reads is an error.
#[test]
fn adv_u9_generated_rust_reads_both_q_and_param_q() {
    let model = FIXTURE.replace(
        "    params: [{name: q, type: String}]\n    filter: {name: {starts_with: {param: q}}}",
        "    params: [{name: q, type: String}, {name: param_q, type: String}]\n    filter: \
         {all: [{name: {starts_with: {param: q}}}, {note: {contains: {param: param_q}}}]}",
    );
    assert_ne!(model, FIXTURE);
    let synthesis = synthesize_for(&compile_text(&model), Target::Rust).expect("Rust");
    let tree = written("rust-param-q", &synthesis);
    let output = std::process::Command::new(
        std::env::var_os("CARGO").expect("Cargo supplies its executable"),
    )
    .args(["check", "--offline", "--workspace"])
    .current_dir(&tree)
    .env(
        "CARGO_TARGET_DIR",
        Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-e-u9-target"),
    )
    .env("RUSTFLAGS", "-D warnings")
    .env("CARGO_INCREMENTAL", "0")
    .env_remove("RUSTC_WRAPPER")
    .output()
    .expect("cargo runs");
    let log = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&tree);
    assert!(
        output.status.success(),
        "{}",
        log.lines()
            .skip_while(|line| !line.starts_with("error"))
            .take(12)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
