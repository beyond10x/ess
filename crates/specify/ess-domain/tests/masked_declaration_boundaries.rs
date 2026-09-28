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
//! lists filled only from `Ok(..)` arms, so all three masked in precisely the way the registries
//! did: `review-result:adversary-wave25-unit2-pass-1` is where that was first measured (F1 and F3).
//! `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` closed both with
//! `spec.rs`'s `Collected::write`, which asks the cross-kind question of every writing, converted
//! or not, and hands `Assembly::claim` only one copy of each name so each is refused once. The
//! last four cases here ask that of every soundness combination and count the refusals.
//!
//! The two first-measured cases each pair a control (both declarations convert — the refusal the
//! tree already made) with the same document under one broken declaration, so a failure separates
//! "this was never refused" from "this stopped being refused when a copy broke". The other cases
//! are green and stay that way: two of `spec.rs`'s argued-rather-than-built rows, `conversion` and `topology`,
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
        "a command and an event of one name is refused, as it was before any copy could break: {errors}"
    );
}

/// The same document, one copy broken, and the refusal is still there.
///
/// This is the unit's own fault shape: the author was told about the duplicate field, fixed it, and
/// only then learned that the name was held twice. `declare` does not catch it because its key
/// carries the kind, and `Assembly::claim` reads a member list that a failed conversion never
/// reaches; `Collected::write` is what refuses it now.
#[test]
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

/// The control: a type declared twice is refused when both declarations convert.
#[test]
fn one_type_name_declared_twice_is_refused_when_both_declarations_convert() {
    let errors = refusals(TYPE_SOUND);
    assert!(
        refused_as_declared_twice(&errors, "shop.cart.Money"),
        "two declarations of one type name, both sound, are refused: {errors}"
    );
}

/// The row the first unit left open, measured rather than assumed.
///
/// `declare` is not called for types at all, and `SpecPart` carried to `SystemSpec::merge` receives
/// only the types that converted. So the second declaration took the name in silence and the
/// author saw one error about variants, which is the exact experience the story was written to
/// end; `Collected::write` refuses it now.
#[test]
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

/// How many refusals say `name` is held twice, by either sentence a claim is refused with.
///
/// Counted rather than tested for presence: the rule is one refusal per extra writing, and a
/// reporter that fires beside another one would satisfy a presence check while telling the
/// author the same thing twice.
fn name_refusals(errors: &ValidationErrors, name: &str) -> usize {
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
                        .starts_with(&format!("`{name}` is declared in")))
        })
        .count()
}

/// A command `shop.cart.Thing`, sound or carrying one input twice.
fn command(sound: bool) -> &'static str {
    if sound {
        "
  - name: shop.cart.Thing
    outcomes:
      - name: added
        emits: [shop.cart.Other]
"
    } else {
        "
  - name: shop.cart.Thing
    input:
      - {name: note, type: String}
      - {name: note, type: String}
    outcomes:
      - name: added
        emits: [shop.cart.Other]
"
    }
}

/// An event `shop.cart.Thing`, sound or carrying one field twice.
fn event(sound: bool) -> &'static str {
    if sound {
        "
  - name: shop.cart.Thing
    fields: []
"
    } else {
        "
  - name: shop.cart.Thing
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
"
    }
}

/// A type `shop.cart.Money`, sound or declaring no variants.
fn money(sound: bool) -> &'static str {
    if sound {
        "
  - name: shop.cart.Money
    kind: newtype
    of: Decimal
"
    } else {
        "
  - name: shop.cart.Money
    kind: enum
    variants: []
"
    }
}

/// The four soundness combinations, each with its label.
const COMBINATIONS: [(&str, bool, bool); 4] = [
    ("sound, sound", true, true),
    ("broken, sound", false, true),
    ("sound, broken", true, false),
    ("broken, broken", false, false),
];

/// One name, a command and an event, in all four soundness combinations: refused exactly once.
///
/// `Collected::write` refuses every row and `Assembly::claim` never sees a second copy, so no row
/// is refused twice. The sentence is the one `Assembly::claim` wrote for the sound row.
#[test]
fn one_name_held_by_two_kinds_is_refused_exactly_once_in_all_four_soundness_combinations() {
    for (label, first, second) in COMBINATIONS {
        let document = format!(
            "\ndomain: shop.cart\ncommands:{}events:\n  - name: shop.cart.Other\n    fields: \
             []{}",
            command(first),
            event(second)
        );
        let errors = refusals(&document);
        assert_eq!(
            name_refusals(&errors, "shop.cart.Thing"),
            1,
            "a command and an event of one name ({label}) is one fault, refused once: {errors}"
        );
        assert!(
            refused_as_declared_twice(&errors, "shop.cart.Thing"),
            "({label}) {errors}"
        );
    }
}

