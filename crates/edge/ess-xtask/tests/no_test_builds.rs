//! No new test spawns cargo.
//!
//! A test that runs `cargo build` compiles a crate graph inside the test: the browser-product
//! lane spent 0.68.0's release about 7.4 hours in its tests' wasm host builds. What a new test
//! needs built is built once, by a Taskfile step before the test binary runs, and handed to the
//! test as a path. The files below spawn cargo today; the list only shrinks.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use syn::visit::Visit;

/// Every file whose test code spawns cargo today, in byte order. Remove an entry when its file
/// stops; never add one.
const ALLOWED: &[&str] = &[
    "crates/edge/ess-cli/tests/adversary_tui_app_pass1.rs",
    "crates/edge/ess-cli/tests/adversary_tui_app_pass2.rs",
    "crates/edge/ess-cli/tests/browser_product_adversary.rs",
    "crates/edge/ess-cli/tests/browser_response_conformance.rs",
    "crates/edge/ess-cli/tests/generate_ui_tui.rs",
    "crates/edge/ess-xtask/tests/layout.rs",
    "crates/generate/ess-cli-project/tests/adversarial.rs",
    "crates/generate/ess-cli-project/tests/json_results.rs",
    "crates/generate/ess-cli-project/tests/projection.rs",
    "crates/generate/ess-cli-project/tests/workspace.rs",
    "crates/generate/ess-publisher/tests/adversary_391_pass1.rs",
    "crates/generate/ess-publisher/tests/generated.rs",
    "crates/generate/ess-synth/tests/actor_grants.rs",
    "crates/generate/ess-synth/tests/adversary_binding_condition_kleene.rs",
    "crates/generate/ess-synth/tests/adversary_c_event_wire.rs",
    "crates/generate/ess-synth/tests/adversary_c_event_wire_2.rs",
    "crates/generate/ess-synth/tests/adversary_e_u5_now_stored_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_e_u7_wire.rs",
    "crates/generate/ess-synth/tests/adversary_e_u9_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_go_behaviour_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_go_behaviour_pass2.rs",
    "crates/generate/ess-synth/tests/adversary_k5_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_names_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_served_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_served_pass1_gatepass.rs",
    "crates/generate/ess-synth/tests/adversary_served_pass2.rs",
    "crates/generate/ess-synth/tests/adversary_served_pass2_at_most_once.rs",
    "crates/generate/ess-synth/tests/adversary_upsert_by_existence_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_view_params_pass1.rs",
    "crates/generate/ess-synth/tests/adversary_view_params_pass2.rs",
    "crates/generate/ess-synth/tests/adversary_w3_2_pass1.rs",
    "crates/generate/ess-synth/tests/binding_condition_boolean.rs",
    "crates/generate/ess-synth/tests/binding_condition_runtime.rs",
    "crates/generate/ess-synth/tests/coalesce_guard_behaviour.rs",
    "crates/generate/ess-synth/tests/declared_behaviour.rs",
    "crates/generate/ess-synth/tests/emit_swap_served.rs",
    "crates/generate/ess-synth/tests/expression_a1_guards.rs",
    "crates/generate/ess-synth/tests/expression_a2_guards.rs",
    "crates/generate/ess-synth/tests/expression_a4_values.rs",
    "crates/generate/ess-synth/tests/expression_roots.rs",
    "crates/generate/ess-synth/tests/expression_utf8_bytes.rs",
    "crates/generate/ess-synth/tests/expression_values.rs",
    "crates/generate/ess-synth/tests/external_before_row_served.rs",
    "crates/generate/ess-synth/tests/external_narrowed_served.rs",
    "crates/generate/ess-synth/tests/external_request_binding.rs",
    "crates/generate/ess-synth/tests/feasibility.rs",
    "crates/generate/ess-synth/tests/feasibility_adversary.rs",
    "crates/generate/ess-synth/tests/finite_boolean.rs",
    "crates/generate/ess-synth/tests/generated_view_queries.rs",
    "crates/generate/ess-synth/tests/http_binding/go.rs",
    "crates/generate/ess-synth/tests/http_binding/shared.rs",
    "crates/generate/ess-synth/tests/identity_changing_updates.rs",
    "crates/generate/ess-synth/tests/invariant_check.rs",
    "crates/generate/ess-synth/tests/json_clap.rs",
    "crates/generate/ess-synth/tests/json_primitive.rs",
    "crates/generate/ess-synth/tests/json_web.rs",
    "crates/generate/ess-synth/tests/minted_value_sources.rs",
    "crates/generate/ess-synth/tests/nested_response_wasm.rs",
    "crates/generate/ess-synth/tests/now_stored_rows.rs",
    "crates/generate/ess-synth/tests/one_time_wasm.rs",
    "crates/generate/ess-synth/tests/optional_generated_port.rs",
    "crates/generate/ess-synth/tests/optional_recursive_codecs.rs",
    "crates/generate/ess-synth/tests/related_guard_served/mod.rs",
    "crates/generate/ess-synth/tests/served_codec_names.rs",
    "crates/generate/ess-synth/tests/served_correction_pass1.rs",
    "crates/generate/ess-synth/tests/served_correction_pass2_bridge.rs",
    "crates/generate/ess-synth/tests/served_entry.rs",
    "crates/generate/ess-synth/tests/served_publication.rs",
    "crates/generate/ess-synth/tests/served_returns.rs",
    "crates/generate/ess-synth/tests/served_unfinished_committed.rs",
    "crates/generate/ess-synth/tests/set_effects.rs",
    "crates/generate/ess-synth/tests/single_crate_layout.rs",
    "crates/generate/ess-synth/tests/subject_state_source.rs",
    "crates/generate/ess-synth/tests/transport_free_entry.rs",
    "crates/generate/ess-synth/tests/typed_text_operands.rs",
    "crates/generate/ess-synth/tests/union_unit_variants.rs",
    "crates/generate/ess-synth/tests/union_unit_variants_adversary.rs",
    "crates/generate/ess-synth/tests/upsert_by_existence.rs",
    "crates/generate/ess-synth/tests/view_grant_security.rs",
    "crates/generate/ess-synth/tests/view_grants.rs",
    "crates/generate/ess-synth/tests/view_params_served.rs",
    "crates/generate/schema-contract/src/realize.rs",
    "crates/generate/schema-contract/tests/adversary_exact_numbers_pass1.rs",
    "crates/generate/schema-contract/tests/adversary_model_ergonomics.rs",
    "crates/generate/schema-contract/tests/binary64_adversary.rs",
    "crates/generate/schema-contract/tests/binary64_structural.rs",
    "crates/generate/schema-contract/tests/binary64_structural_adversary.rs",
    "crates/generate/schema-contract/tests/enum_attributes.rs",
    "crates/generate/schema-contract/tests/model_ergonomics.rs",
    "crates/generate/schema-contract/tests/model_ergonomics/shorthand.rs",
    "crates/generate/schema-contract/tests/normalization_binary64_targets.rs",
    "crates/generate/schema-contract/tests/normalization_positional_adversary.rs",
    "crates/generate/schema-contract/tests/normalization_positional_targets.rs",
    "crates/generate/schema-contract/tests/normalization_raw_adversary.rs",
    "crates/generate/schema-contract/tests/normalization_raw_targets.rs",
    "crates/generate/schema-contract/tests/normalization_rust.rs",
    "crates/generate/schema-contract/tests/rust_realization.rs",
    "crates/generate/schema-contract/tests/types_manifest_features.rs",
    "crates/ui/ess-ui-react/tests/live_binding.rs",
    "crates/ui/ess-ui-tui/tests/http_adapter.rs",
    "crates/verify/ess-conformance/tests/aggregate_group_selection.rs",
    "crates/verify/ess-conformance/tests/support_initial_state/mod.rs",
];

