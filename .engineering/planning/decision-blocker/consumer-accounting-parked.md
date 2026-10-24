---
format: aep.planning-md/3
id: decision-blocker:consumer-accounting-parked
kind: decision-blocker
status: cleared
title: Consumer accounting is parked until an external adopter needs it
relations:
- blocks: story:consumer-accounting-baseline-never-extended
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-23T13:57:35Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
# Consumer accounting is parked until an external adopter needs it

Parked by ESS-EVOLUTION.md revision 3 (2026-09-25) and the amendment to
`architecture-decision-record:ess-evolution-01-evolve-without-feature-loss`.

Clears when a named external adopter pins a released `ess/N` and generated artifacts, and a change reaches
that adopter undetected by conformance, `ess verify diff` or a format refusal. At 0.31.0 the lane stopped
past classification on one held reviewed case, one stale requirement model, and a reconciliation authority
of about 728 mechanical and 7,680 judgement decisions that main has never had.

## Answer (2026-10-23)

Decided by the operator, relayed by the orchestrator: "consumer-accounting-parked: not built; reopen only when an external adopter asks."

Consumer accounting stays parked as revision 3 of the evolution plan left it. `consumer-check` stays opt-in and outside every CI lane and the release bar. A request from a named external adopter reopens it as a new blocker.
