---
format: aep.planning-md/3
id: story:feature-request-317
kind: story
status: active
title: Synthesize recreation of an identity after deletion
refs:
- provider: github
  reference: beyond10x/ess#317
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/upsert_by_existence.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:12:16Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:12:17Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Generated conformance distinguishes an identity removed by deletes from an existing record, by creating, deleting and creating the same identity again wherever the declared deletion path can be arranged.

## Fit review

1. Need: removed identities must be available to a creating branch, not found by a stale existence lookup. Issue #317 is an adversary finding against #310.
2. Class: conformance coverage gap under an existing semantic obligation. docs/design/outcome-shapes.md:79-81 says deleted identities resolve as unknown. synthesize/existence.rs:918-984 checks duplicate creation but never deletion.
3. Existing expression: creation/existing_instance branches and a separate delete command already express this; authored scenarios can cover it. Generated coverage is the missing promise, not new syntax.
4. Fit: use existing commands, captured identities, outcomes and absence observations. deletion_witness (synthesize.rs:8577-8644) currently repeats only deletion. Prove a reachable delete route and its guards/transitions; never claim a witness when no declared path is arrangeable. Preserve other target obligations and all existing suite vocabulary.
5. Second adopter: revoke then register a device under the same externally supplied identifier. This follows the same unknown-after-deletion contract.
6. Cost: new executable obligations and changed generated suite content, no new persisted step or format expected. tests/upsert_by_existence.rs:562 already holds the family to existing vocabulary.
7. Alternatives: change nothing misses stale tombstone lookups; requiring each consumer to author it repeats the same omission; synthesize bounded create-delete-create wherever reachability is proved.

## Decisions

Accept as proposed. Reuse the existing existence and deletion machinery. Implement after #342 within the same managed synthesis batch because it must not inherit the false post-deletion state. This order is not a claim that #342 alone satisfies this story.

## Acceptance

- issue_317_deleted_identity_is_created_again: honest Rust target passes create/delete/recreate using exactly the same identity.
- issue_317_lookup_of_removed_rows_fails_recreation: mutant removes visible data but retains the existence lookup entry; the new scenario must fail on the expected creating outcome.
- Existing duplicate-create refusal remains checked. Cover multiple creation branches and singleton identity where existing grammar permits; no declared deletion path must not produce an invented sequence.
- Nontrivial reachable deletion paths retain their guard/state setup. Unsupported paths receive honest existing refusals, never a fabricated successful witness.

## Scope

Cited: crates/verify/ess-conformance/src/synthesize/existence.rs; crates/verify/ess-conformance/tests/upsert_by_existence.rs; docs/design/outcome-shapes.md.
Cited read seam, possible edit inferred: crates/verify/ess-conformance/src/synthesize.rs; tests/outcome_shapes.rs.
Confidence: medium; execute red before implementation and report any missing design capability.

## Implementation refinement

The existing deletion witness chooses one creator; merely appending a retry there would miss the accepted multiple-creation-branch requirement. Reuse subject_fact's bounded search_from with a specific creator and a declared reachable deletion goal, through a small wrapper in subject_fact.rs. This preserves stored guards and lifecycle routing without adding another graph search. Existence-family scenarios then append recreation using the same captured identity. Search failures remain explicit refusals rather than invented sequences. This scope refinement was reviewed before implementation; #342's current diff remains frozen until independent review ends.
