---
format: aep.planning-md/3
id: story:the-caller-swapped-run-is-appended
kind: story
status: draft
title: The caller-swapped run of a caller-written set effect is appended, and a test fails without it
relations:
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Outcome

The caller-swapped run of a set effect over a caller-written field is appended to the scenario it
belongs to, and a test fails when it is not.

## Evidence

The 2026-10-19 adversarial review of the integration branch (finding T6): with the swapped append in
`crates/verify/ess-conformance/src/synthesize/caller.rs` (the `adopted` loop, `append(ir, scenario,
again, identities)`) removed, the suite synthesized from the tokens fixture of
`tests/caller_arranged_rows.rs` is byte-identical. Either the append never runs for that model, or
its result is discarded. Not yet traced: which.

## Acceptance

- One test in `tests/caller_arranged_rows.rs` goes red when the swapped append is removed.
- If the append cannot run for any model, it is removed and the module docs say so.
