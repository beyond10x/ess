---
title: Track specification change
sidebar_position: 6
description: Compare compiled specification revisions and explain which scenarios and generated artifacts need another check.
---

# Track specification change

`ess verify diff` describes changes between two compiled models. `ess verify impact` follows their
dependencies to identify conformance scenarios owed another run and generated artifacts owed
regeneration. Neither command establishes that an earlier conformance result still holds.

## Compare revisions

```sh
ess verify diff --from examples/revision-pair/before --to examples/revision-pair/after
ess verify diff --from examples/revision-pair/before --to examples/revision-pair/after --format json
```

The committed example pair changes currency variants, an entity invariant, an outcome condition
and actor grants while also moving files and reordering declarations. The comparison uses the
compiled models, so comments and source-file layout do not themselves produce changes.

The report follows these rules:

- Ten typed construct families are compared: system header, types, entities, commands, events,
  errors, views, actors, components and bindings. Residual model changes outside those typed
  comparisons produce `unclassified-changed`; they are not silently treated as equality.
- Changes carry stable content-derived ids and a relation: widening, narrowing or changed.
  Predicate comparison uses canonical equality, not a proof that one predicate implies another.
- Renames are not inferred. A removed declaration and an added declaration remain two changes.
- Inputs must describe revisions of the same system. A different system identity is refused.

`--format` accepts `text` or `json`. JSON output now uses **`ess-diff/2`**. The library retains
the frozen `/1` vocabulary and reader support; explicit legacy writing refuses changes outside that
vocabulary. The endpoint digests identify compact compiled models, not raw YAML or the pretty JSON
shown by `ess compile`.
[Delta writer](https://github.com/beyond10x/ess/blob/main/crates/verify/ess-diff/src/delta.rs),
[comparison](https://github.com/beyond10x/ess/blob/main/crates/verify/ess-diff/src/diff.rs).

## Find the work owed again

Without `--suite`, impact reports construct dependencies and generated-artifact obligations:

```sh
ess verify impact --from examples/revision-pair/before --to examples/revision-pair/after
```

To include scenarios, supply a suite for the **before** revision. For example, generate one from
the committed pair and then compare:

```sh
ess verify conform synthesize --path examples/revision-pair/before --out before-suite.json
ess verify impact --from examples/revision-pair/before --to examples/revision-pair/after \
  --suite before-suite.json
```

The report starts with the delta and explains each reached construct, scenario and artifact.
A scenario about a price list can be affected by a currency change through the price list's money
field. The dependency path explains why it is owed even when the scenario does not mention the
currency type directly. Adding `--format json` writes **`ess-impact/3`**, with its embedded `/2`
delta and current dependency-relation vocabulary.
[Impact implementation](https://github.com/beyond10x/ess/blob/main/crates/verify/ess-diff/src/impact.rs),
[dependency graph](https://github.com/beyond10x/ess/blob/main/crates/specify/ess-compiler/src/graph.rs).

The CLI computes artifact obligations from the generators' output for the compared models. It
does **not** read your committed generated directory, and there is no `--generated` CLI option.
Library callers can supply a `GeneratedTree` to `ess_diff::impact` for the separate committed-stamp
check. An unreadable, mismatching or obsolete slice-profile claim then owes regeneration. A command
that runs without that input has not performed that check.
[CLI entry point](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/src/main.rs).

## When narrowing is unavailable

| Situation | Result |
|---|---|
| A system-wide change has no construct from which to start a dependency walk | Whole obligations apply. |
| Residual model content changes outside the typed comparisons | `unclassified-changed` makes whole obligations apply. |
| A scenario or artifact depends on a construct absent from both dependency graphs | The answer stays whole rather than silently omitting that dependency. |
| The supplied suite belongs to another system or does not match the before model/contract digest | The comparison is refused. |

An item absent from the narrowed answer was not reached by this analysis. That is not a claim that
its prior evidence remains valid or that the suite has complete coverage. Suite/4 carries model and
whole-contract identities; it does not carry an exact digest of the complete suite bytes.

Design documents, runbooks and prose references outside the compiled model are not part of the
dependency walk. Their owner must track their freshness separately.

See [Formats and digests](../reference/formats.md) for current version markers, reader boundaries
and the distinct compiled-model, whole-contract and sliced-contract digest profiles.
