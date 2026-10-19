---
format: aep.planning-md/3
id: story:verify-diff-decides-what-the-model-settles
kind: story
status: draft
title: ess verify diff rates a domain summary edit as documentation and decides what the model settles
tags:
- adopter-report
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- informed_by: release-plan:ess-063-downstream
scope:
- confidence: cited
  path: crates/verify/ess-diff/src/compatibility.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/domain_entry.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/narrowed_acceptance.rs
revision: 2
---
## Outcome

`ess verify diff` decides the changes the model itself settles, and stops rating documentation as
`unknown`:

1. **Defect.** A domain's `summary:` or `display` edit is documentation, compatible everywhere,
   instead of `system/<name>/unclassified-changed` (`unknown`, failing `--fail-on
   breaking-or-unknown`). A domain `wire` change stays decided as today.
2. **Gap.** An added `Optional` command input is compatible for callers and history; an added
   `Optional` stored field is compatible for history; a refusal added or widened through a
   `when_related` (or `exists`) guard is breaking for callers, as a plain `when:` refusal already is.

Added outcomes and errors for readers, and added view or event fields for readers, stay `unknown`:
they depend on the reader, and "Unknown is an answer" (`crates/verify/ess-diff/src/compatibility.rs:34-62`).

## Fit review

Finding covered: downstream specification report of 2026-10-19, finding 18 (`ess-diff/17`, 180
changes: 119 `unknown`, 48 compatible, 13 breaking, identical on 0.60.0, 0.62.0 and the integration
build). Of the 119: 41 command outcome-added, 20 view field-added, 8 outcome-sets-changed, 7 entity
field-added, 5 error field-added, 1 `system/<name>/unclassified-changed`, the rest single kinds.

1. **Need.** A caller-facing gate on a growing specification fails on changes the model decides.
   Minimal pairs under `.engineering/repro/diff-unknown-verdicts/` (`before.yaml` against each
   `after-*.yaml`), integration build:
   - `after-domain-summary.yaml` (domain `summary:` only): `system/cascade/unclassified-changed`,
     `unknown for callers, readers, history`, `fails --fail-on breaking-or-unknown`;
   - `after-related-refusal.yaml` (a `when_related … exists: false` refusal added): outcome-added,
     unknown for callers and readers;
   - `after-when-refusal.yaml` (an `Optional` input and a `when:` refusal on it): input-added and
     outcome-added unknown;
   - `after-optional-field.yaml` (an `Optional` entity and view field): unknown, history included;
   - `after-error-added.yaml`: compatible (control).
2. **Class.** Part 1 defect: the documented table rates "display name, summary or example moved"
   compatible everywhere (`compatibility.rs:39`), while `residual()` strips only a domain's fallback
   `wire`/`display` (`crates/verify/ess-diff/src/diff.rs:2361-2385`), so a summary edit falls into the
   residual. Part 2 gap: only plain `when:`/`otherwise` branches are decided
   (`compatibility.rs:930-951`; https://github.com/beyond10x/ess/issues/514).
3. **Already expressible?** `--dimension` narrows the gate and `ess-diff-acknowledgements/1`
   records reviewed changes; neither decides these cases.
4. **Fit.** Reuses the change vocabulary: a domain naming change kind beside the construct naming
   kinds; the related-guard refusal reuses the narrowed-acceptance rule. `domain_entry.rs:209`
   deliberately pins a domain *naming* edit as unclassified because a `wire` change reaches every
   path; the new kind keeps `wire` decided and only moves `summary`/`display`.
5. **Second adopter.** Any specification edited between releases runs the gate.
6. **Cost.** New decided rows and one change kind in the delta format; the unreleased `ess-diff/20`
   (`release-plan:ess-063-downstream`, delta row) carries them. `task test-xtask` pins move.
7. **Alternatives.** (a) Change nothing: adopters acknowledge each documentation edit by hand.
   (b) Decide what the model settles (chosen). (c) Rate every added outcome compatible: refused, it
   guesses about readers.

## Decisions

Accept. Part 1 is a defect fix that can land alone; part 2 is a gap on the same surface, landing in
`ess-diff/20`.

## Acceptance

- `a_domain_summary_edit_is_documentation`: `after-domain-summary.yaml` reports a domain naming
  change, compatible, and `--fail-on breaking-or-unknown` exits 0; a domain `wire` change stays
  decided as today (`domain_entry.rs:209` pin adjusted, not removed).
- `an_optional_input_added_is_compatible_for_callers`, `an_optional_stored_field_added_is_compatible_for_history`,
  `a_related_guard_refusal_added_is_breaking_for_callers`: the three other pairs.

## Scope

- `crates/verify/ess-diff/src/diff.rs` (`residual()`, ~2361) — cited.
- `crates/verify/ess-diff/src/compatibility.rs` (~39, ~930-951) and the change kinds — cited.
- Tests: `crates/verify/ess-diff/tests/domain_entry.rs`, `crates/verify/ess-diff/tests/narrowed_acceptance.rs`.

Part 1 is one unit today; part 2 is one unit in the `ess-diff/20` release.
