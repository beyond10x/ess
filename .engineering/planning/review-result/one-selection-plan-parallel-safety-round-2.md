---
format: aep.planning-md/3
id: review-result:one-selection-plan-parallel-safety-round-2
kind: review-result
status: active
title: 'Plan critic (parallel-safety), round 2: one selection plan'
relations:
- reviews: epic:one-selection-plan
- reviews: story:selection-plan-design-and-type
- reviews: story:interpreter-reads-selection-plan
- reviews: story:entity-runtime-lowering-reads-selection-plan
- reviews: story:generated-behaviour-reads-selection-plan
- reviews: story:synthesis-reads-selection-plan
- reviews: story:overlap-witnesses-per-phase
- reviews: story:validation-reads-selection-plan
revision: 1
---
approve

What I read: 8 artifacts, with `aep plan artifact show` on each of the 8, and `aep plan artifact waves --kind story --status draft --format json` plus round 1's parallel-safety record. I also ran `git grep` for the cross-module call sites, `gh pr view 487 --json state,files`, and `git status` in the tree. Placement: 7 stories cited-placed (each has cited paths, and also some inferred files), 0 inferred-only, 0 unplaceable. The epic is a container.

Why the epic's waves hold:
- **Wave 3** (interpreter, lowering, emitters) lands on three disjoint file sets: `crates/verify/ess-conformance/src/interpret/execute*`, `crates/generate/ess-entity-runtime/src/lib.rs`, and `crates/generate/ess-synth/src/{rust,go}/behaviour.rs`, `determined.rs` and `plan.rs`. The functions the interpreter and emitters delete have no caller outside their own crate's `src/`: `orders_present_related_refusal`, `is_present_related_refusal` and `related::several`.
- **Wave 4 units** (U3, U4, U5) stay disjoint. U3 owns `subject_fact.rs` and U4 owns `related_guard.rs` plus `stored.rs`. Their cross-calls (`guard_truth_with`, `grounded`, `stored::step`) are not `Order`, `selects` or `leaves_external`, so those rewrites leave the callers alone. U5 owns `row_set.rs`, and the story keeps `input_for`'s call site in `synthesize.rs` (U2's file, which lands first).
- **Wave 5** pairs overlap (`ess-conformance`) with validation (`ess-domain`). Neither shares a file with the other.
- **Shared files** between in-set stories that land in different waves are ordered by `depends_on` or by the epic's wave order. Validation shares `command.rs` with design, and overlap shares `synthesize.rs`, `subject_fact.rs`, `related_guard.rs` and `external-beside-held-guard-base.tsv` with synthesis.
- **Round 1's three findings are closed.**
  - The design story now delivers the test seam (`selection-plan-design-and-type.md` "What it delivers" 3), so consumers no longer each add one to the plan file.
  - `story:synthesis-reads-selection-plan` records #487 as a base requirement in its acceptance.
  - The epic now gates every wave on a `main` that contains #487 (`.engineering/planning/epic/one-selection-plan.md`, under the Waves table).

What I could not establish:
- #487 is still OPEN (`gh pr view 487`: `state` OPEN, `mergedAt` null), and `main` at `985f58cc3a` lacks it. The epic's every-wave gate is a precondition nothing in the store enforces, so wave 2 cannot start until #487 merges. That is sequencing, so I did not count it as a finding.
- Overlap's re-pins (`tests/mutation_audit.rs:336,525`, `adversary-244b-base-digests.tsv`, `adversary-285-base-digests.tsv`) are in its decisions and digest-table lines but only partly in its scope. No other in-set story touches them.
- The waves tool places the set differently from the epic: design in 1, the lowering, emitters and validation in 2, synthesis in 3, interpreter and overlap in 4. It reads only the `depends_on` edges and scope overlaps. The collisions it reports for this set are with stories outside it, which I did not assess. Every in-set pair it reports (design/validation, synthesis/overlap) is ordered by `depends_on`.
- Out of my lane, for the acceptance critic: synthesis decision 3 swaps the plan through a scoped thread-local, while its acceptance says to use the design story's seam. Validation decision 5 says `#[cfg(test)]` where the design story's seam is `#[doc(hidden)] pub`.

```findings
[]
```
