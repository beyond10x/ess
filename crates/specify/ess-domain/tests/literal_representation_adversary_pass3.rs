//! Third adversary pass, against the change `story:structured-ring-is-refused-by-two-passes` made.
//!
//! That change gave the `Structured` arm of the representation walk an ownership test: a struct or
//! a union the inhabitation fixpoint does not mark is now [silent], on the stated grounds that
//! `check_inhabitation` refuses the declaration itself. This file attacks the second half of that
//! sentence, because it is the half nothing else checks — the pass-2 matrix builds its three names
//! only out of each other, so no shape in it ever names a type nobody declared, and
//! `check_inhabitation`'s own `names_something_undeclared` skip is never reached.
//!
//! Everything here goes through `RawSpecFile::parse` + `Specification::assemble`, so "silence"
//! means silence from the whole compiler and not from one pass.

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;

/// Assemble one document; `None` when it was admitted, the joined refusals when it was not.
fn refusals(document: &str) -> Option<String> {
    let raw = match RawSpecFile::parse(document) {
        Ok(raw) => raw,
        Err(error) => return Some(error.to_string()),
    };
    Specification::assemble(vec![(Source::new("adversary3.yaml"), raw)])
        .err()
        .map(|errors| errors.to_string())
}

/// A document whose binding writes a bare literal into a command input of `target`.
fn binding_document(types: &str, target: &str) -> String {
    format!(
        r"
format: ess/1
system: notifications
version: v1
domain: notifications.core
types:
{types}commands:
  - name: notifications.core.Reject
    input:
      - name: reason
        type: {target}
    outcomes:
      - name: rejected
        emits: [notifications.core.Rejected]
events:
  - name: notifications.core.Refused
  - name: notifications.core.Rejected
bindings:
  - id: reject-on-refusal
    when:
      event: notifications.core.Refused
    invoke:
      command: notifications.core.Reject
    mapping:
      reason: just-text
    delivery: at_most_once
    on_failure: drop
"
    )
}

/// A struct with one field naming `field_type`.
fn pick(field_type: &str) -> String {
    format!(
        "  - name: notifications.core.Pick\n    kind: struct\n    fields:\n      - name: \
         only\n        type: {field_type}\n"
    )
}

/// A union with one variant naming `variant_type`.
fn choice(variant_type: &str) -> String {
    format!(
        "  - name: notifications.core.Pick\n    kind: union\n    tag: kind\n    variants:\n      \
         only: {variant_type}\n"
    )
}

/// The premise: the literal *is* wrong, and somebody says so when the struct is whole.
///
/// Without this, the two cases below could be read as "that document was fine all along". It
/// declares the field's type, changing nothing else, and the literal check speaks.
#[test]
fn a_literal_written_into_a_whole_struct_is_refused_by_the_literal_check() {
    let types = format!(
        "{}  - name: notifications.core.Other\n    kind: newtype\n    of: String\n",
        pick("notifications.core.Other")
    );
    let found = refusals(&binding_document(&types, "notifications.core.Pick"));
    let errors = found.expect("a literal written into a struct is not a literal");
    assert!(
        errors.contains("has structure"),
        "the literal check owns a struct that can be built, and did not speak: {errors}"
    );
}

/// A struct whose one field names a type nobody declared, filled by a literal.
///
/// The mistake under test is the **literal**, not the missing declaration: they are two mistakes in
/// one document and each is entitled to its own diagnostic. The walk now answers `Uninhabited` here,
/// because `inhabited_names` cannot mark a struct whose field names nothing — and
/// `check_inhabitation`, the pass the silence defers to, declines this exact shape at
/// `system.rs:609` (`names_something_undeclared`) precisely because the missing declaration is
/// already reported elsewhere. So the deferral is to a pass that does not take it, and no
/// diagnostic anywhere in the compiler mentions the binding's mapping.
#[test]
fn a_literal_written_into_a_struct_whose_field_names_nothing_is_still_refused_by_somebody() {
    let found = refusals(&binding_document(
        &pick("notifications.core.Missing"),
        "notifications.core.Pick",
    ));
    let errors = found.expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "the walk deferred this struct to `check_inhabitation`, which skips a type that names \
         something undeclared, so nothing in the compiler refuses the literal: {errors}"
    );
}

/// The same hole reached through a union, which the change treats identically.
#[test]
fn a_literal_written_into_a_union_whose_variant_names_nothing_is_still_refused_by_somebody() {
    let found = refusals(&binding_document(
        &choice("notifications.core.Missing"),
        "notifications.core.Pick",
    ));
    let errors = found.expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "the walk deferred this union to `check_inhabitation`, which skips a type that names \
         something undeclared, so nothing in the compiler refuses the literal: {errors}"
    );
}

