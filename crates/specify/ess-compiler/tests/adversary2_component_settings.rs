//! Adversary pass 2 against `story:component-declares-its-settings`.
//!
//! The unit's correction rests on one sentence: for every *accepted* document
//! `required.unwrap_or(!is_optional)` is identically `!is_optional`, "so deleting the line changes
//! no answer" (`ess-domain/src/component.rs`, `requires_a_value`). That is true of the boolean the
//! two readers compute. It is not true of the compiled bytes: `ResolvedComponentSetting.required`
//! is carried into the IR as the `Option<bool>` the document wrote, and `EssIr::source_digest` is
//! SHA-256 over the serialised IR.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const HEADER: &str = "\
format: ess/1
system: connectors
version: v1
domains:
  - connectors.config
";

const DOMAIN: &str = "\
domain: connectors.config
types:
  - name: connectors.config.StateRoot
    kind: newtype
    of: String
  - name: connectors.config.MaybeRoot
    kind: newtype
    of: Optional<String>
  - name: connectors.config.IndirectRoot
    kind: newtype
    of: connectors.config.MaybeRoot
";

/// A components file, with the settings block indented into the list entry.
fn components(settings: &str) -> String {
    let mut block = String::new();
    for line in settings.lines().filter(|line| !line.trim().is_empty()) {
        block.push_str("      ");
        block.push_str(line);
        block.push('\n');
    }
    format!(
        "\
components:
  - component: connectors-cli
    owns:
      domains:
        - connectors.config
    settings:
{block}"
    )
}