/// Files that must never enter [`ALLOWED`]: the Firefox harness every browser test shares.
const NEVER_ALLOWED: &[&str] = &["crates/edge/ess-cli/tests/support/browser.rs"];

/// The browser-product tests build their wasm hosts with cargo. They build into one shared
/// target, named by `CARGO_TARGET_TMPDIR` or `ESS_BROWSER_TARGET_DIR`, on cargo's default job
/// count: `CARGO_BUILD_JOBS=1` compiled the whole `ess-conformance` tree on one core per host.
const BROWSER_BUILDERS: &[&str] = &[
    "crates/edge/ess-cli/tests/browser_product_adversary.rs",
    "crates/edge/ess-cli/tests/browser_response_conformance.rs",
];

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

/// Whether `tokens` spawn cargo: `Command::new("cargo")`, `env!("CARGO")`, or
/// `var("CARGO")`/`var_os("CARGO")`, at any depth.
fn spawns_cargo(tokens: TokenStream) -> bool {
    let tokens: Vec<TokenTree> = tokens.into_iter().collect();
    let first_literal = |tree: &TokenTree| match tree {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => group
            .stream()
            .into_iter()
            .next()
            .map(|token| token.to_string()),
        _ => None,
    };
    for (index, tree) in tokens.iter().enumerate() {
        if let TokenTree::Group(group) = tree {
            if spawns_cargo(group.stream()) {
                return true;
            }
        }
        let TokenTree::Ident(ident) = tree else {
            continue;
        };
        let name = ident.to_string();
        let next = tokens.get(index + 1);
        let found = match name.as_str() {
            "new" => next.and_then(first_literal).as_deref() == Some("\"cargo\""),
            "var" | "var_os" => next.and_then(first_literal).as_deref() == Some("\"CARGO\""),
            "env" => {
                matches!(next, Some(TokenTree::Punct(punct)) if punct.as_char() == '!')
                    && tokens.get(index + 2).and_then(first_literal).as_deref() == Some("\"CARGO\"")
            }
            _ => false,
        };
        if found {
            return true;
        }
    }
    false
}

