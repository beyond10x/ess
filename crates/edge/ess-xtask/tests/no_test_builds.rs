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
