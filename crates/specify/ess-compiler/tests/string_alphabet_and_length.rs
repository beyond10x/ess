//! `alphabet:`, `example:` and `.count` on text in the IR, the diagnostic codes, and the one walk
//! over every predicate the IR holds (beyond10x/ess#103, beyond10x/ess#104).
//!
//! `docs/design/string-alphabet-and-length.md`, sections 1, 2, 3 and 6.

use std::path::{Path, PathBuf};

use ess_compiler::ir::{EssIr, ResolvedBody};
use ess_compiler::resolve::{compile, diagnose};
use ess_compiler::source::SourceMap;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../ess-domain/tests/fixtures/text-length-positions.yaml");

fn assemble(text: &str) -> Result<Specification, ess_primitives::error::ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("keypad.yaml"), raw)])
}

fn ir(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the fixture carries {from:?}");
    model.replacen(from, to, 1)
}

fn codes(text: &str) -> Vec<String> {
    let errors = assemble(text).expect_err("must refuse");
    diagnose(&errors, &SourceMap::new())
        .as_slice()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect()
}

fn keypad() -> String {
    with(
        &with(
            MODEL,
            "    # ALPHABET",
            r#"    alphabet: "0123456789*#ABCD""#,
        ),
        "{name: keys, type: keypad.dial.KeySequence}",
        r#"{name: keys, type: keypad.dial.KeySequence, example: "12#"}"#,
    )
}