fn file_spawns_cargo(source: &str) -> bool {
    TokenStream::from_str(source).map_or_else(
        |_| source.contains("Command::new(\"cargo\")") || source.contains("env!(\"CARGO\")"),
        spawns_cargo,
    )
}

fn is_test_attribute(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        let path = attribute.path();
        if path.is_ident("test") {
            return true;
        }
        path.is_ident("cfg")
            && attribute
                .meta
                .require_list()
                .is_ok_and(|list| list.tokens.to_string() == "test")
    })
}

/// The test code of one `src` file: `#[test]` functions, inline `#[cfg(test)]` modules, and the
/// files of out-of-line `#[cfg(test)] mod name;` declarations.
struct TestCode<'a> {
    file: &'a Path,
    module_directory: PathBuf,
    spawns: bool,
    test_files: Vec<PathBuf>,
}

impl<'ast> Visit<'ast> for TestCode<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if is_test_attribute(&item.attrs) {
            self.spawns |= spawns_cargo(quote::ToTokens::to_token_stream(item));
        } else {
            syn::visit::visit_item_fn(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let test = is_test_attribute(&item.attrs);
        let name = item.ident.to_string();
        match &item.content {
            Some(_) if test => self.spawns |= spawns_cargo(quote::ToTokens::to_token_stream(item)),
            Some(_) => {
                let outer = self.module_directory.clone();
                self.module_directory = outer.join(&name);
                syn::visit::visit_item_mod(self, item);
                self.module_directory = outer;
            }
            None if test => {
                let explicit = item.attrs.iter().find_map(|attribute| {
                    let syn::Meta::NameValue(pair) = &attribute.meta else {
                        return None;
                    };
                    let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(path),
                        ..
                    }) = &pair.value
                    else {
                        return None;
                    };
                    pair.path
                        .is_ident("path")
                        .then(|| self.file.parent().unwrap().join(path.value()))
                });
                let flat = self.module_directory.join(format!("{name}.rs"));
                let nested = self.module_directory.join(&name).join("mod.rs");
                self.test_files.push(explicit.unwrap_or(if flat.is_file() {
                    flat
                } else {
                    nested
                }));
            }
            None => {}
        }
    }
}

/// Every repository-relative file whose test code spawns cargo.
fn spawning_files(root: &Path) -> BTreeSet<String> {
    let files = rust_files(root);
    let mut whole: BTreeSet<PathBuf> = BTreeSet::new();
    let mut found = BTreeSet::new();
    for relative in &files {
        let path = Path::new(relative);
        let in_tests = path.components().any(|part| part.as_os_str() == "tests")
            || path.file_name().is_some_and(|name| name == "tests.rs");
        if in_tests {
            whole.insert(root.join(path));
            continue;
        }
        if !path.components().any(|part| part.as_os_str() == "src") {
            continue;
        }
        let file = root.join(path);
        let source = fs::read_to_string(&file).unwrap();
        let Ok(parsed) = syn::parse_file(&source) else {
            continue;
        };
        let stem = file.file_stem().unwrap();
        let module_directory = if ["main", "lib", "mod"].iter().any(|root| stem == *root) {
            file.parent().unwrap().to_path_buf()
        } else {
            file.parent().unwrap().join(stem)
        };
        let mut code = TestCode {
            file: &file,
            module_directory,
            spawns: false,
            test_files: Vec::new(),
        };
        code.visit_file(&parsed);
        if code.spawns {
            found.insert(relative.clone());
        }
        whole.extend(code.test_files);
    }
    for file in whole {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        if file_spawns_cargo(&source) {
            let relative = file.strip_prefix(root).unwrap();
            found.insert(relative.to_string_lossy().into_owned());
        }
    }
    found
}