/// Compiles the fixture with the given settings block, or renders why it would not.
fn try_compiled(settings: &str) -> Result<ess_compiler::EssIr, String> {
    let files = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", DOMAIN.to_owned()),
        ("components.yaml", components(settings)),
    ];
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = files
        .iter()
        .map(|(label, text)| {
            sources.insert((*label).to_owned(), text.clone());
            (
                Source::new(*label),
                RawSpecFile::parse(text)
                    .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
            )
        })
        .collect();
    let specification = Specification::assemble(parsed).map_err(|errors| errors.to_string())?;
    compile(&specification, &sources).map_err(|diagnostics| {
        diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic| {
                format!(
                    "{} {} {}",
                    diagnostic.code,
                    diagnostic.message,
                    diagnostic.hint.as_deref().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Compiles the fixture with the given settings block.
fn compiled(settings: &str) -> ess_compiler::EssIr {
    let files = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", DOMAIN.to_owned()),
        ("components.yaml", components(settings)),
    ];
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = files
        .iter()
        .map(|(label, text)| {
            sources.insert((*label).to_owned(), text.clone());
            (
                Source::new(*label),
                RawSpecFile::parse(text)
                    .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
            )
        })
        .collect();
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("assembles: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| {
        panic!("compiles: {:?}", diagnostics.as_slice());
    })
}

/// Two documents the unit says mean the same thing must compile to one identity.
///
/// `required:` over a type the restatement agrees with is free — `ComponentSpec::validate_settings`
/// refuses it the moment it disagrees, and the unit's own rustdoc says deleting the line changes no
/// answer. The two documents in each group below therefore state one specification, and
/// `is_required()` agrees across the group, which the first assertion establishes.
///
/// `EssIr::source_digest` is SHA-256 over the serialised IR (`ir.rs`), and it is the
/// `semantic_digest` every realization, runtime, build and release document in this repository
/// pins. So if a restatement that changes no answer changes that digest, adding or deleting the
/// line invalidates the whole pinned chain for a statement the model does not distinguish.
///
/// # The class, not the instance
///
/// `review-result:adversary-wave25-unit3-pass-2` F3 names one document pair. The class is *every*
/// accepted cell of the `(required, type)` grid: a cell is accepted exactly when the restatement
/// agrees with the type, so every accepted cell has a same-meaning twin with the line deleted, and
/// each twin pair must digest identically. The enumeration below is that grid rather than the one
/// pair, so a future field that carries a restatement into the IR fails here.
///
/// The last assertion is the other half, and it is what stops the field simply being dropped: the
/// two *groups* mean different things and must not collapse onto one digest.
#[test]
fn a_required_line_that_changes_no_answer_does_not_move_the_model_digest() {
    let plain = "connectors.config.StateRoot";
    let optional = "Optional<connectors.config.StateRoot>";
    // Every accepted cell of the grid, grouped by what it means.
    let groups = [
        (plain, vec![None, Some(true)], true),
        (optional, vec![None, Some(false)], false),
    ];

    let mut digests = Vec::new();
    for (type_ref, restatements, required) in groups {
        let mut group: Vec<(String, ess_compiler::EssIr)> = Vec::new();
        for restatement in restatements {
            let line =
                restatement.map_or_else(String::new, |value| format!("  required: {value}\n"));
            let document = format!("\n- name: state-root\n  type: {type_ref}\n{line}");
            let ir = compiled(&document);
            let answer = ir
                .components()
                .values()
                .next()
                .expect("the component resolved")
                .settings[0]
                .is_required();
            assert_eq!(
                answer, required,
                "`required: {restatement:?}` over `{type_ref}` is accepted, so it restates the \
                 type; the two documents must mean the same thing before the digest question is \
                 worth asking"
            );
            group.push((document, ir));
        }
        let (first_document, first) = &group[0];
        for (document, ir) in &group[1..] {
            assert_eq!(
                ir.source_digest(),
                first.source_digest(),
                "these two documents state one specification — `requires_a_value` answers \
                 {required} for both, and the unit's own rustdoc says deleting the line changes \
                 no answer — yet they digest differently, and `EssIr::source_digest` is the \
                 `semantic_digest` every realization, runtime, build and release document \
                 pins:\n{first_document}\nagainst\n{document}"
            );
        }
        digests.push(first.source_digest().clone());
    }

    assert_ne!(
        digests[0], digests[1],
        "a setting that must carry a value and one that may be absent are different \
         specifications; collapsing the restatement must not collapse the answer with it"
    );
}

/// The seventh cell, end to end: `compile` is the reader that actually runs the rule.
///
/// `review-result:adversary-wave25-unit3-pass-2` F4. `ess-domain`'s own case calls
/// `validate_setting_types` directly, because the rules that need the whole specification run from
/// `compile_locating` rather than from `Specification::validate` and the one-line move is in a file
/// another unit of this wave owns. This is the form every caller of this repository actually takes,
/// and it is the one that decides whether the refusal reaches anybody.
///
/// Transitive, because `newtype of: newtype of: Optional<String>` admits absence just as directly
/// and a rule that stopped at one hop would be evaded by adding a second.
#[test]
fn compiling_a_setting_typed_by_a_name_that_admits_absence_is_refused() {
    for type_ref in [
        "connectors.config.MaybeRoot",
        "connectors.config.IndirectRoot",
    ] {
        let refusal = try_compiled(&format!("\n- name: state-root\n  type: {type_ref}\n"))
            .err()
            .unwrap_or_else(|| {
                panic!(
                    "`{type_ref}` reaches `Optional<String>` through its representation, so the \
                     model says a value of it may be absent; `TypeRef::is_optional` cannot see a \
                     name, so `is_required()` answers `true` and the deployment projector derives \
                     a required slot for a value the model permits to be missing"
                )
            });
        assert!(
            refusal.contains(type_ref) && refusal.contains("Optional<String>"),
            "the refusal must name the type and the representation that admits absence, because \
             the author cannot see the second from the first: {refusal}"
        );
    }

    compiled("\n- name: state-root\n  type: Optional<connectors.config.StateRoot>\n");
    compiled("\n- name: state-root\n  type: connectors.config.StateRoot\n");
}
