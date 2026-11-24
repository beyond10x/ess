//! Everything synthesis writes for every repository model, byte for byte
//! (`story:synthesis-reads-selection-plan`, unit 1).
//!
//! Two tables, each opening with a header stating how many models it pins and how many lines
//! follow, written at the story's base, before synthesis read the selection plan, so a digest that
//! moves is the plan reaching the suite's bytes:
//!
//! - `tests/fixtures/synthesis-suite-bytes-table.tsv`, one line per model through
//!   [`synthesize`](ess_conformance::synthesize::synthesize): the stage it stopped at, or the count
//!   and digest of its whole suite's canonical JSON, claimable scenarios kept, and of its refusals,
//!   notes and out-of-scope scenarios, each as it prints. The last is empty for a whole-system
//!   suite by construction; the column states that rather than leaving it unread.
//! - `tests/fixtures/synthesis-component-suites-table.tsv`, one line per component of every model
//!   declaring one, through [`synthesize_for`](ess_conformance::synthesize::synthesize_for): the
//!   component's suite and the scenarios it leaves to another component. Its refusals are the whole
//!   system's, which the first table pins.
//!
//! `repository_model_suites_change_only_where_claimed` (`tests/external_beside_held_guard.rs`)
//! strips the scenarios forcing an external branch beside a guard that reads a row before its
//! digest; these tables keep them, so they are the check that a change to the order synthesis
//! refutes siblings in moved no byte. The models the #464 claim search decides a witness on, which
//! `external_beside_held_guard.rs` and `adversary_464_pass1.rs` build in the test, are written out
//! under `tests/fixtures/claim-search/` so the walker reads them.
//!
//! Not pinned here: `synthesize_with_seeds` and `synthesize_for_with_seeds`, which need admitted
//! seeds no repository model carries beside it, and `authored::compile` (with the `not_taken`
//! claim check of an authored step), which needs authored scenario sources the walker does not
//! pair with a model. Their own tests hold them.
//!
//! `SYNTHESIS_SUITE_BYTES_TABLE_WRITE=<path>` and `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE=<path>`
//! write their table, header included, to `<path>` instead of comparing it, and print each model's
//! synthesis time; after it, the test passes with nothing else edited. The walker is that of
//! `crates/generate/ess-entity-runtime/tests/lowered_definitions_table.rs`: each ESS source file
//! alone, and each directory holding a `system.yaml` as one system, as the `ess` CLI reads it.
//!
//! The test step synthesizes nothing here: `the_synthesis_sources_digest_matches_the_committed_stamp`
//! compares a SHA-256 over the `src/` of the crates synthesis runs through and over every model
//! file with `tests/fixtures/synthesis-sources.digest`. The tests that re-synthesize every model
//! are marked for the slow lane (`task test-slow`) and compare the bytes with the tables.
//! `SYNTHESIS_SOURCES_STAMP_WRITE=<path>` writes the stamp to `<path>` instead of comparing it:
//! `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/\
//! synthesis-sources.digest" cargo test -p ess-conformance --test synthesis_suite_bytes_table
//! the_synthesis_sources_digest_matches_the_committed_stamp`.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Instant;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, synthesize_for, with_jobs};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const TABLE: &str = include_str!("fixtures/synthesis-suite-bytes-table.tsv");
const COMPONENT_TABLE: &str = include_str!("fixtures/synthesis-component-suites-table.tsv");

/// A table's first line: how many models it pins and how many lines follow it, written with the
/// table, so a table cut short or a line lost fails against its own count.
const HEADER: &str = "#\tmodels=";

/// How to re-pin the whole-suite table: the write mode writes the header with the table, so
/// nothing else is edited.
const REPIN: &str = "re-pin from the repository root with \
     SYNTHESIS_SUITE_BYTES_TABLE_WRITE=\"$PWD/crates/verify/ess-conformance/tests/fixtures/\
     synthesis-suite-bytes-table.tsv\" cargo test -p ess-conformance --locked --test \
     synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes, \
     which writes the header counts with the table; nothing else \
     needs editing";

/// How to re-pin the component table.
const COMPONENT_REPIN: &str = "re-pin from the repository root with \
     SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE=\"$PWD/crates/verify/ess-conformance/tests/fixtures/\
     synthesis-component-suites-table.tsv\" cargo test -p ess-conformance --locked --test \
     synthesis_suite_bytes_table every_repository_component_keeps_its_synthesized_bytes, \
     which writes the header counts with the table; nothing else \
     needs editing";