#[test]
fn no_test_spawns_cargo_outside_the_shrinking_allowlist() {
    let mut sorted = ALLOWED.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted, ALLOWED,
        "ALLOWED is sorted and holds each file once"
    );
    for file in NEVER_ALLOWED {
        assert!(
            !ALLOWED.contains(file),
            "{file} is in ALLOWED: its lane runs for hours when its tests build"
        );
    }
    let root = workspace_root();
    let found = spawning_files(&root);
    let unexpected: Vec<&String> = found
        .iter()
        .filter(|file| !ALLOWED.contains(&file.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "these test files spawn cargo: {unexpected:#?}\n\
         A test that builds a crate graph turns its lane into a multi-hour one. Build what the test \
         needs once in a Taskfile step before the test binary runs, and hand the test the \
         artifact's path."
    );
    let stale: Vec<&&str> = ALLOWED
        .iter()
        .filter(|file| !found.contains(**file))
        .collect();
    assert!(
        stale.is_empty(),
        "these ALLOWED files no longer spawn cargo; remove them from ALLOWED: {stale:#?}"
    );
}

/// Whether `attributes` hold `#[cfg(not(test))]`: an item left out of the unit-test build.
fn is_production_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .meta
                .require_list()
                .is_ok_and(|list| list.tokens.to_string().replace(' ', "") == "not(test)")
    })
}

/// One call a body makes: `name(…)`, `<qualifier>::name(…)` or `.name(…)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Call {
    qualifier: Option<String>,
    method: bool,
    name: String,
}

/// Every call in `tokens`, at any depth.
fn calls(tokens: TokenStream, into: &mut BTreeSet<Call>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        match tree {
            TokenTree::Group(group) => calls(group.stream(), into),
            TokenTree::Ident(ident) => {
                let called = matches!(trees.get(index + 1),
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis);
                let declared = index >= 1
                    && matches!(&trees[index - 1], TokenTree::Ident(word) if word == "fn");
                if !called || declared {
                    continue;
                }
                let punct = |at: usize, ch: char| matches!(trees.get(at), Some(TokenTree::Punct(punct)) if punct.as_char() == ch);
                let method = index >= 1 && punct(index - 1, '.');
                let qualifier = (index >= 3 && punct(index - 1, ':') && punct(index - 2, ':'))
                    .then(|| match &trees[index - 3] {
                        TokenTree::Ident(qualifier) => Some(qualifier.to_string()),
                        _ => None,
                    })
                    .flatten();
                into.insert(Call {
                    qualifier,
                    method,
                    name: ident.to_string(),
                });
            }
            _ => {}
        }
    }
}

/// The name of every function `tokens` declare, at any depth.
fn declared(tokens: TokenStream, into: &mut BTreeSet<String>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        match tree {
            TokenTree::Group(group) => declared(group.stream(), into),
            TokenTree::Ident(word) if word == "fn" => {
                if let Some(TokenTree::Ident(name)) = trees.get(index + 1) {
                    into.insert(name.to_string());
                }
            }
            _ => {}
        }
    }
}

/// A function of one crate's `src` compiled into its unit tests.
struct Function {
    file: PathBuf,
    name: String,
    method: bool,
    spawns: bool,
    calls: BTreeSet<Call>,
}

/// The functions of one crate's `src` compiled into its unit tests, and the calls its test code
/// makes, by file.
#[derive(Default)]
struct UnitTests {
    functions: Vec<Function>,
    tests: Vec<(PathBuf, BTreeSet<Call>)>,
    file: PathBuf,
}

impl UnitTests {
    fn function(
        &mut self,
        (name, method): (&syn::Ident, bool),
        attributes: &[syn::Attribute],
        body: TokenStream,
    ) {
        if is_production_only(attributes) {
            return;
        }
        let mut called = BTreeSet::new();
        calls(body.clone(), &mut called);
        self.functions.push(Function {
            file: self.file.clone(),
            name: name.to_string(),
            method,
            spawns: spawns_cargo(body),
            calls: called,
        });
    }

