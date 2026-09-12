//! Fourth adversary pass, against the repair `story:structured-ring-is-refused-by-two-passes`
//! made after `review-result:adversary-wave23-unit1-pass-1`.
//!
//! Pass 1's blocker was that the `Structured` arm deferred to a pass that declines. The repair
//! moved the decision into `system::Inhabitation`, so the arm now asks *does `check_inhabitation`
//! refuse this name* rather than *can a value of it exist*. That closes the hole pass 1 measured.
//!
//! This file attacks the half the repair did not touch: **the arm still ignores `base_cases`**.
//! The walk's own documentation says the ring answers turn on whether an `Optional` was crossed
//! *inside* the loop — a ring reached through an `Optional` is inhabited, nothing under it is
//! spellable as text, nobody else reports it, so `Resolution::Cyclic` refuses it here. The struct
//! and union arm reads `inhabitation.refuses(named)` and nothing else, so the same `Optional`
//! buys nothing: a declared, inhabited type whose only content is a refused struct behind an
//! `Optional` is answered `Uninhabited` and the literal written into it is reported by nobody.
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
    Specification::assemble(vec![(Source::new("adversary4.yaml"), raw)])
        .err()
        .map(|errors| errors.to_string())
}

/// A document whose binding writes the bare literal `just-text` into a command input of `target`.
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

/// `Alpha = newtype of Optional<…>`: declared, and inhabited because absence is a value.
fn alpha_over_optional(inner: &str) -> String {
    format!("  - name: notifications.core.Alpha\n    kind: newtype\n    of: Optional<{inner}>\n")
}

/// A struct with one field naming `field_type`.
fn pick_struct(field_type: &str) -> String {
    format!(
        "  - name: notifications.core.Pick\n    kind: struct\n    fields:\n      - name: \
         only\n        type: {field_type}\n"
    )
}

/// A union with one variant naming `variant_type`.
fn pick_union(variant_type: &str) -> String {
    format!(
        "  - name: notifications.core.Pick\n    kind: union\n    tag: kind\n    variants:\n      \
         only: {variant_type}\n"
    )
}

/// The premise: when the struct behind the `Optional` can be built, the literal check speaks.
///
/// Without this, the cases below could be read as "that document was broken for some other
/// reason". Only the struct's field changes between this case and the next one.
#[test]
fn a_literal_into_an_optional_of_a_whole_struct_is_refused_by_the_literal_check() {
    let types = format!(
        "{}{}",
        alpha_over_optional("notifications.core.Pick"),
        pick_struct("String")
    );
    let errors = refusals(&binding_document(&types, "notifications.core.Alpha"))
        .expect("a literal written into a struct behind an Optional is not a literal");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "the literal check owns a struct that can be built, and did not speak: {errors}"
    );
}

/// The contrast the walk's own documentation draws, and it holds: an `Optional` crossed inside a
/// ring of **bare names** makes the ring inhabited, `check_inhabitation` says nothing, and the
/// walk refuses the literal itself as `Cyclic`.
///
/// This is the rule the struct arm is supposed to share. It does not, which is the next three
/// cases.
#[test]
fn a_literal_into_a_ring_of_bare_names_through_an_optional_is_refused_by_the_walk() {
    let types = alpha_over_optional("notifications.core.Alpha");
    let errors = refusals(&binding_document(&types, "notifications.core.Alpha"))
        .expect("a ring with nothing spellable under it is not a literal's type");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "`Cyclic` owns an inhabited ring with no text under it, and did not speak: {errors}"
    );
}

/// **The finding.** `Alpha` is declared and inhabited — absence is a value of `Optional<Pick>` —
/// and `check_inhabitation` therefore says nothing whatever about `Alpha`. It refuses `Pick`, and
/// only `Pick`: a different declaration, about a different mistake, which stands on its own
/// whether or not any literal is written anywhere.
///
/// The walk crosses the `Optional`, meets `Pick`, asks `inhabitation.refuses("Pick")` — true —
/// and answers `Uninhabited`, which the caller reads as *somebody else speaks*. Nobody else
/// speaks about the literal. This is `OwnedByNobody` reached through the arm pass 1 did not
/// measure, and it is the same silence, one `Optional` further along.
///
/// Had `Alpha` been `newtype of Optional<Alpha>` the walk would have answered `Cyclic` and refused
/// the literal here, which is the case directly above. The only difference between the two
/// documents is whether the ring closes through a bare name or through a struct.
#[test]
fn a_literal_into_an_optional_of_a_refused_struct_is_still_refused_by_somebody() {
    let types = format!(
        "{}{}",
        alpha_over_optional("notifications.core.Pick"),
        pick_struct("notifications.core.Pick")
    );
    let errors = refusals(&binding_document(&types, "notifications.core.Alpha"))
        .expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "the walk deferred an inhabited `Alpha` to a `self_reference` that is about `Pick`, so \
         nothing in the compiler refuses the literal: {errors}"
    );
}

