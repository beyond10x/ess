---
format: aep.planning-md/1
id: story:enum-variant-in-an-entity-invariant
kind: story
status: draft
title: An entity invariant may name an enum variant that does not exist, and validate accepts it
relations:
- informed_by: story:review-expression-typechecking
revision: 1
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
