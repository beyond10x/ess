//! `cargo xtask slow-list`: the tests marked `#[ignore = "slow lane: <reason>"]`.
//!
//! The marker sits on the test, so it moves with the function when test binaries are merged, and
//! nextest's own `--run-ignored only` runs it. This module reads the integration tests of every
//! crate (`crates/<area>/<crate>/tests/**/*.rs`) with `syn` and prints what `task test-slow` needs:
//! the marked tests, their packages, the test targets that hold them and a nextest filterset.
//!
//! It names no `crate::` path, so `tests/slow_lane.rs` includes it with `#[path]`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use syn::visit::Visit;

/// The text an `ignore` reason starts with when it marks a test for the slow lane.
const MARKER: &str = "slow lane:";

/// One marked test.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Marked {
    /// The package name from the crate's `Cargo.toml`.
    pub package: String,
    /// The first path component below `tests/`: a file stem or a directory name.
    pub target: String,
    /// The function name.
    pub test: String,
    /// The file that holds it, relative to the workspace root, with `/` separators.
    pub file: String,
}

/// What `cargo xtask slow-list` prints.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Print the package names that hold a marked test, one per line.
    #[arg(long)]
    pub packages: bool,
    /// Restrict to one package; with `--targets` or `--filter`.
    #[arg(long)]
    pub package: Option<String>,
    /// Print the test targets of `--package` that hold a marked test, one per line.
    #[arg(long, requires = "package", conflicts_with_all = ["packages", "filter"])]
    pub targets: bool,
    /// Print one nextest filterset that selects the marked tests of `--package`.
    #[arg(long, requires = "package", conflicts_with_all = ["packages", "targets"])]
    pub filter: bool,
}

/// Runs the command against the workspace at `root`.
pub fn run(root: &Path, args: &Args) -> Result<String, String> {
    let marked = scan(root)?;
    let mut out = String::new();
    if args.packages {
        let packages: BTreeSet<&str> = marked.iter().map(|m| m.package.as_str()).collect();
        for package in packages {
            out.push_str(package);
            out.push('\n');
        }
        return Ok(out);
    }
    if let Some(package) = &args.package {
        let own = marked.iter().filter(|m| &m.package == package);
        if args.targets {
            let targets: BTreeSet<&str> = own.map(|m| m.target.as_str()).collect();
            for target in targets {
                out.push_str(target);
                out.push('\n');
            }
        } else if args.filter {
            let names: BTreeSet<&str> = own.map(|m| m.test.as_str()).collect();
            if names.is_empty() {
                return Err(format!("no test of package `{package}` is marked"));
            }
            let names: Vec<&str> = names.into_iter().collect();
            out = format!("test(/(^|::)({})$/)\n", names.join("|"));
        } else {
            return Err("--package needs --targets or --filter".to_owned());
        }
        return Ok(out);
    }
    for m in &marked {
        let _ = writeln!(out, "{}\t{}\t{}", m.package, m.target, m.test);
    }
    Ok(out)
}

/// Every marked test under `root`, sorted by package, target and test name.
pub fn scan(root: &Path) -> Result<Vec<Marked>, String> {
    let mut marked = Vec::new();
    for area in sorted_dirs(&root.join("crates")) {
        for krate in sorted_dirs(&area) {
            let tests = krate.join("tests");
            let manifest = krate.join("Cargo.toml");
            if !tests.is_dir() || !manifest.is_file() {
                continue;
            }
            let package = package_name(&manifest)?;
            let mut files = Vec::new();
            rust_files(&tests, &mut files)?;
            files.sort();
            for file in files {
                let text = fs::read_to_string(&file)
                    .map_err(|error| format!("reading {}: {error}", file.display()))?;
                if !text.contains(MARKER) {
                    continue;
                }
                let relative = file
                    .strip_prefix(root)
                    .unwrap_or(&file)
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let parsed = syn::parse_file(&text)
                    .map_err(|error| format!("{relative}: does not parse: {error}"))?;
                let target = target_of(&tests, &file);
                let mut visitor = Visitor {
                    file: &relative,
                    found: Vec::new(),
                    errors: Vec::new(),
                };
                visitor.visit_file(&parsed);
                if let Some(error) = visitor.errors.into_iter().next() {
                    return Err(error);
                }
                if !visitor.found.is_empty() && !is_target_file(&tests, &file) {
                    let line = text
                        .lines()
                        .position(|line| line.contains(MARKER))
                        .map_or(1, |index| index + 1);
                    return Err(format!(
                        "{relative}:{line}: a `{MARKER}` marker sits in a file that is no test \
                         target and that a target may include; a marker belongs in a file cargo \
                         compiles as a test target: `tests/<name>.rs` or `tests/<name>/main.rs`"
                    ));
                }
                for test in visitor.found {
                    marked.push(Marked {
                        package: package.clone(),
                        target: target.clone(),
                        test,
                        file: relative.clone(),
                    });
                }
            }
        }
    }
    let mut seen: BTreeMap<(&str, &str), &str> = BTreeMap::new();
    for m in &marked {
        if let Some(first) = seen.insert((&m.package, &m.test), &m.file) {
            return Err(format!(
                "package `{}` has two marked tests named `{}`: {first} and {}",
                m.package, m.test, m.file
            ));
        }
    }
    marked.sort();
    Ok(marked)
}

