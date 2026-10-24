---
format: aep.planning-md/3
id: story:rows-a-guarded-move-reaches-are-made-once
kind: story
status: draft
title: Synthesis makes the rows a guarded move reaches once
summary: 0.69.0 synthesized a downstream specification for over 13 minutes where 0.68.0 took 56 s; the creating branches' rows are now made once per model and context.
tags:
- adopter-report
- ess-0.70.0
- regression
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- informed_by: story:seeded-state-reaches-related-rows
revision: 1
---
## Need

On 0.69.0 `ess verify conform synthesize` over the downstream specification of beyond10x/patchback (9 domains, 125 authored scenarios) ran for 13 minutes at 99 % CPU without output before it was killed; 0.68.0 finishes it in 56 s (patchback wave coordinator, measured 2026-10-24). Synthesis must finish within twice 0.68.0's time and keep what 0.69.0 added.

## Reproduction

- `git bisect` between 0.68.0 and 0.69.0 on the specification's first seven domains with one authored scenario, `timeout 150`, `ESS_TOOLCHAIN_DELEGATED=1` so the built binary runs instead of the pinned release: the first slow commit is 8fe9e8798 (`fix(conformance): seeded state reaches related rows`, story:seeded-state-reaches-related-rows).
- Runtime samples (gdb, release build) put the stall in `SelectCandidate/outranked`: the row-set search arranges the sibling candidate through `arrange_toward_with`, which calls `created_bound` once per creator input, and each call arranges a fresh run row (628 or more run searches in one outcome, about 20 nested searches each).
- 8fe9e8798 lets each creating branch also leave rows taken toward the guards of the moves on the way, so every level of that nesting creates several rows. Counted on the seven domains with an instrumented build: run rows were asked for under 29 distinct keys and 1009 repeats, topology rows under 83 and 13020; with the memo, 6283 requests are answered from 1311 computations.

## Fix

The rows a creating branch leaves (`subject_fact::creations`) are made once per model and context and answered from memory after that (`synthesize/made.rs`), keyed on the arguments and every thread-local the computation reads (held rows, earlier captures, the branch under test, the open beside searches, the dropped write). A search answered from memory counts the beside searches it cut again, so the search decides as before.

## Acceptance

- `tests/related_guard_ranked_candidates_work.rs`: on fixture `related-guard-ranked-candidates.yaml` the creating branches' rows are made at most 120 times (`creations_made`); 54 with the memo, 574 without it.
- The pinned suite bytes of `synthesis_suite_bytes_table` and the culprit's own tests stay unchanged.
- The downstream specification synthesizes in 50 s (0.68.0 on the same host: 50 s), with 464 scenarios and 27 refusals against 0.68.0's 397 and 92.