/// The models these tables do not synthesize, each pinned as `EXCLUDED` in the whole-suite table:
/// synthesis of `conditional-measures-generated.yaml` takes minutes in a debug build (about 224
/// seconds measured). Its suite bytes are pinned by the slow probe
/// `adversary_244b_window_free_bytes` (`#[ignore = "slow lane: ..."]`, run by `task test-slow`), as `external_beside_held_guard.rs`
/// excludes it.
const UNAFFORDABLE: &[&str] =
    &["crates/generate/ess-synth/tests/fixtures/conditional-measures-generated.yaml"];

/// The counts the header line states.
fn header(table: &str, repin: &str) -> (usize, usize) {
    let first = table.lines().next().unwrap_or_default();
    let counts = first
        .strip_prefix(HEADER)
        .and_then(|rest| rest.split_once("\tlines="))
        .and_then(|(models, lines)| Some((models.parse().ok()?, lines.parse().ok()?)));
    counts.unwrap_or_else(|| {
        panic!(
            "the table opens with `#\\tmodels=<n>\\tlines=<n>`, written with it: {first:?}; \
             {repin}"
        )
    })
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &[u8]) -> String {
    let hex = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    format!("{hex}:{}", bytes.len())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !["target", "node_modules", ".git", ".engineering"].contains(&name.as_str()) {
                walk(&path, out);
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

/// Whether a YAML file is an ESS source: a line starts `format: ess/`.
fn is_ess_source(text: &str) -> bool {
    text.lines().any(|line| line.starts_with("format: ess/"))
}

/// The documents and sources of one model; `None` for one that does not parse.
type Model = Option<(Vec<(Source, RawSpecFile)>, SourceMap)>;

/// Every model this tree holds, by label: each ESS source file alone, and each directory holding a
/// `system.yaml` as one system of every YAML file under it, as the `ess` CLI reads a specification
/// directory.
fn models(root: &Path) -> Vec<(String, Model)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    let mut systems = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !is_ess_source(&text) {
            continue;
        }
        if path.file_name().is_some_and(|name| name == "system.yaml") {
            systems.push(path.parent().unwrap().to_path_buf());
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        let model = RawSpecFile::parse(&text).ok().map(|raw| {
            let mut sources = SourceMap::new();
            sources.insert(label.clone(), text.clone());
            (vec![(Source::new(label.clone()), raw)], sources)
        });
        found.push((label, model));
    }
    for dir in systems {
        let mut members = Vec::new();
        walk(&dir, &mut members);
        members.sort();
        let mut sources = SourceMap::new();
        let mut documents = Vec::new();
        let mut unparsed = false;
        for path in members {
            let text = std::fs::read_to_string(&path).unwrap();
            let label = path.strip_prefix(&dir).unwrap().display().to_string();
            match RawSpecFile::parse(&text) {
                Ok(raw) => {
                    sources.insert(label.clone(), text);
                    documents.push((Source::new(label), raw));
                }
                Err(_) => unparsed = true,
            }
        }
        let name = dir.strip_prefix(root).unwrap().display().to_string();
        found.push((
            format!("system-dir:{name}"),
            (!unparsed).then_some((documents, sources)),
        ));
    }
    found
}

/// The count and digest of `items` as each prints, one per line, in synthesis order.
fn listed<T: ToString>(items: &[T]) -> String {
    let printed: Vec<String> = items.iter().map(ToString::to_string).collect();
    format!(
        "{} {}",
        printed.len(),
        digest(printed.join("\n").as_bytes())
    )
}

/// A model compiled, or the stage it stopped at.
fn compiled(model: Model) -> Result<EssIr, &'static str> {
    let (documents, sources) = model.ok_or("PARSE")?;
    let spec = Specification::assemble(documents).map_err(|_| "ASSEMBLE")?;
    compile(&spec, &sources).map_err(|_| "COMPILE")
}

/// One model's line: the stage it stopped at, or its suite's canonical JSON, claimable scenarios
/// kept, and its refusals, notes and out-of-scope scenarios.
fn model_lines(label: &str, model: Model) -> Vec<String> {
    if UNAFFORDABLE.contains(&label) {
        return vec![format!("{label}\tEXCLUDED")];
    }
    let ir = match compiled(model) {
        Ok(ir) => ir,
        Err(stage) => return vec![format!("{label}\t{stage}")],
    };
    let synthesis = synthesize(&ir);
    let suite = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| format!("SUITE-ERROR {error:?}"));
    vec![format!(
        "{label}\tsuite={} {}\trefusals={}\tnotes={}\toutside={}",
        synthesis.suite.scenarios.len(),
        digest(suite.as_bytes()),
        listed(&synthesis.refusals),
        listed(&synthesis.notes),
        listed(&synthesis.outside),
    )]
}

