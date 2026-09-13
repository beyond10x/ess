//! The toolchain model's copies of five source declarations, held to the source.
//!
//! `models/toolchain/` states, as ESS enums, five closed sets this repository owns elsewhere: the
//! specification formats a build admits, the diagnostic families and classes, the conformance check
//! codes, and the target failure codes. Each was copied by hand from the crate that declares it.
//!
//! `ess specify validate` cannot see a copy that has fallen behind — a stale variant list is a
//! perfectly valid document, and the model's own gate lines exit 0 on one. Four of the five were
//! stale on the day the model was written, every one of them for the same reason: they were read
//! from a checkout that was four commits behind, and nothing re-read them afterwards. That is the
//! failure this file exists to make impossible rather than to remember.
//!
//! The comparison is by set rather than by order. A reordering in the source is not a defect in a
//! closed set of names; a variant appearing or disappearing is.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The workspace root, found by walking up rather than by counting `..`.
///
/// The same spelling `layout.rs` uses, for the same reason: a fixed number of `..` is a claim about
/// where this file sits, which is exactly the thing a move breaks silently.
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

/// One file of the repository, read as text.
fn source(root: &Path, relative: &str) -> String {
    let path = root.join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// Every item of a parsed file, with the contents of nested modules flattened in.
///
/// `codes::family` and `codes::class` are two modules deep, so a scan of the top level alone finds
/// neither and reports an empty set — which would pass against any model at all.
fn flattened<'a>(items: &'a [syn::Item], into: &mut Vec<&'a syn::Item>) {
    for item in items {
        into.push(item);
        if let syn::Item::Mod(module) = item {
            if let Some((_, inner)) = &module.content {
                flattened(inner, into);
            }
        }
    }
}

/// Every item of one source file, modules flattened.
fn items(text: &str) -> Vec<syn::Item> {
    let file = syn::parse_file(text).expect("the source file parses");
    let mut collected = Vec::new();
    flattened(&file.items, &mut collected);
    collected.into_iter().cloned().collect()
}

/// The variants of one `enum`, wherever in the file it is declared.
fn enum_variants(text: &str, name: &str) -> BTreeSet<String> {
    let variants = items(text)
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Enum(declaration) if declaration.ident == name => Some(declaration.variants),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`enum {name}` is declared in the file named for it"));
    variants
        .into_iter()
        .map(|variant| variant.ident.to_string())
        .collect()
}

/// The items of one named module, wherever it is declared.
fn module_items(text: &str, name: &str) -> Vec<syn::Item> {
    items(text)
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Mod(module) if module.ident == name => {
                module.content.map(|(_, inner)| inner)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("`mod {name}` is declared with a body"))
}

