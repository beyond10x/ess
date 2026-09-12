//! Whether `MASKED`'s first-copy half is checked, or only observed to be red.
//!
//! `spec.rs`'s `every_masked_row_is_broken_then_sound_as_this_table_claims` asks two things of each
//! row. The second copy's half is exact — `assemble` must return `Ok`, which nothing collateral can
//! satisfy. The first copy's half is `outcome.is_err()`, and that is satisfied by *any* refusal of
//! the whole assembly: the preamble, a support file, a rule about the document as a whole. The
//! table's comment claims more than `is_err()` can see — "the first copy carries an error of its
//! own **and nothing else**" — so a row whose first copy became sound while its assembly stayed
//! red would keep the check green and stop testing masking, which is the failure mode the second
//! copy's half was added to catch on the other side.
//!
//! `MASKED` is private to `spec.rs`'s test module, so each row's first copy is *copied* here.
//! A copy can go stale where an in-crate reading could not; it is the only way to ask this
//! question from outside, and the question is whether the in-crate check is asking it at all.
//!
//! The assertion is the weakest one that still means something: every refusal of a row's first
//! copy, assembled alone, must name that row's declaration. A refusal that names something else is
//! a refusal `is_err()` would accept while the row said nothing about masking.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// Parses one file's worth of YAML, through the parser the CLI uses.
fn file(source: &str, yaml: &str) -> (Source, RawSpecFile) {
    (
        Source::new(source),
        RawSpecFile::parse(yaml).expect("the document is well formed YAML"),
    )
}

/// `spec.rs`'s `SHOP_CART`, the sound domain the `component` and `binding` rows refer into.
const SHOP_CART: &str = r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields: []
commands:
  - name: shop.cart.AddItem
    outcomes:
      - name: added
        emits: [shop.cart.ItemAdded]
";

/// One `MASKED` row, reduced to what this file asks of it.
struct Row {
    /// The kind whose name is declared twice, as `spec.rs` labels it.
    kind: &'static str,
    /// The file the copy is written in.
    label: &'static str,
    /// The name the row declares twice, which every refusal of its first copy must name.
    name: &'static str,
    /// `spec.rs`'s `preamble` followed by its `first`, which is its `first_alone()`.
    first_alone: &'static str,
    /// `spec.rs`'s `support`.
    support: &'static [(&'static str, &'static str)],
}

/// Every row of `MASKED` whose `first_is_broken` is `true`; the `actor` control has no first-copy
/// refusal to attribute and is not here.
const ROWS: &[Row] = &[
    Row {
        kind: "entity",
        label: "domains/cart.yaml",
        name: "shop.cart.Cart",
        first_alone: r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
",
        support: &[],
    },
    Row {
        kind: "command",
        label: "domains/cart.yaml",
        name: "shop.cart.AddItem",
        first_alone: r"
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
",
        support: &[],
    },
    Row {
        kind: "event",
        label: "domains/cart.yaml",
        name: "shop.cart.ItemAdded",
        first_alone: r"
domain: shop.cart
events:
  - name: shop.cart.ItemAdded
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
",
        support: &[],
    },
    Row {
        kind: "error",
        label: "domains/cart.yaml",
        name: "shop.cart.CartFull",
        first_alone: r"
domain: shop.cart
errors:
  - name: shop.cart.CartFull
    fields:
      - {name: total, type: Decimal}
      - {name: total, type: Decimal}
",
        support: &[],
    },
    Row {
        kind: "view",
        label: "domains/cart.yaml",
        name: "shop.cart.CartById",
        first_alone: r"
domain: shop.cart
entities:
  - name: shop.cart.Cart
    identity: {name: id, type: Uuid}
    lifecycle: {states: [Open], initial: Open, terminal: [Open]}
views:
  - name: shop.cart.CartById
    source: shop.cart.Cart
",
        support: &[],
    },
    Row {
        kind: "component",
        label: "components.yaml",
        name: "cart-service",
        first_alone: r"
components:
  - component: cart-service
",
        support: &[("domains/cart.yaml", SHOP_CART)],
    },
    Row {
        kind: "binding",
        label: "bindings.yaml",
        name: "tell-the-shopper",
        first_alone: r"
bindings:
  - id: tell-the-shopper
    when: {}
    invoke: {command: shop.cart.AddItem}
    delivery: at_least_once
    on_failure: drop
",
        support: &[("domains/cart.yaml", SHOP_CART)],
    },
];

/// Each row's first copy is refused for itself, not for the company it keeps.
///
/// If this is red, `every_masked_row_is_broken_then_sound_as_this_table_claims`'s `is_err()` is
/// being satisfied by a refusal that is not about the first copy, and that row's half of the
/// table's claim is unmeasured.
#[test]
fn every_masked_rows_first_copy_is_refused_for_itself() {
    for row in ROWS {
        let mut files = vec![file(
            "system.yaml",
            "\nformat: ess/1\nsystem: shop\nversion: v1\n",
        )];
        files.extend(
            row.support
                .iter()
                .map(|(label, support)| file(label, support)),
        );
        files.push(file(row.label, row.first_alone));

        let Err(errors) = Specification::assemble(files) else {
            panic!(
                "the {} row's first copy is not refused at all, and `spec.rs` asserts that it is",
                row.kind
            )
        };

        let stray: Vec<&str> = errors
            .as_slice()
            .iter()
            .filter(|error| !error.location.contains(row.name))
            .map(|error| error.location.as_str())
            .collect();
        assert!(
            stray.is_empty(),
            "the {} row's first copy, assembled alone, is refused at {stray:?} — locations that \
             do not name `{}`. `is_err()` cannot tell those refusals from the copy's own, so the \
             row would stay green with a sound first copy and would then say nothing about \
             masking: {errors}",
            row.kind,
            row.name
        );
    }
}
