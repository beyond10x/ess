//! No test writes into `CARGO_TARGET_TMPDIR` before creating it.
//!
//! A CI test shard unpacks its test binaries into a fresh `target/`, which has no `target/tmp`; a
//! local tree keeps the directory from earlier runs. A test that puts a file or directory straight
//! into it passes locally and fails in the shard with `NotFound`, as `fc1ffeb859` did. This check
//! refuses such a call unless an earlier `create_dir_all` in the same function creates the directory.

use quote::ToTokens;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::visit::Visit;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.contains("[workspace]"))
        })
        .expect("a workspace manifest stands above this crate")
        .to_path_buf()
}

/// Tracked and untracked-but-not-ignored Rust files, so a new test is seen before its commit.
fn rust_files(root: &Path) -> Vec<String> {
    let listed = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            "*.rs",
        ])
        .current_dir(root)
        .output()
        .expect("git ls-files runs");
    assert!(listed.status.success(), "git ls-files failed");
    listed
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8(path.to_vec()).expect("a UTF-8 path"))
        .filter(|path| root.join(path).is_file())
        .collect()
}

/// One call that needs the directory to exist.
#[derive(Debug, PartialEq, Eq)]
struct Finding {
    function: String,
    call: String,
    argument: String,
}

/// Every finding of `source`, in source order.
fn findings(source: &str) -> Result<Vec<Finding>, syn::Error> {
    let file = syn::parse_file(source)?;
    let mut functions = Functions { findings: vec![] };
    functions.visit_file(&file);
    Ok(functions.findings)
}

/// Visits every function, nested ones included, and walks each body on its own.
struct Functions {
    findings: Vec<Finding>,
}

impl Functions {
    fn walk(&mut self, name: &syn::Ident, block: &syn::Block) {
        let mut body = Body {
            function: name.to_string(),
            roots: BTreeSet::new(),
            children: BTreeSet::new(),
            created: false,
            findings: vec![],
        };
        body.visit_block(block);
        self.findings.append(&mut body.findings);
    }
}

impl<'ast> Visit<'ast> for Functions {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.walk(&function.sig.ident, &function.block);
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.walk(&function.sig.ident, &function.block);
        syn::visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast syn::TraitItemFn) {
        if let Some(block) = &function.default {
            self.walk(&function.sig.ident, block);
        }
        syn::visit::visit_trait_item_fn(self, function);
    }
}

/// The walker of one function body.
struct Body {
    function: String,
    roots: BTreeSet<String>,
    children: BTreeSet<String>,
    created: bool,
    findings: Vec<Finding>,
}

fn strip(mut expression: &syn::Expr) -> &syn::Expr {
    loop {
        match expression {
            syn::Expr::Reference(inner) => expression = &inner.expr,
            syn::Expr::Paren(inner) => expression = &inner.expr,
            other => return other,
        }
    }
}

fn is_target_tmpdir_macro(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|last| last.ident == "env")
        && mac.tokens.to_string() == "\"CARGO_TARGET_TMPDIR\""
}

fn single_ident(path: &syn::ExprPath) -> Option<String> {
    (path.qself.is_none() && path.path.segments.len() == 1)
        .then(|| path.path.segments[0].ident.to_string())
}

fn segments(path: &syn::ExprPath) -> Vec<String> {
    path.path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect()
}

impl Body {
    fn is_root(&self, expression: &syn::Expr) -> bool {
        match strip(expression) {
            syn::Expr::Macro(call) => is_target_tmpdir_macro(&call.mac),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = strip(&call.func) else {
                    return false;
                };
                let names = segments(path);
                let [.., owner, name] = names.as_slice() else {
                    return false;
                };
                matches!(
                    (owner.as_str(), name.as_str()),
                    ("Path", "new") | ("PathBuf", "from")
                ) && call.args.len() == 1
                    && self.is_root(&call.args[0])
            }
            syn::Expr::MethodCall(call) => {
                matches!(
                    call.method.to_string().as_str(),
                    "to_path_buf" | "to_owned" | "into"
                ) && call.args.is_empty()
                    && self.is_root(&call.receiver)
            }
            syn::Expr::Path(path) => single_ident(path).is_some_and(|n| self.roots.contains(&n)),
            _ => false,
        }
    }

    fn is_child(&self, expression: &syn::Expr) -> bool {
        match strip(expression) {
            syn::Expr::MethodCall(call) => match call.method.to_string().as_str() {
                "join" => call.args.len() == 1 && self.is_root(&call.receiver),
                "to_path_buf" | "to_owned" => call.args.is_empty() && self.is_child(&call.receiver),
                _ => false,
            },
            syn::Expr::Path(path) => single_ident(path).is_some_and(|n| self.children.contains(&n)),
            _ => false,
        }
    }

    fn mentions(&self, expression: &syn::Expr) -> bool {
        struct Mention<'a> {
            body: &'a Body,
            found: bool,
        }
        impl<'ast> Visit<'ast> for Mention<'_> {
            fn visit_expr_macro(&mut self, call: &'ast syn::ExprMacro) {
                self.found |= is_target_tmpdir_macro(&call.mac);
                syn::visit::visit_expr_macro(self, call);
            }
            fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
                self.found |= single_ident(path).is_some_and(|n| {
                    self.body.roots.contains(&n) || self.body.children.contains(&n)
                });
            }
        }
        let mut mention = Mention {
            body: self,
            found: false,
        };
        mention.visit_expr(expression);
        mention.found
    }

    fn record(&mut self, call: &str, argument: &syn::Expr) {
        if !self.created {
            self.findings.push(Finding {
                function: self.function.clone(),
                call: call.to_string(),
                argument: argument.to_token_stream().to_string(),
            });
        }
    }
}