/// The names of the `const … : u16` declarations in one module.
///
/// `ALL` is the aggregate of them and is skipped by type: it is a slice, not a number, so nothing
/// here has to know its name.
fn u16_constant_names(text: &str, module: &str) -> BTreeSet<String> {
    module_items(text, module)
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Const(declaration) => {
                let syn::Type::Path(path) = declaration.ty.as_ref() else {
                    return None;
                };
                path.path
                    .is_ident("u16")
                    .then(|| declaration.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

/// The string values of the `const … : &str` declarations in one module.
fn str_constant_values(text: &str, module: &str) -> BTreeSet<String> {
    module_items(text, module)
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Const(declaration) => match declaration.expr.as_ref() {
                syn::Expr::Lit(literal) => match &literal.lit {
                    syn::Lit::Str(text) => Some(text.value()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The integers of a `const … : &[u32]` declaration.
fn u32_slice(text: &str, name: &str) -> Vec<u32> {
    let expression = items(text)
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Const(declaration) if declaration.ident == name => Some(declaration.expr),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`const {name}` is declared"));
    let syn::Expr::Reference(reference) = expression.as_ref() else {
        panic!("`{name}` is a reference to a slice literal");
    };
    let syn::Expr::Array(array) = reference.expr.as_ref() else {
        panic!("`{name}` is a slice literal");
    };
    array
        .elems
        .iter()
        .map(|element| match element {
            syn::Expr::Lit(literal) => match &literal.lit {
                syn::Lit::Int(number) => number.base10_parse().expect("a decimal major"),
                _ => panic!("`{name}` holds integer literals"),
            },
            _ => panic!("`{name}` holds literals"),
        })
        .collect()
}

/// The `variants:` of one named type in one authored domain document.
fn model_variants(root: &Path, domain: &str, type_name: &str) -> BTreeSet<String> {
    let text = source(root, &format!("models/toolchain/domains/{domain}.yaml"));
    let document: serde_yaml::Value = serde_yaml::from_str(&text).expect("the domain is YAML");
    let declarations = document
        .get("types")
        .and_then(serde_yaml::Value::as_sequence)
        .expect("the domain declares types");
    let declaration = declarations
        .iter()
        .find(|value| value.get("name").and_then(serde_yaml::Value::as_str) == Some(type_name))
        .unwrap_or_else(|| panic!("{type_name} is declared in {domain}.yaml"));
    declaration
        .get("variants")
        .and_then(serde_yaml::Value::as_sequence)
        .unwrap_or_else(|| panic!("{type_name} declares `variants:`"))
        .iter()
        .map(|variant| {
            variant
                .as_str()
                .expect("every variant is a name")
                .to_owned()
        })
        .collect()
}

/// `TYPE_MISMATCH` in the spelling an ESS enum uses for it: `TypeMismatch`.
fn pascal_case(screaming_snake: &str) -> String {
    screaming_snake
        .split('_')
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_ascii_uppercase().to_string() + &characters.as_str().to_ascii_lowercase()
            })
        })
        .collect()
}

/// What to do about a difference, said once rather than five times.
fn assert_same(
    what: &str,
    source_of_truth: &str,
    expected: &BTreeSet<String>,
    model: &BTreeSet<String>,
) {
    assert_eq!(
        expected,
        model,
        "`{what}` in models/toolchain has fallen behind {source_of_truth}. \
         Missing from the model: {:?}. In the model and not in the source: {:?}. \
         Re-derive the variant list from the source rather than editing one end of it",
        expected.difference(model).collect::<Vec<_>>(),
        model.difference(expected).collect::<Vec<_>>(),
    );
}

/// Every specification format this build admits is a variant, and no other.
#[test]
fn the_model_lists_every_admitted_specification_format() {
    let root = workspace_root();
    let text = source(&root, "crates/specify/ess-domain/src/system.rs");
    let expected: BTreeSet<String> = u32_slice(&text, "SUPPORTED_FORMATS")
        .into_iter()
        .map(|major| format!("ess/{major}"))
        .collect();
    let model = model_variants(&root, "specify", "toolchain.specify.SpecificationFormat");
    assert_same(
        "SpecificationFormat",
        "`SUPPORTED_FORMATS`",
        &expected,
        &model,
    );
}

/// Every diagnostic family is a variant, and no other.
#[test]
fn the_model_lists_every_diagnostic_family() {
    let root = workspace_root();
    let text = source(&root, "crates/specify/ess-compiler/src/resolve.rs");
    let expected = str_constant_values(&text, "family");
    let model = model_variants(&root, "specify", "toolchain.specify.DiagnosticFamily");
    assert_same("DiagnosticFamily", "`codes::family`", &expected, &model);
}

/// Every diagnostic class is a variant, and no other.
///
/// The four accessor classes arrived with `ess/3` bounded accessors, and the module's own
/// doc-comment still says twelve — so a count read from prose is exactly the wrong thing to check
/// against, and this reads the constants.
#[test]
fn the_model_lists_every_diagnostic_class() {
    let root = workspace_root();
    let text = source(&root, "crates/specify/ess-compiler/src/resolve.rs");
    let expected: BTreeSet<String> = u16_constant_names(&text, "class")
        .iter()
        .map(|name| pascal_case(name))
        .collect();
    let model = model_variants(&root, "specify", "toolchain.specify.DiagnosticClass");
    assert_same("DiagnosticClass", "`codes::class`", &expected, &model);
}

/// Every rule the conformance runner checks is a variant, and no other.
#[test]
fn the_model_lists_every_conformance_check_code() {
    let root = workspace_root();
    let text = source(&root, "crates/verify/ess-conformance/src/report.rs");
    let expected = enum_variants(&text, "CheckCode");
    let model = model_variants(&root, "verify", "toolchain.verify.CheckCode");
    assert_same("CheckCode", "`CheckCode`", &expected, &model);
}

/// Every rule under which a target refuses to emit is a variant, and no other.
#[test]
fn the_model_lists_every_target_failure_code() {
    let root = workspace_root();
    let text = source(&root, "crates/generate/ess-synth/src/failure.rs");
    let expected = enum_variants(&text, "TargetFailureCode");
    let model = model_variants(&root, "generate", "toolchain.generate.TargetFailureCode");
    assert_same(
        "TargetFailureCode",
        "`TargetFailureCode`",
        &expected,
        &model,
    );
}

/// The scan reads the declarations it names, rather than reporting an empty set.
///
/// The half without which every case above is a claim about nothing: a module walker that never
/// descends, an enum name that no longer resolves, or a renamed constant would each produce two
/// empty sets and compare equal. `layout.rs` carries the same guard for the same reason.
#[test]
fn every_source_declaration_this_scan_reads_is_non_empty() {
    let root = workspace_root();
    let resolve = source(&root, "crates/specify/ess-compiler/src/resolve.rs");
    let report = source(&root, "crates/verify/ess-conformance/src/report.rs");
    let failure = source(&root, "crates/generate/ess-synth/src/failure.rs");
    let system = source(&root, "crates/specify/ess-domain/src/system.rs");

    assert!(
        !u32_slice(&system, "SUPPORTED_FORMATS").is_empty(),
        "no supported specification format was read"
    );
    assert!(
        !str_constant_values(&resolve, "family").is_empty(),
        "no diagnostic family was read"
    );
    assert!(
        !u16_constant_names(&resolve, "class").is_empty(),
        "no diagnostic class was read"
    );
    assert!(
        !enum_variants(&report, "CheckCode").is_empty(),
        "no conformance check code was read"
    );
    assert!(
        !enum_variants(&failure, "TargetFailureCode").is_empty(),
        "no target failure code was read"
    );
}
