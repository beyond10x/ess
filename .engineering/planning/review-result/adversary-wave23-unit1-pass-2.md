---
format: aep.planning-md/2
id: review-result:adversary-wave23-unit1-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: the same silence, one Optional further along'
relations:
- reviews: story:structured-ring-is-refused-by-two-passes
revision: 1
---
## Pass

`aep-drive:adversary`, pass 2, against `wt-dcb4b24b4ed8` over base `46c7281e`.
Verdict **NEEDS-CHANGE**. Cases executed 438 → 445, red 4. Origin: introduced 3.

Seven cases added in `crates/specify/ess-domain/tests/literal_representation_adversary_pass4.rs`,
all driven through `RawSpecFile::parse` + `Specification::assemble`, so "silence" is silence from
the whole compiler.

## The blocker: `OwnedByNobody`, one `Optional` further along

The repaired arm reads `inhabitation.refuses(named)` and **ignores `base_cases`**. An `Optional`
crossed on the way to the struct makes the type the literal actually fills *inhabited*, so
`check_inhabitation` says nothing about it — it refuses only the inner struct, a different
declaration about a different mistake. The arm still answers `Uninhabited`, the caller reads that as
*somebody else speaks*, and nobody does.

Whole-compiler output for `Alpha = newtype of Optional<Pick>` / `Pick = struct { only: Pick }` with
a literal into `Alpha` is **one** line, `[self_reference] types.notifications.core.Pick`, and
nothing at the mapping site. Both consumers. The plainest reproducer declares no wrapper at all:
an input typed `Optional<notifications.core.Pick>` directly.

The neighbouring `Cyclic` arm applies exactly the missing rule, and a green contrast case proves it.
`Optional<…>` appears 64 times in this repository's own shipped models, examples and fixtures.

## The adversary's own correction

Its seventh case failed on a typo it made — it spelled a lifecycle enum `calls.core.CallState`
where the name is `calls.core.Call.State`. Corrected and re-run alone: **green**, the two registries
agree. Reported as a probe that came back clean rather than as a finding.

## Attacked and could not break

`refused` is exactly what base `check_inhabitation` emitted: `TypeRegistry` is a
`BTreeMap<QualifiedName, NamedType>` whose `insert` refuses a second declaration, so `iter()` yields
each name once, and both versions emit inside the same name-ordered loop. Pass 1's blocker is
genuinely closed — `Pick = struct { only: Missing }` now answers `Structured` and is refused here.
A union with one undeclared and one good variant stays inhabited. The bare-name ring arm looks like
the same hole and is not: reaching it requires every step to be a bare `Named` into a declared
newtype. The two registries agree, exercised by a struct naming a lifecycle enum inside a ring. The
hoist takes the same `&registry` in both callers. `Resolved<'a>` is three shared references in a
`Copy` struct from one stack frame; no caller sees a different registry.

```findings
- file: crates/specify/ess-domain/src/binding.rs
  line: 1821
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the repaired struct and union arm reads inhabitation.refuses(named) without consulting base_cases, so an Optional crossed on the way to a refused struct leaves the literal's own inhabited type refused by nobody, on both the binding and the payload consumer.
- file: crates/specify/ess-domain/src/binding.rs
  line: 1769
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the walk's new doc claims the arm applies the same base-case rule as check_inhabitation, Resolution::Uninhabited's claims check_inhabitation refuses the type the walk reached, and command.rs:2175 claims Uninhabited is exactly the set self_reference is emitted for, and all three are false once an Optional sits between the literal's type and the struct.
- file: crates/specify/ess-domain/src/system.rs
  line: 625
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: refuses(name) answers whether check_inhabitation speaks about that name, but a pass deciding to stay silent needs whether it speaks about the type the literal fills, and the name plus its doc present the first as if it were the second.
```
