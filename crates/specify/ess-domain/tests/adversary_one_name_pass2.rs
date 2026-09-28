//! Adversary pass 2 against `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts`.
//!
//! The correction keeps every second copy of a name away from `Assembly::claim`: `Collected::write`
//! answers whether a converted copy is kept as a domain member, and `declare`'s refusal is now the
//! only one a same-kind repeat gets. These cases ask what that suppression took away with it, and
//! whether the count holds in every file order.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn file(source: &str, yaml: &str) -> (Source, RawSpecFile) {
    (
        Source::new(source),
        RawSpecFile::parse(yaml).expect("the document is well formed YAML"),
    )
}

const HEADER: &str = "\nformat: ess/1\nsystem: shop\nversion: v1\n";

fn assemble(files: &[(&str, &str)]) -> ValidationErrors {
    let mut all = vec![file("system.yaml", HEADER)];
    all.extend(files.iter().map(|(name, yaml)| file(name, yaml)));
    Specification::assemble(all).expect_err("refused on purpose")
}

/// Every refusal saying `name` is held more than once, in either sentence.
fn refusals_of(errors: &ValidationErrors, name: &str) -> usize {
    errors
        .as_slice()
        .iter()
        .filter(|error| {
            error.code == ValidationCode::DuplicateDeclaration
                && [
                    format!("`{name}` is declared twice"),
                    format!("`{name}` is declared in"),
                    format!("`{name}` is declared more than once"),
                ]
                .iter()
                .any(|opening| error.message.starts_with(opening.as_str()))
        })
        .count()
}

const EVENT_A: &str = "\ndomain: shop.a\nevents:\n  - name: shop.a.Thing\n    fields: []\n";
const EVENT_A_UNDER_B: &str = "\ndomain: shop.b\nevents:\n  - name: shop.a.Thing\n    fields: []\n";

/// One event name, written in `shop.a` and again in a file whose `domain:` is `shop.b`.
///
/// Before the correction the author was told both claimants: `Assembly::claim`'s hint read
/// "claimed by `shop.a` and by `shop.b`". That refusal is now suppressed as a same-kind repeat, and
/// `declare`'s sentence names files, not domains — so nothing in the run says that `shop.b` holds a
/// name that is not its own.
#[test]
fn a_same_kind_copy_filed_under_another_domain_names_that_domain() {
    let errors = assemble(&[("a.yaml", EVENT_A), ("b.yaml", EVENT_A_UNDER_B)]);
    assert!(
        refusals_of(&errors, "shop.a.Thing") >= 1,
        "premise: the name is refused: {errors}"
    );
    assert!(
        errors.as_slice().iter().any(|error| {
            error.message.contains("`shop.b`")
                || error
                    .hint
                    .as_deref()
                    .is_some_and(|hint| hint.contains("`shop.b`"))
                || error.location.contains("shop.b")
        }),
        "b.yaml files `shop.a.Thing` under `shop.b`; no refusal names `shop.b`: {errors}"
    );
}

/// The same pair: b.yaml's copy is also misplaced — `shop.a.Thing` is not inside `shop.b` — and
/// `DomainSpec::validate` refuses exactly that when the name is written once. The duplicate drops
/// the copy before any domain sees it, so the misplacement is reported only after the author has
/// fixed the duplicate: the masking class, by a different fault.
#[test]
fn a_misplaced_copy_is_refused_for_its_domain_even_when_it_repeats_a_name() {
    let alone = assemble(&[("b.yaml", EVENT_A_UNDER_B)]);
    assert!(
        alone.as_slice().iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
                && error.location == "domain shop.b.events"
        }),
        "control: the misplacement alone is refused: {alone}"
    );
    let errors = assemble(&[("a.yaml", EVENT_A), ("b.yaml", EVENT_A_UNDER_B)]);
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
                && error.location == "domain shop.b.events"
        }),
        "the misplacement is hidden by the duplicate: {errors}"
    );
}

const DOMAINLESS_EVENT: &str = "\nevents:\n  - name: shop.cart.Thing\n    fields: []\n";
const DOMAIN_COMMAND: &str = "\ndomain: shop.cart\nevents:\n  - name: shop.cart.Other\n    fields: []\ncommands:\n  - name: shop.cart.Thing\n    outcomes:\n      - name: added\n        emits: [shop.cart.Other]\n";
const DOMAIN_EVENT: &str =
    "\ndomain: shop.cart\nevents:\n  - name: shop.cart.Thing\n    fields: []\n";