/// One line per component of a model that compiles and declares one: the component's suite and
/// the scenarios it leaves to another component. None for any other model.
fn component_lines(label: &str, model: Model) -> Vec<String> {
    if UNAFFORDABLE.contains(&label) {
        return Vec::new();
    }
    let Ok(ir) = compiled(model) else {
        return Vec::new();
    };
    ir.components()
        .keys()
        .map(|component| {
            let synthesis = synthesize_for(&ir, component.as_str())
                .expect("a component the model declares is synthesized");
            let suite = synthesis
                .suite
                .to_canonical_json()
                .unwrap_or_else(|error| format!("SUITE-ERROR {error:?}"));
            format!(
                "{label}\t{component}\tsuite={} {}\toutside={}",
                synthesis.suite.scenarios.len(),
                digest(suite.as_bytes()),
                listed(&synthesis.outside),
            )
        })
        .collect()
}

/// Every model's lines by `lines`, in walk order, leaving out a model with none; written to the
/// path `write` names when set, with each model's time printed.
fn table(lines: fn(&str, Model) -> Vec<String>, write: &str) -> (Vec<(String, Vec<String>)>, bool) {
    let writing = std::env::var(write).ok();
    let here: Vec<(String, Vec<String>)> = models(&root())
        .into_iter()
        .filter_map(|(label, model)| {
            let started = Instant::now();
            let found = lines(&label, model);
            if writing.is_some() {
                eprintln!("{:>9.3}s\t{label}", started.elapsed().as_secs_f64());
            }
            (!found.is_empty()).then_some((label, found))
        })
        .collect();
    let Some(path) = writing else {
        return (here, false);
    };
    let count = here.iter().map(|(_, lines)| lines.len()).sum::<usize>();
    let mut text = format!("{HEADER}{}\tlines={count}\n", here.len());
    for line in here.iter().flat_map(|(_, lines)| lines) {
        writeln!(text, "{line}").unwrap();
    }
    std::fs::write(path, text).unwrap();
    (here, true)
}

/// Compares `here` with the pinned `table`, failing on a table cut short, an unpinned or missing
/// model, a moved line, or counts that disagree, each naming `repin`. Returns the pinned models in
/// table order with their lines.
fn compare<'t>(
    table: &'t str,
    here: &[(String, Vec<String>)],
    repin: &str,
) -> Vec<(&'t str, Vec<&'t str>)> {
    let (models, lines_stated) = header(table, repin);
    let mut pinned: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut order: Vec<&str> = Vec::new();
    for line in table.lines().skip(1).filter(|line| !line.is_empty()) {
        let (label, _) = line.split_once('\t').expect("a label, then a tab");
        if !pinned.contains_key(label) {
            order.push(label);
        }
        pinned.entry(label).or_default().push(line);
    }
    let lines = pinned.values().map(Vec::len).sum::<usize>();
    assert_eq!(
        pinned.len(),
        models,
        "every pinned model is read: the table is cut short or edited by hand; {repin}"
    );
    assert_eq!(
        lines, lines_stated,
        "every pinned line is read: the table is cut short or edited by hand; {repin}"
    );

    let unpinned: Vec<&str> = here
        .iter()
        .map(|(label, _)| label.as_str())
        .filter(|label| !pinned.contains_key(label))
        .collect();
    assert_eq!(
        unpinned,
        Vec::<&str>::new(),
        "the tree holds models the table does not pin; in the change that adds them, {repin}"
    );

    let mut moved = Vec::new();
    for label in &order {
        let Some((_, lines)) = here.iter().find(|(name, _)| name == label) else {
            moved.push(format!("{label}: no longer found"));
            continue;
        };
        let base = &pinned[label];
        if lines.iter().map(String::as_str).collect::<Vec<_>>() != *base {
            moved.push(format!(
                "pinned {}\n  here   {}",
                base.join("\n         "),
                lines.join("\n         ")
            ));
        }
    }
    assert_eq!(
        moved,
        Vec::<String>::new(),
        "synthesized bytes moved; where the move is intended, name it, then {repin}"
    );
    let here_lines = here.iter().map(|(_, lines)| lines.len()).sum::<usize>();
    assert_eq!(
        (here.len(), here_lines),
        (models, lines_stated),
        "the tree holds as many models and lines as the table pins; {repin}"
    );
    order
        .into_iter()
        .map(|label| (label, pinned.remove(label).unwrap_or_default()))
        .collect()
}

