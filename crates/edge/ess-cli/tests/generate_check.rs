//! `ess generate --check`: regenerate in memory, compare with what `--out` already holds, write
//! nothing, and exit 1 naming every file that drifted. Also the adopter guidance built on it: the
//! drift-check section of the generation guide and the page on which generated files to commit.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::SystemTime,
};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("examples/billing").is_dir())
        .expect("the workspace holds examples/billing")
        .to_path_buf()
}

fn ess(args: &[&str], out: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["generate", "--path"])
        .arg(workspace().join("examples/billing"))
        .args(args);
    if let Some(out) = out {
        command.arg("--out").arg(out);
    }
    command.output().expect("ess runs")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Every file under `root`, with its bytes and modification time.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
        for entry in fs::read_dir(dir).expect("readable directory") {
            let path = entry.expect("directory entry").path();
            let meta = fs::symlink_metadata(&path).expect("metadata");
            if meta.is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).expect("under root").to_path_buf(),
                    (
                        fs::read(&path).expect("readable file"),
                        meta.modified().expect("mtime"),
                    ),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

/// The generated files under `root`, outside the `.ess-output` ownership record.
fn generated_files(root: &Path) -> Vec<PathBuf> {
    snapshot(root)
        .into_keys()
        .filter(|path| !path.starts_with(".ess-output"))
        .collect()
}

/// The one `--check` line that names `drifted`.
fn drift_line<'a>(err: &'a str, drifted: &Path) -> &'a str {
    err.lines()
        .find(|line| line.contains(&*drifted.to_string_lossy()))
        .unwrap_or_else(|| panic!("stderr names {}: {err}", drifted.display()))
}

#[test]
fn generate_check_reports_drift_and_exits_nonzero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("openapi");
    let first = ess(&["--kind", "openapi"], Some(&out));
    assert!(first.status.success(), "generation: {first:?}");
    let files = generated_files(&out);
    assert!(
        files.len() >= 2,
        "billing projects at least two OpenAPI documents: {files:?}"
    );
    let edited = &files[0];
    let deleted = &files[1];
    fs::write(out.join(edited), "edited by hand\n").expect("edit");
    fs::remove_file(out.join(deleted)).expect("delete");
    let before = snapshot(&out);

    let check = ess(&["--kind", "openapi", "--check"], Some(&out));

    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    let err = stderr(&check);
    // Regeneration recreates a missing owned file.
    assert!(
        drift_line(&err, deleted).contains("regenerate it with `ess generate`"),
        "the drift line says how to repair it: {err}"
    );
    // It refuses an edited one (beyond10x/ess#484), so the line names the re-enroll route and
    // does not promise that regenerating repairs it.
    let edited_line = drift_line(&err, edited);
    for named in [
        "move it aside",
        "remove `.ess-output`",
        "ess generate output adopt",
    ] {
        assert!(
            edited_line.contains(named),
            "the drift line names {named}: {edited_line}"
        );
    }
    assert!(
        !edited_line.contains("regenerate it with"),
        "the drift line promises a repair regeneration refuses: {edited_line}"
    );
    assert_eq!(
        err.lines()
            .filter(|line| line.starts_with(&*out.to_string_lossy()))
            .count(),
        2,
        "one line per drifted file and no other: {err}"
    );
    assert_eq!(snapshot(&out), before, "--check wrote nothing");

    let regenerate = ess(&["--kind", "openapi"], Some(&out));
    let refused = stderr(&regenerate);
    assert!(
        !regenerate.status.success(),
        "regeneration replaced it: {refused}"
    );
    for named in [
        edited.to_string_lossy().into_owned(),
        format!(
            "ess generate output adopt --ownership-root {}",
            out.display()
        ),
    ] {
        assert!(
            refused.contains(&named),
            "the refusal names {named}: {refused}"
        );
    }
    assert_eq!(snapshot(&out), before, "the refusal wrote");
}

#[test]
fn generate_check_passes_on_fresh_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("openapi");
    let first = ess(&["--kind", "openapi"], Some(&out));
    assert!(first.status.success(), "generation: {first:?}");
    let before = snapshot(&out);

    let check = ess(&["--kind", "openapi", "--check"], Some(&out));

    assert_eq!(
        check.status.code(),
        Some(0),
        "current output passes: {check:?}"
    );
    let err = stderr(&check);
    assert!(
        !err.contains("regenerate it with"),
        "no drift line on current output: {err}"
    );
    assert_eq!(
        snapshot(&out),
        before,
        "--check changed no file's bytes or modification time"
    );
}

#[test]
fn generate_check_without_kind_compares_every_projection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("generated");
    let first = ess(&[], Some(&out));
    assert!(first.status.success(), "generation: {first:?}");
    let check = ess(&["--check"], Some(&out));
    assert_eq!(
        check.status.code(),
        Some(0),
        "current output passes: {check:?}"
    );

    let docs = generated_files(&out)
        .into_iter()
        .find(|path| path.starts_with("docs"))
        .expect("the default generation writes docs");
    fs::write(out.join(&docs), "stale\n").expect("edit");
    let check = ess(&["--check"], Some(&out));
    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    assert!(
        stderr(&check).contains(&*docs.to_string_lossy()),
        "a drifted docs page is named: {}",
        stderr(&check)
    );
}

