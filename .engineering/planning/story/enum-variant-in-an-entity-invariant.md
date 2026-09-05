---
format: aep.planning-md/1
id: story:enum-variant-in-an-entity-invariant
kind: story
status: draft
title: An entity invariant may name an enum variant that does not exist, and validate accepts it
relations:
- informed_by: story:review-expression-typechecking
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/tests/billing.rs
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
revision: 6
---
## What is wrong

An entity invariant may compare an enum-typed field to a variant the enum does not declare, and
`ess specify validate` accepts it.

Reproduced 2026-09-04 against `ess 0.15.0`, on a copy of the ACD model in `acd/specs`:

```console
$ sed -i '242s/line_status == selected/line_status == NOT_A_VARIANT_AT_ALL/' domains/routing.yaml
$ ess specify validate --path .
acd v3 — 4 file(s), valid
$ echo $?
0
```

`line_status` is `acd.routing.LineStatus`, an enum with nine declared variants.
`NOT_A_VARIANT_AT_ALL` is not one of them.

**View filters are checked.** The same comparison in a view's `filter:` is refused. So the machinery
exists and the entity-invariant path does not use it.

## Why it is worth fixing rather than noting

The invariants most worth lifting into a model are the ones about lifecycle state — *an agent in an
active-call status is not available*, *Selected implies an agent*. Every one of them compares a field
to a variant, and every one is a place a typo turns into an invariant that is never violated because
nothing can match it. It reads as a passing constraint and it constrains nothing.

This was found while modelling ACD's routing core, where the whole point of the exercise was to lift
invariants out of Go regression tests named after outages. An invariant that silently checks nothing
is worse than the test it replaced.

## Acceptance

1. An entity invariant naming an undeclared enum variant is refused by `ess specify validate`, with
   a stable refusal code, the file it was read from, and the variants that are declared.
2. The refusal names the enum, not only the field, so the fix is one lookup away.
3. A test covers it — and a second covers the case that already works, the same comparison in a view
   filter, so a future refactor cannot lose one while keeping the other.

## What this does not cover

Only equality against a variant. Whether every other predicate form over an enum resolves its
right-hand side is not established here and should be checked while fixing this, not assumed.


## Integration Provenance

Reconciled through AEP from wt-e46db550dce9 at original revision 1 and status draft. Source artifact SHA-256: 3a90e2d966fdab5e49f7f4655b3fbe80d294a78d181c9c31d0db8e7039850cf9. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.

## Scope

Derived 2026-09-06 by `aep-drive:story-scoper` against clean ESS `dcb84be861d2f906b3dd95254f03701cb264faa2` — cited.

- **Primary surface:** `crates/specify/ess-domain` — cited; entity invariant admission and the existing view enum-literal checker both live here.
- **Entity implementation and unit tests:** `crates/specify/ess-domain/src/entity.rs` — cited; `EntitySpec::validate` at line 826 checks fact paths but not their compared literals, and the existing invariant regressions are inline.
- **Shared checking and view regression:** `crates/specify/ess-domain/src/view.rs` — inferred; reuse or extract its existing `validate_filter_values`, `enumeration`, and `compared_values` machinery so entity and view equality checks share enum membership behavior, while retaining the view's existing negative and positive cases.
- **Diagnostic integration tests:** `crates/specify/ess-compiler/tests/billing.rs` — inferred; extend its existing assembly-to-diagnostic test pattern to assert the entity refusal code, source filename, enum name and declared variants.
- **Symbols:** `EntitySpec::validate`, `observable_fields`, `state_type`, `ViewSpec::validate_filter_values`, `enumeration`, `compared_values`, and `collect_compared_values` — cited; these provide the entity environment, enum registry lookup and existing predicate traversal.
- **Bounded change:** reject equality between an enum-typed entity field and an undeclared literal; preserve declared literals, optional enum fields, lifecycle state access and the equivalent view-filter behavior — inferred; these are focused controls around the demonstrated defect.
- **Diagnostics:** the existing `UndeclaredReference` bridge can yield `ESS-ENTITY-001` for entity invariants and `ESS-VIEW-001` for view filters, retaining the domain message and hint — cited; no new refusal code or persisted format is established as necessary.
- **Documents:** none independently required by this imported defect story — inferred; the related expression-typechecking design must govern any broader resolver extraction.
- **Confidence:** high for the validation defect and existing reuse surface — cited; source-attribution completeness and the eventual shared helper placement remain unresolved.
- **Would collide with:** entity/view predicate validation and their inline tests, compiler billing diagnostic tests, and broader expression-typechecking work in ess-domain — inferred; this story remains outside the immediate wave.
