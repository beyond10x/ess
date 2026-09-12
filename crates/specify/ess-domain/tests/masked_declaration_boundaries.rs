//! Where the masking `declare` closes stops, measured at both edges of it.
//!
//! `story:a-masked-first-declaration-hides-a-duplicate-name` replaced `spec.rs`'s `insert` (*is
//! this name in the registry*) with `declare` (*has this name been written*) plus `record`. The
//! fault it closes is that a reporter which only sees **converted** declarations cannot see a
//! declaration whose own `try_from` failed, so the other copy of its name finds the registry empty
//! and takes it in silence.
//!
//! `declare`'s key is `(kind, name)`, so it closes exactly the same-kind half. The cross-kind half
//! has different reporters — `system.rs`'s `Assembly::claim` and `domain.rs`'s
//! `DomainSpec::validate` — and a declared *type* is carried to `SystemSpec::merge`. All three read
//! lists filled only from `Ok(..)` arms, so all three mask in precisely the way the registries did.
//! Both halves are still open, they are
//! `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts`, and the two cases that
//! pin them are `#[ignore]`d against it at the attribute rather than deleted or weakened:
//! `review-result:adversary-wave25-unit2-pass-1` is where they were first measured (F1 and F3).
//!
//! Each of those pairs a control (both declarations convert — the refusal the tree already makes)
//! with the same document under one broken declaration, so a failure separates "this was never
//! refused" from "this stopped being refused when a copy broke". The other cases are green and
//! stay that way: two of `spec.rs`'s argued-rather-than-built rows, `conversion` and `topology`,
//! built here instead; and the `entity` row's second declaration read as the contract its table
//! claims.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// Parses one file's worth of YAML, through the parser the CLI uses.
fn file(source: &str, yaml: &str) -> (Source, RawSpecFile) {
    (
        Source::new(source),
        RawSpecFile::parse(yaml).expect("the document is well formed YAML"),
    )
}

/// The smallest header a specification needs, plus one domain document.
fn refusals(document: &str) -> ValidationErrors {
    Specification::assemble(vec![
        file(
            "system.yaml",
            "\nformat: ess/1\nsystem: shop\nversion: v1\n",
        ),
        file("domains/cart.yaml", document),
    ])
    .expect_err("the document is refused on purpose")
}

/// `true` when something in `errors` refuses `name` for being declared twice.
///
/// Matched on the sentence both cross-kind reporters write, rather than on the code alone: a
/// duplicate *field* inside the broken declaration is a `DuplicateDeclaration` too, and a case that
/// accepted it would pass while saying nothing.
fn refused_as_declared_twice(errors: &ValidationErrors, name: &str) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::DuplicateDeclaration
            && error
                .message
                .contains(&format!("`{name}` is declared twice"))
    })
}

/// One command, one event, one name, both declarations sound.
const CROSS_KIND_SOUND: &str = r"
domain: shop.cart
events:
  - name: shop.cart.Other
    fields: []
  - name: shop.cart.Thing
    fields: []
commands:
  - name: shop.cart.Thing
    outcomes:
      - name: added
        emits: [shop.cart.Other]
";

/// The same document, with the command carrying an error of its own: one input, twice.
///
/// The broken half is the unit's own `broken` literal from
/// `a_name_declared_twice_is_refused_in_all_four_soundness_combinations`, so the only thing that
/// differs from a case the unit already pins is that the *second* declaration of the name is an
/// event rather than a command.
const CROSS_KIND_FIRST_BROKEN: &str = r"
domain: shop.cart
events:
  - name: shop.cart.Other
    fields: []
  - name: shop.cart.Thing
    fields: []
commands:
  - name: shop.cart.Thing
    input:
      - {name: note, type: String}
      - {name: note, type: String}
    outcomes:
      - name: added
        emits: [shop.cart.Other]
";

/// The control: two kinds, one name, and the tree says so.
#[test]
fn one_name_held_by_two_kinds_is_refused_when_both_declarations_convert() {
    let errors = refusals(CROSS_KIND_SOUND);
    assert!(
        refused_as_declared_twice(&errors, "shop.cart.Thing"),
        "a command and an event of one name is the fault `Assembly::claim` exists for: {errors}"
    );
}

/// The same document, one copy broken, and the refusal is gone.
///
/// This is the unit's own fault shape: the author is told about the duplicate field, fixes it, and
/// only then learns that the name was held twice. `declare` does not catch it because its key
/// carries the kind, and the reporters it names read a member list that a failed conversion never
/// reaches — which is what `declare`'s comment now says, after this case showed that the
/// unqualified version of it was false.
#[test]
#[ignore = "story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts — `declare` \
            is keyed by (kind, name), and the cross-kind reporters read member lists a failed \
            conversion never reaches; measured as F1 of \
            review-result:adversary-wave25-unit2-pass-1"]
fn one_name_held_by_two_kinds_is_still_refused_when_the_first_declaration_is_broken() {
    let errors = refusals(CROSS_KIND_FIRST_BROKEN);
    assert!(
        errors.contains(ValidationCode::DuplicateDeclaration),
        "the broken command's own duplicate input is the premise of this case: {errors}"
    );
    assert!(
        refused_as_declared_twice(&errors, "shop.cart.Thing"),
        "`shop.cart.Thing` is written as a command and as an event, and nothing says so once the \
         command fails its own conversion — the masking `declare` closes for one kind, still open \
         across two: {errors}"
    );
}

/// One type name, twice, both declarations sound.
const TYPE_SOUND: &str = r"
domain: shop.cart
types:
  - name: shop.cart.Money
    kind: enum
    variants: [Gbp]
  - name: shop.cart.Money
    kind: newtype
    of: Decimal
