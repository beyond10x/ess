---
title: "0.42 — unkillable mutants scored apart, overlap precedence, and counter limits"
description: >
  0.42.0 scores a mutant no scenario could kill apart from a survivor, gives two overlapping
  accepting guards a declared answer, witnesses a stored counter at its limit and a guard over a
  link field, and lets a relation ride on the identity.
slug: unkillable-mutants
tags: [release, ess]
date: 2026-09-29T20:00:00+02:00
release_tag: "0.42.0"
release_commit: 50cb2b3db48caab265daa8bfec6149e930b7a26e
---

0.42.0 adds `ess-diff/11`, `ess-mutation-report/3` and `ess-mutation-manifest/3`, described in
[the version history](/ess/docs/reference/spec-versions).

{/* truncate */}

## Mutants no scenario could kill

A guard mutant that leaves its outcome's guard satisfied by no input is `equivalent`
(`ESS-MUTATE-005`) and names the guard it left dead. A mutant on an outcome whose scenario the
baseline already refused at synthesis is `unwitnessed` and names that refusal. Neither is scored
`survived` any more.

## Overlapping guards

Of two accepting guarded branches that both hold, the first declared answers; input-guarded
refusals are still taken first, and an `external:` branch takes its place in the same order. The
interpreter answers this way, and synthesis sends each overlap and requires the first branch.

## Synthesis

Synthesis witnesses a guard comparing a link field with an input, and a branch selected by a stored
counter at its limit, each side of it, so an off-by-one limit fails. A suite from a model whose
actors declare `attributes:` records that model's digests.

## Relations and deltas

`via:` may name the entity's own identity, a one-to-one link keyed by the same id. `ess verify
diff` names a newtype's `prefix:` added, removed or changed.
