---
format: aep.planning-md/3
id: epic:retrofit-findings-20260927
kind: epic
status: draft
title: 'Retrofit findings of 2026-09-27: what ess/13 could not state'
revision: 2
---
## Problem

25 issues filed on 2026-09-27 (beyond10x/ess#132–#157) from retrofits of existing services onto
0.35.0 (`ess/13`) name behaviour the implementations have and the specification language cannot
state, or synthesis that misses a mutant.

## Outcome

Every issue is implemented, answered by an existing construct, or recorded here as deferred with
its reason. Order and grouping: `ESS-ISSUES-PLAN-2026-09-27.md` in the workspace root.

## Done

- Value expressions E1–E5 (#133, #134, #135, #136, #137), source format `ess/14`, release 0.36.0,
  PR #159, design `docs/design/value-expressions.md`.

## Answered in part by existing constructs

- #144: `preserves:` (ess/6) under `when:` is an accepted no-op on an existing subject.
- #145: 0.35.1 (#158) takes a declared `external:` not-found refusal for an unknown instance.

## Remaining, one story each

- `story:subject-guard-input-and-case-folding` — #157, #140
- `story:synthesis-kills-connective-and-source-mutants` — #154, #155, #132
- `story:outcome-shapes-beyond-ess-14` — #145, #150, #151, #144, #152
- `story:field-names-wire-and-value-types` — #141, #143, #142, #139, #138, #146
- `story:aggregates-over-optional-fields` — #148
- `story:external-mutation-explorer-and-toolchain` — #153, #156, #147
