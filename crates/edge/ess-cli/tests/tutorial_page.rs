//! The getting-started walkthrough runs, and prints what the page says it prints.
//!
//! `website/docs/getting-started.md` takes an adopter from an empty directory to a green
//! conformance run. Every file it has the reader write and every `ess` command it has the reader
//! run is a fenced block carrying the bare attribute `ess-tutorial` on its fence. This test writes
//! those files into a temporary home directory, runs those commands in page order against the
//! `ess` this package builds, and compares every output line the page records.
//!
//! | fence | meaning |
//! |---|---|
//! | `` ```yaml ess-tutorial file=tasks/ess-inputs.yaml `` | write the body to that path, relative to the home directory |
//! | `` ```yaml ess-tutorial expect=tasks/ess-inputs.yaml `` | the file at that path now holds exactly the body |
//! | `` ```shell-session ess-tutorial `` (or `console`) | commands, each `$ ` line followed by the output it prints |
//! | `… requires=node` | run only with `ESS_TUTORIAL_NETWORK=1` set and `node` and `npm` on `PATH`; otherwise print the skip and its reason |
//! | `… checkout` | run in a copy of the repository's `examples/`, apart from the walkthrough's directory |
//!
//! Any other fence attribute, such as Docusaurus's `title="…"`, is ignored.
//!
//! A command block runs line by line from the directory the previous block left, starting in the
//! home directory. `cd <dir>` and `mkdir -p <dir>` are carried out by the test itself; `ess` runs
//! the built binary; `npm` runs only in a `requires=node` block. Leading `NAME=value` words set
//! the environment for that one command. Nothing else is a command here, and no word may hold a
//! quote or a `$`, because the test does not interpret them as a shell would.
//!
//! Every command must exit 0. Its output is its stdout followed by its stderr (stdout alone for
//! `npm`, whose own notices go to stderr), with trailing blank lines dropped. Each recorded line is
//! compared exactly, with two documented allowances and no others:
//!
//! - a line holding only `…` stands for any number of lines, including none, so a long or
//!   timing-dependent output (a test reporter's durations, `npm install`'s summary) is shown in part;
//! - the home directory prints as `~`, so a path `ess` prints is the path a reader who started in
//!   their home directory sees.
//!
//! `ess specify toolchain install --pin` downloads a release. Here it reads a release directory
//! this test assembles instead (`ESS_TOOLCHAIN_BASE_URL`): an archive holding the built binary
//! under this package's version, with its `SHA256SUMS`, so the command runs offline and prints
//! what it prints for the real download.
//!
//! `ESS_TUTORIAL_NETWORK=1` is the one switch this test reads. The `requires=node` blocks run
//! `npm install`, which fetches from the npm registry, so the offline gate skips them by default;
//! CI's workspace test shards set the variable, and they install Node.js.
//!
//! A shell block on the page that shows `$ ess` and is not an `ess-tutorial` block is refused, so a
//! command cannot be added to the walkthrough without being run.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use sha2::{Digest, Sha256};

const PAGE: &str = "website/docs/getting-started.md";
const MARK: &str = "ess-tutorial";
const ELISION: &str = "…";
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Set to `1` to run the `requires=node` blocks, whose `npm install` reaches the npm registry.
const NETWORK: &str = "ESS_TUTORIAL_NETWORK";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

/// One fenced block of the page: its language, its attributes and its lines.
struct Block {
    line: usize,
    language: String,
    flags: BTreeSet<String>,
    values: BTreeMap<String, String>,
    body: Vec<String>,
}

impl Block {
    fn at(&self) -> String {
        format!("{PAGE}:{}", self.line)
    }

    fn tutorial(&self) -> bool {
        self.flags.contains(MARK)
    }

    fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    fn shell(&self) -> bool {
        matches!(self.language.as_str(), "shell-session" | "console")
    }
}

