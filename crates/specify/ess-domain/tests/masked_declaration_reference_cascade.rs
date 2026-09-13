//! What `record` does to a name whose *first* declaration failed its own conversion.
//!
//! `story:a-masked-first-declaration-hides-a-duplicate-name` split `spec.rs`'s `insert` into
//! `declare` (has this name been written) and `record` (what the name means), and for one round
//! `record` answered the second question with `declare`'s answer alone: keep the copy `first` is
//! `true` for, drop every other.
//!
//! That rule has a case, and it is the exact case the story is named after. When the first
//! declaration is the *broken* one, `declare` answers `first = true` for it — and it never reaches
//! `record`, because its `try_from` failed. The second copy then gets `first = false`, and a
//! `record` that reads only `first` drops it too. The name is then in **no** registry at all, and
//! every other declaration that refers to it is refused as a reference to something nobody
//! declared, in the same run that also says the name is declared more than once.
//!
//! `insert` did not do that. It recorded the copy that converted, because the registry it looked
//! in was empty — the very blindness the story closed is also what kept the registry populated. So
//! `record` keeps the first copy that **converts**, which is the question that registry was really
//! answering; `first` decides only between copies that both converted. Both refusals still fire,
//! and the cascade behind them does not. These cases are what says so.
//!
//! Each case below pairs a control — the same document with the first copy sound, where the name
//! resolves — with the masked document, so a failure separates "this reference never resolved"
//! from "this reference stopped resolving when the first copy broke". The rule is asked of all
//! eight registries at once by `spec.rs`'s
//! `every_masked_rows_name_is_recorded_from_the_copy_that_converts`; these two are where an author
//! meets it.

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
    .expect_err("the document declares one name twice, so it is refused on purpose")
}

/// `true` when something in `errors` says the reference at `location` names nothing declared.
fn undeclared_at(errors: &ValidationErrors, location: &str) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::UndeclaredReference && error.location == location
    })
}

/// `true` when something in `errors` is `declare`'s whole-name duplicate for `location`.
fn declared_twice_at(errors: &ValidationErrors, location: &str) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::DuplicateDeclaration && error.location == location
    })
}

/// One entity name declared twice, the first copy carrying a duplicate field, and a view on it.
///
/// The two entity copies are `spec.rs`'s own `MASKED` `entity` row verbatim; the view is that
/// table's `view` row's *second* copy, which the same table asserts is sound on its own.
const ENTITY_FIRST_BROKEN: &str = r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
views:
  - name: shop.cart.CartById
    source: shop.cart.Cart
    fields:
      - {name: id, type: Uuid}
";

/// The same document with the first copy sound, so `record` keeps it and the view resolves.
const ENTITY_BOTH_SOUND: &str = r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
views:
  - name: shop.cart.CartById
    source: shop.cart.Cart
    fields:
      - {name: id, type: Uuid}
";

/// A view is not told its source is undeclared when the source is declared twice.
#[test]
fn a_view_on_a_twice_declared_entity_is_not_refused_for_an_undeclared_source() {
    let control = refusals(ENTITY_BOTH_SOUND);
    assert!(
        declared_twice_at(&control, "entity shop.cart.Cart"),
        "the control's premise: two declarations of one entity name are refused: {control}"
    );
    assert!(
        !undeclared_at(&control, "view.shop.cart.CartById.source"),
        "the control's premise: when the first copy converts, `record` keeps it and the view \
         resolves against it: {control}"
    );

    let masked = refusals(ENTITY_FIRST_BROKEN);
    assert!(
        declared_twice_at(&masked, "entity shop.cart.Cart"),
        "the masked document's premise, which is what the story closed: the second declaration \
         is refused even though the first one never converted: {masked}"
    );
    assert!(
        !undeclared_at(&masked, "view.shop.cart.CartById.source"),
        "`shop.cart.Cart` is declared twice in this document and the same run says so, and the \
         view on it is refused for projecting an entity `no domain declares`. The first copy \
         failed its `try_from` so it never reached `record`, and `record` dropped the second \
         because `declare` had already answered `first = false` — so the name is in no registry \
         and every reference to it is a lie about the document: {masked}"
    );
}

/// One command name declared twice, the first copy carrying a duplicate input, and an actor grant.
///
/// The two command copies are `spec.rs`'s own `MASKED` `command` row verbatim — the same literals
/// its `a_name_declared_twice_is_refused_in_all_four_soundness_combinations` calls `broken` and
/// `sound`.
const COMMAND_FIRST_BROKEN: &str = r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:
  - name: shop.cart.AddItem
    input:
      - {name: note, type: String}
      - {name: note, type: String}
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
actors:
  - name: shop.cart.Shopper
    may: [shop.cart.AddItem]
";

/// The same document with the first copy sound.
const COMMAND_BOTH_SOUND: &str = r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:
  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
actors:
  - name: shop.cart.Shopper
    may: [shop.cart.AddItem]
";

/// An actor's grant is not called undeclared when the command it names is declared twice.
#[test]
fn an_actor_grant_on_a_twice_declared_command_is_not_refused_as_undeclared() {
    let control = refusals(COMMAND_BOTH_SOUND);
    assert!(
        declared_twice_at(&control, "command shop.cart.AddItem"),
        "the control's premise: two declarations of one command name are refused: {control}"
    );
    assert!(
        !undeclared_at(&control, "actor shop.cart.Shopper.may"),
        "the control's premise: when the first copy converts, the grant resolves: {control}"
    );

    let masked = refusals(COMMAND_FIRST_BROKEN);
    assert!(
        declared_twice_at(&masked, "command shop.cart.AddItem"),
        "the masked document's premise: the second declaration is refused: {masked}"
    );
    assert!(
        !undeclared_at(&masked, "actor shop.cart.Shopper.may"),
        "`shop.cart.AddItem` is declared twice here and the run says so, and the actor that may \
         invoke it is told no domain declares it as a command. Nothing recorded either copy: the \
         first failed its own conversion, the second was dropped as a later copy: {masked}"
    );
}
