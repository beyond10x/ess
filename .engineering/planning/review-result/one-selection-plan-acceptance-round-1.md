---
format: aep.planning-md/3
id: review-result:one-selection-plan-acceptance-round-1
kind: review-result
status: active
title: 'Plan critic (acceptance), round 1: one selection plan'
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
story:synthesis-reads-selection-plan — the first acceptance names `repository_model_suites_change_only_where_claimed` as the byte-identity check, but the story's own Scope shows that test strips every claimable scenario and so cannot see what the story changes; the acceptance must name the full-bytes pin (unit U1) as the check — .engineering/planning/story/synthesis-reads-selection-plan.md:50 (gap stated at :74)
story:overlap-witnesses-per-phase — the first acceptance (the #472 reproduction sends `quantity: 0, discount: -1` and the mutant is killed) probably already holds on the base through #455, so it reads the same before the work as after; it must name what is false on the base, or the story must drop it for a regression test — .engineering/planning/story/overlap-witnesses-per-phase.md:53 (the Scope says #472 is probably already fixed, and `:47` says the same)
story:overlap-witnesses-per-phase — no acceptance observes the held-state, stored-row or related overlap paths (gaps 1-3) or `mutate.rs` `precedence_sites`, which "What it delivers" promises, and the second acceptance is vacuous because the Scope puts models gaining scenarios at 0 — .engineering/planning/story/overlap-witnesses-per-phase.md:56 (Scope `:76`)
story:interpreter-reads-selection-plan — the exchanged-phases test is pinned to `external-beside-held-guard.yaml`, but the story could not establish that exchanging two phases changes that fixture's answer (`stale` and `unlisted` share one loop today), so the check may be unsatisfiable; the acceptance must name the phases exchanged and a fixture whose answer is known to change — .engineering/planning/story/interpreter-reads-selection-plan.md:41 (`:90`)
story:entity-runtime-lowering-reads-selection-plan — "No ordering rule remains in the lowering that the plan does not supply" contradicts design decision 1, which lets a target rule (default last) stay in the lowering, so a conforming implementation could fail it; the acceptance must say which rules the plan supplies — .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md:45 (decision 1 at `:70`)
story:entity-runtime-lowering-reads-selection-plan — the exchanged-phases acceptance points to `tests/selection_precedence.rs` as its model, but no such file exists on the base (`git ls-files | grep selection_precedence` is empty); it must cite a test that exists, or say the file arrives with PR #487 — .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md:43
story:generated-behaviour-reads-selection-plan — "`determined.rs` no longer carries its own copy of which branches answer first" names no symbol, while decision 4 keeps `subject_guarded` and `collision_answer`; the acceptance must name what is deleted (`orders_present_related_refusal`, `is_present_related_refusal`) so a grep can check it — .engineering/planning/story/generated-behaviour-reads-selection-plan.md:49
story:selection-plan-design-and-type — no acceptance observes that the phase classification is readable from `ess-domain`, the first design decision that story:validation-reads-selection-plan depends on, or the type's name; the acceptance names only the design doc, the `ResolvedCondition` match, the table and the unchanged IR — .engineering/planning/story/selection-plan-design-and-type.md:53 (deliverable at `:42`)
epic:one-selection-plan — the byte-identity constraint says lowered definitions change only "where a story names an addition (#472)", but story:entity-runtime-lowering-reads-selection-plan and story:generated-behaviour-reads-selection-plan each name a different permitted byte change (a held-state branch after a disjoint accepting branch), so the epic's bar fails stories that satisfy their own acceptance — .engineering/planning/epic/one-selection-plan.md:48

What I read: 8 of 8 given ids, through `aep plan artifact show <id>` for each, plus `aep plan artifact list --format json`, `kinds` and `lifecycle story`. I checked the named tests, fixtures and tasks with `git grep`, `ls` and `git ls-files` in the tree.

Could not establish:
- `tests/held_state_order.rs`, which the validation story's acceptance names, is not in the tree because PR #487 is unmerged. The epic and the story's decision 1 already gate that story on #487, so I raised no finding.
- Whether any acceptance is met on the base, beyond what the stories' own Scope sections state. I ran no tests.
- "None matches on condition kinds to decide order" appears in the interpreter, synthesis and validation acceptances. It is checkable only by reading each site and is partly judgement. I did not raise it.
- Out of my lane: the epic has no Acceptance section, but that is the convention of the other epics in the store (pre-existing, so not raised). Overlap on `synthesize.rs` between the synthesis and overlap stories, and `mutate.rs` ownership, belong to parallel-safety and scope.

```findings
- file: .engineering/planning/story/synthesis-reads-selection-plan.md
  line: 50
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the first acceptance names repository_model_suites_change_only_where_claimed as the byte-identity check, but the story's own Scope shows that test strips every claimable scenario and so cannot see what the story changes; the acceptance must name the full-bytes pin (unit U1) as the check"
- file: .engineering/planning/story/overlap-witnesses-per-phase.md
  line: 53
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the first acceptance (the #472 reproduction sends quantity: 0, discount: -1 and the mutant is killed) probably already holds on the base through #455, so it reads the same before the work as after; it must name what is false on the base, or the story must drop it for a regression test"
- file: .engineering/planning/story/overlap-witnesses-per-phase.md
  line: 56
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "no acceptance observes the held-state, stored-row or related overlap paths (gaps 1-3) or mutate.rs precedence_sites, which What it delivers promises, and the second acceptance is vacuous because the Scope puts models gaining scenarios at 0"
- file: .engineering/planning/story/interpreter-reads-selection-plan.md
  line: 41
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the exchanged-phases test is pinned to external-beside-held-guard.yaml, but the story could not establish that exchanging two phases changes that fixture's answer (stale and unlisted share one loop today), so the check may be unsatisfiable; the acceptance must name the phases exchanged and a fixture whose answer is known to change"
- file: .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md
  line: 45
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "No ordering rule remains in the lowering that the plan does not supply contradicts design decision 1, which lets a target rule (default last) stay in the lowering, so a conforming implementation could fail it; the acceptance must say which rules the plan supplies"
- file: .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md
  line: 43
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the exchanged-phases acceptance points to tests/selection_precedence.rs as its model, but no such file exists on the base (git ls-files | grep selection_precedence is empty); it must cite a test that exists, or say the file arrives with PR #487"
- file: .engineering/planning/story/generated-behaviour-reads-selection-plan.md
  line: 49
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "determined.rs no longer carries its own copy of which branches answer first names no symbol, while decision 4 keeps subject_guarded and collision_answer; the acceptance must name what is deleted (orders_present_related_refusal, is_present_related_refusal) so a grep can check it"
- file: .engineering/planning/story/selection-plan-design-and-type.md
  line: 53
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "no acceptance observes that the phase classification is readable from ess-domain, the first design decision that story:validation-reads-selection-plan depends on, or the type's name; the acceptance names only the design doc, the ResolvedCondition match, the table and the unchanged IR"
- file: .engineering/planning/epic/one-selection-plan.md
  line: 48
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the byte-identity constraint says lowered definitions change only where a story names an addition (#472), but the lowering and emitter stories each name a different permitted byte change (a held-state branch after a disjoint accepting branch), so the epic's bar fails stories that satisfy their own acceptance"
```
