//! Adversary pass 1 against `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts`.
//!
//! The story's claim: a name written more than once is refused once per extra writing, whichever
//! kinds hold it and whichever copies convert, and never twice for one writing.

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

/// Every refusal that is about `name` being held more than once: `declare`'s same-kind sentence
/// (location `<kind> <name>`) and the cross-kind sentence of `Claim::refuse`.
fn refusals_of(errors: &ValidationErrors, name: &str) -> usize {
    errors
        .as_slice()
        .iter()
        .filter(|error| {
            error.code == ValidationCode::DuplicateDeclaration
                && (error
                    .message
                    .starts_with(&format!("`{name}` is declared twice"))
                    || error
                        .message
                        .starts_with(&format!("`{name}` is declared in"))
                    || error
                        .message
                        .starts_with(&format!("`{name}` is declared more than once")))
        })
        .count()
}

const SOUND_EVENT: &str = "  - name: shop.cart.Thing\n    fields: []\n";
const BROKEN_EVENT: &str = "  - name: shop.cart.Thing\n    fields:\n      - {name: total, type: Decimal}\n      - {name: total, type: Decimal}\n";
const SOUND_COMMAND: &str = "  - name: shop.cart.Thing\n    outcomes:\n      - name: added\n        emits: [shop.cart.Other]\n";
const BROKEN_COMMAND: &str = "  - name: shop.cart.Thing\n    input:\n      - {name: note, type: String}\n      - {name: note, type: String}\n    outcomes:\n      - name: added\n        emits: [shop.cart.Other]\n";
const OTHER: &str = "  - name: shop.cart.Other\n    fields: []\n";

/// Two sound events of one name: one extra writing, one refusal.
#[test]
fn two_sound_copies_of_one_kind_are_refused_once() {
    let doc = format!("\ndomain: shop.cart\nevents:\n{OTHER}{SOUND_EVENT}{SOUND_EVENT}");
    let errors = assemble(&[("domains/cart.yaml", &doc)]);
    assert_eq!(
        refusals_of(&errors, "shop.cart.Thing"),
        1,
        "two sound events of one name, one extra writing: {errors}"
    );
}

/// Two sound actors, the `MASKED` table's own control row: one extra writing, one refusal.
#[test]
fn two_sound_actors_of_one_name_are_refused_once() {
    let doc =
        "\ndomain: shop.cart\nactors:\n  - name: shop.cart.Shopper\n  - name: shop.cart.Shopper\n";
    let errors = assemble(&[("domains/cart.yaml", doc)]);
    assert_eq!(
        refusals_of(&errors, "shop.cart.Shopper"),
        1,
        "two sound actors of one name: {errors}"
    );
}

/// Sound event, broken command, sound command: two extra writings, two refusals.
///
/// The broken command is `Collected::write`'s. The sound command is `declare`'s (a command again)
/// and also reaches `Assembly::claim` beside the sound event, and nothing stops the second.
#[test]
fn a_sound_copy_after_a_broken_copy_of_its_own_kind_is_refused_once() {
    let doc = format!(
        "\ndomain: shop.cart\nevents:\n{OTHER}{SOUND_EVENT}commands:\n{BROKEN_COMMAND}{SOUND_COMMAND}"
    );
    let errors = assemble(&[("domains/cart.yaml", &doc)]);
    assert_eq!(
        refusals_of(&errors, "shop.cart.Thing"),
        2,
        "three writings of one name are two refusals: {errors}"
    );
}

/// A sound event in a file with no `domain:`, and a sound command of the same name in a file with
/// one. The domainless file is refused for its missing domain; the name is still written twice,
/// and nothing says so — the masking the story closes, by a file-level error instead of a
/// member-level one.
#[test]
fn a_name_is_refused_when_one_copy_sits_in_a_file_with_no_domain() {
    let events = format!("\nevents:\n{SOUND_EVENT}");
    let commands = format!("\ndomain: shop.cart\nevents:\n{OTHER}commands:\n{SOUND_COMMAND}");
    let errors = assemble(&[
        ("domains/events.yaml", &events),
        ("domains/commands.yaml", &commands),
    ]);
    assert!(
        errors.contains(ValidationCode::MissingDeclaration),
        "premise: the domainless file is refused: {errors}"
    );
    assert_eq!(
        refusals_of(&errors, "shop.cart.Thing"),
        1,
        "`shop.cart.Thing` is written twice; fixing the missing `domain:` reveals it: {errors}"
    );
}

/// The same, with the domainless copy broken: refused (by `Collected::write`). Paired with the
/// case above, the only difference being that the domainless copy converts.
#[test]
fn a_broken_copy_in_a_file_with_no_domain_is_refused_like_a_sound_one() {
    let events = format!("\nevents:\n{BROKEN_EVENT}");
    let commands = format!("\ndomain: shop.cart\nevents:\n{OTHER}commands:\n{SOUND_COMMAND}");
    let errors = assemble(&[
        ("domains/events.yaml", &events),
        ("domains/commands.yaml", &commands),
    ]);
    assert_eq!(refusals_of(&errors, "shop.cart.Thing"), 1, "{errors}");
}

