---
format: aep.planning-md/3
id: story:the-first-of-overlapping-related-refusals-is-synthesized
kind: story
status: draft
title: The first of several overlapping related refusals over one row is synthesized
summary: Synthesis refuses reachable related refusals that share a guard with a later one (ESS-SYNTH-003), as in 0.68.0 (found by the U14 adversary, 2026-10-23).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Found while landing `story:an-authored-step-expecting-a-shadowed-related-refusal-is-refused`
(0.69.0, adversary pass 1, finding F3). A command declares refusals over one related row whose guards
overlap, for example `staff-closed` and `locker-closed` both reading `accepting == false` through
the same `via:`. The interpreter answers the first declared one, as the precedence design says
(`docs/design/cross-record-and-stored-field-guards.md:828-830`, `:868`). Synthesis refuses the
reachable first refusal as well as the shadowed one, with ESS-SYNTH-003 "no candidate of the 4
tried satisfies `parcels.locker.Locker` stored related row selecting this branch, over the rows 4
bounded arrangements left". `ess 0.68.0` refuses the same 4 outcomes, so this predates 0.69.0.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs`: the accepting-phase related
  refusals have no order among themselves, so the goal for the first one also asks its row to miss
  the later one's guard.
- Pinned: `tests/fixtures/synthesis-suite-bytes-table.tsv`, row `related-refusal-shadowed.yaml`
  (`refusals=4`).
- Failing case: `adversary_u14_pass1::the_shadowed_reproduction_synthesizes_its_reachable_refusals`
  (ignored, naming this story).

## Acceptance

- The reproduction synthesizes a scenario for every reachable refusal (`staff-closed`,
  `locker-closed`) that passes on the interpreted target, and refuses only the shadowed ones, naming
  the branch that answers first.
- The ignored adversary case runs and passes.
- No existing pinned suite digest moves except the reproduction's own row.

## Format

None.
