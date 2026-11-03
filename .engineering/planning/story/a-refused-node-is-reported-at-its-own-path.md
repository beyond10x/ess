---
format: aep.planning-md/3
id: story:a-refused-node-is-reported-at-its-own-path
kind: story
status: implemented
title: A refused ess-ui node is reported at its own path
summary: Load errors name the real path when an earlier form has a field named action; export params take a map and an empty state takes a title (C22, C22b; beyond10x/ess#723)
tags:
- ess-ui-adopter-wave-1
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/filter.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: cited
  path: crates/ui/ess-ui/src/locate.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T16:24:12Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-11-03T16:24:13Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-11-03T21:47:49Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
# Story: a refused node is reported at its own path

## Outcome

An author whose ess-ui document is refused at load (`ess ui check`, `ess generate ui`) is told the
real path and key of the refused node, whatever field names earlier forms use; and a bulk export
may pass params taken from the selection, and an empty state may carry a title.

## Context

A downstream adopter's report on ESS 0.84.0 (beyond10x/ess#723, items C22 and C22b): a refused key on
one page was reported as `pages/<an earlier page>/overlays/create/fields/action: unknown field 'field'`,
on a node that is valid. The adopter's hint: renaming the earlier form field `action` to `verb`
makes the report name the right path. Wave 1 of the ESS UI-adopter waves (D418, D420), release 0.86.0.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes. The
documents are minimal and written for the test (invented names), not copied from the adopter.

- `crates/ui/ess-ui/tests/load_error_path.rs`
  - `a_form_field_named_action_is_not_read_as_an_action` — a document with an earlier overlay form field `{field: action}` and a later refused key reports the later node's path, not `.../fields/action`.
  - `a_refused_key_in_an_empty_state_names_the_empty_state` — `states: {empty: {message: M, colour: red}}` after such a form is reported at `pages/<page>/sections/<section>/states/empty` (or deeper), naming `colour`.
  - `a_refused_key_in_a_bulk_export_names_the_export` — a refused key inside a bulk action's `export` after such a form is reported at that bulk action's path.
  - `a_field_named_like_any_action_position_is_read_as_a_field` — the same holds for a field whose name is any other key the schema uses as a single-action position (property test over `Schema::embedded().positions().actions`).
- `crates/ui/ess-ui/tests/export_params.rs`
  - `an_export_takes_params_written_out` — `export: {reads: V, as: zip, params: {ids: selection.id}}` loads, and the typed export carries the map.
  - `an_export_keeps_the_expression_form` — `params: same_as(list)` loads as before.
- `crates/ui/ess-ui-check/tests/export_params.rs`
  - `a_selection_param_outside_a_bulk_action_is_an_error` — `selection.<field>` in the params of a non-bulk export is refused by a named rule, as it is in `bind`.
- `crates/ui/ess-ui-react/tests/export_params.rs`
  - `a_bulk_export_sends_its_selection_params` — the generated runtime evaluates each param in the action's scope and reads the view with `{ids: [<selected ids>]}`.
- `crates/ui/ess-ui/tests/empty_title.rs`
  - `an_empty_state_takes_a_title` — `empty: {title: No files, message: M}` loads.
- `crates/ui/ess-ui-react/tests/empty_title.rs`
  - `an_empty_state_renders_its_title_above_its_message` — a heading with the title, then the message.
- `crates/ui/ess-ui-tui/tests/empty_title.rs`
  - `an_empty_state_shows_its_title` — the terminal shows the title line before the message.
- The reference page (`website/docs/reference/ess-ui.md`) is a fresh render of the schema.

## Out of Scope

- Any other load diagnostic; test-runner items (beyond10x/ess#722).
- Params on a header or row export beyond what the map form gives every export.

## Problem

When a whole document is refused, `locate` walks the expanded value and reads each node as the type
its position implies (`crates/ui/ess-ui/src/locate.rs:1-6`). A sequence entry's segment is the
entry's `name`/`field`/`view` (`crates/ui/ess-ui/src/expand.rs:305-315`), and `classify` then treats
that segment as a container key: a field entry `{field: action}` gives the last segment `action`,
which the schema holds as a single-action position (`EmptyState.action`, `Button.action`), so the
field is read as an `Action` and refused with "unknown field `field`"
(`locate.rs:99-104`). That false refusal comes first in document order and hides the real one
(inferred from the code and the adopter's rename experiment; the unit confirms it with a failing
test first).

The two real refusals: `Export.params` is a single expression (`crates/ui/ess-ui/src/model.rs:2444-2453`)
and an expression is a scalar (`model.rs:33-45`), so `params: {ids: selection.id}` is refused;
`EmptyState` has only `message` and `action` (`model.rs:880-888`), so `title` is refused.

## Fit review

1. **Need.** (a) A load error must name the node at fault. (b) A bulk export needs the selected rows'
   keys as view params. (c) An empty state needs a heading above its message. Requester's asks:
   "the diagnostic names the real path"; "`params` on a bulk export is either accepted (selection-
   derived values such as `selection.<field>`) or refused with a named error"; "accept `title` on an
   empty state (heading plus message) or state it is out of scope".
2. **Class.** (a) defect; (b) gap: the expression form cannot build a map; (c) gap.
3. **Already expressible?** (b) only `same_as(<section>)`, which copies another read's params, not
   the selection. (c) no.
4. **Fit.** (a) `classify` distinguishes an entry segment from a key. (b) the map form
   `{<param>: <expr>}` is the shape `navigate.params` and `reads.params` already use
   (`model.rs:2431-2439`, `model.rs:2315-2317`); `selection.<field>` follows the bulk `bind` rule.
   (c) `title` beside `message`, as section `title` (story:feature-request-281) and inline confirm
   `title` already spell it.
5. **Second adopter.** (b) a ticket list exporting the selected tickets as CSV; (c) any empty list
   with a heading and a hint.
6. **Cost.** A fix in `locate.rs`; two optional additive shapes in `ess-ui/1`; one check rule; React
   and terminal rendering.
7. **Considered.** (b) refuse the map with a named error: rejected, it leaves a real need
   inexpressible while two sibling constructs already take the map. (c) out of scope: rejected, the
   cost is one optional string.

## Decisions

Accept (2026-11-03, wave lead under D420): (a) fixed at its cause; (b) `params` on an export accepts
a map of params beside the expression form; (c) `title` on an empty state accepted.

## Delivered

UI-adopter wave 1 (D418, D420), release 0.86.0.

- Unit `c75123668d` merged at `af1451bf4b`; fix round 1 `086515559c` at `082647b2d2`.
- `locate.rs` reports a refused node at its own path. A list entry whose name or `field` matches an action key is no longer read as an action. A section's `states` and its entries are read as such only on a page section. Entries of the document maps an author names (`themes`, `tone_maps`, `label_maps`) are read as names.
- An export's `params` is a map or one expression. A hand-written reader names both shapes on a wrong one.
- `export_selection` admits `selection.<field>` only on an action of `bulk_actions`.
- An empty state takes a `title`; an empty title is no title in the terminal.
- Every named acceptance scenario passes. Adversary pass 1 needed revision and was fixed in round 1; pass 2 approved with two infeasible notes (a node named `bulk_actions`, a whitespace-only title), recorded and not fixed.
- Out of scope: an author key named `action` with a map value is `story:an-author-key-named-action-is-not-an-action`, and a node named like an action position is `story:a-node-named-like-an-action-position-loads` (both backlog).