fn sorted_dirs(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("reading {}: {error}", directory.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "fixtures") {
                continue;
            }
            rust_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Whether `file` belongs to a test target cargo compiles: `tests/<name>.rs`, or a file below
/// `tests/<name>/` when that directory holds a `main.rs`. A file in a directory without one is
/// a module another target includes, and no `--test` name reaches it.
fn is_target_file(tests: &Path, file: &Path) -> bool {
    let Ok(rest) = file.strip_prefix(tests) else {
        return false;
    };
    let mut parts = rest.components();
    let (Some(first), second) = (parts.next(), parts.next()) else {
        return false;
    };
    second.is_none() || tests.join(first.as_os_str()).join("main.rs").is_file()
}

fn target_of(tests: &Path, file: &Path) -> String {
    let first = file
        .strip_prefix(tests)
        .ok()
        .and_then(|rest| rest.components().next())
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    first.strip_suffix(".rs").unwrap_or(&first).to_owned()
}

fn package_name(manifest: &Path) -> Result<String, String> {
    let text = fs::read_to_string(manifest)
        .map_err(|error| format!("reading {}: {error}", manifest.display()))?;
    let mut in_package = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package {
            if let Some(rest) = line.strip_prefix("name") {
                if let Some(value) = rest.trim_start().strip_prefix('=') {
                    return Ok(value.trim().trim_matches('"').to_owned());
                }
            }
        }
    }
    Err(format!("{}: no package name", manifest.display()))
}

struct Visitor<'a> {
    file: &'a str,
    found: Vec<String>,
    errors: Vec<String>,
}

impl Visitor<'_> {
    fn reason(attribute: &syn::Attribute) -> Option<String> {
        if !attribute.path().is_ident("ignore") {
            return None;
        }
        let syn::Meta::NameValue(pair) = &attribute.meta else {
            return None;
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(text),
            ..
        }) = &pair.value
        else {
            return None;
        };
        text.value().strip_prefix(MARKER).map(str::to_owned)
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let tokens = mac.tokens.to_string();
        if tokens.contains(&format!("ignore = \"{MARKER}")) {
            self.errors.push(format!(
                "{}: a `{MARKER}` marker sits inside a macro body or call, so the test it marks \
                 cannot be named; mark a plain `fn` that carries #[test]",
                self.file
            ));
        }
        syn::visit::visit_macro(self, mac);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let name = function.sig.ident.to_string();
        let is_test = function.attrs.iter().any(|attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "test")
        });
        for attribute in &function.attrs {
            let Some(reason) = Self::reason(attribute) else {
                continue;
            };
            if !is_test {
                self.errors.push(format!(
                    "{}: `{name}` carries a `{MARKER}` marker but is not a #[test]",
                    self.file
                ));
            } else if reason.trim().is_empty() {
                self.errors.push(format!(
                    "{}: `{name}` carries a `{MARKER}` marker with no reason",
                    self.file
                ));
            } else {
                self.found.push(name.clone());
            }
        }
        syn::visit::visit_item_fn(self, function);
    }
}