/// The crates whose `src/` synthesis runs through: `ess-conformance` and what it depends on
/// (`ess-gen` through `ess-transport` and `ess-http`, `ess-compiler`, `ess-domain`,
/// `ess-primitives`, `ess-yaml`). `ess-synth` depends on `ess-conformance`, so it is not here.
const SYNTHESIS_CRATES: &[&str] = &[
    "ess-compiler",
    "ess-conformance",
    "ess-domain",
    "ess-gen",
    "ess-http",
    "ess-primitives",
    "ess-transport",
    "ess-yaml",
];

/// The stamp of the sources of synthesis, written by the command the failing test prints.
const STAMP_PATH: &str = "tests/fixtures/synthesis-sources.digest";

/// How to write the stamp.
const STAMP_REPIN: &str = "re-stamp from the repository root with \
     SYNTHESIS_SOURCES_STAMP_WRITE=\"$PWD/crates/verify/ess-conformance/tests/fixtures/\
     synthesis-sources.digest\" cargo test -p ess-conformance --locked --test \
     synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp, in \
     the same change as the source or model it follows";

/// Every file under `dir`, recursively, in no order.
fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// The SHA-256 of the sources of synthesis, the crate count and the model count. It hashes, as
/// sorted records, every file under `src/` of each crate of [`SYNTHESIS_CRATES`] found under
/// `crates/<area>/<crate>/` (`S`, by path) and every model file [`walk`] finds (`M`, by label).
/// Manifests, lock files and `.tsv` tables are not hashed, so a version bump re-stamps nothing.
fn sources_digest(root: &Path) -> (String, usize, usize) {
    let mut records: Vec<(u8, String, Vec<u8>)> = Vec::new();
    let mut crates = 0;
    for area in std::fs::read_dir(root.join("crates")).unwrap() {
        let area = area.unwrap().path();
        let Ok(members) = std::fs::read_dir(&area) else {
            continue;
        };
        for member in members {
            let member = member.unwrap().path();
            let name = member.file_name().unwrap().to_string_lossy().to_string();
            if !SYNTHESIS_CRATES.contains(&name.as_str()) || !member.join("src").is_dir() {
                continue;
            }
            crates += 1;
            let mut files = Vec::new();
            files_under(&member.join("src"), &mut files);
            for file in files {
                let label = file
                    .strip_prefix(root)
                    .unwrap()
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("/");
                records.push((b'S', label, std::fs::read(&file).unwrap()));
            }
        }
    }
    let mut yaml = Vec::new();
    walk(root, &mut yaml);
    let mut models = 0;
    for path in yaml {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !is_ess_source(&text) {
            continue;
        }
        models += 1;
        let label = path.strip_prefix(root).unwrap().display().to_string();
        records.push((b'M', label, text.into_bytes()));
    }
    records.sort_by(|a, b| (a.0, a.1.as_bytes()).cmp(&(b.0, b.1.as_bytes())));
    let mut hasher = Sha256::new();
    for (kind, label, bytes) in &records {
        hasher.update([*kind, 0]);
        hasher.update(label.as_bytes());
        hasher.update([0]);
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update([0]);
        hasher.update(bytes);
    }
    let hex = hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    (hex, crates, models)
}

#[test]
fn the_synthesis_sources_digest_matches_the_committed_stamp() {
    let (hex, crates, models) = sources_digest(&root());
    let computed = format!("#\tsources={hex}\tcrates={crates}\tmodels={models}\n");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(STAMP_PATH);
    if let Ok(target) = std::env::var("SYNTHESIS_SOURCES_STAMP_WRITE") {
        std::fs::write(target, computed).unwrap();
        return;
    }
    let stored = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        stored, computed,
        "the sources of synthesis or a model moved since the stamp was written: stored {stored:?}, \
         computed {computed:?} ({crates} crates, {models} models); {STAMP_REPIN}"
    );
}

