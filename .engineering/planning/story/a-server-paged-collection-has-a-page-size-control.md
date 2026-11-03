---
format: aep.planning-md/3
id: story:a-server-paged-collection-has-a-page-size-control
kind: story
status: active
title: A server-paged collection has a page-size control and a pager footer
summary: reads.pager names the page and size params of a server-paged read; React renders a per-page select and a pager footer on the bound state paths (D12; beyond10x/ess#723)
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
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T16:24:12Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-03T16:24:12Z", actor: "human:timo", revision: 9}
---
# Story: a server-paged collection has a page-size control and a pager footer

## Outcome

A person paging through a long server-paged list in the generated React app can choose how many
rows a page shows and always sees where they are (page number, previous and next, next disabled
on the last page when the total is known).

## Context

A downstream adopter's review of ESS 0.84.0 generated pages (beyond10x/ess#723, item D12): paged
lists render no per-page selector and no pager footer when the document declares paging. Wave 1 of
the ESS UI-adopter waves (D418, D420), release 0.86.0.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.

- `crates/ui/ess-ui/tests/page_size.rs`
  - `a_read_takes_a_pager` — `reads: {view: V, params: {page: state.page, size: state.size}, paging: server, pager: {page: page, size: size, sizes: [25, 50, 100]}}` loads and the typed read carries the pager.
  - `pager_sizes_default` — a pager that names `size` and no `sizes` gets the schema default `[25, 50, 100]`.
- `crates/ui/ess-ui-check/tests/page_size.rs`
  - `a_pager_without_server_paging_is_an_error` — named rule, at the read's path.
  - `a_pager_param_must_be_a_read_param_bound_to_state` — `pager.page` or `pager.size` naming no param, or a param bound to something other than `state.<path>`, is refused by a named rule.
  - `pager_sizes_are_positive` — an empty `sizes` or a size below 1 is refused.
- `crates/ui/ess-ui-react/tests/page_size.rs`
  - `the_pager_sets_the_state_path_its_read_binds` — with `params: {page: state.p}`, Previous and Next set `state.p`, not a hardcoded `state.page`.
  - `a_pager_with_a_size_renders_a_per_page_select` — the footer shows a labelled per-page select offering `sizes`, showing the current size.
  - `choosing_a_size_sets_the_size_and_returns_to_page_one` — the select sets the size param's state path and resets the page param's state path to 1.
  - `next_is_disabled_on_the_last_page_when_the_total_is_known` — total 60, size 25, page 3: Next disabled; page 1: enabled.
  - `a_declared_pager_always_shows_its_footer` — the footer renders even when the total fits one page.
  - `a_server_read_without_a_pager_keeps_todays_footer` — `paging: server` with a `page` param and no `pager` renders as before, except that it sets the state path the `page` param binds.
- The reference page (`website/docs/reference/ess-ui.md`) is a fresh render of the schema.

## Out of Scope

- The terminal renderer: it has no pager today (no paging in `crates/ui/ess-ui-tui/src/`), so it gains
  none here.
- `cursor` and `append` paging, client paging controls, `header.total` text (G19) and list totals
  (B13).
- Server-side sort sending its initial direction (a defect the adopter noted with G6; not this wave).

## Problem

The pager renders only when the total exceeds the rows and a `page` param exists, has no size
control, and always writes `state.page`
(`crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl:246-260`). The read
declares only who pages (`Reads.paging`, `crates/ui/ess-ui/src/model.rs:2318-2319`;
`Paging`, `model.rs:2333-2350`), not which params carry the page and its size.

## Fit review

1. **Need.** A server-paged read has a page param and a size param, and the renderer must know
   which is which to page and to offer sizes. The adopter's reference fix recognised a size param
   by name (`size`, `max`, `limit`, `per_page`) and assumed `state.page`/`state.size`, and said the
   emitter should hand the collection the state paths its read binds and the grammar could carry
   the size options.
2. **Class.** Gap, plus a defect: the hardcoded `state.page` writes the wrong state when the read
   binds another path.
3. **Already expressible?** No: no key names a size param or its options.
4. **Fit.** A `pager` record on `reads` beside `paging`, naming read params rather than state paths,
   so the state path stays where it is declared (`params:`) and the emitter resolves it. It is
   refused unless `paging: server`. Reuses the param names a read already declares.
5. **Second adopter.** An audit log paged by `page` and `per_page`, offering
   50, 100 and 500 rows a page.
6. **Cost.** One optional record in `ess-ui/1` (additive), check rules, runtime and emitter changes.
7. **Considered.** (a) change nothing: rejected. (b) recognise params by name (the reference fix):
   rejected, a guess. (c) state paths in the pager (`size: state.size`): rejected, duplicates what
   `params:` already says and can disagree with it. (d) chosen: `pager: {page, size, sizes}` naming
   read params.

## Decisions

Accept, redesigned (2026-11-03, wave lead under D420): declared pager naming read params; `sizes`
optional with a documented default; no name-based recognition.

## Delivered

UI-adopter wave 1 (D418, D420), release 0.86.0.

- Unit `05114f9e2f` merged at `29c551f188`; fix round 1 `2f504a8c57` at `6b1798614c`; fix round 2 `c22f85428a` at `4815c9fbb8`.
- A read with `paging: server` takes `pager: {page, size, sizes}`; `sizes` defaults to `[25, 50, 100]`.
- The React collection footer shows Previous, the page (of N when known) and Next, and with a `size` a per-page select. The select writes page and size in one update and returns to page 1. It offers the current size, and lists a held value the pager does not count under its own text.
- The footer writes the state paths the params bind.
- `ess ui check` adds `pager_paging`, `pager_params` (including overlapping paths), `pager_sizes` and `pager_outside_collection`.
- Every named acceptance scenario passes. Adversary passes 1 and 2 needed revision and were fixed; pass 3 approved. The pass-3 cases are kept (`adversary_w086_u3_pass3.rs`).
- Out of scope: the fixture adapter paging by the pager params is now `story:the-fixture-adapter-pages-by-the-pager-params` (backlog).
