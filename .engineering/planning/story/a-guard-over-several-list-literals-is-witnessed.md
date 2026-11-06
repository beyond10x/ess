---
format: aep.planning-md/3
id: story:a-guard-over-several-list-literals-is-witnessed
kind: story
status: draft
title: Synthesis witnesses a guard that needs several literals in one identity list
summary: 'all: [exists a, exists b] refused ESS-SYNTH-003, per-child steps send unbacked literals (0.90.0 U1 adversary; from-finding)'
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 1
---
## Need

Found by the U1 adversary of wave-2026-11-07-ess (ESS 0.90.0). Pre-existing, beside issue 761.

With a guard `all: [exists p in <list>: (p == telephony), exists p in <list>: (p == voice)]`:

- the guarded branch is refused ESS-SYNTH-003;
- the emitted `updated` and `grant/denied` scenarios fail, because their per-child steps send
  `["voice"]` or `["telephony"]` without arranging a row that backs it.

The red case is `adversary_761_two_literal_guard_is_witnessed` in
`crates/verify/ess-conformance/tests/adversary_761_list_identity_guard.rs`. It is ignored with
this story's id until fixed.

## Acceptance

- That case passes. The guarded branch's scenario sends a list holding every literal, each
  arranged as a row.
- No emitted scenario sends a literal that no row backs.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize/related_list.rs`,
  `related_guard.rs`, the per-child step planner in `synthesize.rs`.
