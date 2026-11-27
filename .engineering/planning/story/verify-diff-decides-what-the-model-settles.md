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
relations:
- serves: vision:O2
- informed_by: release-plan:ess-063-downstream
- decomposes: epic:adopter-gaps-verdicts
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
revision: 8
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

## Entity field and lifecycle changes for stored history (downstream request, 2026-11-27)

A downstream runtime that stores entity rows asked `ess verify diff` to answer one question: does the later
specification fit the rows written under the earlier one. This section extends part 2 above, which names only the added
`Optional` stored field; the request is the same gap with its other half.

### Need

On 0.99.2, and on `main` at 0.99.8 (`CHANGELOG.md` 0.99.3 to 0.99.8 and `[Unreleased]` name no change to the
compatibility of an entity field), `crates/verify/ess-diff/src/compatibility.rs` answers every entity change other than
`Added` and `Removed` with `[unknown, unknown, unknown]` (callers, readers, history; the arm at lines 745-748). That
holds for `FieldAdded`, `FieldRemoved` and `FieldTypeChanged`. A runtime that decides whether stored rows still fit
cannot tell an added `Optional` field (fits) from an added required field, a removed field or a retyped field (do not
fit). The runtime ships its own rule meanwhile and moves to `ess-diff` once a release classifies these. A removed entity
already answers history `breaking` (`EntityChange::Removed`, `[compatible, compatible, breaking]`); the removed lifecycle
state is reached through the entity's `<Entity>.State` type, a `VariantRemoved` of a type with a `Stored` use, which the
type table reads as narrowed (breaking for history where the state type has a `Stored` use; not checked by a run). Neither has a test naming it as stored-row compatibility today.

### Fit review

1. **Need.** Whether a specification change fits the rows already stored under the earlier one, decided from the two
   revisions alone. The consumer is "a downstream runtime that stores entity rows".
2. **Class.** Gap: the history dimension of an entity field change is `unknown` where the model settles it.
3. **Already expressible?** `--dimension history` narrows the gate to the dimension and
   `ess-diff-acknowledgements/1` records a reviewed change; every field change of an entity stays `unknown` and so
   needs an acknowledgement by hand, which defeats the gate.
4. **Fit.** Reuses the change vocabulary (`EntityChange::FieldAdded`, `FieldRemoved`, `FieldTypeChanged`, no new kind)
   and the type table's history column. Adds decided rows, not new fields in the delta, so no new `ess-diff` version
   unless a decision below needs one.
5. **Second adopter.** Any specification whose entities are persisted between releases.
6. **Cost.** New decided rows; `task test-xtask` pins and a regenerated example delta may move.
7. **Alternatives.** (a) Change nothing: the downstream keeps its own rule, and every adopter writes one. (b) Decide
   what the model settles (chosen). (c) Take a compatibility profile that states the consumer's assumptions: more
   general, a larger design, and not needed for the history dimension, where the model holds the answer.
8. **Generic, not per finding.** One rule over stored rows covers the added, removed and retyped field, the removed
   entity and the removed lifecycle state; it is not one story per finding.

### Decisions

- **D1. How is a retyped entity field compared for history?** (a) By the written type name: a rename of a named type
  with the same body is breaking, which fails closed and is wrong for a pure rename. (b) Structurally: both sides are
  resolved through newtypes, structs, unions and lists to their wire forms, and compared with the same comparison the
  type table already uses (`expanded`, `narrowed`, `changed`); equal wire form is compatible, `expanded` is compatible
  (every stored value is still one), `narrowed` and `changed` are breaking. (c) As (b), but every difference is breaking,
  `expanded` included, as the request words it. **Recommended: (b).** A widening is compatible for history in the table
  already (a widened type), and a second rule for fields would disagree with it; (c) is a one-line difference if the
  owner wants the request as worded. Fails closed: (a) and (c); (b) fails closed for what it cannot resolve
  (`Unmodelled` stays `unknown`).
- **D2. Are callers and readers classified in the same change?** (a) History only: callers and readers stay `unknown`
  for a field change. (b) Also callers (an entity field is not sent: compatible) and readers by the same type rule.
  (c) All three, and the added `Optional` view or event field for readers too. **Recommended: (a).** The request asks
  for the history dimension; readers of an entity field depend on the views and events that expose it, which those
  changes already report ("Unknown is an answer"); (b) can follow as its own story on this surface.
- **D3. How is an added field told `Optional` from required?** (a) From the field's type text in `FieldAdded`. (b) From
  the resolved type of the later revision, so an alias of `Optional<T>` counts. **Recommended: (b).** A text match
  misses an alias.
- **D4. Where does the removed lifecycle state land?** (a) Pin the current answer through the `<Entity>.State` variant
  with a test and change nothing if it is already breaking for history. (b) Add a state-level entity change kind.
  **Recommended: (a);** a new kind is a delta format change for an answer the type table already holds, and moves to
  (b) only if the test is red.
- **D5. Does the delta open a new `ess-diff` version?** Recommended: no, if the rows only change the classification of
  kinds that exist; a reader of the earlier version derives the written classification from the content and refuses a
  mismatch (`compatibility.rs` header), so a changed classification of an old kind is a version question the story
  answers before the first row lands.

### Acceptance

Every bullet runs in `cargo test -p ess-diff --locked --test entity_stored_history` (new, one file), over a model with
an entity that has stored fields and a lifecycle.

- `entity/<name>/field-added/<field>` of an `Optional<T>` field answers history `compatible`; the current `unknown`
  for history goes.
- The same for a required field answers history `breaking`.
- `field-removed` answers history `breaking`.
- `field-type-changed` answers history by D1: a structurally equal retype `compatible`, a narrowed or changed one
  `breaking`; the `unknown` for history goes in each case.
- A removed entity and a removed lifecycle state (a variant of `<Entity>.State`) each answer history `breaking`, named
  by a test of their own.
- Callers and readers of every kind above answer as D2 says (under the recommended (a): still `unknown`), and a test
  names that, so the choice is pinned and not an accident.
- A type the classifier cannot place (`Unmodelled`) keeps `unknown` in every dimension.
- `ess verify diff --dimension history --fail-on breaking` exits 4 for the required, removed and narrowed cases and 0 for
  the added `Optional` field, on the reproduction pairs under `.engineering/repro/diff-unknown-verdicts/` plus one
  pair per kind added there.
- No existing test file changes beyond the pins `task test-xtask` names; two revisions without an entity field change
  diff as before.