/// The same hole through a union, which the arm treats identically.
#[test]
fn a_literal_into_an_optional_of_a_refused_union_is_still_refused_by_somebody() {
    let types = format!(
        "{}{}",
        alpha_over_optional("notifications.core.Pick"),
        pick_union("notifications.core.Pick")
    );
    let errors = refusals(&binding_document(&types, "notifications.core.Alpha"))
        .expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "the walk deferred an inhabited `Alpha` to a `self_reference` that is about `Pick`, so \
         nothing in the compiler refuses the literal: {errors}"
    );
}

/// The same hole with no intermediate declaration at all: the command input is written
/// `Optional<Pick>` directly, which is the plainest thing an author can write.
///
/// Nothing here is a wrapper somebody has to have declared first. `Optional<Pick>` is a type
/// reference in ordinary use, the literal is a bare word, and the compiler has nothing to say
/// about the mapping.
#[test]
fn a_literal_into_a_bare_optional_of_a_refused_struct_is_still_refused_by_somebody() {
    let errors = refusals(&binding_document(
        &pick_struct("notifications.core.Pick"),
        "Optional<notifications.core.Pick>",
    ))
    .expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("binding.reject-on-refusal.mapping.reason"),
        "an `Optional` of a refused struct takes the `Uninhabited` answer, so nothing in the \
         compiler refuses the literal: {errors}"
    );
}

/// The other consumer the story names, on the same shape: an outcome payload's literal.
///
/// `command.rs`'s `check_payload_literal` reads the same `Resolution` through the same
/// `Inhabitation`, so the silence arrives there too.
#[test]
fn a_payload_literal_into_an_optional_of_a_refused_struct_is_still_refused_by_somebody() {
    let types = format!(
        "{}{}",
        alpha_over_optional("notifications.core.Pick"),
        pick_struct("notifications.core.Pick")
    );
    let document = format!(
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
        type: notifications.core.Alpha
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
"
    );
    let errors = refusals(&document).expect("this document is wrong twice over and was admitted");
    assert!(
        errors.contains("outcomes.rejected.payload"),
        "the payload literal is owned by nobody once the refused struct sits behind an \
         `Optional`: {errors}"
    );
}

/// A consistency probe, not an accusation: `check_inhabitation` runs over the registry the merge
/// built, which carries every entity's lifecycle enum; `Inhabitation::of` is now run again in
/// `validate_bindings` over the registry `Specification::validate` assembled. If those two ever
/// held different names, a struct naming a lifecycle enum would fall on different sides of
/// `names_something_undeclared` in the two of them, and this document would grow a diagnostic
/// about the mapping that the single-pass design says it should not have.
///
/// `Ringed` is uninhabited and names only declared types, so `check_inhabitation` refuses it and
/// the walk is entitled to its silence. The assertion is that the silence is intact — one
/// `self_reference`, nothing about the mapping.
#[test]
fn the_two_registries_agree_about_a_struct_naming_a_lifecycle_enum() {
    let document = r"
format: ess/3
system: calls
version: v1
domain: calls.core
types:
  - name: calls.core.Ringed
    kind: struct
    fields:
      - name: state
        type: calls.core.Call.State
      - name: self_again
        type: calls.core.Ringed
entities:
  - name: calls.core.Call
    identity: {name: call_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Bridged
      states: [Bridged]
      terminal: [Bridged]
commands:
  - name: calls.core.Reject
    input:
      - name: reason
        type: calls.core.Ringed
    outcomes:
      - name: rejected
        emits: [calls.core.Rejected]
events:
  - name: calls.core.Refused
  - name: calls.core.Rejected
bindings:
  - id: reject-on-refusal
    when:
      event: calls.core.Refused
    invoke:
      command: calls.core.Reject
    mapping:
      reason: just-text
    delivery: at_most_once
    on_failure: drop
";
    let errors = refusals(document).expect("`Ringed` cannot be built and somebody must say so");
    assert!(
        errors.contains("self_reference"),
        "`check_inhabitation` owns `calls.core.Ringed` and did not speak: {errors}"
    );
    assert!(
        !errors.contains("binding.reject-on-refusal.mapping.reason"),
        "the walk spoke about a struct `check_inhabitation` already refuses, which means the \
         registry `Inhabitation::of` sees in `validate_bindings` is not the registry \
         `check_inhabitation` ran over: {errors}"
    );
}
