//! `ess generate ui --target tui --path <ui.yaml> --model <spec> --out <dir>`: an `ess-ui/1`
//! document becomes a Rust terminal application crate (beyond10x/ess#311, S4).
//!
//! The generated crate holds the document, the binding `ess_ui_check::binding` computes and a
//! clap-derived `main` — no renderer code: it depends on `ess-ui-tui` by the Git tag of the ESS
//! version that generated it. These cases build it with a `[patch]` of that dependency to this
//! workspace, into a target directory of their own under `CARGO_TARGET_TMPDIR`, and run the built
//! binary headless (`--screen-once`) against the Rust gatepass server from
//! `examples/gatepass-realization`, started on `127.0.0.1:0`.

use std::collections::BTreeMap;
use std::io::BufRead as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;

use ess_ui::binding::Answer;
use serde_yaml::Value;

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

/// One run of the built `ess` binary, from the workspace root so example paths resolve.
fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(workspace_root())
        .env_remove("ESS_UI_AUTHORIZATION")
        .output()
        .expect("the ess binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("output is UTF-8")
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

/// A desk over `examples/gatepass`, whose one served component is `pass-service`: the expected
/// visits as its home page, with a register action (which the seeding below sends).
const DESK: &str = "format: ess-ui/1\napp: desk\nmodel: gatepass\nplacement_profile: fat\n\
    shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
    navigation: {home: desk, sections: [{name: all, pages: [desk]}]}\n\
    pages: {desk: {kind: detail_page, title: Desk, sections: [{name: expected, \
    component: collection, reads: visit.ExpectedVisits, columns: [visitor, building], \
    actions: [{name: register, does: visit.RegisterVisit}], \
    row_actions: [{name: depart, label: Depart, does: visit.SignOutVisitor, \
    bind: {visit_id: row.visit_id}}]}]}}\n";

/// A fresh scratch directory under this test binary's own target directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("generate-ui-tui")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

/// Writes the desk under `dir` and generates its terminal app into `dir/app`.
fn generate(dir: &Path) -> (Output, PathBuf) {
    let document = dir.join("desk.yaml");
    std::fs::write(&document, DESK).expect("writable");
    let out = dir.join("app");
    let output = ess(&[
        "generate",
        "ui",
        "--target",
        "tui",
        "--path",
        utf8(&document),
        "--model",
        "examples/gatepass",
        "--out",
        utf8(&out),
    ]);
    (output, out)
}

/// Every file under `dir`, by its path relative to `dir`.
fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
            .map(|entry| entry.expect("a directory entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, into);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("under the root")
                    .to_string_lossy()
                    .replace('\\', "/");
                into.insert(relative, std::fs::read(&path).expect("readable"));
            }
        }
    }
    let mut into = BTreeMap::new();
    walk(dir, dir, &mut into);
    into
}

/// The generated desk, built once with its `ess-ui-tui` dependency patched to this workspace;
/// returns the built binary.
fn built_desk() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        let dir = scratch("built");
        let (output, out) = generate(&dir);
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
        // The lock an online resolve with the patch below would write: the workspace's own, so
        // the offline build takes what is already here, and the generated package locked to the
        // patched `ess-ui-tui`. Without that entry Cargo loads the Git source the patch replaces,
        // which an offline build cannot fetch.
        let mut lock =
            std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("the lock reads");
        lock.push_str(
            "\n[[package]]\nname = \"desk\"\nversion = \"0.1.0\"\n\
             dependencies = [\n \"clap\",\n \"ess-ui-tui\",\n]\n",
        );
        std::fs::write(out.join("Cargo.lock"), lock).expect("the lock is written");
        let patch = format!(
            "patch.\"https://github.com/beyond10x/ess\".ess-ui-tui.path = \"{}\"",
            workspace_root().join("crates/ui/ess-ui-tui").display()
        );
        let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("generate-ui-tui-target");
        let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args([
                "build",
                "--offline",
                "--quiet",
                "--config",
                &patch,
                "--manifest-path",
            ])
            .arg(out.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(&target)
            .env_remove("CARGO_TARGET_DIR")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        assert!(
            built.status.success(),
            "the generated crate builds: {}",
            text(&built.stderr)
        );
        target
            .join("debug")
            .join(format!("desk{}", std::env::consts::EXE_SUFFIX))
    })
}