impl<'ast> Visit<'ast> for Body {
    fn visit_item(&mut self, _item: &'ast syn::Item) {}

    fn visit_local(&mut self, local: &'ast syn::Local) {
        syn::visit::visit_local(self, local);
        let mut pattern = &local.pat;
        if let syn::Pat::Type(typed) = pattern {
            pattern = &typed.pat;
        }
        let syn::Pat::Ident(ident) = pattern else {
            return;
        };
        if ident.subpat.is_some() {
            return;
        }
        let Some(init) = &local.init else {
            return;
        };
        let name = ident.ident.to_string();
        self.roots.remove(&name);
        self.children.remove(&name);
        if self.is_root(&init.expr) {
            self.roots.insert(name);
        } else if self.is_child(&init.expr) {
            self.children.insert(name);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        syn::visit::visit_expr_call(self, call);
        let syn::Expr::Path(path) = strip(&call.func) else {
            return;
        };
        let names = segments(path);
        let Some(last) = names.last() else {
            return;
        };
        let before = names.len().checked_sub(2).map(|i| names[i].as_str());
        let first = call.args.first();
        if last == "create_dir_all" {
            if call.args.iter().any(|argument| self.mentions(argument)) {
                self.created = true;
            }
            return;
        }
        let hit = match (last.as_str(), before) {
            ("tempdir_in" | "tempfile_in" | "new_in", _) => {
                first.filter(|argument| self.is_root(argument))
            }
            ("create_dir", _) | ("write", Some("fs")) | ("create", Some("File")) => {
                first.filter(|argument| self.is_child(argument))
            }
            ("copy", Some("fs")) => call.args.iter().nth(1).filter(|a| self.is_child(a)),
            _ => None,
        };
        if let Some(argument) = hit {
            self.record(last, argument);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        syn::visit::visit_expr_method_call(self, call);
        let method = call.method.to_string();
        if matches!(
            method.as_str(),
            "tempdir_in" | "tempfile_in" | "current_dir"
        ) {
            if let Some(argument) = call.args.first().filter(|a| self.is_root(a)) {
                self.record(&method, argument);
            }
        }
    }
}

fn names(source: &str) -> Vec<(String, String)> {
    findings(source)
        .expect("the source parses")
        .into_iter()
        .map(|f| (f.function, f.call))
        .collect()
}

fn in_f(body: &str) -> Vec<(String, String)> {
    names(&format!("fn f() {{ {body} }}"))
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

const EVERY_SHAPE: &str = r#"
    let a = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let b = tempfile::Builder::new().prefix("p-").tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let c = tempfile::TempDir::new_in(root).unwrap();
    Command::new("go").current_dir(env!("CARGO_TARGET_TMPDIR")).status().unwrap();
    let made = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("x-{}", std::process::id()));
    std::fs::create_dir(&made).expect("fresh");
    let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&written, actual).expect("w");
    let mut file = fs::File::create(Path::new(env!("CARGO_TARGET_TMPDIR")).join("suite.json")).unwrap();
    fs::copy("a", &written).unwrap();
"#;

#[test]
fn a_tempdir_in_the_bare_directory_is_a_finding() {
    let got = in_f(r#"let d = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();"#);
    assert_eq!(got, pairs(&[("f", "tempdir_in")]));
}

#[test]
fn the_shape_fc1ffeb859_fixed_is_a_finding_before_and_none_after() {
    let body = r#"let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
        let adapter = dir.path().join("adapter.sh"); fs::write(&adapter, "x").unwrap();"#;
    let before = names(&format!("fn left_behind_after() {{ {body} }}"));
    assert_eq!(before, pairs(&[("left_behind_after", "tempdir_in")]));
    let fixed = format!(
        r#"fn left_behind_after() {{ fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).unwrap(); {body} }}"#
    );
    assert_eq!(names(&fixed), pairs(&[]));
}

#[test]
fn a_builder_tempdir_in_is_a_finding() {
    let got = in_f(
        r#"let d = tempfile::Builder::new().prefix("p-").tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();"#,
    );
    assert_eq!(got, pairs(&[("f", "tempdir_in")]));
}

#[test]
fn a_tempdir_new_in_a_bound_root_is_a_finding() {
    let got = in_f(
        r#"let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")); let d = tempfile::TempDir::new_in(root).unwrap();"#,
    );
    assert_eq!(got, pairs(&[("f", "new_in")]));
}

#[test]
fn a_current_dir_of_the_bare_directory_is_a_finding() {
    let got =
        in_f(r#"Command::new("go").current_dir(env!("CARGO_TARGET_TMPDIR")).status().unwrap();"#);
    assert_eq!(got, pairs(&[("f", "current_dir")]));
}

#[test]
fn a_create_dir_of_a_direct_child_is_a_finding() {
    let got = in_f(
        r#"let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("x-{}", std::process::id())); std::fs::create_dir(&root).expect("fresh");"#,
    );
    assert_eq!(got, pairs(&[("f", "create_dir")]));
}

#[test]
fn a_write_of_a_direct_child_is_a_finding() {
    let got = in_f(
        r#"let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name); std::fs::write(&written, actual).expect("w");"#,
    );
    assert_eq!(got, pairs(&[("f", "write")]));
}

#[test]
fn a_file_create_of_a_direct_child_is_a_finding() {
    let got = in_f(
        r#"let mut file = fs::File::create(Path::new(env!("CARGO_TARGET_TMPDIR")).join("suite.json")).unwrap();"#,
    );
    assert_eq!(got, pairs(&[("f", "create")]));
}

#[test]
fn creating_the_directory_first_clears_every_shape() {
    let got = in_f(&format!(
        r#"fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).unwrap(); {EVERY_SHAPE}"#
    ));
    assert_eq!(got, pairs(&[]));
    assert_eq!(in_f(EVERY_SHAPE).len(), 8);
}

#[test]
fn create_dir_all_of_a_path_below_clears_it() {
    let got = in_f(
        r#"let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("a"); fs::create_dir_all(root.join("model")).unwrap(); fs::create_dir(&root).ok();"#,
    );
    assert_eq!(got, pairs(&[]));
}

#[test]
fn writing_below_a_subdirectory_is_not_a_finding() {
    let got = in_f(
        r#"let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("a"); let file = dir.join("f"); fs::write(dir.join("f"), "").unwrap(); fs::write(&file, "").unwrap();"#,
    );
    assert_eq!(got, pairs(&[]));
}

#[test]
fn handing_a_subdirectory_to_a_command_is_not_a_finding() {
    let got = in_f(
        r#"Command::new("cargo").env("CARGO_TARGET_DIR", Path::new(env!("CARGO_TARGET_TMPDIR")).join("t")).status().unwrap();"#,
    );
    assert_eq!(got, pairs(&[]));
}

#[test]
fn a_name_bound_again_is_no_longer_tracked() {
    let got = in_f(
        r#"let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("a"); let root = PathBuf::from("elsewhere").join("b"); fs::create_dir(&root).unwrap();"#,
    );
    assert_eq!(got, pairs(&[]));
}

#[test]
fn a_later_create_does_not_clear_an_earlier_call() {
    let got = in_f(
        r#"let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("a"); std::fs::create_dir(&root).unwrap(); std::fs::create_dir_all(root.join("src")).unwrap();"#,
    );
    assert_eq!(got, pairs(&[("f", "create_dir")]));
}

#[test]
fn a_method_and_a_nested_function_are_walked() {
    let got = names(
        r#"struct S; impl S { fn m(&self) { let d = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")); } }
        fn outer() { fn inner() { let d = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")); } }"#,
    );
    assert_eq!(got, pairs(&[("m", "tempdir_in"), ("inner", "tempdir_in")]));
}

#[test]
fn an_unparsable_source_is_an_error() {
    assert!(findings("fn (").is_err());
}

#[test]
fn no_test_writes_into_cargo_target_tmpdir_before_creating_it() {
    let root = workspace_root();
    let mut lines = vec![];
    for path in rust_files(&root) {
        if !path.contains("/tests/") {
            continue;
        }
        let Ok(text) = fs::read_to_string(root.join(&path)) else {
            continue;
        };
        if !text.contains("CARGO_TARGET_TMPDIR") {
            continue;
        }
        match findings(&text) {
            Err(error) => lines.push(format!("{path}: could not be parsed: {error}")),
            Ok(found) => lines.extend(found.into_iter().map(|f| {
                format!(
                    "{path}: fn {}: {}({}) needs CARGO_TARGET_TMPDIR to exist and nothing before it in the function creates it; add std::fs::create_dir_all(env!(\"CARGO_TARGET_TMPDIR\")) before it",
                    f.function, f.call, f.argument
                )
            })),
        }
    }
    assert!(lines.is_empty(), "\n{}", lines.join("\n"));
}
