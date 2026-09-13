//! Adversary pass 2 against `story:component-declares-its-settings`, at the declaration.
//!
//! `an_accepted_required_restates_the_type_and_never_overrides_it` enumerates six combinations of
//! `(required, type)` and calls that the grid. It is the grid of one *syntactic* question:
//! `TypeRef::is_optional` is `matches!(self, Self::Optional(_))`, so it answers about the wrapper a
//! document wrote and not about what the named type is. A newtype whose representation is
//! `Optional<…>` is therefore on the wrong side of every settings rule, and this repository already
//! declares that shape — `crates/specify/ess-compiler/tests/fixtures/adversary_expression.yaml`
//! holds three of them.

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;

const HEADER: &str = "\
format: ess/1
system: connectors
version: v1
domains:
  - connectors.config
";

/// A domain whose `MaybeRoot` is a newtype over `Optional<String>` — the same shape
/// `adversary_expression.yaml` declares, and one the type system accepts.
const DOMAIN: &str = "\
domain: connectors.config
types:
  - name: connectors.config.StateRoot
    kind: newtype
    of: String
  - name: connectors.config.MaybeRoot
    kind: newtype
    of: Optional<String>
";

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

fn assemble(settings: &str) -> Result<Specification, ess_primitives::error::ValidationErrors> {
    let parsed: Vec<_> = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", DOMAIN.to_owned()),
        ("components.yaml", components(settings)),
    ]
    .into_iter()
    .map(|(label, text)| {
        (
            Source::new(label),
            RawSpecFile::parse(&text)
                .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
        )
    })
    .collect();
    Specification::assemble(parsed)
}

/// The seventh cell of a six-cell grid: a declared type that is itself an `Optional`.
///
/// `TypeRef::is_optional` is `matches!(self, Self::Optional(_))`, so it answers about the wrapper a
/// document wrote and not about what the named type is. `connectors.config.MaybeRoot` is
/// `newtype of: Optional<String>` — the model does say the value may be absent, it says it through
/// a name — and `is_optional` cannot see a name. The setting is then on the wrong side of every
/// settings rule at once: `requires_a_value` answers `true`, the projector derives
/// `ConfigKind::Required` for a value the model permits to be missing, and `required: false`,
/// which is the author's only way to say what they meant, is refused.
///
/// The resolution taken is to refuse the *type* in a settings position, in
/// `validate_setting_types`, which is the one settings rule site that holds the type registry. That
/// is what makes the wrapper question total: for every setting in an accepted specification,
/// `TypeRef::is_optional` is exactly "this type admits absence", so the three sites that ask it
/// syntactically are all correct rather than all correctable.
///
/// # Why this calls `validate_setting_types` and not `assemble`
///
/// The same seam the two `#[ignore]`d cases in `component_settings.rs` name: the rules that need
/// the whole specification run from `compile_locating` rather than from `Specification::validate`,
/// and the one-line fix is in a file another unit of this wave owns. This case calls the rule
/// directly so that it tests the rule and not the placement; `ess-compiler`'s own
/// `adversary2_component_settings.rs` holds the end-to-end form, through `compile`.
#[test]
fn a_declared_type_that_is_an_optional_is_not_treated_as_admitting_no_absence() {
    let specification = assemble(
        r"
- name: retry-window
  type: connectors.config.MaybeRoot
",
    )
    .unwrap_or_else(|errors| panic!("a setting typed by a declared newtype assembles: {errors}"));
    let errors = ess_domain::component::validate_setting_types(&specification);
    assert!(
        !errors.is_empty(),
        "`connectors.config.MaybeRoot` is `newtype of: Optional<String>`, so the model says this \
         value may be absent; `requires_a_value` answers `true` for it because \
         `TypeRef::is_optional` only recognises a wrapper a document wrote, and the derived slot \
         is therefore `ConfigKind::Required`"
    );
    let text = errors.to_string();
    assert!(
        text.contains("connectors.config.MaybeRoot") && text.contains("Optional<String>"),
        "the refusal must name the type and the representation that admits absence, because the \
         author cannot see the second from the first: {text}"
    );

    let refusal = assemble(
        r"
- name: retry-window
  type: connectors.config.MaybeRoot
  required: false
",
    )
    .expect_err("`required: false` over this type is refused today")
    .to_string();
    assert!(
        !refusal.contains("admits no absence"),
        "and the author's only way to say what they meant is refused with a sentence that is \
         false about their type: {refusal}"
    );
}

