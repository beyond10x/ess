---
format: aep.planning-md/3
id: story:related-rows-keep-distinct-names-and-named-refusals
kind: story
status: active
title: Related rows keep distinct names, existence refusals name their guard, and a references-only relation is arranged
summary: 'Follow-up to the U6 related-guard stories: a suite-level no-double-capture check after per-row distinctions, the existence-outcome refusal test, and TagMachine on a references-only relation (0.70.0).'
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/arrangement-through-writers.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/beside-row-once.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/nested_list_membership_arranged.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/one_name_captured_once.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/references_only_relation_arranged.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/unknown_site_on_existing_subject.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:43Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:45Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Left open by unit U6 of the 0.69.0 batch and its adversary pass (2026-10-23). Checked again on
ess 0.92.0; the items that no longer apply are listed under "Closed since" below. This is still a
grab-bag of five items and should be split into one story per item before a wave takes it.

1. **A suite never captures one name twice.** `story:a-row-arranged-beside-is-created-once-under-its-name`
   fixed the row arranged beside a searched row, but synthesizing
   `crates/verify/ess-conformance/tests/fixtures/arrangement-through-writers.yaml` still writes four
   scenarios that capture one name twice (every scenario passes on the interpreted target):

   ```
   $ ess verify conform synthesize --path arrangement-through-writers.yaml --target ir --out suite.json
   ledger.lock.Annotate/outcome/annotated: captures review-194041 more than once
   ledger.lock.Approve/outcome/approved: captures review-194041 more than once
   ledger.lock.Review/transition/approve/by/ledger.lock.Approve/approved: captures review-194041 more than once
   ledger.lock.Seal/outcome/not-approved: captures incident-720721 more than once
   ```

   `Seal/not-approved` does so because `successors` calls `stored::step` with `Distinction::PLAIN`.
   A suite-level check that withdraws such a scenario would withdraw all four, so nested related
   rows first need a distinction of their own per row.
2. **A references-only relation arranges `TagMachine`.** In
   `crates/verify/ess-conformance/tests/fixtures/beside-row-once.yaml` (22 scenarios, no refusal),
   moving the relation from `Site` (`{name: machines, kind: owns, target: Machine, cardinality: many,
   via: site_id}`) onto `Machine` (`{name: site, kind: references, target: Site, cardinality: one,
   via: site_id}`) leaves a valid model whose `TagMachine` branches `elsewhere`, `not-alpha` and
   `tagged` are refused:

   ```
   refused: refusal[ESS-SYNTH-003]: outcome demo.yard.TagMachine/elsewhere has no scenario
     no candidate of the 4 tried satisfies ``demo.yard.Machine` stored related row selecting this branch, over the rows 4 bounded arrangements left`
   (the same for not-alpha and tagged)
   19 scenario(s) (0 authored), 3 refusal(s)
   ```
3. **Nested-list membership.** A related-list membership guard over a list nested inside the related
   row is not arranged; the existence-outcome fix covers a top-level list member only.
4. **Unknown-site on an existing subject.** Where the subject row exists in the wrong state, the
   wrong-state refusal answers first and the unknown-site branch is never synthesized.
5. **`with_row` call sites.** The four `with_row` call sites in
   `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`:2034`, `:2122`, `:2648`,
   `:3638` on main) were not surveyed for the `ordered`/`order` split U6 introduced.

## Acceptance

Every test below is a target of `cargo test -p ess-conformance --locked --test <name>`.

- `one_name_captured_once`: the suite synthesized from `arrangement-through-writers.yaml` holds no
  scenario with two `capture_instance` steps carrying one name, and the four scenarios of item 1
  (`Annotate/annotated`, `Approve/approved`, `Review/transition/approve/by/…/approved`,
  `Seal/not-approved`) are still in it and pass on the interpreted target.
- `one_name_captured_once`: a suite built by hand with one name captured twice is refused by the
  suite check before output, the refusal naming the scenario and the name.
- `references_only_relation_arranged`: the references-only variant of `beside-row-once.yaml`
  above synthesizes 22 scenarios with no refusal, and the scenarios for `elsewhere`, `not-alpha` and
  `tagged` pass on the interpreted target.
- `nested_list_membership_arranged`: an existence outcome guarded by membership of an input in a
  list nested inside the related row (`row.lines[].codes`) gets a scenario that passes on the
  interpreted target.
- `unknown_site_on_existing_subject`: on a command whose subject row exists and which declares a
  wrong-state refusal beside an unknown-site branch, the suite holds a scenario answered by the
  unknown-site branch, and it passes on the interpreted target.
- The four `with_row` call sites of `related_guard.rs` (`:2034`, `:2122`, `:2648`, `:3638`) call
  `with_row_where` filtered by the `order` check of `related_guard/stored.rs` (a read shows no
  `with_row(` call outside its definition), and `stored_order_adversary` stays green.
- `synthesis_suite_bytes_table` passes, and each row this change re-pins names its item beside it.

## Closed since

- The existence-outcome refusal naming its guard has a test:
  `tests/an_existence_outcome_passes_a_related_list_membership_guard.rs::an_existence_outcome_names_the_membership_guard_no_creator_can_pass`
  (commit 3b8dbf3862).
- The distinction-block overlap is gone: `COPIED` is `27_720 * 13` (`synthesize/subject_fact.rs:5161`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- cited: crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs
- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- inferred: crates/verify/ess-conformance/src/synthesize/existence.rs
- cited: crates/verify/ess-conformance/tests/fixtures/arrangement-through-writers.yaml
- cited: crates/verify/ess-conformance/tests/fixtures/beside-row-once.yaml
- inferred: crates/verify/ess-conformance/tests/one_name_captured_once.rs (new)
- inferred: crates/verify/ess-conformance/tests/references_only_relation_arranged.rs (new)
- inferred: crates/verify/ess-conformance/tests/nested_list_membership_arranged.rs (new)
- inferred: crates/verify/ess-conformance/tests/unknown_site_on_existing_subject.rs (new)

confidence: medium

Shared surfaces moved: re-pins the suite-bytes rows of `arrangement-through-writers.yaml` and
`beside-row-once.yaml` where they move; proposes its `CHANGELOG.md` sentence; no format row.

## Decisions

- The references-only `TagMachine` gets scenarios, not named refusals: the model is valid and the
  `owns` variant of the same rows already arranges all three branches.
- A nested-list membership guard is arranged, not refused, as `held_in_body` already arranges the
  top-level list member.
- Every `with_row` site takes the `order` filter: it replaces only a row no order of the
  arrangement reaches, so it cannot drop a scenario that passes today.
