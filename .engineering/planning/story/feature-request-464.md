---
format: aep.planning-md/3
id: story:feature-request-464
kind: story
status: implemented
title: 'synthesize: external outcome witness triggers an earlier when_subject guard (Loom RevalidateSelection)'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#464
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-463
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/external_beside_held_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/guarded_external.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T01:58:04Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-06T01:58:05Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-06T17:49:27Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":8}}}
---
## Outcome
Resolve beyond10x/ess#464: synthesize: an external outcome's witness triggers an earlier when_subject guard.

## Origin
beyond10x/ess#464, filed 2026-10-05; an adopter's specification on ess 0.53.0 (`ess/20`), whose unmodified suite fails two scenarios against ESS's own interpreter, so `mutate` refuses its baseline (ESS-MUTATE-001). Reproduced minimally and fresh in `<fit-review scratch>/probe-464/`.

## Fit review
1. Need: an external branch's scenario must send an input and arrange a row that no earlier-answering guard claims. Minimal reproduction `probe-464/b/` (`ess/20`, installed ess 0.53.0):
   - `Pick {revision}`. `CheckPick {pick_id, revision}` declares `stale` (`when_subject: revision != input.revision`, moves `refuse`), then `unlisted` (`external:`, moves `refuse`), then `accepted` and `wrong-state`.
   - Synthesis writes 10 scenarios with 0 refusals. `CheckPick/outcome/unlisted` arranges `revision: 1.0`, forces `unlisted` and sends `revision: 2.0`.
   - The interpreter answers `outcome = stale` (`b.run.json`): 8 passed, 2 failed (`unlisted` and `Pick/transition/refuse/by/CheckPick/unlisted`), and `mutate` exits 3 with ESS-MUTATE-001 (`b.mutate.out`). Same shape as the issue.
   - `accepted`, which is not external, is sent `revision: 1.0` and passes. Only the external path ignores the guard.
   - The requester asks (theirs): satisfy or neutralize the preceding guards when witnessing an external outcome, or refuse explicitly. The interpreter keeps its precedence.
