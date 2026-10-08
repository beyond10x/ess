---
format: aep.planning-md/3
id: review-result:one-selection-plan-parallel-safety-round-1
kind: review-result
status: active
title: 'Plan critic (parallel-safety), round 1: one selection plan'
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
needs-revision

- story:interpreter-reads-selection-plan — decision 5 says the exchanged-phases test needs a `#[doc(hidden)] pub` constructor or a feature "on the plan's crate", but scope lists neither `crates/specify/ess-compiler/src/ir/precedence.rs` nor `crates/specify/ess-domain/src/command/precedence.rs`, and `story:selection-plan-design-and-type` has no acceptance for such a seam. Wave 3 (interpreter, lowering, generated) and `story:validation-reads-selection-plan` each need a swappable plan (lowering decision 6, generated decision 1, validation decision 5), so up to four concurrent stories would add a seam to the same new plan file. Surface is inferred, from the bodies and the file not existing yet. Either the design story delivers the seam and the consumers name it as a dependency, or each seam stays in its own crate and the body says so — `.engineering/planning/story/interpreter-reads-selection-plan.md:87`
- story:synthesis-reads-selection-plan — scope lists `crates/verify/ess-conformance/tests/external_beside_held_guard.rs`, and PR #487 modifies it (+15/-11); the "Would collide with" line names the other stories but not #487. The body never says the story must wait for #487 or rebase on it. Surface is cited. Either add an ordering note that records #487 as the reason, or take the file out of scope — `.engineering/planning/story/synthesis-reads-selection-plan.md:76`
- epic:one-selection-plan — the only #487 gate is "Validation needs PR #487 … before wave 5", but #487 also touches files owned by earlier waves. Wave 3 lowering's acceptance cites `tests/selection_precedence.rs`, which exists only on #487's branch (`ls` finds it on neither `crates/generate/ess-entity-runtime/tests` nor `crates/verify/ess-conformance/tests` at 985f58cc3a). Wave 4 synthesis scopes `external_beside_held_guard.rs`, which #487 edits. Wave 3 interpreter names its own #487 overlap. Surface is cited, from `gh pr view 487 --json files`. Either move the gate to before wave 3, or have each affected story name its #487 file — `.engineering/planning/epic/one-selection-plan.md:65`

What I read: 8 artifacts, via `aep plan artifact show` on each, the `waves --kind story --status draft` JSON, `gh pr view 487 --json files`, `git grep` and `ls` in the tree. By surface, 7 stories are cited-placed, 0 inferred-only and 0 unplaceable. The epic is a container. Beyond those findings the epic's waves hold:
- **Wave 3:** `interpret/execute*` in ess-conformance, `ess-entity-runtime/src/lib.rs`, and `ess-synth/{rust,go}/behaviour.rs` and `determined.rs` are disjoint file sets.
- **Wave 4 units:** U3, U4 and U5 land on `subject_fact.rs`, `related_guard.rs` and `row_set.rs`, with no shared file.
- **Shared files:** every other in-set pair that shares a file is ordered by `depends_on`: design with validation on `command.rs`, and synthesis with overlap on `synthesize.rs`, `subject_fact.rs`, `related_guard.rs` and `external-beside-held-guard-base.tsv`.
- **Waves tool:** its placement differs only through collisions with stories outside the set, which I did not assess. For example it puts interpreter and overlap in wave 4.

What I could not establish:
- Whether overlap's re-pins (`mutation_audit.rs:336,525`, `adversary-244b-base-digests.tsv`, `adversary-285-base-digests.tsv`) collide with anything. They are in decision 5 and the digest-table lines but not in its scope, and no other in-set story touches them.
- Whether `entry-classifications.json` is touched at all. Grep finds none of the deleted or renamed private functions in it, and the check is parked.
- Out of my lane, for the acceptance critic: lowering's and interpreter's "exchanged phases" acceptances depend on a seam the design story does not promise (acceptance checkability).

```findings
- file: .engineering/planning/story/interpreter-reads-selection-plan.md
  line: 87
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "decision 5 needs a doc(hidden) pub constructor or feature on the plan's crate for the exchanged-phases test, but scope lists no plan-module file and the design story promises no such seam; wave-3 interpreter, lowering and generated (and validation) would each add a swap seam to the same new plan file (inferred: the file does not exist yet); either the design story delivers the seam and consumers depend on it, or each seam stays in its own crate and the body says so"
- file: .engineering/planning/story/synthesis-reads-selection-plan.md
  line: 76
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "scope lists crates/verify/ess-conformance/tests/external_beside_held_guard.rs, which open PR #487 modifies (+15/-11), and the Would-collide line does not name #487 nor require it merged first (cited, gh pr view 487 --json files); either record #487 as an ordering reason or remove the file from scope"
- file: .engineering/planning/epic/one-selection-plan.md
  line: 65
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the only #487 gate is before wave 5, but #487 also edits external_beside_held_guard.rs (wave 4 synthesis scope) and adds tests/selection_precedence.rs that wave-3 lowering's acceptance cites and that is absent on main (cited); either move the gate before wave 3 or have each affected story name its #487 file"
```