/// Two domains, two kinds, same last segment: not one name, never refused as one.
#[test]
fn the_same_short_name_in_two_domains_is_not_a_duplicate() {
    for (command, event) in [
        (SOUND_COMMAND, SOUND_EVENT),
        (BROKEN_COMMAND, SOUND_EVENT),
        (SOUND_COMMAND, BROKEN_EVENT),
        (BROKEN_COMMAND, BROKEN_EVENT),
    ] {
        let cart = format!("\ndomain: shop.cart\nevents:\n{OTHER}commands:\n{command}");
        let billing = format!(
            "\ndomain: shop.billing\nevents:\n{}",
            event.replace("shop.cart.Thing", "shop.billing.Thing")
        );
        let Err(errors) = Specification::assemble(vec![
            file("system.yaml", HEADER),
            file("domains/cart.yaml", &cart),
            file("domains/billing.yaml", &billing),
        ]) else {
            continue;
        };
        assert_eq!(refusals_of(&errors, "shop.cart.Thing"), 0, "{errors}");
        assert_eq!(refusals_of(&errors, "shop.billing.Thing"), 0, "{errors}");
    }
}

/// Four kinds, four broken copies: three extra writings, three refusals, in every order.
#[test]
fn four_broken_copies_of_one_name_are_three_refusals() {
    let entity = "entities:\n  - name: shop.cart.Thing\n    identity: {name: id, type: Uuid}\n    fields:\n      - {name: total, type: Decimal}\n      - {name: total, type: Decimal}\n    lifecycle: {states: [Open], initial: Open, terminal: [Open]}\n";
    let command = format!("commands:\n{BROKEN_COMMAND}");
    let event = format!("events:\n{OTHER}{BROKEN_EVENT}");
    let error = "errors:\n  - name: shop.cart.Thing\n    fields:\n      - {name: total, type: Decimal}\n      - {name: total, type: Decimal}\n";
    let parts = [entity.to_string(), command, event, error.to_string()];
    // One file per kind, in both file orders.
    for order in [[0usize, 1, 2, 3], [3, 2, 1, 0]] {
        let docs: Vec<(String, String)> = order
            .iter()
            .map(|&i| {
                (
                    format!("domains/k{i}.yaml"),
                    format!("\ndomain: shop.cart\n{}", parts[i]),
                )
            })
            .collect();
        let refs: Vec<(&str, &str)> = docs.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let errors = assemble(&refs);
        assert_eq!(
            refusals_of(&errors, "shop.cart.Thing"),
            3,
            "{order:?}: {errors}"
        );
    }
}

/// A type and an entity of one name, every soundness combination: one refusal.
#[test]
fn a_type_and_an_entity_of_one_name_are_refused_once() {
    let sound_type = "types:\n  - name: shop.cart.Thing\n    kind: newtype\n    of: Decimal\n";
    let broken_type = "types:\n  - name: shop.cart.Thing\n    kind: enum\n    variants: []\n";
    let sound_entity = "entities:\n  - name: shop.cart.Thing\n    identity: {name: id, type: Uuid}\n    lifecycle: {states: [Open], initial: Open, terminal: [Open]}\n";
    let broken_entity = "entities:\n  - name: shop.cart.Thing\n    identity: {name: id, type: Uuid}\n    fields:\n      - {name: total, type: Decimal}\n      - {name: total, type: Decimal}\n    lifecycle: {states: [Open], initial: Open, terminal: [Open]}\n";
    for (t, e) in [
        (sound_type, sound_entity),
        (broken_type, sound_entity),
        (sound_type, broken_entity),
        (broken_type, broken_entity),
    ] {
        for doc in [
            format!("\ndomain: shop.cart\n{t}{e}"),
            format!("\ndomain: shop.cart\n{e}{t}"),
        ] {
            let errors = assemble(&[("domains/cart.yaml", &doc)]);
            assert_eq!(
                refusals_of(&errors, "shop.cart.Thing"),
                1,
                "{doc}\n{errors}"
            );
        }
    }
}

/// An error and a view of one name, one document, every soundness combination: the refusal is
/// one sentence at one location whichever reporter writes it (`Claim::refuse`'s own doc claim).
///
/// `Collected` writes errors before views; `Assembly::absorb_domain` claims views before errors.
#[test]
fn an_error_and_a_view_of_one_name_are_refused_at_one_location_whichever_copy_converts() {
    let entity = "entities:\n  - name: shop.cart.Cart\n    identity: {name: id, type: Uuid}\n    lifecycle: {states: [Open], initial: Open, terminal: [Open]}\n";
    let sound_view = "views:\n  - name: shop.cart.Thing\n    source: shop.cart.Cart\n    fields:\n      - {name: id, type: Uuid}\n";
    let broken_view = "views:\n  - name: shop.cart.Thing\n    source: shop.cart.Cart\n";
    let sound_error = "errors:\n  - name: shop.cart.Thing\n    fields: []\n";
    let broken_error = "errors:\n  - name: shop.cart.Thing\n    fields:\n      - {name: total, type: Decimal}\n      - {name: total, type: Decimal}\n";
    let mut seen = Vec::new();
    for (e, v) in [
        (sound_error, sound_view),
        (broken_error, sound_view),
        (sound_error, broken_view),
        (broken_error, broken_view),
    ] {
        let doc = format!("\ndomain: shop.cart\n{entity}{e}{v}");
        let errors = assemble(&[("domains/cart.yaml", &doc)]);
        let refusal = errors
            .as_slice()
            .iter()
            .find(|error| {
                error
                    .message
                    .starts_with("`shop.cart.Thing` is declared twice")
            })
            .unwrap_or_else(|| panic!("no refusal: {errors}"));
        seen.push((refusal.location.clone(), refusal.message.clone()));
    }
    assert!(
        seen.windows(2).all(|pair| pair[0] == pair[1]),
        "one fault, four sentences: {seen:#?}"
    );
}