2. Class: defect. The documented order puts the held state ("`when_subject_state:` and `when_subject:` select by it") before "the accepting and external branches, in declaration order" (`website/docs/guides/specify/guards-and-predicates.md:107-117`; `docs/design/cross-record-and-stored-field-guards.md:749-762`, which says synthesis follows it, :771-773). The cause:
   - `subject_fact::routes` excludes `External`/`ExternalWhen` (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:118-129`).
   - So `arranged` gives the external branch `prepare` + `reach` (`synthesize.rs:3668-3678`), and `reach` hands it to `reach_external` (:5889-5891).
   - `reach_external` refutes input-guarded refusals (#178) and earlier accepting `when:` (#217) only (:5826-5848). No stored row is read.
   - The arranging driver has the same gap (`invoke`, :5108-5150: an `InjectFault` driver goes to `reach`).
3. Existing idiom: none in the specification. The adopter cannot reorder the branches without changing what the command means. Moving `unlisted` before `stale` (`probe-464/c/`) passes 10/10 on the interpreter, which answers held-subject and external branches in one declaration-order loop (`interpret/execute.rs:864-925`). The guide says otherwise (Q2), so a target that follows the guide fails it. Hand-editing the suite, the issue's control, is not an idiom.
4. Fit: the change is synthesis only, with no key, IR or format change.
   - (a) when_subject. An `InjectFault` branch on a command where `subject_fact::uses` holds is arranged by the subject-fact search. It looks for a row in a state its move starts from, and an input, such that `selects` (`subject_fact.rs:1820-1893`) picks no guarded sibling and the branch's own `ExternalWhen` guard holds. The miss-every-sibling search is the one `refusal_witness` already runs for wrong-state (:1953). It refutes every held-state guard, not only the earlier-declared ones. That witness is right under the guide's order and under the interpreter's declaration order alike.
   - (b) Driver. `invoke` (`synthesize.rs:5108`) chooses an external driver's input the same way against the row as arranged.
   - (c) Siblings. A `when_related` sibling has the same defect: `probe-464/d/` sends a `list_id` no row carries, and the interpreter answers `no-list` in three scenarios, one a state refusal driven through `unlisted` (`d.run.out`). `related_guard::routes` excludes externals too (`related_guard.rs:166-176`). So an external branch is arranged through `prepare_at_in` (:1368) on a present row no related refusal claims. A row-set guard (`row_set::routes`, `row_set.rs:102-115`) is inferred to have the same gap. It is either witnessed the same way or refused by name (ESS-SYNTH-003), never emitted unrefuted.
   - (d) Where no row and input miss every earlier-answering guard, the scenario is refused naming the guard. A suite the reference model fails is never written.
   - Targets: none change. The interpreter keeps its order (the requester's ask). Rust/Go/TypeScript runners execute the same steps.
   - Incidental, for the PR (not a new issue): the interpreter's declaration-order loop disagrees with the guide's held-state-before-external order (probe `c`). This story's witnesses avoid the overlap, so they hold under either order.
5. Second adopter: a payment capture whose stored amount must equal the input (`when_subject: amount != input.amount` → `mismatch`) beside an external `acquirer-declined`. Also a document publish refused on a stale revision beside an external `scan-failed`.
6. Cost: no source format, suite format, keyword or diagnostic. Suite bytes change, at every source format, only for external-branch scenarios (and external drivers) of commands with a `when_subject` or `when_related` sibling. Each of these is a scenario whose row or input such a guard claims. Existing steps only. Measure with before/after synthesis of every repository model. #461's `complete_refusal` (ess/23) is not touched: an external branch is not a when_subject refusal.
7. Alternatives:
   - (a) Change nothing and document "declare the external branch first". This contradicts the guide's order, and a guide-following target fails it.
   - (b) Refuse every external branch beside a held-state guard. That throws away coverage that (a)-(c) of Q4 keep.
   - (c) Refute only earlier-declared guards. Right under the interpreter, wrong under the guide.
   - (d) Chosen: refute every guard that answers before an external branch, and refuse by name where none can be refuted.

## Decisions
accept, redesigned. Synthesis witnesses an external branch on a row and input that no input refusal, held-state guard (`when_subject`, `when_subject_state`) or related guard claims. It refutes every held-state guard whatever its declaration order, and does the same for an external branch used as an arranging driver. Where no such witness exists, the scenario is refused naming the guard. The interpreter's precedence is unchanged. No format bump: neither ess/23 nor ess-conformance/44-45 is touched. Suite bytes change at every source format, which this story decides as a synthesis defect fix, only for external-branch scenarios a sibling guard claims today. Out of scope: aligning the interpreter's declaration-order loop with the guide (PR note).

## Acceptance
- external_witness_refutes_when_subject: probe `b` synthesizes `CheckPick/outcome/unlisted` and `Pick/transition/refuse/by/CheckPick/unlisted` with `revision` equal to the arranged row's, and the interpreter passes all 10 scenarios.
- external_witness_mutate_baseline_passes: `ess verify conform mutate --target interpreted` on probe `b` passes its baseline (no ESS-MUTATE-001).
- external_declared_first_still_refutes: probe `c` (external declared before `stale`) sends an input `stale` does not claim, and passes.
- external_witness_arranges_related_row: probe `d` arranges a present `List` whose `revision` equals the input for `unlisted`, and all three scenarios that now fail pass.
- external_driver_refutes_sibling_guards: a state refusal arranged through an external driver reaches its state on the interpreter.
- external_guarded_by_when_keeps_own_guard: an `ExternalWhen` branch beside a `when_subject` sibling is sent an input its own guard admits.
- unrefutable_sibling_refused_by_name: a `when_subject` guard that every row satisfies makes the external scenario a refusal naming that guard, not a failing scenario.
- row_set_sibling_witnessed_or_refused: an external branch beside a row-set guard either passes the interpreter or is refused by name.
- repository_model_suites_change_only_where_claimed: before/after synthesis of every repository model differs only in external-branch scenarios a sibling guard claimed.

## Scope
- crates/verify/ess-conformance/src/synthesize.rs  cited — `run_as` routing :3402-3408; `arranged` :3656-3678; `reach_external` :5820-5869; driver `invoke` :5108-5150; `earlier_accepting_branches` :6306
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs  cited — `routes` :118-129; `selects` :1820-1893; `refusal_witness` :1953 (no overlap with #463's `evaluate_row` :420-524 / `identity_selectors` :525)
- crates/verify/ess-conformance/src/synthesize/related_guard.rs  cited — `routes` :166-176; `prepare_at_in` :1368
- crates/verify/ess-conformance/src/synthesize/row_set.rs  inferred — `routes` :102-115 only (no overlap with #463's `scoped`, `Reading::truth`, decoys)
- crates/verify/ess-conformance/tests/external_beside_held_guard.rs  inferred — new cases
- crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard.yaml  inferred — brand-free fixture (probe `b`/`c`/`d` shapes)
- crates/verify/ess-conformance/tests/guarded_external.rs  cited — existing external-guard cases must stay green
- docs/design/cross-record-and-stored-field-guards.md  cited — the "Synthesis … follow it" line :771-773 gains the external-witness sentence