    /// The calls of one piece of test code, but for a bare call of a function it declares itself.
    fn test(&mut self, tokens: TokenStream) {
        let mut called = BTreeSet::new();
        calls(tokens.clone(), &mut called);
        let mut own = BTreeSet::new();
        declared(tokens, &mut own);
        called.retain(|call| call.method || call.qualifier.is_some() || !own.contains(&call.name));
        self.tests.push((self.file.clone(), called));
    }
}

impl<'ast> Visit<'ast> for UnitTests {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if is_test_attribute(&item.attrs) {
            self.test(quote::ToTokens::to_token_stream(item));
        } else {
            self.function(
                (&item.sig.ident, false),
                &item.attrs,
                quote::ToTokens::to_token_stream(&item.block),
            );
        }
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.function(
            (&item.sig.ident, true),
            &item.attrs,
            quote::ToTokens::to_token_stream(&item.block),
        );
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if !is_production_only(&item.attrs) {
            syn::visit::visit_item_impl(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if is_production_only(&item.attrs) {
            return;
        }
        if is_test_attribute(&item.attrs) {
            self.test(quote::ToTokens::to_token_stream(item));
        } else {
            syn::visit::visit_item_mod(self, item);
        }
    }
}

/// The directory a source file's own child modules live in.
fn module_directory(file: &Path) -> PathBuf {
    let stem = file.file_stem().unwrap();
    if ["main", "lib", "mod"].iter().any(|root| stem == *root) {
        file.parent().unwrap().to_path_buf()
    } else {
        file.parent().unwrap().join(stem)
    }
}

/// The functions a call made in `file` may run, read by name and path: a bare call runs the
/// function of that name in `file` where it declares one; `crate::`, `self::` and `super::` name
/// the crate root, the file and its parent module; any other qualifier names the module of that
/// name; a method call, or a bare call of a name `file` does not declare (imported by `use`), runs
/// any function of that name.
fn callees<'a>(
    functions: &'a [Function],
    file: &Path,
    call: &Call,
    in_test: bool,
) -> Vec<&'a Function> {
    let named = |function: &&Function| function.name == call.name;
    let pick = |within: &dyn Fn(&Function) -> bool| -> Vec<&'a Function> {
        functions
            .iter()
            .filter(named)
            .filter(|function| within(function))
            .collect()
    };
    if call.method {
        return pick(&|function| function.method);
    }
    let free = |function: &Function| !function.method;
    let own = pick(&|function| free(function) && function.file == file);
    match call.qualifier.as_deref() {
        None if !own.is_empty() => own,
        None => pick(&free),
        Some("self") => own,
        Some("crate") => pick(&|function| {
            free(function)
                && function
                    .file
                    .file_name()
                    .is_some_and(|name| name == "main.rs" || name == "lib.rs")
                && function.file.parent() == Some(crate_src(file))
        }),
        Some("super") if in_test && !own.is_empty() => own,
        Some("super") => {
            let parent = module_directory(file);
            let parent = parent.parent();
            pick(&|function| {
                free(function) && Some(module_directory(&function.file).as_path()) == parent
            })
        }
        Some(module) => pick(&|function| {
            free(function)
                && (function.file.file_stem().is_some_and(|stem| stem == module)
                    || (function.file.file_stem().is_some_and(|stem| stem == "mod")
                        && function
                            .file
                            .parent()
                            .and_then(Path::file_name)
                            .is_some_and(|name| name == module)))
        }),
    }
}

/// The `src` directory above `file`.
fn crate_src(file: &Path) -> &Path {
    file.ancestors()
        .find(|directory| directory.file_name().is_some_and(|name| name == "src"))
        .unwrap_or(file)
}

