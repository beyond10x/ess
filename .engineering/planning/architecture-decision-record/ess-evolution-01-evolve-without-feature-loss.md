---
format: aep.planning-md/2
id: architecture-decision-record:ess-evolution-01-evolve-without-feature-loss
kind: architecture-decision-record
status: proposed
title: 01 — Evolve ESS without feature loss
relations:
- decides: initiative:ess-evolution
revision: 2
---
## Context
Existing consumers depend on package identities, formats and tested behavior.

## Decision
Preserve ESS, its repository, package identities and all implemented contracts. Migrate incrementally.

## Rejected alternatives
Replacement repository, renaming ESS and dropping features to simplify lowering.

## Compatibility and migration
Retain every public format, fixture, generated snapshot and CLI behavior. The preservation mapping names destinations; full existing gates and independent compatibility prove each move.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.


## Amendment 2026-09-25: the accounting lane is parked (ESS-EVOLUTION.md revision 3)

The principle stands: ESS evolves without silently dropping a feature an adopter relies on. The mechanism
that was to prove it per model × consumer-profile cell (`task consumer-check`, a frozen baseline of 157,677
pairs) leaves `task check` and the release bar; it runs only with `CONSUMER_CHECKS=true`.

Why: its consumers are 87 in-repository profiles, it never executed more than 22 cases, it caught only drift
in its own ledgers, and at ESS 0.31.0 it needed about 7,680 per-cell judgement decisions that could only have
been rubber stamps. Preservation is shown by the conformance suites, the retained legacy readers,
`ess verify diff`/`impact` and `ess/N` format discipline, and adopters are protected at upgrade time (stories
`specify-upgrade-command`, `change-fragment-upgrade-obligation`, `adopter-reviewed-delta`).

Re-enable when a named external adopter pins a released `ess/N` and generated artifacts, and a change reaches
that adopter undetected by conformance, `ess verify diff` or a format refusal. The code, the baseline, the 490
classifications and the 47 rebound cases (ess#82) stay in the tree. `decision-blocker:consumer-accounting-parked`
records the stop on `story:consumer-accounting-baseline-never-extended`.