/// Splits a fence's info string after the language into bare flags and `key=value` pairs. A
/// value may be quoted, as Docusaurus's `title="…"` is.
fn attributes(meta: &str) -> (BTreeSet<String>, BTreeMap<String, String>) {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in meta.chars() {
        match character {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    let mut flags = BTreeSet::new();
    let mut values = BTreeMap::new();
    for token in tokens {
        match token.split_once('=') {
            Some((key, value)) => {
                values.insert(key.to_owned(), value.to_owned());
            }
            None => {
                flags.insert(token);
            }
        }
    }
    (flags, values)
}

/// Every fenced block of `page`, read as `predicate_reference_page.rs` reads its page.
fn blocks(page: &str) -> Vec<Block> {
    let mut found = Vec::new();
    let mut open: Option<Block> = None;
    for (index, line) in page.lines().enumerate() {
        let trimmed = line.trim_start();
        match open.take() {
            None => {
                if let Some(info) = trimmed.strip_prefix("```") {
                    // CommonMark does not open a backtick fence whose info string holds a
                    // backtick, so the site would pair every later fence with the wrong partner.
                    assert!(
                        !info.contains('`'),
                        "{PAGE}:{}: a fence's info string holds a backtick",
                        index + 1
                    );
                    let language = info.split_whitespace().next().unwrap_or_default();
                    let (flags, values) = attributes(info[language.len()..].trim());
                    open = Some(Block {
                        line: index + 1,
                        language: language.to_owned(),
                        flags,
                        values,
                        body: Vec::new(),
                    });
                }
            }
            Some(mut block) => {
                if trimmed.starts_with("```") {
                    found.push(block);
                } else {
                    block.body.push(line.to_owned());
                    open = Some(block);
                }
            }
        }
    }
    assert!(open.is_none(), "{PAGE} ends inside a code fence");
    found
}

/// One `$ ` line of a command block and the output lines the page records under it.
struct Step {
    line: usize,
    words: Vec<String>,
    expected: Vec<String>,
}

fn steps(block: &Block) -> Result<Vec<Step>, String> {
    let mut steps: Vec<Step> = Vec::new();
    for (offset, text) in block.body.iter().enumerate() {
        let line = block.line + 1 + offset;
        if let Some(command) = text.strip_prefix("$ ") {
            let words: Vec<String> = command.split_whitespace().map(str::to_owned).collect();
            if let Some(word) = words.iter().find(|w| w.contains(['"', '\'', '$', '`'])) {
                return Err(format!(
                    "{PAGE}:{line}: `{word}` holds a quote or `$`, which this test does not \
                     interpret as a shell would"
                ));
            }
            steps.push(Step {
                line,
                words,
                expected: Vec::new(),
            });
        } else if let Some(step) = steps.last_mut() {
            step.expected.push(text.clone());
        } else if !text.trim().is_empty() {
            return Err(format!(
                "{PAGE}:{line}: output before the block's first `$ ` command"
            ));
        }
    }
    for step in &mut steps {
        while step.expected.last().is_some_and(|l| l.trim().is_empty()) {
            step.expected.pop();
        }
    }
    if steps.is_empty() {
        return Err(format!("{}: a command block with no `$ ` line", block.at()));
    }
    Ok(steps)
}

/// Whether `actual` is `expected` line for line, where an expected `…` line stands for any number
/// of actual lines.
fn matches(expected: &[String], actual: &[String]) -> bool {
    // reach[j]: the expected lines read so far can account for exactly the first j actual lines.
    let mut reach = vec![false; actual.len() + 1];
    reach[0] = true;
    for line in expected {
        let mut next = vec![false; actual.len() + 1];
        if line == ELISION {
            let mut any = false;
            for (j, slot) in next.iter_mut().enumerate() {
                any |= reach[j];
                *slot = any;
            }
        } else {
            for j in 0..actual.len() {
                if reach[j] && actual[j] == *line {
                    next[j + 1] = true;
                }
            }
        }
        reach = next;
    }
    reach[actual.len()]
}

fn trailing_blank_lines_dropped(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines
}

fn on_path(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Why the `requires=node` blocks cannot run here, or `None` when they can. `npm install` reaches
/// the npm registry, and the gate is offline unless `NETWORK` opts in.
fn node_skipped() -> Option<&'static str> {
    if std::env::var_os(NETWORK).is_none_or(|value| value != "1") {
        Some("ESS_TUTORIAL_NETWORK=1 is not set, and `npm install` needs the npm registry")
    } else if !(on_path("node") && on_path("npm")) {
        Some("node or npm is not on PATH")
    } else {
        None
    }
}

/// The release target this platform's archives are named for, as `toolchain.rs` names it.
fn target() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        (arch, os) => panic!("ess publishes no archive for {arch}-{os}"),
    }
}