/// Where test code of `files` (path, source, whether the whole file is a `#[cfg(test)] mod name;`
/// file) calls a function compiled into the unit tests that spawns cargo, directly or through the
/// functions it calls: one line per test file and function first called.
fn unit_tests_reaching_cargo(files: &[(PathBuf, String, bool)]) -> Vec<String> {
    let mut scan = UnitTests::default();
    for (path, source, whole) in files {
        scan.file.clone_from(path);
        let parsed = syn::parse_file(source).expect("an ess-xtask source parses");
        if *whole {
            scan.test(quote::ToTokens::to_token_stream(&parsed));
        }
        scan.visit_file(&parsed);
    }
    let mut reaching: Vec<bool> = scan
        .functions
        .iter()
        .map(|function| function.spawns)
        .collect();
    loop {
        let mut grew = false;
        for (index, function) in scan.functions.iter().enumerate() {
            if reaching[index] {
                continue;
            }
            let reaches = function.calls.iter().any(|call| {
                callees(&scan.functions, &function.file, call, false)
                    .iter()
                    .any(|callee| {
                        let at = scan
                            .functions
                            .iter()
                            .position(|candidate| std::ptr::eq(candidate, *callee))
                            .unwrap();
                        reaching[at]
                    })
            });
            if reaches {
                reaching[index] = true;
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    let mut found: BTreeSet<String> = BTreeSet::new();
    for (file, called) in &scan.tests {
        for call in called {
            for callee in callees(&scan.functions, file, call, true) {
                let at = scan
                    .functions
                    .iter()
                    .position(|candidate| std::ptr::eq(candidate, callee))
                    .unwrap();
                if reaching[at] {
                    found.insert(format!(
                        "{}: test code calls `{}` of {}, which spawns cargo",
                        file.display(),
                        call.name,
                        callee.file.display()
                    ));
                }
            }
        }
    }
    found.into_iter().collect()
}

/// The `src` files of ess-xtask, each marked when the whole file is a `#[cfg(test)] mod name;`.
fn xtask_sources(root: &Path) -> Vec<(PathBuf, String, bool)> {
    let prefix = "crates/edge/ess-xtask/src/";
    let files: Vec<PathBuf> = rust_files(root)
        .into_iter()
        .filter(|path| path.starts_with(prefix))
        .map(PathBuf::from)
        .collect();
    let mut test_files: BTreeSet<PathBuf> = BTreeSet::new();
    for relative in &files {
        let file = root.join(relative);
        let source = fs::read_to_string(&file).unwrap();
        let parsed = syn::parse_file(&source).expect("an ess-xtask source parses");
        let mut code = TestCode {
            file: &file,
            module_directory: module_directory(&file),
            spawns: false,
            test_files: Vec::new(),
        };
        code.visit_file(&parsed);
        test_files.extend(
            code.test_files
                .into_iter()
                .filter_map(|path| path.strip_prefix(root).ok().map(Path::to_path_buf)),
        );
    }
    files
        .into_iter()
        .map(|relative| {
            let source = fs::read_to_string(root.join(&relative)).unwrap();
            let whole = test_files.contains(&relative);
            (relative, source, whole)
        })
        .collect()
}

/// The unit tests of ess-xtask run the `ess` binary that `task test-xtask` builds before them: a
/// test that reached `cargo run` compiled `ess-cli` inside the test, and the other test processes
/// waited on cargo's lock for it, 68-102 s each on the build node.
#[test]
fn no_ess_xtask_unit_test_reaches_a_function_that_spawns_cargo() {
    let found = unit_tests_reaching_cargo(&xtask_sources(&workspace_root()));
    assert!(
        found.is_empty(),
        "ess-xtask unit tests reach a function that spawns cargo:\n{}\nBuild what the test needs \
         in the build step of `task test-xtask` and hand the test the artifact's path, as \
         `cli_output` runs the `ess` binary beside the test executable.",
        found.join("\n")
    );
}

#[test]
fn the_unit_test_scan_follows_calls_to_a_cargo_spawn() {
    let source = |test: &str| {
        format!(
            "fn spawn() {{ std::process::Command::new(std::env::var_os(\"CARGO\").unwrap()); }}\n\
             #[cfg(not(test))]\nfn gated() {{ std::process::Command::new(\"cargo\"); }}\n\
             fn render() {{ spawn(); }}\nfn quiet() {{ let spawn = 1; }}\n\
             #[cfg(test)]\nmod tests {{ #[test] fn case() {{ super::{test}(); }} }}\n"
        )
    };
    let main = PathBuf::from("src/main.rs");
    let scan = |test: &str| unit_tests_reaching_cargo(&[(main.clone(), source(test), false)]);
    assert_eq!(
        scan("render").len(),
        1,
        "a call through `render` is followed"
    );
    assert_eq!(scan("spawn").len(), 1, "a direct call is seen");
    assert!(scan("quiet").is_empty(), "a quiet function is not");
    assert!(
        scan("gated").is_empty(),
        "a `#[cfg(not(test))]` function is out of the unit tests"
    );
    let whole = unit_tests_reaching_cargo(&[
        (main.clone(), source("quiet"), false),
        (
            PathBuf::from("src/leaks.rs"),
            "fn check() { crate::render(); }".to_owned(),
            true,
        ),
    ]);
    assert_eq!(
        whole.len(),
        1,
        "a `#[cfg(test)] mod name;` file is test code: {whole:?}"
    );
    let elsewhere = unit_tests_reaching_cargo(&[
        (main.clone(), source("quiet"), false),
        (
            PathBuf::from("src/other.rs"),
            "fn render() {}\n#[cfg(test)]\nmod tests { #[test] fn case() { super::render(); } }"
                .to_owned(),
            false,
        ),
    ]);
    assert!(
        elsewhere.is_empty(),
        "a function of the same name in another module is not the one called: {elsewhere:?}"
    );
}

/// Every string literal in `tokens`, at any depth, with its quotes.
fn literals(tokens: TokenStream, into: &mut BTreeSet<String>) {
    for tree in tokens {
        match tree {
            TokenTree::Group(group) => literals(group.stream(), into),
            TokenTree::Literal(literal) => {
                into.insert(literal.to_string());
            }
            _ => {}
        }
    }
}

/// What is wrong with how one browser-product test file builds, if anything.
fn browser_build_faults(source: &str) -> Vec<&'static str> {
    let mut found = BTreeSet::new();
    literals(
        TokenStream::from_str(source).expect("a browser test lexes"),
        &mut found,
    );
    let mut faults = Vec::new();
    if !found.contains("\"CARGO_TARGET_TMPDIR\"") && !found.contains("\"ESS_BROWSER_TARGET_DIR\"") {
        faults.push("takes its cargo target dir from neither CARGO_TARGET_TMPDIR nor ESS_BROWSER_TARGET_DIR");
    }
    if found.contains("\"CARGO_BUILD_JOBS\"") {
        faults.push("sets CARGO_BUILD_JOBS, which holds each host build to fewer cores than cargo's default");
    }
    faults
}

#[test]
fn browser_tests_build_into_one_shared_target_on_every_core() {
    let root = workspace_root();
    let faults: Vec<String> = BROWSER_BUILDERS
        .iter()
        .flat_map(|file| {
            let source = fs::read_to_string(root.join(file)).unwrap();
            browser_build_faults(&source)
                .into_iter()
                .map(move |fault| format!("{file} {fault}"))
        })
        .collect();
    assert!(
        faults.is_empty(),
        "the browser-product lane builds its wasm hosts in its tests, so how it builds decides \
         whether the lane takes minutes or hours:\n{}",
        faults.join("\n")
    );
}

#[test]
fn the_browser_build_check_sees_both_faults() {
    let none: Vec<&str> = Vec::new();
    assert_eq!(
        browser_build_faults("fn f() { var_os(\"CARGO_TARGET_TMPDIR\"); }"),
        none
    );
    assert_eq!(
        browser_build_faults("fn f() { env!(\"CARGO_TARGET_TMPDIR\"); }"),
        none
    );
    assert_eq!(
        browser_build_faults("fn f() { root.join(\"compiled-host\"); }").len(),
        1
    );
    assert_eq!(
        browser_build_faults(
            "fn f() { var_os(\"ESS_BROWSER_TARGET_DIR\"); c.env(\"CARGO_BUILD_JOBS\", \"1\"); }"
        )
        .len(),
        1
    );
}

#[test]
fn the_scan_sees_each_way_of_spawning_cargo() {
    for spawning in [
        "fn f() { Command::new(\"cargo\").status(); }",
        "fn f() { let cargo = env!(\"CARGO\"); }",
        "fn f() { std::env::var_os(\"CARGO\"); }",
        "fn f() { std::env::var(\"CARGO\").unwrap(); }",
        "fn f() { if x { std::process::Command::new(\"cargo\"); } }",
    ] {
        assert!(file_spawns_cargo(spawning), "missed: {spawning}");
    }
    for quiet in [
        "fn f() { Command::new(\"rustc\"); }",
        "fn f() { json!({\"CARGO\": \"cargo\"}); }",
        "/// Command::new(\"cargo\")\nfn f() {}",
    ] {
        assert!(!file_spawns_cargo(quiet), "false positive: {quiet}");
    }
}
