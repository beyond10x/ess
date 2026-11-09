---
format: aep.planning-md/3
id: story:verify-diff-decides-what-the-model-settles
kind: story
status: draft
title: ess verify diff rates a domain summary edit as documentation and decides what the model settles
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- informed_by: release-plan:ess-063-downstream
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: .engineering/repro/diff-unknown-verdicts/
- confidence: cited
  path: crates/verify/ess-diff/src/compatibility.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/domain_entry.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/narrowed_acceptance.rs
revision: 6
---
## Outcome

`ess verify diff` decides the changes the model itself settles (part 2 of the original story; part 1, a domain
`summary:` edit rated documentation, shipped and is pinned by
`crates/verify/ess-diff/tests/domain_entry.rs::a_domain_summary_edit_is_documentation`):

- an added `Optional` command input is compatible for callers and history;
- an added `Optional` stored entity field is compatible for history;
- a refusal added or widened through a `when_related` (or `exists`) guard is breaking for callers, as a plain `when:`
  refusal already is.

Added outcomes and errors for readers, and added view or event fields for readers, stay `unknown`: they depend on the
reader, and "Unknown is an answer" (`crates/verify/ess-diff/src/compatibility.rs:33-62`).

## Fit review

Finding covered: downstream specification report of 2026-10-19, finding 18 (`ess-diff/17`, 180 changes: 119
`unknown`, 48 compatible, 13 breaking, identical on 0.60.0, 0.62.0 and the integration build). Of the 119: 41 command
outcome-added, 20 view field-added, 8 outcome-sets-changed, 7 entity field-added, 5 error field-added, 1
`system/<name>/unclassified-changed`, the rest single kinds.

1. **Need.** A caller-facing gate on a growing specification fails on changes the model decides. Minimal pairs under
   `.engineering/repro/diff-unknown-verdicts/` (`before.yaml` against each `after-*.yaml`), re-run on 0.92.0:
   - `after-related-refusal.yaml` (a `when_related: {via: input.feed_id, exists: false}` refusal added to `Post`):
     `command/cascade.feed.Post/outcome-added/unknown-feed`, `unknown for callers, readers`; it should be breaking for
     callers, and today `--fail-on breaking` passes it;
   - `after-when-refusal.yaml` (an `Optional<Boolean>` input `quiet` and a `when: quiet == true` refusal):
     `command/cascade.feed.Post/input-added/quiet`, `unknown for callers, readers`; the input should be compatible for
     callers and history (the refusal over an input the earlier revision does not declare stays undecided, as
     documented);
   - `after-optional-field.yaml` (an `Optional<String>` entity and view field `note`):
     `entity/cascade.feed.Item/field-added/note`, `unknown for callers, readers, history`; it should be compatible for
     history;
   - `after-domain-summary.yaml`: `domain/cascade.feed/summary-changed`, compatible, gate passes (part 1, shipped);
   - `after-error-added.yaml`: compatible (control).
2. **Class.** Gap: only plain `when:`/`otherwise` branches are decided (`Compatibility::narrowing`,
   `compatibility.rs:955-978`, `refuses_more` at :1091; https://github.com/beyond10x/ess/issues/514), and an added
   `Optional` input or field has no decided row in the table at `compatibility.rs:38-62`.
3. **Already expressible?** `--dimension` narrows the gate and `ess-diff-acknowledgements/1` records reviewed
   changes; neither decides these cases.
4. **Fit.** Reuses the change vocabulary; the related-guard refusal reuses the narrowed-acceptance rule.
5. **Second adopter.** Any specification edited between releases runs the gate.
6. **Cost.** New decided rows in the delta format, behind a new `ess-diff/<n>` version; `task test-xtask` pins move.
7. **Alternatives.** (a) Change nothing: adopters acknowledge each change by hand. (b) Decide what the model settles
   (chosen). (c) Rate every added outcome compatible: refused, it guesses about readers.

## Decisions

Accept (2026-10-19). Part 1 was a defect fix and shipped alone. Part 2 is a gap on the same surface and lands in a new
`ess-diff` delta version. Reconciled 2026-11-02 (store hygiene H0 of wave W1, section 2i): narrowed to part 2.

## Acceptance

Tests over the repro pairs, each red on the current tree:

- `crates/verify/ess-diff/tests/narrowed_acceptance.rs::a_related_guard_refusal_added_is_breaking_for_callers`:
  `after-related-refusal.yaml` reports `command/cascade.feed.Post/outcome-added/unknown-feed` as `refusal-added`,
  breaking for callers, and `--fail-on breaking` exits 4.
- `crates/verify/ess-diff/tests/narrowed_acceptance.rs::an_optional_input_added_is_compatible_for_callers`:
  `after-when-refusal.yaml` reports `command/cascade.feed.Post/input-added/quiet` compatible for callers and history.
- `crates/verify/ess-diff/tests/domain_entry.rs::an_optional_stored_field_added_is_compatible_for_history` (or a new
  file): `after-optional-field.yaml` reports `entity/cascade.feed.Item/field-added/note` compatible for history; readers
  stay unknown.
- A `when_related` refusal that refuses only inputs the earlier revision already refused stays unclassified.
- `domain_entry.rs::a_domain_summary_edit_is_documentation` and the `after-error-added.yaml` control keep their
  verdicts.

## Scope

- cited: crates/verify/ess-diff/src/compatibility.rs
- cited: crates/verify/ess-diff/tests/narrowed_acceptance.rs
- cited: crates/verify/ess-diff/tests/domain_entry.rs
- inferred: crates/verify/ess-diff/src/diff.rs (delta version and change rows)
- inferred: .engineering/repro/diff-unknown-verdicts/

confidence: medium