/// A release directory in the layout `ESS_TOOLCHAIN_BASE_URL` reads: `<version>/SHA256SUMS` and
/// `<version>/ess-<version>-<target>.tar.gz`, holding the built binary.
fn release(directory: &Path) {
    let package = format!("ess-{VERSION}-{}", target());
    let archive_name = format!("{package}.tar.gz");
    let binary = fs::read(env!("CARGO_BIN_EXE_ess")).expect("read the built ess");
    let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let mut header = tar::Header::new_gnu();
    header.set_size(binary.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder
        .append_data(&mut header, format!("{package}/ess"), binary.as_slice())
        .expect("archive the built ess");
    let archive = builder
        .into_inner()
        .and_then(flate2::write::GzEncoder::finish)
        .expect("compress the archive");
    let digest = Sha256::digest(&archive)
        .iter()
        .fold(String::new(), |mut text, byte| {
            use std::fmt::Write as _;
            let _ = write!(text, "{byte:02x}");
            text
        });
    let versioned = directory.join(VERSION);
    fs::create_dir_all(&versioned).expect("create the release directory");
    fs::write(versioned.join(&archive_name), &archive).expect("write the archive");
    fs::write(
        versioned.join("SHA256SUMS"),
        format!("{digest}  {archive_name}\n"),
    )
    .expect("write SHA256SUMS");
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("create {}: {e}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("read {}: {e}", from.display())) {
        let entry = entry.expect("a directory entry");
        let path = entry.path();
        let destination = to.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &destination);
        } else {
            fs::copy(&path, &destination)
                .unwrap_or_else(|e| panic!("copy {}: {e}", path.display()));
        }
    }
}

/// The walkthrough's state as it runs: where it is, and what it may reach.
struct Walkthrough {
    home: PathBuf,
    releases: PathBuf,
    here: PathBuf,
    checkout_here: PathBuf,
    /// Why the `requires=node` blocks are skipped, or `None` when they run.
    node_skipped: Option<&'static str>,
    npm_cache: Option<std::ffi::OsString>,
}