/// The rule the refusal above buys, stated as the rule rather than as its one instance.
///
/// The class is not "`MaybeRoot` is mishandled"; it is "a settings rule asks a *type* question
/// while holding a type *reference*". Three sites do it — this crate's `validate_settings` grid,
/// `ComponentSetting::is_required`, and the compiled IR's `ResolvedComponentSetting` — and none of
/// them can be fixed one at a time, because two of them hold no registry at all.
///
/// So the fix is an invariant, and this is the invariant: over a specification declaring a type of
/// every shape that could hide an `Optional` — directly, transitively, inside a struct field,
/// inside a union variant, under a `List` — every setting that is *accepted* has `is_optional()`
/// equal to what the registry says. A future named type that admits absence and is not refused
/// fails here, whatever route it admits absence by.
///
/// # The cells are counted
///
/// Written first with a `continue` on a refused assembly, this case passed against the unfixed
/// tree: the fixture also declared a newtype cycle, `Specification::assemble` refused the whole
/// document for it, and every cell was skipped. A test that reports green having examined nothing
/// is worse than no test, so the fixture declares only shapes that assemble and the two counts are
/// asserted. The cycle moved out rather than being kept with its refusal tolerated — the
/// `self_reference` check already owns it, and a fixture that is refused as a whole cannot
/// enumerate anything.
#[test]
fn every_accepted_setting_answers_the_wrapper_question_the_way_the_registry_does() {
    let names = [
        "connectors.config.StateRoot",
        "connectors.config.MaybeRoot",
        "connectors.config.IndirectRoot",
        "connectors.config.ListOfMaybe",
        "connectors.config.Shaped",
        "connectors.config.Tagged",
        "connectors.config.Colour",
    ];

    let mut checked = 0_usize;
    let mut refused = 0_usize;
    for name in names {
        for written in [name.to_owned(), format!("Optional<{name}>")] {
            let specification = assemble_shapes(&written);
            if !ess_domain::component::validate_setting_types(&specification).is_empty() {
                refused += 1;
                continue;
            }
            let setting = &specification
                .components()
                .values()
                .next()
                .expect("the component survived conversion")
                .settings[0];
            checked += 1;
            if written.starts_with("Optional<") {
                // A written wrapper is the model's own statement and needs no registry.
                assert!(
                    setting.type_ref.is_optional(),
                    "`{written}` is written `Optional<…>` and must answer as one"
                );
                continue;
            }
            assert_eq!(
                setting.type_ref.is_optional(),
                registry_says_absent(&specification, name),
                "the setting typed `{written}` is accepted, so `TypeRef::is_optional` — which \
                 every settings rule, `is_required` and the deployment projector read — must be \
                 exactly what the registry says about absence; it is not, and a type that admits \
                 absence through a name is therefore on the wrong side of every settings rule"
            );
        }
    }

    assert_eq!(
        (checked, refused),
        (12, 2),
        "the enumeration must examine every cell it claims to: seven declared shapes, each bare \
         and each wrapped, with exactly the two that admit absence through a name refused"
    );
}

/// A domain declaring one type of every shape that could hide an `Optional`, and one setting.
///
/// No cycle: `A = newtype of B` beside `B = newtype of A` is refused at assembly by the
/// `self_reference` check, which would refuse this whole document and leave the enumeration above
/// with nothing to examine — which is exactly how it first passed against the unfixed tree.
const SHAPES: &str = "\
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
  - name: connectors.config.ListOfMaybe
    kind: newtype
    of: List<Optional<String>>
  - name: connectors.config.Shaped
    kind: struct
    fields:
      - name: maybe
        type: Optional<String>
  - name: connectors.config.Tagged
    kind: union
    tag: kind
    variants:
      absent: Optional<String>
      present: String
  - name: connectors.config.Colour
    kind: enum
    variants: [Red, Green]
";

/// [`SHAPES`] with one setting of the written type, assembled.
fn assemble_shapes(written: &str) -> Specification {
    let parsed: Vec<_> = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", SHAPES.to_owned()),
        (
            "components.yaml",
            components(&format!("\n- name: retry-window\n  type: {written}\n")),
        ),
    ]
    .into_iter()
    .map(|(label, text)| {
        (
            Source::new(label),
            RawSpecFile::parse(&text)
                .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
        )
    })
    .collect();
    Specification::assemble(parsed).unwrap_or_else(|errors| {
        panic!("every shape in this fixture assembles; `{written}` did not: {errors}")
    })
}

/// Whether the registry says a value of this declared name may be absent.
///
/// Walked here, from the specification's own `types:`, rather than through the code under test.
fn registry_says_absent(specification: &Specification, name: &str) -> bool {
    use ess_domain::types::{TypeBody, TypeRef};

    let bodies: std::collections::BTreeMap<String, &TypeBody> = specification
        .system()
        .types
        .iter()
        .map(|named| (named.name.to_string(), &named.body))
        .collect();
    let mut cursor = Some(name.to_owned());
    while let Some(current) = cursor.take() {
        let Some(TypeBody::Newtype { of, .. }) = bodies.get(&current) else {
            return false;
        };
        if of.is_optional() {
            return true;
        }
        if let TypeRef::Named(next) = of {
            cursor = Some(next.to_string());
        }
    }
    false
}