";

/// The same document, with the first declaration declaring no variants.
const TYPE_FIRST_BROKEN: &str = r"
domain: shop.cart
types:
  - name: shop.cart.Money
    kind: enum
    variants: []
  - name: shop.cart.Money
    kind: newtype
    of: Decimal
";

/// The control: a type declared twice is refused, and `Assembly::claim` is what refuses it.
#[test]
fn one_type_name_declared_twice_is_refused_when_both_declarations_convert() {
    let errors = refusals(TYPE_SOUND);
    assert!(
        refused_as_declared_twice(&errors, "shop.cart.Money"),
        "two declarations of one type name is the fault `Assembly::claim` exists for: {errors}"
    );
}

/// The row the unit left open, measured rather than assumed.
///
/// `declare` is not called for types at all, and the reporter the unit names instead —
/// `SpecPart` carried to `SystemSpec::merge` — receives only the types that converted. So the
/// second declaration takes the name in silence and the author sees one error about variants,
/// which is the exact experience the story was written to end.
#[test]
#[ignore = "story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts — types do \
            not go through `declare` at all, and `SystemSpec::merge` receives only the types that \
            converted; measured as F3 of review-result:adversary-wave25-unit2-pass-1"]
fn one_type_name_declared_twice_is_still_refused_when_the_first_declaration_is_broken() {
    let errors = refusals(TYPE_FIRST_BROKEN);
    assert!(
        errors.contains(ValidationCode::EmptyDeclaration),
        "the first declaration's own error is the premise of this case: {errors}"
    );
    assert!(
        refused_as_declared_twice(&errors, "shop.cart.Money"),
        "`shop.cart.Money` is declared twice and the second declaration silently owns the name: \
         {errors}"
    );
}

/// The `conversion` row of the unit's table, built rather than argued.
///
/// The table says a conversion "is refused by `ConversionRegistry::insert` without converting
/// anything", so no copy of one can be masked. `RawSpecFile::conversions` is
/// `Vec<Conversion>` — a `serde::Deserialize` with no `try_from` between the document and the
/// registry — so there is no conversion step to fail, and the row holds.
#[test]
fn one_crossing_declared_twice_is_refused_with_no_conversion_step_to_mask_it() {
    let errors = refusals(
        r"
domain: shop.cart
types:
  - name: shop.cart.Money
    kind: newtype
    of: Decimal
  - name: shop.cart.Price
    kind: newtype
    of: Decimal
conversions:
  - from: shop.cart.Money
    to: shop.cart.Price
    because: the first reason
  - from: shop.cart.Money
    to: shop.cart.Price
    because: a second, different reason
",
    );
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::DuplicateDeclaration
                && error.location == "conversions.shop.cart.Money -> shop.cart.Price"
        }),
        "one crossing, one reason: {errors}"
    );
}

/// The `topology` row of the unit's table, built rather than argued.
///
/// The table says a topology "records the source that carried it before converting it", so a first
/// topology whose own `try_from` fails still blocks the second. `Cart-Service` is not a component
/// name, which is the one way a topology fails its conversion.
#[test]
fn a_second_topology_is_refused_even_when_the_first_fails_its_own_conversion() {
    let errors = Specification::assemble(vec![
        file(
            "system.yaml",
            "\nformat: ess/1\nsystem: shop\nversion: v1\n",
        ),
        file(
            "a.yaml",
            "\ntopology:\n  workloads:\n    Cart-Service: {}\n",
        ),
        file(
            "b.yaml",
            "\ntopology:\n  workloads:\n    cart-service: {}\n",
        ),
    ])
    .expect_err("two topologies");
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::DuplicateDeclaration
                && error.location == "b.yaml.topology"
        }),
        "the first topology's own failure must not let the second one through: {errors}"
    );
}

/// The `entity` row of the unit's `MASKED` table, read as the contract its comment claims.
///
/// The table's comment says of every row: "The first copy carries an error of its own and nothing
/// else … The second is sound and takes the same name." This is the `entity` row's second copy,
/// verbatim, on its own. If it is not sound, that row is a broken-then-*broken* case wearing a
/// broken-then-sound label, and no case anywhere exercises a sound entity taking a name a broken
/// entity declared first — which is the one shape the story is named after.
///
/// `spec.rs`'s `a_name_declared_twice_is_refused_even_when_a_copy_is_broken_itself` cannot notice:
/// it asserts only that `declare`'s refusal is present, and `declare` runs before either copy is
/// converted, so it is satisfied whether the second copy is sound or not. It was not one row but
/// two — the `component` row's second copy owned a domain nothing declared — so `spec.rs` now asks
/// the question of every row in `every_masked_row_is_broken_then_sound_as_this_table_claims`, and
/// this case stays as the one that is written where the table cannot reach itself.
///
/// `MASKED` is private to `spec.rs`'s test module, so the document below is a *copy* of that row's
/// second declaration rather than a reading of it, and a copy can go stale where the in-crate
/// check cannot. That check is the one to trust for the whole table; this one is here because it
/// is what found the row.
#[test]
fn the_masked_tables_entity_row_has_a_sound_second_declaration() {
    let outcome = Specification::assemble(vec![
        file(
            "system.yaml",
            "\nformat: ess/1\nsystem: shop\nversion: v1\n",
        ),
        file(
            "domains/cart.yaml",
            r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
",
        ),
    ]);
    if let Err(errors) = outcome {
        panic!(
            "the `entity` row's second declaration is not sound, so that row never tests a sound \
             copy taking a broken copy's name: {errors}"
        );
    }
}
