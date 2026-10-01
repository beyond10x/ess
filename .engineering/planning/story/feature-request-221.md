---
format: aep.planning-md/3
id: story:feature-request-221
kind: story
status: draft
title: the explorer draws commands with existing_instance, subject_state or subject_predicate outcomes
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#221
relations:
- serves: vision:O2
revision: 3
---
## Outcome

the explorer draws commands with existing_instance, subject_state or subject_predicate outcomes (beyond10x/ess#221).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: story:feature-request-221 (beyond10x/ess#221)

Read tree `gaps-270` HEAD `e3bc9a2ff` (contains 0.48.0). Runs: `ess` 0.44.0.

1. **Need.** The generated model explorer never draws a command whose answer depends on the stored row: whether the row exists, its state, or its fields. These are often the commands most worth exploring: rebind, rotate, revoke. A downstream specification compensated with a hand-written metamorphic test (issue). Requester's syntax: none; the ask is behavioural ("draw against rows the sequence created, assert the outcome the model selects"). Repro: `~/.cache/ess-gaps/fit2/repro-221`, with `Bind` (`existing_instance`), `Rotate` (`when_subject`) and `Revoke` (`when_subject_state`). It validates and synthesizes 10 scenarios (ess 0.44.0), and the IR condition kinds are `existing_instance`, `subject_predicate` and `subject_state` (`ess specify compile`).
2. **Class: gap** in a verification tool. The behaviour is documented as deferred, so it is not a defect: "Explorer conditions: subject fields, subject state, state changes … needs arranged state" (`docs/design/mutation-audit-and-model-runner.md:702`).
3. **Already expressible?** No. The plan excludes every condition kind other than `when`, `otherwise`, `wrong_state`, `external` and `external_when` (`crates/verify/ess-conformance/src/go/explore.go:614-619`; TS `src/ts/explore.ts:478`). The only escape is `AllowExcluded`, which accepts those outcomes as never tried (`explore.go:1863-1872`). Synthesis already covers these branches as fixed scenarios. The gap is random sequences over them.
4. **Fit.**
   - The model already holds what is needed. `exploreModel.records` keeps each row's fields and state (`explore.go:746-762`). `exploreDecide` already finds the supplied row (`explore.go:936-942`) and already decides `wrong_state` from its state (`explore.go:988-1000`). Supplied identities already reuse created rows with chance 0.8 (`explore.go:1150-1160`, design `:421`).
   - The decision must follow the one precedence order (`docs/design/cross-record-and-stored-field-guards.md:605-628`), as #235 did for `wrong_state`.
   - Sibling kinds the request does not name sit behind the same gate and must be handled or excluded by name: `subject_field`, `state_change`, `unknown_instance`, `input_absent` and `related` (`when_related`, ess/18–20; `crates/specify/ess-compiler/src/ir.rs:584-700`).
   - `related` reads another entity's row and widens what the model has to hold. Candidate to exclude by name in the first cut.
   - Go and TS must stay draw-for-draw identical (`explore.go:25-29`).
5. **Second adopter.** An inventory system whose `Reserve` refuses `when_subject: available < input.quantity` and whose `Close` refuses `when_subject_state: Closed`. Random sequences of receive/reserve/close are exactly where over-reservation bugs hide.
6. **Cost.**
   - No authored surface, no format version, no diagnostic.
   - Generated Go and TS explorer code changes.
   - Commands formerly excluded now draw. That changes the command set picked from (`design:407`), so a seed names a different trace than before: a release note item.
   - New `excluded` reasons for kinds left out.
   - Tests in both lanes, as `explore_guards_first.rs` does.
7. **Alternatives.**
   - (a) Change nothing and point at `AllowExcluded` plus synthesis: refused, because these commands get no sequence coverage at all.
   - (b) The request exactly as written (three kinds): refused as partial, since it leaves sibling kinds silently behind the same gate.
   - (c) Chosen: the explorer decides every stored-row condition kind from its model in the documented precedence order, and excludes by name only those it cannot model (proposed: `related`).

## Decisions

- **accept, redesigned (proposed):**
  - Broaden the request from three condition kinds to every stored-row kind the explorer's model can decide: `existing_instance`, `unknown_instance`, `subject_state`, `subject_field`, `subject_predicate` and `state_change`.
  - Decide them in the precedence order of `cross-record-and-stored-field-guards.md`.
  - Exclude `related`/`when_related` by name. `input_absent` is optional.
  - Go and TS port in one unit.
  - Overlaps: #223 is the same explorer and the same seed contract, so schedule them together. Precedent: #156 (externals) and #235 (order), both closed. Covers the design doc's Deferred row "Explorer conditions".
  - Not a duplicate. Not fixed at HEAD `e3bc9a2ff` (`explore.go:614-619`).

- Coordinator (2026-10-01): adopted as proposed above. Scheduled together with #223.
