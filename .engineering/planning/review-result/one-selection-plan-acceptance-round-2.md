---
format: aep.planning-md/3
id: review-result:one-selection-plan-acceptance-round-2
kind: review-result
status: active
title: 'Plan critic (acceptance), round 2: one selection plan'
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
story:selection-plan-design-and-type — the last acceptance, "`EssIr::to_canonical_json` output for every repository model is unchanged (`cargo test -p ess-compiler` green, no fixture … rewritten)", names a check that cannot see every model, because `ess-compiler`'s tests pin canonical JSON only for `billing` and small fixtures. The acceptance must name an all-models check, such as the table test also pinning each model's canonical-JSON digest or the E-U6 probe's compiled-IR hash column — .engineering/planning/story/selection-plan-design-and-type.md:71 (`crates/generate/ess-synth/tests/adversary_e_u6_bytes.rs:3-8` hashes compiled IR per model)
story:synthesis-reads-selection-plan — the fourth acceptance, "The story's base contains PR #487, which edits `tests/external_beside_held_guard.rs`", is a precondition the work cannot change, so it reads the same before and after. It belongs in Scope, not Acceptance — .engineering/planning/story/synthesis-reads-selection-plan.md:60
story:entity-runtime-lowering-reads-selection-plan — "No other ordering rule remains in `lower_command`" does not say whether slot numbering by declaration index counts, though design decision 4 keeps it and `lib.rs:1794-1808` sits in the same function. The acceptance must say "no other branch-selection-order rule" so a reviewer gets the same answer twice — .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md:49
story:overlap-witnesses-per-phase — "What it delivers" promises an overlap send "for each pair of branches the plan places in one phase", but the acceptances observe only the stateless refusal pair, a held-state `when_subject` pair and a stored-row pair. No acceptance observes a related-row or row-set pair, though the Scope counts 12 commands with two or more `when_related` refusals. Either add a related fixture acceptance or narrow "What it delivers" — .engineering/planning/story/overlap-witnesses-per-phase.md:46 (acceptances at :63-66; Scope `:76`)

What I read: 8 of 8 ids (`epic:one-selection-plan` and the seven stories) through `aep plan artifact show <id>` in the named tree, plus `aep plan artifact kinds` and `lifecycle epic|story`, and round 1's `review-result`. I checked PR #487's `selection_precedence.rs` and `held_state_order.rs` with `git show fix/selection-precedence:<path>`, and checked named tests, probes, CI shards and the `refusal_pair_overlap.rs` tests with `git grep`, `grep` and `sed`.

Round 1 findings that landed (checked):
- The interpreter exchanged-phases fixture is now satisfiable. #487's `declared_first_the_held_state_branch_answers_before_the_accepting_one` shows `stale` for revision 2 with rush true, and exchanging the phases yields `rushed`.
- The synthesis story now names the full-bytes pin.
- The #472 case is now a conditional regression test, and `refusal_pair_overlap.rs` has no `bad-quantity` test today.
- The lowering story cites #487's test.
- The emitters story names the deleted symbols.
- The "classification readable from `ess-domain`" acceptance exists.

What I could not establish:
- Whether the #472 mutant is killed on the base. I ran nothing.
- Whether story 4's `git grep` for `orders_present_related_refusal` in `ess-synth` also hits the local variable at `plan.rs:724`. The story treats that file as inferred, so that bullet may force a rename there.
- Out of my lane (design): story 1 says the seam is reachable "from `ess-domain`'s own validation", but its plan constructor lives in `ess-compiler`. Story 7 decision 5 adds its own injection point, though story 1 says consumers add no seam of their own. Story 3 decision 6 says "crate-private seam" against story 1's public `#[doc(hidden)]` seam.
- "None matches on condition kinds to decide order" in stories 2, 5 and 7 stays partly a judgement call, as in round 1. I did not raise it.

```findings
- file: .engineering/planning/story/selection-plan-design-and-type.md
  line: 71
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'EssIr::to_canonical_json output for every repository model is unchanged (cargo test -p ess-compiler green, no fixture rewritten)' names a check that cannot see every model, since ess-compiler's tests pin canonical JSON only for billing and small fixtures; it must name an all-models check such as a per-model canonical-JSON digest in the table test or the E-U6 probe's compiled-IR hash column"
- file: .engineering/planning/story/synthesis-reads-selection-plan.md
  line: 60
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'The story's base contains PR #487, which edits tests/external_beside_held_guard.rs' is a precondition the work cannot change, so it reads the same before and after; it belongs in Scope, not Acceptance"
- file: .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md
  line: 49
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "'No other ordering rule remains in lower_command' does not say whether slot numbering by declaration index counts, though design decision 4 keeps it in the same function; the acceptance must say 'no other branch-selection-order rule'"
- file: .engineering/planning/story/overlap-witnesses-per-phase.md
  line: 46
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "What it delivers promises an overlap send for every same-phase pair, but no acceptance observes a related-row or row-set pair (the Scope counts 12 commands with two or more when_related refusals); add a related fixture acceptance or narrow the delivery"
```