impl Walkthrough {
    fn new(scratch: &Path) -> Self {
        let home = scratch.join("home");
        let releases = scratch.join("releases");
        let checkout = scratch.join("checkout");
        fs::create_dir_all(&home).expect("create the home directory");
        release(&releases);
        copy_tree(&root().join("examples"), &checkout.join("examples"));
        // The test replaces HOME, so npm keeps the cache it would have used under the real one.
        let npm_cache = std::env::var_os("npm_config_cache").or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".npm").into_os_string())
        });
        Walkthrough {
            here: home.clone(),
            checkout_here: checkout,
            home,
            releases,
            node_skipped: node_skipped(),
            npm_cache,
        }
    }

    /// The printed form of `text`: the home directory reads `~`.
    fn printed(&self, text: &str) -> String {
        let mut text = text.to_owned();
        let mut homes = vec![self.home.display().to_string()];
        if let Ok(canonical) = self.home.canonicalize() {
            homes.insert(0, canonical.display().to_string());
        }
        for home in homes {
            text = text.replace(&home, "~");
        }
        text
    }

    fn block(&mut self, block: &Block) -> Result<(), String> {
        if let Some(path) = block.value("file") {
            let destination = self.home.join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", block.at()))?;
            }
            let mut body = block.body.join("\n");
            body.push('\n');
            return fs::write(&destination, body)
                .map_err(|e| format!("{}: write {path}: {e}", block.at()));
        }
        if let Some(path) = block.value("expect") {
            let actual = fs::read_to_string(self.home.join(path))
                .map_err(|e| format!("{}: read {path}: {e}", block.at()))?;
            let mut expected = block.body.join("\n");
            expected.push('\n');
            return if actual == expected {
                Ok(())
            } else {
                Err(format!(
                    "{}: the page says {path} now holds\n{expected}\nit holds\n{actual}",
                    block.at()
                ))
            };
        }
        if !block.shell() {
            return Err(format!(
                "{}: an `{MARK}` block is a shell block, or names `file=` or `expect=`",
                block.at()
            ));
        }
        let checkout = block.flags.contains("checkout");
        let requires = block.value("requires");
        let skip = match requires {
            None => false,
            Some("node") => self.node_skipped.is_some(),
            Some(other) => {
                return Err(format!(
                    "{}: unknown `requires={other}`; only `node`",
                    block.at()
                ))
            }
        };
        if skip {
            println!(
                "skipped {}: {}",
                block.at(),
                self.node_skipped.unwrap_or_default()
            );
        }
        for step in steps(block)? {
            self.step(&step, checkout, requires == Some("node"), skip)?;
        }
        Ok(())
    }

    fn step(&mut self, step: &Step, checkout: bool, node: bool, skip: bool) -> Result<(), String> {
        let at = format!("{PAGE}:{}", step.line);
        let here = if checkout {
            &mut self.checkout_here
        } else {
            &mut self.here
        };
        let mut words = step.words.iter().map(String::as_str).peekable();
        let mut environment = Vec::new();
        while let Some((name, value)) = words.peek().and_then(|word| word.split_once('=')) {
            environment.push((name.to_owned(), value.to_owned()));
            words.next();
        }
        let program = words
            .next()
            .ok_or_else(|| format!("{at}: an empty command"))?;
        let arguments: Vec<&str> = words.collect();
        // The directory builtins run even in a skipped block, so later blocks start where the
        // page says they do.
        match (program, arguments.as_slice()) {
            ("cd", [directory]) => {
                let next = here.join(directory);
                if !skip && !next.is_dir() {
                    return Err(format!("{at}: cd {directory}: no such directory"));
                }
                *here = next;
                return Ok(());
            }
            ("mkdir", ["-p", directory]) => {
                return fs::create_dir_all(here.join(directory))
                    .map_err(|e| format!("{at}: mkdir -p {directory}: {e}"));
            }
            ("cd" | "mkdir", _) => {
                return Err(format!(
                    "{at}: only `cd <dir>` and `mkdir -p <dir>` are run"
                ));
            }
            _ => {}
        }
        if skip {
            return Ok(());
        }
        let mut command = match program {
            "ess" => {
                let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
                command
                    .env_remove("XDG_CACHE_HOME")
                    .env_remove("ESS_TOOLCHAIN")
                    .env_remove("ESS_TOOLCHAIN_DIR")
                    .env_remove("ESS_TOOLCHAIN_DELEGATED")
                    .env("ESS_TOOLCHAIN_BASE_URL", &self.releases);
                command
            }
            "npm" if node => {
                let mut command = Command::new("npm");
                command
                    .env("npm_config_update_notifier", "false")
                    .env("npm_config_fund", "false");
                if let Some(cache) = &self.npm_cache {
                    command.env("npm_config_cache", cache);
                }
                command
            }
            other => {
                return Err(format!(
                    "{at}: `{other}` is not run here; `ess`, `cd`, `mkdir -p`, and `npm` in a \
                     `requires=node` block"
                ))
            }
        };
        let output = command
            .args(&arguments)
            .current_dir(&*here)
            .env("HOME", &self.home)
            .envs(environment)
            .output()
            .map_err(|e| format!("{at}: run {program}: {e}"))?;
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        if program == "ess" {
            text.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        let actual = trailing_blank_lines_dropped(&self.printed(&text));
        if !output.status.success() {
            return Err(format!(
                "{at}: `{}` exited {:?}\n{}\n{}",
                step.words.join(" "),
                output.status.code(),
                actual.join("\n"),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        if !matches(&step.expected, &actual) {
            return Err(format!(
                "{at}: `{}` printed other lines than the page records\n--- page\n{}\n--- printed\n{}",
                step.words.join(" "),
                step.expected.join("\n"),
                actual.join("\n")
            ));
        }
        Ok(())
    }
}

#[test]
fn the_walkthrough_runs_and_prints_what_the_page_records() {
    let blocks = blocks(&read(PAGE));
    let tutorial: Vec<&Block> = blocks.iter().filter(|b| b.tutorial()).collect();
    assert!(
        tutorial.iter().any(|b| b.value("file").is_some()) && tutorial.iter().any(|b| b.shell()),
        "{PAGE} has no `{MARK}` file block or no `{MARK}` command block"
    );
    let scratch = tempfile::tempdir().expect("a temporary directory");
    let mut walkthrough = Walkthrough::new(scratch.path());
    let mut commands = 0;
    for block in &tutorial {
        if block.shell() {
            commands += steps(block).map_or(0, |steps| steps.len());
        }
        if let Err(failure) = walkthrough.block(block) {
            panic!("{failure}");
        }
    }
    println!(
        "{PAGE}: {} block(s), {commands} command line(s) run{}",
        tutorial.len(),
        if walkthrough.node_skipped.is_none() {
            ""
        } else {
            ", the requires=node blocks skipped"
        }
    );
    std::io::stdout().flush().ok();
}

#[test]
fn every_ess_command_the_page_shows_is_run_by_the_walkthrough() {
    let unrun: Vec<String> = blocks(&read(PAGE))
        .iter()
        .filter(|b| b.shell() && !b.tutorial())
        .filter(|b| b.body.iter().any(|line| line.starts_with("$ ess ")))
        .map(Block::at)
        .collect();
    assert!(
        unrun.is_empty(),
        "these shell blocks show `$ ess` and carry no `{MARK}`, so nothing runs them: {unrun:?}"
    );
}

#[test]
fn the_walkthrough_writes_the_newest_format_and_pins_its_toolchain() {
    let newest = ess_domain::system::SUPPORTED_FORMATS
        .iter()
        .max()
        .expect("a supported format");
    let blocks = blocks(&read(PAGE));
    let tutorial: Vec<&Block> = blocks.iter().filter(|b| b.tutorial()).collect();
    let format = format!("format: ess/{newest}");
    assert!(
        tutorial
            .iter()
            .any(|b| b.value("file").is_some() && b.body.contains(&format)),
        "no `{MARK}` file block on {PAGE} writes `{format}`"
    );
    let pin = format!("$ ess specify toolchain install --pin {VERSION}");
    assert!(
        tutorial.iter().any(|b| b.shell() && b.body.contains(&pin)),
        "no `{MARK}` command block on {PAGE} runs `{pin}`"
    );
    let requires = format!("requires: ess {VERSION}");
    assert!(
        tutorial.iter().any(|b| b
            .value("expect")
            .is_some_and(|path| path.ends_with("ess-inputs.yaml"))
            && b.body.contains(&requires)),
        "no `{MARK}` block on {PAGE} shows the manifest holding `{requires}`"
    );
    assert!(
        tutorial.iter().any(|b| b.value("requires") == Some("node")),
        "{PAGE} has no `requires=node` block, so the TypeScript run is never checked"
    );
}

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|l| (*l).to_owned()).collect()
}

#[test]
fn a_changed_expected_line_does_not_match_and_an_elision_stands_for_any_lines() {
    let printed = lines(&["a", "b", "c", "d"]);
    assert!(matches(&lines(&["a", "b", "c", "d"]), &printed));
    assert!(!matches(&lines(&["a", "b", "x", "d"]), &printed));
    assert!(!matches(&lines(&["a", "b", "c"]), &printed));
    assert!(!matches(&lines(&["a", "b", "c", "d", "e"]), &printed));
    assert!(matches(&lines(&["a", ELISION, "d"]), &printed));
    assert!(matches(&lines(&[ELISION, "c", ELISION]), &printed));
    assert!(matches(&lines(&["a", "b", ELISION, "c", "d"]), &printed));
    assert!(!matches(&lines(&[ELISION, "x", ELISION]), &printed));
    assert!(!matches(&lines(&["b", ELISION]), &printed));
    assert!(matches(&lines(&[]), &[]));
    assert!(!matches(&lines(&[]), &printed));
}

#[test]
fn fence_attributes_read_flags_values_and_quoted_titles() {
    let (flags, values) =
        attributes("ess-tutorial file=tasks/spec/system.yaml title=\"spec/system.yaml\"");
    assert!(flags.contains(MARK));
    assert_eq!(
        values.get("file").map(String::as_str),
        Some("tasks/spec/system.yaml")
    );
    assert_eq!(
        values.get("title").map(String::as_str),
        Some("spec/system.yaml")
    );
}