#[test]
fn the_sources_digest_moves_with_a_source_byte_a_model_byte_and_a_new_model_and_ignores_a_manifest()
{
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sources-digest-scratch");
    let _ = std::fs::remove_dir_all(&root);
    let put = |relative: &str, text: &str| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    put("crates/area/ess-gen/src/lib.rs", "pub fn a() {}\n");
    put("crates/area/ess-yaml/src/lib.rs", "pub fn b() {}\n");
    put(
        "crates/area/ess-gen/Cargo.toml",
        "[package]\nversion = \"1\"\n",
    );
    put("crates/area/ess-gen/tests/fixtures/t.tsv", "a\tb\n");
    put("crates/area/other/src/lib.rs", "pub fn c() {}\n");
    put("models/m.yaml", "format: ess/1\nname: m\n");
    put("models/not-a-model.yaml", "name: plain\n");
    let base = sources_digest(&root);
    assert_eq!((base.1, base.2), (2, 1), "two crates, one model");

    put("crates/area/other/src/lib.rs", "pub fn d() {}\n");
    put(
        "crates/area/ess-gen/Cargo.toml",
        "[package]\nversion = \"2\"\n",
    );
    put("crates/area/ess-gen/tests/fixtures/t.tsv", "c\td\n");
    assert_eq!(
        sources_digest(&root),
        base,
        "a manifest, a table and another crate"
    );

    put("crates/area/ess-gen/src/lib.rs", "pub fn a() {} \n");
    let source = sources_digest(&root);
    assert_ne!(source.0, base.0, "a source byte");
    put("crates/area/ess-gen/src/lib.rs", "pub fn a() {}\n");
    assert_eq!(sources_digest(&root), base);

    put("models/m.yaml", "format: ess/1\nname: n\n");
    assert_ne!(sources_digest(&root).0, base.0, "a model byte");
    put("models/m.yaml", "format: ess/1\nname: m\n");

    put("models/second.yaml", "format: ess/1\n");
    let added = sources_digest(&root);
    assert_ne!(added.0, base.0, "a new model");
    assert_eq!(added.2, 2);
    std::fs::remove_file(root.join("models/second.yaml")).unwrap();

    std::fs::rename(
        root.join("crates/area/ess-gen/src/lib.rs"),
        root.join("crates/area/ess-gen/src/main.rs"),
    )
    .unwrap();
    assert_ne!(sources_digest(&root).0, base.0, "a renamed source file");
}

#[test]
#[ignore = "slow lane: re-synthesizes every repository model and compares its suite bytes"]
fn every_repository_model_keeps_its_synthesized_bytes() {
    let (here, written) = table(model_lines, "SYNTHESIS_SUITE_BYTES_TABLE_WRITE");
    if written {
        return;
    }
    let pinned = compare(TABLE, &here, REPIN);
    let excluded: Vec<&str> = pinned
        .iter()
        .filter(|(_, lines)| lines.iter().any(|line| line.ends_with("\tEXCLUDED")))
        .map(|(label, _)| *label)
        .collect();
    assert_eq!(
        excluded, UNAFFORDABLE,
        "every excluded model is named in UNAFFORDABLE, with its reason; {REPIN}"
    );
}

#[test]
#[ignore = "slow lane: re-synthesizes every repository component and compares its suite bytes, 31.8 s at load 50"]
fn every_repository_component_keeps_its_synthesized_bytes() {
    let (here, written) = table(component_lines, "SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE");
    if written {
        return;
    }
    compare(COMPONENT_TABLE, &here, COMPONENT_REPIN);
}

/// Every model's lines by `lines` inside `with_jobs(count, ..)`, in walk order.
fn lines_at(count: usize, lines: fn(&str, Model) -> Vec<String>) -> Vec<(String, Vec<String>)> {
    with_jobs(NonZeroUsize::new(count).unwrap(), || {
        models(&root())
            .into_iter()
            .map(|(label, model)| {
                let made = lines(&label, model);
                (label, made)
            })
            .collect()
    })
}

#[test]
#[ignore = "slow lane: every model and component synthesized at 1, 2 and 7 jobs, about 450 s"]
fn every_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs() {
    for (name, lines) in [
        ("model", model_lines as fn(&str, Model) -> Vec<String>),
        ("component", component_lines),
    ] {
        let one = lines_at(1, lines);
        for count in [2, 7] {
            let many = lines_at(count, lines);
            assert_eq!(one.len(), many.len(), "{name} lines at {count} jobs");
            for ((label, at_one), (_, at_many)) in one.iter().zip(&many) {
                assert!(
                    at_one == at_many,
                    "{name} `{label}` at {count} jobs differs from 1 job\n  1 job : {at_one:?}\n  \
                     {count} jobs: {at_many:?}"
                );
            }
        }
    }
}
