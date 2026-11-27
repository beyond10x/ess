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
revision: 9
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

## A refusal added beside outcomes that read rows (downstream report, 2026-11-27)

## A refusal added beside outcomes that read rows (downstream report, 2026-11-27)

A downstream hardening pass put `ess verify diff --fail-on breaking-or-unknown` in a gate and read its history. This
section is the same gap as part 2 above, seen from the command that gains the refusal and not from the guard the new
refusal uses.

### Need

On 0.99.2, and on `main` at 0.99.8 (`CHANGELOG.md` 0.99.3 to 0.99.8 and `[Unreleased]` name no change to a diff rating
of a refusal), a plain `when:` refusal added to a command is rated `unknown` as soon as any outcome of the command is
not a plain `when:` or `otherwise` branch: a `when_related:`, a `when_subject:`, a `wrong_state:` or an
`unknown_instance:` outcome of the earlier revision, or one declared before the new refusal in the later revision.
`refuses_more` (`crates/verify/ess-diff/src/compatibility.rs`, the `_ => return false` arms in both loops) gives up on
the first such outcome, because a stored-state guard decides whether an input was accepted and the check reads inputs
only. The report: of the 292 changes of 8 merges that were not compatible, 254 were `unknown`, 83 of them an added
refusal and 29 a changed outcome set; nearly every command of that specification has a row guard or a wrong-state
outcome, so every change to the specification needed an acknowledgement and the gate no longer told a narrowing from a
harmless addition.

Minimal pair, run on 0.99.2 (the model of `.engineering/repro/diff-unknown-verdicts/before.yaml` with a required
`String` input `note` on `Cancel`; the later revision adds `{name: blank-note, when: note == "", error: WrongState}`
before `cancelled`):

- with the command's `wrong-state` outcome (`wrong_state: true`): `command/Cancel/outcome-added/blank-note`, `unknown for
  callers, readers`;
- with that outcome deleted from both revisions: the same change, `breaking for callers; unknown for readers`.

So the verdict depends on an outcome the new refusal does not touch.

### Fit review

1. **Need.** A caller-facing gate that rates an added refusal over inputs, on commands that also read rows, without an
   acknowledgement for each. The consumer is "a downstream control plane".
2. **Class.** Gap: the rating is decided only where every outcome of the command is a plain input guard.
3. **Already expressible?** `ess-diff-acknowledgements/1` records a reviewed change, which is the cost the report names.
   `--dimension` narrows the gate and decides nothing.
4. **Fit.** Reuses `Narrowing::RefusalAdded` and `satisfiable_by_declared_values`. The new part is what the check does
   with an outcome it cannot read.
5. **Second adopter.** Any specification whose commands mix input refusals with row guards, which is every model that
   has a lifecycle.
6. **Cost.** New decided rows; the delta's classification of an existing change kind moves, so D2 asks whether that
   needs a new `ess-diff` version. `task test-xtask` pins move.
7. **Alternatives.** (a) Change nothing: every change to the specification is acknowledged by hand, and a reviewer
   cannot tell the one narrowing from the rest. (b) Decide what the inputs settle (chosen, D1). (c) Rate every added
   refusal `breaking`: it fails closed and calls a refusal of an input no caller sent narrowing.
8. **Generic, not per finding.** One rule over "an outcome the check cannot read" covers the added refusal, the changed
   outcome set and the changed condition on every such command; it is not a rule per guard kind.

### Decisions

- **D1. How does the check treat an outcome whose guard reads a row or the stored state?** (a) As an outcome that
  does not refuse in some stored state: the check looks for an input the earlier revision's plain refusals let through
  and a plain accepting outcome takes, and claims `refusal-added` for the later revision's refusal of it, meaning "in
  a state where the earlier revision accepted the input, the later one refuses it". The state itself is not searched
  for or proven reachable. (b) Keep `unknown`, and name in the verdict which outcome made it so, so a reviewer knows what to read.
  (c) Rate such a change `breaking` whenever the new guard is satisfiable at all. **Recommended: (a).** It keeps
  `breaking` sound for the inputs it names and shrinks `unknown` to the cases that need the state; (b) can ship first
  as a diagnostic and does not conflict; (c) over-reports.
- **D2. Does the classification of an existing change kind move the delta format?** The header of
  `compatibility.rs` says a reader of an earlier version derives the written classification and refuses a mismatch.
  Recommended: the story answers it before the first row lands, and moves to a new `ess-diff/<n>` if an earlier
  reader would refuse the new verdict.
- **D3. Is the changed outcome set (`outcome-sets-changed`, 29 of the 254) in this story?** Recommended: no; it is a
  different change kind and a decision of its own on the same surface. Name it as a follow-up if the first rows land.

### Acceptance

Tests in `crates/verify/ess-diff/tests/narrowed_acceptance.rs`, each red on the current tree.

- The minimal pair above, with the `wrong_state` outcome kept, reports `command/<name>/outcome-added/blank-note` as
  `refusal-added`, breaking for callers, and `--fail-on breaking` exits 4; the same pair without that outcome keeps its
  verdict.
- The same with a `when_related:` outcome and with a `when_subject:` outcome in the earlier revision.
- An added refusal over inputs the earlier revision already refused by any outcome stays unclassified, as part 2
  says.
- A refusal whose guard holds only for stored state stays `unknown`.
- Part 2's own tests and the `after-error-added.yaml` control keep their verdicts.