#[test]
fn the_ir_carries_the_alphabet_and_the_example() {
    let compiled = ir(&keypad());
    let sequence = &compiled.types()[&"keypad.dial.KeySequence".parse().unwrap()];
    let ResolvedBody::Newtype { alphabet, .. } = &sequence.body else {
        panic!("a newtype")
    };
    assert_eq!(alphabet.as_deref(), Some("0123456789*#ABCD"));
    let command = &compiled.commands()[&"keypad.dial.SendKeys".parse().unwrap()];
    assert_eq!(command.examples["keys"], Node::Text("12#".to_owned()));

    let bytes = serde_json::to_string(&compiled).unwrap();
    assert!(
        bytes.contains(r#""alphabet":"0123456789*#ABCD""#),
        "{bytes}"
    );
    assert!(bytes.contains(r#""examples":{"keys":"12#"}"#), "{bytes}");
}

#[test]
fn a_model_that_declares_neither_keeps_its_ir_keys() {
    let bytes = serde_json::to_string(&ir(MODEL)).unwrap();
    assert!(!bytes.contains("\"alphabet\""), "{bytes}");
    assert!(!bytes.contains("\"examples\""), "{bytes}");
}

#[test]
fn a_newtype_is_constrained_by_an_alphabet_or_an_invariant_and_a_plain_one_is_not() {
    let compiled = ir(&with(
        &with(MODEL, "    # INNER-ALPHABET", r#"    alphabet: "123""#),
        "    invariants: [value != \"\"]",
        "",
    ));
    let body = |name: &str| &compiled.types()[&name.parse().unwrap()].body;
    assert!(
        body("keypad.dial.Short").is_constrained(),
        "an alphabet alone"
    );
    assert!(body("keypad.dial.Amount").is_constrained(), "an invariant");
    assert!(
        body("keypad.dial.Label").is_constrained(),
        "a struct invariant"
    );
    assert!(!body("keypad.dial.KeySequence").is_constrained(), "neither");
    assert!(!body("keypad.dial.Channel").is_constrained(), "an enum");
}

#[test]
fn each_validation_row_has_the_code_the_design_names() {
    let cases: &[(String, &str)] = &[
        (
            with(
                MODEL,
                "    of: String\n    # ALPHABET",
                "    of: Uuid\n    alphabet: \"a\"",
            )
            .replace(
                "    invariants: [value != \"\"]\n  - name: keypad.dial.Short",
                "  - name: keypad.dial.Short",
            ),
            "ESS-TYPE-002",
        ),
        (
            with(MODEL, "    # ALPHABET", r#"    alphabet: """#),
            "ESS-TYPE-007",
        ),
        (
            with(MODEL, "    # ALPHABET", r#"    alphabet: "aba""#),
            "ESS-TYPE-006",
        ),
        (
            with(
                &with(MODEL, "    # ALPHABET", r#"    alphabet: "ab""#),
                "    # INNER-ALPHABET",
                r#"    alphabet: "cd""#,
            ),
            "ESS-TYPE-004",
        ),
        (
            with(
                &with(MODEL, "    # ALPHABET", r#"    alphabet: "ab""#),
                "format: ess/11",
                "format: ess/10",
            ),
            "ESS-TYPE-009",
        ),
        (
            with(
                MODEL,
                "{name: tags, type: List<String>}",
                "{name: tags, type: List<String>, example: [a]}",
            ),
            "ESS-COMMAND-002",
        ),
        (
            with(
                MODEL,
                "{name: channel, type: keypad.dial.Channel}",
                "{name: channel, type: keypad.dial.Channel, example: Rotary}",
            ),
            "ESS-COMMAND-002",
        ),
        (
            with(&keypad(), r#"example: "12#""#, r#"example: "12x""#),
            "ESS-COMMAND-004",
        ),
        (
            with(&keypad(), "format: ess/11", "format: ess/10")
                .replace("    alphabet: \"0123456789*#ABCD\"\n", ""),
            "ESS-COMMAND-009",
        ),
        (
            with(
                MODEL,
                "        when: n > 64",
                "        when: ref.count > 64",
            ),
            "ESS-COMMAND-003",
        ),
        (
            with(
                &with(
                    MODEL,
                    "        when: n > 64",
                    "        when: keys.count > 64",
                ),
                "format: ess/11",
                "format: ess/10",
            ),
            "ESS-COMMAND-009",
        ),
    ];
    for (model, code) in cases {
        let found = codes(model);
        assert!(
            found.iter().any(|found| found == code),
            "expected {code}, got {found:?}"
        );
    }
}

/// Every spec model under `examples/`: a directory holding a `system.yaml`, read whole.
fn example_models() -> Vec<(String, Vec<(String, String)>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples");
    let mut models = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("examples/ is readable")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("system.yaml").is_file())
        .collect();
    entries.sort();
    for base in entries {
        let mut files = Vec::new();
        let mut pending = vec![base.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).expect("readable") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|kind| kind == "yaml") {
                    let label = path.strip_prefix(&base).unwrap().display().to_string();
                    files.push((label, std::fs::read_to_string(&path).unwrap()));
                }
            }
        }
        files.sort();
        models.push((base.display().to_string(), files));
    }
    assert!(models.len() >= 3, "examples/ holds {} models", models.len());
    models
}

fn specification(files: &[(String, String)]) -> Option<Specification> {
    let mut parts = Vec::new();
    for (label, text) in files {
        parts.push((Source::new(label.as_str()), RawSpecFile::parse(text).ok()?));
    }
    Specification::assemble(parts).ok()
}

#[test]
fn predicate_sites_walks_exactly_the_predicates_admission_gates() {
    let mut checked = 0;
    let mut saw_selection = false;
    let fixtures = [
        (
            "positions".to_owned(),
            vec![("keypad.yaml".to_owned(), MODEL.to_owned())],
        ),
        (
            "binding selections".to_owned(),
            vec![(
                "selection.yaml".to_owned(),
                include_str!("../../ess-domain/tests/fixtures/binding-selection.yaml").to_owned(),
            )],
        ),
        (
            "every position reads a length".to_owned(),
            vec![(
                "keypad.yaml".to_owned(),
                with(
                    &with(
                        &with(
                            MODEL,
                            "        when: n > 64",
                            "        when: keys.count > 64",
                        ),
                        "    invariants: [text != \"\"]",
                        "    invariants: [text.count >= 1]",
                    ),
                    "    filter: name != \"\"",
                    "    filter: name.count > 0",
                ),
            )],
        ),
    ];
    for (label, files) in fixtures.into_iter().chain(example_models()) {
        let Some(spec) = specification(&files) else {
            continue;
        };
        let compiled =
            compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"));
        let admitted = ess_domain::primitive_admission::predicates(&spec).len();
        saw_selection |= !spec
            .bindings()
            .values()
            .all(|binding| binding.selections.is_empty());
        let sites = ess_compiler::expression::predicate_sites(&compiled);
        assert_eq!(sites.len(), admitted, "{label}: {sites:#?}");
        checked += 1;
    }
    assert!(checked >= 5, "only {checked} models were compared");
    assert!(
        saw_selection,
        "no compared model declares a binding selection"
    );
}

#[test]
fn predicate_sites_resolves_a_text_length_where_one_is_read() {
    let compiled = ir(&with(
        &with(
            MODEL,
            "        when: n > 64",
            "        when: keys.count > 64",
        ),
        "    filter: name != \"\"",
        "    filter: name.count > 0",
    ));
    let lengths: Vec<String> = ess_compiler::expression::predicate_sites(&compiled)
        .iter()
        .flat_map(|site| {
            site.check(&compiled)
                .reads
                .into_iter()
                .filter(|read| read.resolution.access.text_length)
                .map(|read| format!("{}: {}", site.site, read.path))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        lengths,
        [
            "command.keypad.dial.SendKeys.outcomes.too-long: keys.count",
            "view.keypad.dial.Sessions.filter: name.count"
        ]
    );
}

/// Every `.rs` file under a crate's `src/`, anywhere in the workspace.
fn workspace_sources() -> Vec<PathBuf> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    let mut pending = vec![crates];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|name| name != "tests" && name != "target")
                {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|kind| kind == "rs")
                && path.components().any(|part| part.as_os_str() == "src")
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The class `is_constrained` exists for, checked rather than listed: no production arm or
/// `matches!` gates on whether a newtype has invariants, because an alphabet constrains a type as
/// much as an invariant does (`docs/design/string-alphabet-and-length.md`, section 6). A site that
/// reads invariants to *assert* them — `value_object_invariants` — binds them in the arm and asks
/// afterwards, and is not what this looks for.
#[test]
fn no_constraint_gate_asks_only_whether_a_newtype_has_invariants() {
    let mut offending = Vec::new();
    let sources = workspace_sources();
    assert!(
        sources.len() > 100,
        "the walk found {} sources",
        sources.len()
    );
    for path in sources {
        let text = std::fs::read_to_string(&path).unwrap();
        for (at, _) in text.match_indices("Body::Newtype {") {
            let rest = &text[at..];
            let end = rest.find("=>").unwrap_or(rest.len()).min(400);
            let arm = &rest[..end];
            if arm.contains("if invariants.is_empty()") || arm.contains("if !invariants.is_empty()")
            {
                let line = text[..at].matches('\n').count() + 1;
                offending.push(format!("{}:{line}", path.display()));
            }
        }
    }
    assert!(
        offending.is_empty(),
        "these gate on invariants alone; ask `ResolvedBody::is_constrained()`: {offending:#?}"
    );
}