#[test]
fn generate_check_reports_a_file_no_longer_generated() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("site");
    let guide = dir.path().join("guide.md");
    fs::write(&guide, "# A guide\n\nAuthored.\n").expect("guide");
    let include = format!("guide={}", guide.display());
    let first = ess(&["--kind", "site", "--include", &include], Some(&out));
    assert!(first.status.success(), "generation: {first:?}");
    assert!(
        out.join("guide.html").is_file(),
        "the included page was published"
    );
    let before = snapshot(&out);

    // Without the include, `guide.html` is still recorded as owned but no longer generated.
    let check = ess(&["--kind", "site", "--check"], Some(&out));

    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    let err = stderr(&check);
    assert!(
        err.lines()
            .any(|line| line.contains("guide.html") && line.contains("regenerate it with")),
        "the file no projection produces any more is named: {err}"
    );
    assert_eq!(snapshot(&out), before, "--check wrote nothing");
}

#[test]
fn generate_check_requires_out() {
    let check = ess(&["--kind", "openapi", "--check"], None);
    assert_eq!(
        check.status.code(),
        Some(2),
        "the argument parser refuses --check without --out: {check:?}"
    );
    assert!(
        stderr(&check).contains("--out"),
        "the refusal names --out: {}",
        stderr(&check)
    );
}

/// The `#### ` section of `page` titled `heading`, up to the next heading of that level or above.
fn section<'a>(page: &'a str, heading: &str) -> Option<&'a str> {
    let level = heading.chars().take_while(|c| *c == '#').count();
    let start = page.lines().position(|line| line == heading)?;
    let lines: Vec<&str> = page.lines().collect();
    let mut fenced = false;
    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        let hashes = line.chars().take_while(|c| *c == '#').count();
        if !fenced && hashes > 0 && hashes <= level && line[hashes..].starts_with(' ') {
            end = index;
            break;
        }
    }
    let offset = |n: usize| {
        lines[..n]
            .iter()
            .map(|line| line.len() + 1)
            .sum::<usize>()
            .min(page.len())
    };
    Some(&page[offset(start)..offset(end)])
}

fn page(relative: &str) -> String {
    let path = workspace().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("{relative} exists"))
}

#[test]
fn cli_reference_lists_generate_check() {
    let page = page("website/docs/reference/cli.md");
    let section = section(&page, "#### `ess generate generate`")
        .expect("the CLI reference has an `ess generate generate` section");
    assert!(
        section
            .lines()
            .any(|line| line.starts_with("| `--check` |")),
        "the generated `ess generate generate` table lists --check:\n{section}"
    );
}

#[test]
fn drift_section_shows_ess_generate_check() {
    let page = page("website/docs/guides/generate-artifacts.md");
    let section = section(&page, "## Drift-checking in CI")
        .expect("generate-artifacts.md keeps the heading `## Drift-checking in CI`");
    assert!(
        section
            .lines()
            .any(|line| line.contains("ess generate") && line.contains("--check")),
        "the drift section shows `ess generate … --check`:\n{section}"
    );
    for sentence in section.split(". ") {
        if sentence.contains("cargo xtask") {
            assert!(
                sentence.contains("this repository's own"),
                "`cargo xtask` appears only in a sentence that says it is this repository's \
                 own: {sentence}"
            );
        }
    }
}

const COMMIT_PAGE: &str = "website/docs/guides/commit-generated-files.md";

#[test]
fn commit_generated_files_page_states_the_table() {
    let path = workspace().join(COMMIT_PAGE);
    let page = fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing page {COMMIT_PAGE}"));
    assert!(
        page.contains("title: Commit what a reviewer reads, regenerate the rest"),
        "{COMMIT_PAGE}: missing title"
    );
    let header = "| artifact | commit? | regenerated by | drift check |";
    let tables = page.lines().filter(|line| line.starts_with("| ")).count();
    assert!(
        page.contains(header),
        "{COMMIT_PAGE}: missing table header {header}"
    );
    let rows: Vec<&str> = page
        .lines()
        .skip_while(|line| *line != header)
        .skip(2)
        .take_while(|line| line.starts_with('|'))
        .collect();
    assert_eq!(
        tables,
        rows.len() + 1,
        "{COMMIT_PAGE}: one table, every table line belongs to it"
    );
    for phrase in [
        "`ess-inputs.yaml`",
        "authored scenarios",
        "`ess-known-failures/1`",
        "compiled IR",
        "`.ess-output`",
        "`--kind`",
        "synthesized runner",
    ] {
        assert!(
            rows.iter().any(|row| row.contains(phrase)),
            "{COMMIT_PAGE}: no table row names {phrase}"
        );
    }
    assert!(
        page.contains("../concepts/test-pyramid.md"),
        "{COMMIT_PAGE}: missing link to concepts/test-pyramid.md"
    );
    let why = section(&page, "## Why the first diff is large").unwrap_or_else(|| {
        panic!("{COMMIT_PAGE}: missing heading `## Why the first diff is large`")
    });
    assert!(
        why.contains("`--kind`"),
        "{COMMIT_PAGE}: `## Why the first diff is large` does not name `--kind`"
    );
}

#[test]
fn default_inventory_is_recorded_from_the_binary() {
    let page = page(COMMIT_PAGE);
    let why = section(&page, "## Why the first diff is large")
        .expect("the page has `## Why the first diff is large`");
    let stated: usize = why
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|pair| {
            (pair[1].trim_end_matches(|c: char| !c.is_alphanumeric()) == "files")
                .then(|| pair[0].parse().ok())
                .flatten()
        })
        .expect("the section states the default inventory as `N files`");

    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("generated");
    let run = ess(&[], Some(&out));
    assert!(run.status.success(), "default generation: {run:?}");
    let written = generated_files(&out).len();
    assert_eq!(
        written, stated,
        "`ess generate --path examples/billing` writes {written} files; the page says {stated}"
    );
}
