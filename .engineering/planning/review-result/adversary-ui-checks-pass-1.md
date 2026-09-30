---
format: aep.planning-md/3
id: review-result:adversary-ui-checks-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-ui wave unit ui-spec-checks
relations:
- reviews: story:ui-spec-checks
revision: 1
---
unit: story:ui-spec-checks — working tree ess-ui-checks (base 90ce849d7, crate crates/ui/ess-ui-check uncommitted)
verdict: NEEDS-CHANGE
cases: executed 43→52, red 8
origin: introduced 9 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 (~/.cache/ess-ui-wave/checks/adv1-red.log, adv1-red2.log, adv1-perf.log, adv1-suite.log, adv1-review.md)
needs-coordinator: none

## 1. diff --stat

`git --no-pager diff --stat`: `Cargo.lock | 13 +`, `Cargo.toml | 1 +` (both the implementor's). The crate is untracked,
so the stat cannot show my file; `git status --short --untracked-files=all` lists one new path from this pass:
`crates/ui/ess-ui-check/tests/adversary_pass1.rs`. No implementation file touched.

## 2. Cases (crates/ui/ess-ui-check/tests/adversary_pass1.rs), each run alone first

| case | asserts | now | red output (first run alone) |
|---|---|---|---|
| widget_expands_fires_when_an_argument_does_not_match_its_param_type | enum param `{enum:[low,high]}` bound to `sideways` → `widget_expands` at the use | red | only finding: `fixture_per_view` warning |
| a_page_param_used_as_a_link_target_is_not_a_page_refs_error_in_the_declaration | `{ref: page}` param in `to: {to: args.target}` → no `page_refs` | red | `widgets/go/body/link page_refs error: \`args.target\` names no page` |
| a_view_param_read_by_a_widget_body_is_not_a_view_in_model_error_in_the_declaration | `{ref: view}` param as `reads: args.source` → no `view_in_model` | red | `widgets/lister/body/rows/reads view_in_model error: \`args.source\` names no view of model \`shop\`` |
| a_section_rendered_by_a_widget_reading_an_unreadable_view_is_reported | section `component: log` whose body reads `audit.Entries` → `section_readable` | red | only three `fixture_per_view` warnings |
| a_foreign_primitive_prop_in_a_widget_is_primitive_props_whatever_the_key_order | unknown prop `icon` on a widget-body text → `primitive_props` for both key orders | red | `widgets_first = false: filed as [("document_loads", "pages/p/sections/summary/children/use/body/caption")]` |
| two_pages_with_one_name_are_names_unique | duplicate `pages.p` key → `names_unique` | red | `filed as [("document_loads", "/")]` |
| an_expression_finding_in_an_unnamed_action_names_the_action_by_its_canonical_path | `channel.ghost` in an unnamed action → finding at the loader's path for that action | red | `left: ["pages/p/sections/summary/actions/visible"]` `right: ["pages/p/sections/summary/actions/archive"]` |
| a_chain_of_2000_widgets_is_checked_within_the_budget | 2,000 widgets, each using the next, checked in < 10 s | red | `took 76.313708514s to check, of which loading alone takes 207.212891ms` |
| a_page_of_2000_sections_is_checked_within_the_budget | 2,000 sections with state, depends_on, opens → < 10 s | green | 0.76 s |

My first runs of the first three cases were red for my own mistake (params without the required `note`), and the
`{ref: view}` case passed because its assertion was absence-only. I fixed the fixtures, added a no-errors assertion,
and reran each one alone. The reds quoted above come from that rerun (adv1-red2.log).

## 3. Suite

`cargo test -p ess-ui-check --no-fail-fast`: EXIT=101. adversary_from_schema_pass1 7 passed; adversary_pass1 1
passed, 8 failed; checks 35 passed; example 1 passed. `<before>` = 43 is the implementing state's gate-test.log.
`cargo clippy -p ess-ui-check --all-targets -- -D warnings` exit 0; `cargo fmt -p ess-ui-check -- --check` exit 0.

## 4. Findings

| # | file:line | verdict | origin | measured | what reaches it |
|---|---|---|---|---|---|
| 1 | src/rules.rs:224 | NEEDS-CHANGE | introduced | `widget_expands.must: args_match_param_types` is never checked; the loader does not check it either | every widget use with typed params (the example's `money`, `status_badge`, `partner_card`) |
| 2 | src/rules.rs:312 | NEEDS-CHANGE | introduced | widget declaration bodies get the reference checks with `args.*` unbound → `page_refs` error, exit 1 | the Widget doc: "body … may read `args.<param>` … expanded at its use site and then checked"; `{ref: page}` is a listed kind |
| 3 | src/model.rs:233 | NEEDS-CHANGE | introduced | same for `--model`: `view_in_model` error on `args.source` in the declaration (command/event refs presumably the same, untested) | `{ref: view}` param, a documented pattern |
| 4 | src/model.rs:268 | NEEDS-CHANGE | introduced | `section_readable` only looks at a composite body; a section whose `component` is a widget is never checked | Section.component is `one_of [composite_kind, widget]` |
| 5 | src/classify.rs:45 | NEEDS-CHANGE | introduced | `is_primitive` walks the authored YAML along the expanded path; when `pages` comes before `widgets` the loader names the instance path (`…/use/body/caption`), and that path is not in the authored text → `document_loads` | any document that places `widgets:` after `pages:`; key order is free |
| 6 | src/classify.rs:39 | CONFIRMED | introduced | a duplicate map key (two `pages.p`) is filed `document_loads` at `/`, not `names_unique` at the node | an author pasting a page twice |
| 7 | src/raw.rs:71 | NEEDS-CHANGE | introduced | an unnamed list entry is walked at its list's path, so its keys become fake segments: `actions/visible` is a node that does not exist, and it is not the loader's `actions/archive` | actions without `name` are an accepted shorthand (Action name derivation) |
| 8 | src/rules.rs:693 | INFEASIBLE | introduced | `cycle_through` clones the chain on every push, starting again from each widget: O(n³) on a deep chain, 76 s for 2,000 (loading takes 0.2 s) | constructed; a 2,000-deep widget chain is not shown to be written by anyone. A flat 2,000-section page takes 0.76 s |
| 9 | src/model.rs:118 | INFEASIBLE | introduced | judgement: "readable when some actor may invoke a command of the owning context" counts a write-only grant (e.g. anyone who may `RecordEntry`) as reading every view of that context, and ignores the document's `actor: anonymous`. ESS has no read grants, so this crate cannot fix it | every model-backed run |

Fixes, named and not applied: (1) check args against `params.*.type` at each use site; (2, 3) skip reference checks
under `widgets/*` (as `opens` already does) or skip values that start with `args.`; (4) for a `Body::Widget`
section, test the reads of its expanded `body` nodes; (5) read the refused node from the expanded value, or classify
from the serde message plus the typed position instead of the authored text; (6) match serde_yaml's
`duplicate entry` message; (7) walk the expanded value, or take the segment from the loader's derived name;
(8) memoise reachability once per graph (a single DFS with colours) instead of one DFS per widget with chain clones.

## 5. Attacked and not broken

- Every `checks.list` id trips on its own document and not on the example (checks.rs plus example.rs), and the severities match the schema.
- A document with only a placeholder warning exits 0 (example.rs, run_exits_1…).
- JSON `ess-ui-check/1` keys and order: a derived serde struct plus sorted, deduplicated findings; no Hash* containers in either crate.
- Names with and without the system prefix, and names under the wrong context (`stock.Missing`, `audit.Items`), resolve or report as they should.
- `--lacks` with an unknown capability, a declared `refuse`, and a default non-refuse fallback all behave per `Degrades.rule`.
- A 2,000-section page takes 0.76 s.

## 6. Paths written outside the worktree

~/.cache/ess-ui-wave/checks/adv1-red.log, adv1-red2.log, adv1-perf.log, adv1-suite.log, adv1-review.md.
The build reused ~/.cache/b10x-target/ess-ui-checks.

## 7. findings

```findings
- file: crates/ui/ess-ui-check/src/rules.rs
  line: 224
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: widget_expands never checks args_match_param_types, so an enum param bound to a value outside its enum passes
- file: crates/ui/ess-ui-check/src/rules.rs
  line: 312
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: widget declaration bodies are reference-checked with args unbound, so a {ref page} param used as a link target is a page_refs error and exit 1
- file: crates/ui/ess-ui-check/src/model.rs
  line: 233
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: with --model a {ref view} param read in a widget body is a view_in_model error on the declaration
- file: crates/ui/ess-ui-check/src/model.rs
  line: 268
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: section_readable skips sections whose component is a widget, so a widget section reading an ungranted context is not reported
- file: crates/ui/ess-ui-check/src/classify.rs
  line: 45
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a foreign primitive prop inside a widget use is filed document_loads instead of primitive_props when pages precede widgets in the file
- file: crates/ui/ess-ui-check/src/classify.rs
  line: 39
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: two pages under one key are filed document_loads at / instead of names_unique at the page
- file: crates/ui/ess-ui-check/src/raw.rs
  line: 71
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an expression finding in an unnamed action is reported at a nonexistent path (actions/visible), not the canonical actions/archive
- file: crates/ui/ess-ui-check/src/rules.rs
  line: 693
  category: property
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: widget cycle detection is cubic on a deep chain, 76 s for 2,000 chained widgets against 0.2 s of loading
- file: crates/ui/ess-ui-check/src/model.rs
  line: 118
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: readability counts any command grant in a context as reading all its views and ignores the document actor setting
```