/// Three writings of one name — a domainless event, a domain command, a domain event — in every
/// file order: two extra writings, two refusals, whichever file comes first.
#[test]
fn file_order_does_not_change_how_often_a_name_is_refused() {
    let files = [
        ("x.yaml", DOMAINLESS_EVENT),
        ("y.yaml", DOMAIN_COMMAND),
        ("z.yaml", DOMAIN_EVENT),
    ];
    let orders: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for order in orders {
        let permuted: Vec<(&str, &str)> = order.iter().map(|&index| files[index]).collect();
        let errors = assemble(&permuted);
        assert_eq!(
            refusals_of(&errors, "shop.cart.Thing"),
            2,
            "order {order:?}: {errors}"
        );
    }
}

/// One event name in three files: the third file's hint names the first file, not the second.
#[test]
fn a_third_copy_is_told_the_first_file() {
    let errors = assemble(&[
        ("a.yaml", DOMAIN_EVENT),
        ("b.yaml", DOMAIN_EVENT),
        ("c.yaml", DOMAIN_EVENT),
    ]);
    let third = errors
        .as_slice()
        .iter()
        .find(|error| error.message.ends_with("c.yaml declares it again"))
        .unwrap_or_else(|| panic!("the third copy is refused: {errors}"));
    assert!(
        third
            .hint
            .as_deref()
            .is_some_and(|hint| hint.starts_with("a.yaml declares it first")),
        "{errors}"
    );
    assert_eq!(refusals_of(&errors, "shop.cart.Thing"), 2, "{errors}");
}

/// A command in a.yaml, an event of the same name in b.yaml, the event again in c.yaml.
///
/// c.yaml's refusal is `declare`'s, and its new hint says which file "declares it first". `declare`
/// is keyed by kind, so it answers b.yaml — the first *event* — while a.yaml declared the name
/// first, and b.yaml's own copy is itself refused and dropped. The hint sends the author to the
/// wrong file, stated as a fact.
#[test]
fn the_first_file_named_in_a_hint_is_the_first_to_declare_the_name() {
    let command = "\ndomain: shop.cart\nevents:\n  - name: shop.cart.Other\n    fields: []\ncommands:\n  - name: shop.cart.Thing\n    outcomes:\n      - name: added\n        emits: [shop.cart.Other]\n";
    let errors = assemble(&[
        ("a.yaml", command),
        ("b.yaml", DOMAIN_EVENT),
        ("c.yaml", DOMAIN_EVENT),
    ]);
    let third = errors
        .as_slice()
        .iter()
        .find(|error| error.message.ends_with("c.yaml declares it again"))
        .unwrap_or_else(|| panic!("the third copy is refused: {errors}"));
    let hint = third.hint.as_deref().unwrap_or_default();
    assert!(
        !hint.starts_with("b.yaml declares it first"),
        "a.yaml declares `shop.cart.Thing` first; the hint says b.yaml: {errors}"
    );
}

/// A system type in a file with no domain and a domain event of the same name: refused once, and
/// the type still reaches the assembly (it is not a domain member).
#[test]
fn a_domainless_type_and_a_member_of_one_name_are_refused_once() {
    let types = "\ntypes:\n  - name: shop.cart.Thing\n    kind: newtype\n    of: Decimal\n";
    let errors = assemble(&[("types.yaml", types), ("cart.yaml", DOMAIN_EVENT)]);
    assert_eq!(refusals_of(&errors, "shop.cart.Thing"), 1, "{errors}");
    let errors = assemble(&[("cart.yaml", DOMAIN_EVENT), ("types.yaml", types)]);
    assert_eq!(refusals_of(&errors, "shop.cart.Thing"), 1, "{errors}");
}

/// A domainless command first, a domain command second (same kind), a domain event third: the
/// domain command is `declare`'s, the event is `Collected::write`'s, and the assembly receives only
/// the domain command.
#[test]
fn a_domainless_first_copy_does_not_let_a_later_pair_through() {
    let domainless_command = "\nevents:\n  - name: shop.cart.Other\n    fields: []\ncommands:\n  - name: shop.cart.Thing\n    outcomes:\n      - name: added\n        emits: [shop.cart.Other]\n";
    let errors = assemble(&[
        ("x.yaml", domainless_command),
        ("y.yaml", DOMAIN_COMMAND),
        ("z.yaml", DOMAIN_EVENT),
    ]);
    assert_eq!(refusals_of(&errors, "shop.cart.Thing"), 2, "{errors}");
}