/// One type name, twice, in all four soundness combinations: refused exactly once.
#[test]
fn one_type_name_declared_twice_is_refused_exactly_once_in_all_four_soundness_combinations() {
    for (label, first, second) in COMBINATIONS {
        let document = format!(
            "\ndomain: shop.cart\ntypes:{}{}",
            money(first),
            money(second)
        );
        let errors = refusals(&document);
        assert_eq!(
            name_refusals(&errors, "shop.cart.Money"),
            1,
            "one type name declared twice ({label}) is one fault, refused once: {errors}"
        );
    }
}

/// The two kinds in two files, in every combination: both files are named, as `Assembly::claim`
/// named them when both copies converted.
#[test]
fn one_name_held_by_two_kinds_across_two_files_names_both_when_a_copy_is_broken() {
    for (label, first, second) in COMBINATIONS {
        let errors = Specification::assemble(vec![
            file(
                "system.yaml",
                "\nformat: ess/1\nsystem: shop\nversion: v1\n",
            ),
            file(
                "domains/commands.yaml",
                &format!("\ndomain: shop.cart\ncommands:{}", command(first)),
            ),
            file(
                "domains/events.yaml",
                &format!(
                    "\ndomain: shop.cart\nevents:\n  - name: shop.cart.Other\n    fields: \
                     []{}",
                    event(second)
                ),
            ),
        ])
        .expect_err("one name, two kinds");
        let named: Vec<_> = errors
            .as_slice()
            .iter()
            .filter(|error| {
                error.message
                    == "`shop.cart.Thing` is declared in `domains/commands.yaml` and in \
                        `domains/events.yaml`"
            })
            .collect();
        assert_eq!(
            named.len(),
            1,
            "({label}) one refusal naming both files: {errors}"
        );
        assert_eq!(named[0].location, "domain shop.cart.events", "({label})");
    }
}

/// A name written three times — twice as one kind, once as another — is refused once per extra
/// writing, not once per reporter.
///
/// The second command is `declare`'s and the event is `Collected::write`'s; `Assembly::claim` gets
/// only the sound command, so it must not refuse the event again.
#[test]
fn a_name_written_three_times_is_refused_once_per_extra_writing() {
    let document = format!(
        "\ndomain: shop.cart\ncommands:{}{}events:\n  - name: shop.cart.Other\n    fields: \
         []{}",
        command(false),
        command(true),
        event(true)
    );
    let errors = refusals(&document);
    assert_eq!(
        name_refusals(&errors, "shop.cart.Thing"),
        1,
        "the event is refused once: {errors}"
    );
    assert_eq!(
        errors
            .as_slice()
            .iter()
            .filter(|error| error.location == "command shop.cart.Thing")
            .count(),
        1,
        "the second command is refused once, by `declare`: {errors}"
    );
}

/// One event name in two files, both sound: one refusal, and that one names both files.
///
/// `Assembly::claim` used to refuse this a second time, and its sentence was the only one that
/// named the first file. With it kept out, `declare`'s refusal is the only one, so it has to name
/// both — in its hint, leaving its sentence and location as they were.
#[test]
fn one_name_of_one_kind_in_two_files_is_refused_once_naming_both_files() {
    let declaration = "\ndomain: shop.cart\nevents:\n  - name: shop.cart.Thing\n    fields: []\n";
    let errors = Specification::assemble(vec![
        file(
            "system.yaml",
            "\nformat: ess/1\nsystem: shop\nversion: v1\n",
        ),
        file("a.yaml", declaration),
        file("b.yaml", declaration),
    ])
    .expect_err("declared twice");
    let about_the_name: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| {
            error.code == ValidationCode::DuplicateDeclaration
                && error.message.contains("`shop.cart.Thing`")
        })
        .collect();
    assert_eq!(about_the_name.len(), 1, "one extra writing: {errors}");
    let refusal = about_the_name[0];
    assert_eq!(refusal.location, "event shop.cart.Thing");
    assert_eq!(
        refusal.message,
        "`shop.cart.Thing` is declared more than once; b.yaml declares it again"
    );
    assert!(
        refusal
            .hint
            .as_deref()
            .is_some_and(|hint| hint.starts_with("a.yaml declares it first")),
        "the first file is named: {errors}"
    );
}
