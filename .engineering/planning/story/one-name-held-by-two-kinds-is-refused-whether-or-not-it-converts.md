---
format: aep.planning-md/3
id: story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts
kind: story
status: active
title: One name held by two kinds is refused whether or not either copy converts
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/tests/locator_citations.rs
- confidence: inferred
  path: crates/specify/ess-compiler/tests/typed_diagnostics.rs
- confidence: cited
  path: crates/specify/ess-domain/src/domain.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/masked_declaration_boundaries.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T10:14:56Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-09-28T10:14:56Z", actor: "human:timo", revision: 10}
---
# One name held by two kinds is refused whether or not either copy converts

`story:a-masked-first-declaration-hides-a-duplicate-name` closed the same-kind half of a class:
a name is declared where it is written, whether or not what is written under it converts.
`Spec::declare` now asks *has this name been written* before the conversion is attempted, and eight
declaration kinds go through it.

**Its key is `(kind, name)`, so it closes only half.**

One name held by **two different kinds** is reported by a different pair of reporters —
`system.rs:989 Assembly::claim` and `domain.rs:243 DomainSpec::validate` — and both read
`DomainMembers`, which is filled only from `Ok(..)` arms. That is the exact blindness the story
removed from the registries, still in place one layer over.

Measured by the wave-25 unit-2 adversary at `masked_declaration_adversary_pass1.rs:125`, exit 101,
with a passing control at `:107`: a command `shop.cart.Thing` carrying a duplicate input field, plus
an event `shop.cart.Thing`. The author is told about the input field and **nothing about the name**.
Make the command sound and the clash is reported. So the refusal exists, and disappears exactly when
a copy breaks.

## A second reporter that is not there

`ess-domain/src/spec.rs:1237` says a declared **type** can be masked and points at
`SystemSpec::merge` as "a second reporter … with a contract of its own". That reporter receives only
types that converted. For the masked case there is no second reporter at all.

Measured at `masked_declaration_adversary_pass1.rs:180`, exit 101: two `shop.cart.Money` types, the
first with `variants: []`. One error about variants, nothing about the name, and the second
declaration silently owns it.

## The doc comment that would stop the next reader looking

`spec.rs:542`'s new comment says, without qualification, that one name held by two different kinds is
what `DomainSpec::validate_all` reports. It does not report it when either copy fails its
conversion — the only case the story is about. **That sentence is wave 25's, and correcting it is
wave 25's; it is not part of this story.**

## Acceptance

A name written twice is refused whether the two writings are the same kind or different kinds, and
whether or not either converts. A declared type is covered by the same rule or the bound is stated
where a reader will find it.

## The two candidate shapes, and what each costs

- **Key `declared` by name alone** and let the message distinguish kinds. Simplest, and changes the
  message of every already-reported same-kind duplicate.
- **Move the cross-kind check ahead of conversion**, the way `declare` does for registries. Leaves
  existing messages alone and adds a third place the rule lives.

Whoever takes this picks one and says why the other is worse. Note that the first touches a
diagnostic contract for cases that are already reported today.

## Scope

Re-derived 2026-09-28 by `story-scoper` on `46e367ab2`. Each line **cited** or **inferred**.

- **Status:** open — `ess specify validate` 0.38.0 on fixtures from `masked_declaration_boundaries.rs`: CROSS_KIND_FIRST_BROKEN reports only ESS-COMMAND-006, TYPE_FIRST_BROKEN only ESS-TYPE-007; neither refuses the duplicate name. SOUND variants refuse (ESS-DOMAIN-006, ESS-SPEC-006) — cited (run)
- **Primary surface:** `crates/specify/ess-domain/src/spec.rs` — `declare` `:623` keyed `(kind, name)` `:624`/`:750`; call sites `:852-:1020`; type path `:843-844` pushes only `Ok` and never calls `declare` — cited
- **Files:** `crates/specify/ess-domain/src/system.rs:1047` `Assembly::claim`, `:479` `SystemSpec::merge` — cited (story's `:989` stale)
- **Files:** `crates/specify/ess-domain/src/domain.rs:242` `DomainSpec::validate`, `:267` `validate_all` — cited
- **Tests:** `crates/specify/ess-domain/tests/masked_declaration_boundaries.rs:119-123`, `:178-181` — two `#[ignore]`d cases pin this story; acceptance un-ignores them — cited
- **Also:** `spec.rs:1588-1593` test doc and `declare` doc `:614-622` say "open"; rewrite on close — cited
- **Also likely:** `crates/specify/ess-compiler/tests/typed_diagnostics.rs`, `locator_citations.rs` if same-kind duplicate messages change — inferred
- **Story drift:** the doc-comment item is already done at `spec.rs:614-622` — cited
- **Confidence:** high
- **Safety fact:** keying by name alone changes every same-kind duplicate message (`spec.rs:640`); a pre-conversion cross-kind check must not double-report with `Assembly::claim` (`system.rs:1041-1045`) — inferred
