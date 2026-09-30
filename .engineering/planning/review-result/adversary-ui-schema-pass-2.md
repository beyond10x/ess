---
format: aep.planning-md/3
id: review-result:adversary-ui-schema-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-ui wave unit ui-spec-schema
relations:
- reviews: story:ui-spec-schema
revision: 1
---
unit: story:ui-spec-schema, working tree ~/.local/state/worktree/trees/b10x/ess/ess-ui-schema (uncommitted over c2df36f75, after correction 1)
verdict: NEEDS-CHANGE
cases: executed 31→40, red 9
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (this file; ~/.cache/ess-ui-wave/schema/adv2-keys.txt scratch) plus the assigned build dir
needs-coordinator: none

## 1. Diff

`git --no-pager diff --stat`: Cargo.lock | 8, Cargo.toml | 2 (the implementor's; the crate is untracked).
Adversary addition, untracked: crates/ui/ess-ui/tests/adversary_pass2.rs (9 cases). No implementation file touched.
One probe file (tests/adversary_pass2_probe.rs) was written, run and deleted. The new file was rustfmt-ed (whitespace only) after its red run.

## 2. Cases (all red, run alone: `cargo test -p ess-ui --test adversary_pass2` → 0 passed; 9 failed)

| case (line) | red output |
|---|---|
| a_misspelt_kind_in_the_composite_shorthand_is_refused (:55) | accepted; expand is Some("Widget(WidgetUse { component: \"recrod\", args: {}, body: [] })") |
| a_misspelt_kind_in_a_board_widget_is_refused (:86) | a board widget of kind `metrc` was accepted |
| a_widget_named_by_the_composite_shorthand_is_not_left_unexpanded (:107) | accepted without the required argument `who`; expand is Some("Widget(WidgetUse { component: \"card\", args: {}, body: [] })") |
| an_overlay_named_export_reads_through_the_shorthand (:136) | refused: pages/p/overlays/export: `record`: invalid type: string "t.Csv", expected struct Reads |
| a_board_widget_named_export_reads_through_the_shorthand (:157) | refused: pages/p/sections/board/widgets/export: `metric`: invalid type: string "t.Exports", expected struct Reads |
| a_list_arg_fills_a_list_typed_position_of_a_widget_body (:186) | refused: widgets/panel/body/rec: `record`: invalid type: string "args.cols", expected a sequence |
| an_unmapped_marker_is_accepted_by_a_boolean_field (:212) | refused: pages/p/sections/summary/columns/x: invalid type: string "UNMAPPED: the grid config is dynamic", expected a boolean |
| an_unmapped_marker_is_accepted_by_a_record_field (:224) | refused: pages/p/sections/summary: `collection`: invalid type: string "UNMAPPED: sorting is done by a plugin", expected struct Sort |
| an_unmapped_marker_at_reads_is_not_read_as_a_view_name (:238) | the marker became view Some("UNMAPPED: the handler builds the query at runtime") |

## 3. Suite

`cargo test -p ess-ui --no-fail-fast` (after the cases existed): adversary_pass1 11 ok, adversary_pass2 0/9, example 8 ok, schema 8 ok, shorthands 4 ok; `error: 1 target failed`; cargo exit non-zero.
Before = 31 (11+8+8+4, adversary_pass2 deselected); after = 40.

## 4. Findings

| # | file:line | verdict | origin | what reaches it |
|---|---|---|---|---|
| 1 | src/expand.rs:414 | NEEDS-CHANGE | introduced | Correction 1 moved the Composite shorthand after widget expansion (expand.rs:115 vs :131), so its output `{component: X}` is never checked. A typo'd bare kind at any single Node position (`expand`, `result`, `record`, `choice`, `Tab.form`) or board `widgets` map value loads as a widget use of a widget that does not exist. Before correction 1 pass 1 found this refused. Reached by any author typo. Board `widgets: {kpi: metric}` is the natural place for this shorthand |
| 2 | src/expand.rs:414 | NEEDS-CHANGE | introduced | Same cause: a bare widget name at a Node position loads as a `WidgetUse` with an empty body. The required-param check and the body are skipped. The fix is one of two: refuse non-kinds in the shorthand (the schema says `accepts: {ref: composite_kind}`), or run the shorthand before widget expansion |
| 3 | src/expand.rs:333 | NEEDS-CHANGE | introduced | `export.reads` is told apart by the key of the enclosing map (`own_key != "export"`). An overlay named `export` or a board widget named `export` also sits under that key, so its `reads: <view>` is refused. Reached by any overlay/board kind named `export`, which is a plausible name. The example uses `export` only as action names in lists, which are not affected |
| 4 | src/model.rs:1689 | NEEDS-CHANGE | introduced | Widget declarations are read as typed nodes with `args.<p>` still in place. A param that stands for a list or record (`fields: args.cols`, `sort: args.s`, `navigate: args.n`) makes the declaration fail before any use is substituted, although WidgetInstance.args says "an expression, or a literal such as a map". Only `reads`, Node positions and scalar/expr positions survive |
| 5 | src/model.rs:48 | CONFIRMED | introduced | Correction 1 made only enums accept `UNMAPPED: …`. Booleans (`sortable`, `multiple`, `danger`…) and record-typed fields (`sort`, `navigate`, `export`, `submit`…) still refuse it, against `unmapped_marker.accepted_by: any field, whatever its declared type`. The schema is self-inconsistent here: Tab.fields and collection.columns declare the string alternative explicitly. The schema or the loader needs a decision |
| 6 | src/expand.rs:335 | CONFIRMED | introduced | The Reads shorthand wraps a marker at `reads:` as `{view: "UNMAPPED: …"}`. The Document then names a view called `UNMAPPED: …`, where `unmapped_marker.renderer` says treat_as_absent. Reached by every retrofit that cannot find a section's API call. note |

## 5. Attacked and not broken

- **Positions**: `positions()` covers every Node/Action/Field/Reads key in `constructs.*.fields`, including one_of (Tab.form, collection.columns), nested records (`SectionStates.empty.action`, `columns.all`) and the board map. Reads/Field occur only in the shapes the loader handles. Key-name collisions were checked (`body` confirm string, `columns` PageLayout, `fields` Channel map, `layout` board, `reads` PlacementProfile). Only `export` misfires (finding 3).
- **Merge by derived name**: two actions deriving `approve` are refused (names_unique), with or without a kind merge. A page action overrides a kind action of the same derived name. A field merges by `field`. A widget use in a kind is overridden by `args` only.
- **Pass order**: same_as works on an overlay inherited from a kind, and derived action names inside it are fine. Tab.form works as an action and as a bare kind. An inline confirm on an `opens` action drops `does`.
- **Error paths in widget bodies**: arg-dependent type errors, missing names and nested-widget errors are all reported at `<instance>/body/<node>…`. Errors in the declaration itself are reported at `widgets/<w>/…`.
- **Round trip**: the model has no Serialize, so there is nothing to round-trip. The UNMAPPED enum variants are `#[serde(untagged)]` and deserialize correctly.

## 6. Paths written outside the worktree

- ~/.cache/ess-ui-wave/schema/adv2-review.md (this file)
- ~/.cache/ess-ui-wave/schema/adv2-keys.txt (scratch: schema field keys by construct/type)
- ~/.cache/b10x-target/ess-ui-schema (the assigned build dir; reused)

## 7. Findings block

```findings
- {file: crates/ui/ess-ui/src/expand.rs, line: 414, category: acceptance, severity: blocker, verdict: NEEDS-CHANGE, origin: introduced, message: "the Composite shorthand now runs after widget expansion, so a misspelt bare kind at a single Node position or board widget loads as a use of a nonexistent widget"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 414, category: boundary, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "a bare widget name at a Node position loads as a widget use with an empty body, skipping required-param checks and expansion"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 333, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "export.reads is detected by the enclosing key, so an overlay or board widget named export has its Reads shorthand skipped and is refused"}
- {file: crates/ui/ess-ui/src/model.rs, line: 1689, category: contract-drift, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "widget declarations are typed before substitution, so a list or record arg used in a list- or record-typed body position refuses the document"}
- {file: crates/ui/ess-ui/src/model.rs, line: 48, category: contract-drift, severity: warning, verdict: CONFIRMED, origin: introduced, message: "boolean and record-typed fields still refuse the UNMAPPED marker that the schema says any field accepts"}
- {file: crates/ui/ess-ui/src/expand.rs, line: 335, category: contract-drift, severity: note, verdict: CONFIRMED, origin: introduced, message: "the Reads shorthand turns an UNMAPPED marker at reads into a view named UNMAPPED: ..."}
```