// ── the server ────────────────────────────────────────────────────────────────────────────────

/// Builds `examples/gatepass-realization`'s server into its own target directory, once: a nested
/// `cargo build` into the running test's target directory would wait on its lock.
fn gatepass_server() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("gatepass-target");
        let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args([
                "build",
                "--offline",
                "--quiet",
                "--bin",
                "gatepass-server",
                "--manifest-path",
            ])
            .arg(workspace_root().join("examples/gatepass-realization/Cargo.toml"))
            .arg("--target-dir")
            .arg(&target)
            .env_remove("CARGO_TARGET_DIR")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        assert!(
            built.status.success(),
            "the Rust gatepass server builds: {}",
            text(&built.stderr)
        );
        target.join("debug/gatepass-server")
    })
}

/// Kills the server when the case ends, pass or fail.
struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts a fresh server on an ephemeral port of `127.0.0.1`; returns it and its base URL.
fn serve() -> (Served, String) {
    let mut child = Command::new(gatepass_server())
        .env("PORT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let served = Served(child);
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines.next().expect("a startup record").expect("text");
    let record: serde_json::Value = serde_json::from_str(&first).expect("the record is JSON");
    let port = record["runtime"]["port"]
        .as_u64()
        .unwrap_or_else(|| panic!("the record names the port: {first}"));
    std::thread::spawn(move || lines.for_each(drop));
    (served, format!("http://127.0.0.1:{port}"))
}

const RECEPTIONIST: &str = "Actor gatepass.visit.Receptionist";

/// Registers one visit by `visitor` at `building` on the server at `base`.
fn register(base: &str, visitor: &str, building: &str) {
    let document = ess_ui::load_str(DESK).expect("the desk loads");
    let model = workspace_root().join("examples/gatepass");
    let sources: Vec<(String, String)> = ["system.yaml", "components.yaml", "domains/visit.yaml"]
        .iter()
        .map(|file| {
            let text = std::fs::read_to_string(model.join(file)).expect("the model reads");
            ((*file).to_owned(), text)
        })
        .collect();
    let binding = ess_ui_check::binding(&document, &sources).expect("the desk binds");
    let bases = BTreeMap::from([("pass-service".to_owned(), base.to_owned())]);
    let adapter = ess_ui_tui::HttpAdapter::new(binding, &bases, Some(RECEPTIONIST.to_owned()))
        .expect("the adapter is built");
    let input: BTreeMap<String, Value> = serde_yaml::from_str(&format!(
        "{{visitor: {visitor}, building: {building}, host: {{kind: employee, value: E1}}, \
         expected_minutes: 30, expected_stay: PT30M, \
         deposit: {{amount: '0', currency: EUR}}, escorts: [], notes: {{}}, on_watchlist: false}}"
    ))
    .expect("the input parses");
    let (status, body) = adapter
        .post("visit.RegisterVisit", &input)
        .unwrap_or_else(|error| panic!("the registration is answered: {error}"));
    assert_eq!(
        ess_ui::binding::classify(status, &body),
        Answer::Accepted,
        "{status} {body}"
    );
}

// ── the cases ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn the_generated_crate_builds_and_its_help_names_base_url() {
    let binary = built_desk();
    for flag in ["--help", "-h"] {
        let help = Command::new(binary)
            .arg(flag)
            .output()
            .expect("the generated binary runs");
        assert_eq!(help.status.code(), Some(0), "{}", text(&help.stderr));
        let help = text(&help.stdout);
        assert!(help.contains("--base-url"), "{flag}: {help}");
        assert!(help.contains("ESS_UI_AUTHORIZATION"), "{flag}: {help}");
    }
    // A frame larger than 1000x1000, or a signed side, is a usage error.
    for size in ["1001x40", "+120x40"] {
        let refused = Command::new(binary)
            .args(["--base-url", "http://127.0.0.1:9", "--screen-once", size])
            .output()
            .expect("the generated binary runs");
        assert_eq!(
            refused.status.code(),
            Some(2),
            "{size}: {}",
            text(&refused.stderr)
        );
    }

    // No renderer code in it: the crate is its document, its binding and a command line.
    let dir = scratch("renderer");
    let (output, out) = generate(&dir);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let written = files(&out);
    let names: Vec<&str> = written.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        ["Cargo.toml", "src/binding.rs", "src/main.rs", "src/ui.yaml"],
        "the generated files"
    );
    for (name, bytes) in &written {
        let source = String::from_utf8_lossy(bytes);
        for renderer in ["ratatui", "crossterm"] {
            assert!(!source.contains(renderer), "{name} names {renderer}");
        }
    }
    let manifest = String::from_utf8_lossy(&written["Cargo.toml"]).into_owned();
    assert!(
        manifest.contains(&format!(
            "ess-ui-tui = {{ git = \"https://github.com/beyond10x/ess\", tag = \"{}\" }}",
            env!("CARGO_PKG_VERSION")
        )),
        "{manifest}"
    );
}