/// The same hole on the other consumer the story names: an outcome payload's literal.
///
/// `command.rs`'s `check_payload_literal` reads the same `Resolution`, so the silence arrives there
/// too — and `command.rs` is in the story's scope with "needed no code change".
#[test]
fn a_payload_literal_written_into_a_struct_field_naming_nothing_is_still_refused_by_somebody() {
    let document = format!(
        r"
format: ess/1
system: notifications
version: v1
domain: notifications.core
types:
{}commands:
  - name: notifications.core.Reject
    input:
      - name: reason
        type: String
    outcomes:
      - name: rejected
        emits: [notifications.core.Rejected]
        payload:
          notifications.core.Rejected:
            detail: just-text
events:
  - name: notifications.core.Refused
  - name: notifications.core.Rejected
    fields:
      - name: detail
        type: notifications.core.Pick
bindings:
  - id: reject-on-refusal
    when:
      event: notifications.core.Refused
    invoke:
      command: notifications.core.Reject
    mapping:
      reason: anything
    delivery: at_most_once
    on_failure: drop
",
        pick("notifications.core.Missing")
    );
    let errors = refusals(&document).expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("outcomes.rejected.payload"),
        "the payload literal is owned by nobody once the struct's field names something \
         undeclared: {errors}"
    );
}

/// The walk's cost per literal stopped being constant in the size of the registry.
///
/// `inhabited_names` is a least fixpoint over the whole registry — O(names²) references examined,
/// with a `QualifiedName` clone per mark — and it is now called *inside* `representation`, once for
/// every literal whose walk stops on a struct or a union. Nothing caches it, so a document with `L`
/// such literals over `N` declarations pays `O(L · N²)` where it used to pay `O(L · N)`.
///
/// Asserted as a **ratio**, not a wall clock, so the case says something about the shape of the
/// curve rather than about this machine: quadrupling the registry and the literal count together
/// should not raise the cost by more than eightfold. Measured on this host, debug profile, with the
/// implementation's own sources against the base at 46c7281e:
///
/// | names | base | this change |
/// |---|---|---|
/// | 100 | 8.4 ms | 33.9 ms |
/// | 400 | 33.4 ms | 515.4 ms |
///
/// — a ratio of 4.0 at the base and 15.2 here.
#[test]
fn the_cost_of_one_document_stays_within_reach_of_linear_in_the_registry() {
    use std::fmt::Write as _;
    use std::time::{Duration, Instant};

    /// A document declaring `n` one-field structs and writing a literal into each of them.
    fn document(n: usize) -> String {
        let mut types = String::new();
        let mut inputs = String::new();
        let mut mapping = String::new();
        for index in 0..n {
            let _ = write!(
                types,
                "  - name: notifications.core.T{index}\n    kind: struct\n    fields:\n      - \
                 name: only\n        type: String\n"
            );
            let _ = write!(
                inputs,
                "      - name: f{index}\n        type: notifications.core.T{index}\n"
            );
            let _ = writeln!(mapping, "      f{index}: text{index}");
        }
        format!(
            r"
format: ess/1
system: notifications
version: v1
domain: notifications.core
types:
{types}commands:
  - name: notifications.core.Reject
    input:
{inputs}    outcomes:
      - name: rejected
        emits: [notifications.core.Rejected]
events:
  - name: notifications.core.Refused
  - name: notifications.core.Rejected
bindings:
  - id: reject-on-refusal
    when:
      event: notifications.core.Refused
    invoke:
      command: notifications.core.Reject
    mapping:
{mapping}    delivery: at_most_once
    on_failure: drop
"
        )
    }

    /// The fastest of three assemblies of `document(n)`, which is the one least disturbed by a
    /// neighbour on the same host.
    fn best(n: usize) -> Duration {
        let text = document(n);
        (0..3)
            .map(|_| {
                let started = Instant::now();
                let found = refusals(&text);
                assert!(
                    found.is_some(),
                    "every literal here is written into a struct"
                );
                started.elapsed()
            })
            .min()
            .expect("three runs")
    }

    let small = best(100);
    let large = best(400);
    assert!(
        large <= small * 8,
        "quadrupling a document's declarations raised the cost {:.1}x ({small:?} at 100 names, \
         {large:?} at 400): `representation` recomputes the whole inhabitation fixpoint for every \
         literal that stops on a struct or a union",
        large.as_secs_f64() / small.as_secs_f64()
    );
}
