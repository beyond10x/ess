---
format: aep.planning-md/1
id: story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts
kind: story
status: draft
title: One name held by two kinds is refused whether or not either copy converts
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/domain.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
revision: 4
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

- `crates/specify/ess-domain/src/spec.rs` — `cited`
- `crates/specify/ess-domain/src/system.rs` — `cited`
- `crates/specify/ess-domain/src/domain.rs` — `cited`
- `crates/specify/ess-domain/tests/` — `cited`; the two red cases live on
  `impl/a-masked-first-declaration`