#[test]
fn generation_is_deterministic() {
    let first_dir = scratch("first");
    let second_dir = scratch("second-elsewhere");
    let (first, first_out) = generate(&first_dir);
    let (second, second_out) = generate(&second_dir);
    assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
    assert_eq!(second.status.code(), Some(0), "{}", text(&second.stderr));
    let first_files = files(&first_out);
    assert!(!first_files.is_empty());
    assert_eq!(
        first_files,
        files(&second_out),
        "two runs write the same bytes"
    );
    for (name, bytes) in &first_files {
        let source = String::from_utf8_lossy(bytes);
        assert!(
            !source.contains(utf8(&first_dir)),
            "{name} names the directory it was generated from"
        );
    }
    // Generating again over the same output rewrites the same bytes.
    let (again, _) = generate(&first_dir);
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    assert_eq!(first_files, files(&first_out));
}

#[test]
fn the_built_crate_reads_a_view_from_a_synthesized_gatepass_server() {
    let binary = built_desk();
    let (server, base) = serve();
    register(&base, "Ada", "North");
    let state = scratch("state");
    let output = Command::new(binary)
        .args(["--base-url", &base, "--screen-once", "120x40"])
        .env("ESS_UI_AUTHORIZATION", RECEPTIONIST)
        .env("XDG_STATE_HOME", &state)
        .stdin(Stdio::null())
        .output()
        .expect("the generated binary runs");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let screen = text(&output.stdout);
    assert_eq!(screen.lines().count(), 40, "one 120x40 frame:\n{screen}");
    let row = screen
        .lines()
        .find(|line| line.contains("Ada"))
        .unwrap_or_else(|| panic!("the ExpectedVisits row is on the screen:\n{screen}"));
    assert!(
        row.contains("North"),
        "the row shows its building:\n{screen}"
    );
    assert!(output.stderr.is_empty(), "{}", text(&output.stderr));

    // With the server gone the frame is still printed, and the run exits 3 with one stderr line
    // naming the read that failed.
    drop(server);
    let output = Command::new(binary)
        .args(["--base-url", &base, "--screen-once", "120x40"])
        .env("ESS_UI_AUTHORIZATION", RECEPTIONIST)
        .env("XDG_STATE_HOME", &state)
        .stdin(Stdio::null())
        .output()
        .expect("the generated binary runs");
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout).lines().count(), 40);
    let stderr = text(&output.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("visit.ExpectedVisits: "), "{stderr}");
}
