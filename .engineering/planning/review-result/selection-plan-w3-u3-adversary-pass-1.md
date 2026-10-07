---
format: aep.planning-md/3
id: review-result:selection-plan-w3-u3-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 3 unit U3 (validation reads the plan)
relations:
- reviews: story:validation-reads-selection-plan
revision: 1
---
unit: story:validation-reads-selection-plan, the uncommitted working tree on 1046621a8 (branch unit/selection-plan-w3-u3-validation)
verdict: INFEASIBLE (2 expected-red cases written but **not run**: the disk was below the 10G build floor the whole time)
cases: executed 1445→not run, red 0 (1445 is the passed count in the implementor's `w3-u3-scratch/green-domain.log`)
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 1 path (a harness task log; see part 6)
needs-coordinator: yes. `/` has 2.2G free and is at 100% (it fell from 7.9G to 2.2G between 17:3x and 18:00 while other units built). I started no build. The 3 cases need one run of `CARGO_INCREMENTAL=0 cargo test -p ess-domain --locked --test adversary_validation_selection_plan_pass1` once at least 10G is free.

I applied your correction: R2 is `wrong_state:` beside a present-related refusal and a present-related acceptance.

**1. Diff stat**
`git --no-pager diff --stat` shows the unit's 3 implementation files (`command.rs` 153, `related_guard.rs` 71, `subject_state.rs` 58). That is the same stat as before my pass; none of it is mine. My only path is a new untracked test file: `crates/specify/ess-domain/tests/adversary_validation_selection_plan_pass1.rs`. I changed no implementation file.

**2. Cases added (none run, so there is no red output)**

| case | asserts | expected at unit |
|---|---|---|
| `an_input_refusal_still_answers_an_accepting_branch_it_is_still_read_before` | RotateSecret variant: input refusal `frozen` overlaps only the accepting `kept`. It validates at default and under `exchanged(InputRefusal, Existence)`. It must still validate under `exchanged(InputRefusal, HeldState)`, where InputRefusal stays before Accepting | red: "select 2 branches: frozen, kept" |
| `an_input_refusal_beside_a_present_related_refusal_is_not_decided_by_the_accepting_phase` | PublishRelease at ess/22 with `wrong_state:`: input refusal `blocked` and related refusal `not-accepted` are selected together, and no accepting branch is selected with them. It validates at default and under `exchanged(PresentRelated, HeldState)`. It must still validate under `exchanged(PresentRelated, Accepting)`, where InputRefusal stays before PresentRelated | red: "select 2 branches: not-accepted, blocked" |
| `r2_wrong_state_beside_a_related_acceptance_stays_refused` | R2 pin: refused with the full "which of the two answers first is not stated" text and hint, at default and under 3 overrides | green |

**3. Suite run:** not run, for the disk reason above.

**4. Findings (cover the working tree above)**

| # | file:line | finding | what reaches it | verdict / origin |
|---|---|---|---|---|
| F1 | `subject_state.rs:137` (used at :303, :369, :389) | `input_refusals_answer_first` is one global yes/no: the refusal-first tie-break applies only if InputRefusal comes before both HeldState and Accepting. Move HeldState alone and the tie-break is dropped even for refusal/accepting pairs whose relative order did not change. That contradicts the unit's claim that answering order comes from `phase_order`. Fix: decide per selected branch with `place(..).phase.position()` | only `with_phase_order` (a `#[doc(hidden)]` test seam); the default order is unaffected | INFEASIBLE / introduced |
| F2 | `related_guard.rs:2113` (used at :2136, :2161) | `present_related_refusals_first` switches off the whole count in `selected_count` and `several_selected`. So a pair with no accepting branch in it (input refusal + related refusal) becomes a conflict when PresentRelated and Accepting are exchanged. Same fix: compare per pair | only the test seam | INFEASIBLE / introduced |
| F3 | `subject_state.rs:326`, `related_guard.rs:1530`, `:1885` | The membership filters still match on `OutcomeCondition`, and the comments at `related_guard.rs:1536` and `:1538` justify the exclusions by answer order. Acceptance bullet 2 and scoper decision 7 say these should be routed through the phase | these lines are unchanged from base (`git show 1046621a8:` shows them at :1533/:1535) | CONFIRMED / pre-existing |

"Introduced" for F1 and F2 comes from reading the diff, not from a base run. Both gates are lines the diff adds, and at base neither function reads the phase order at all.

**5. Attacked and could not break (by reading the code, not by running it)**
- **#486 check:** `OnePass::of` classifies the same branches as the old rule (`reads_held_state() || reads_subject_fact()` against When-without-error, External, ExternalWhen). The accepting/external word and the message and hint text are byte-identical, and composition has no effect on any branch it classifies.
- **`is_input_refusal`:** matches the old rule (`is_unconditional` on a `When` is `is_trivially_true`, `is_refusal` is `error.is_some()`), and its answer does not depend on composition.
- **`selected_count`:** whenever validation's `orders_present_refusals` is true (a stored `via`, or `wrong_state:` at ess/22 with every related branch refusing), the composition is also ordered. So the count is unchanged at default.
- **`several_selected`:** it only runs at ess/22 or later with at least 2 input rows (the gates at :795 refuse below that). There the composition is ordered whenever a refusal is selected.
- **R2 and `orders_present_refusals` (:866):** the diff does not touch them.
- **Changed public signatures:** nothing in `~/beyond10x` outside ess calls `is_input_refusal` or `subject_state::validate_shape`.
- **Override reach:** validation has no thread hop or cache, so the thread-local override reaches it.
- **`row_set.rs:401`:** its refusal-first rule is unchanged, as the brief says.

**6. Paths written outside the worktree**
- `~/.cache/claude-tmp/claude-1000/-home-timo-beyond10x/649551ea-2c22-4172-bfe0-b0d47cfc46be/tasks/biaonyg2q.output`, the log of the background disk wait. It may still be running and ends on its own by about 18:05.

My worktree lease is released.

**7.**
```findings
- file: crates/specify/ess-domain/src/command/subject_state.rs
  line: 137
  category: property
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: input_refusals_answer_first is one global comparison, so exchanging InputRefusal with HeldState turns an input-refusal/accepting overlap into a conflict although InputRefusal still precedes Accepting (case written, not run; reachable only through the with_phase_order test seam)
- file: crates/specify/ess-domain/src/command/related_guard.rs
  line: 2113
  category: property
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: present_related_refusals_first switches off the whole count, so exchanging PresentRelated with Accepting turns an input-refusal/related-refusal overlap with no accepting branch into a conflict (case written, not run; reachable only through the with_phase_order test seam)
- file: crates/specify/ess-domain/src/command/related_guard.rs
  line: 1530
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: the partition membership filters (here, related_guard.rs:1885 and subject_state.rs:326) still match OutcomeCondition with answer-order rationale, against acceptance bullet 2 and scoper decision 7
```
