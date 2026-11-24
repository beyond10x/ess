---
format: aep.planning-md/3
id: story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once
kind: story
status: draft
title: The loader resolves every state's store and scope and every read's cache store from the placement profile, once, and both renderers and the checker read that resolution
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in
revision: 1
---
## Need

Three defaults of `ess-ui/1` are not constants. Each is resolved from the document's placement
profile:

- `State.store`, by `PlacementProfile.resolution` (`schemas/ui/ess-ui.schema.yaml:1439`,
  `:1563-1570`);
- `State.scope`, by the resolved store (`:1442`);
- `Reads.cache`, `{store: {resolve: {class: view_cache}}}` (`:1291`).

Today three crates each resolve them, and they disagree.

- **React** resolves the store in `crates/ui/ess-ui-react/src/emit.rs:366-445`. For a `pinned`
  state with no `store`, it skips the section and page profiles (`:375-380`).
- **The terminal** resolves it in `crates/ui/ess-ui-tui/src/placement.rs:82-126`, and **the
  checker** in `crates/ui/ess-ui-check/src/rules.rs:1392-1433`. Both ignore `pinned`, so a section
  or page profile moves a pinned state.
  - The schema's `pinned` step has `effect: profile_ignored` (`:1565`).
  - The state's note says "no profile may move it" (`:1440`).
- **`Reads.cache`'s default cannot be read** into `Document`: `ReadCache.store` is a `Store`, with
  no `resolve` form (`crates/ui/ess-ui/src/model.rs:3007`).
  - The terminal decides whether its read cache survives a page change from the document-level
    `view_cache` class only (`app.rs:2770-2774`, `placement.rs:130-139`). It ignores a section's
    or page's profile and the read's own `cache`.
  - React has no read cache.
- **React's server state** is keyed `scope:key` only when `scope` is written, and bare `key`
  otherwise (`templates/runtime/state/server.ts.tmpl:16`). Filling `scope: user` would move the
  key of every server state an adopter has already stored. The terminal never reads `scope`.

## Outcome

- `ess-ui` resolves placement once, in a new `placement` module, in the schema's order:
  1. the explicit store;
  2. `pinned`, which ignores every profile;
  3. the section profile;
  4. the page profile;
  5. `placement_defaults`;
  6. the document profile.

  It reads the profiles' tables from the embedded schema (`PlacementProfile.profiles`).
- The fill step writes onto every `State` the `store` it resolves to and the `scope` the by-store
  table gives. It writes `cache: {store: …}` onto every `Reads`, resolved for the class
  `view_cache` in that read's section and page. Where resolution yields nothing (a `hybrid`
  document with no default for a class, `unresolved: refuse`), nothing is written, and
  `ess ui check` refuses it as it does today, with the same message and path.
- React's emitter, the terminal's `StateStore` and the checker read the written store. Their own
  resolution code goes.
- The terminal keeps or clears each read's cache by that read's resolved store.
- React keeps the bare server key for a state whose `scope` is its store's default scope, and
  writes `scope:key` only for any other scope (P0-D14 A). No stored key moves for a document that
  leaves `scope` out.
- The refusals (sensitive states in `url` or browser storage, credentials outside `memory` or
  `server_session`) stay in `ess-ui-check`.

## Acceptance

New test file `crates/ui/ess-ui/tests/placement.rs`, plus cases in
`crates/ui/ess-ui-tui/tests/adversary_pass1_placement.rs` and in an `ess-ui-react` placement test.
Each test is expected under 3 s, or 10 s where it runs the generated runtime, and is timed. None
may run over 60 s.

1. `the_loader_resolves_every_store`. Red today. A `hybrid` document with `placement_defaults`, a
   `thin` page, a `fat` section, and states of every class, including a `pinned` state with no
   `store` under the `fat` section. Every `State` in the loaded tree holds the store the schema's
   order gives; the expected table is in the test. The pinned state takes `placement_defaults`,
   not the section's profile.
2. `the_renderers_place_where_the_loader_placed`. For every state of `examples/partner-portal/ui.yaml`
   and of the placement corpus of `ess-ui-check`, React's emitted placement and the terminal's
   `StateStore::store_of` equal the tree's `store`.
3. `a_pinned_state_is_not_moved_by_a_section_profile`. In the terminal, the pinned state of test 1
   is stored where the loader placed it. Today the section profile moves it.
4. `a_thin_section_keeps_its_reads_cached_on_the_server`. In the terminal, the cache of a read in a
   `thin` section of a `fat` document follows that section's `view_cache` store across a page
   change, not the document's.
5. `server_keys_do_not_move`. A generated server state without `scope` keeps the bare key. One
   with `scope: account` keeps `account:<key>`.
6. `ess ui check` reports the same findings, with the same wording and paths, for every corpus
   document.

Also:

- The schema's `State.store` note says the loader writes the resolved store. The generated
  reference is regenerated.
- `CHANGELOG.md`, under Changed: the terminal now honours `pinned` and a section's `view_cache`.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** An own generator gets each state's placement without re-implementing the profile
   table. The stock renderers stop disagreeing on `pinned`.
2. **Class.** It is a *gap* for the resolved document. For `pinned` and `view_cache` in the
   terminal, it is a *defect*.
3. **Already expressible?** An author can write `store` on every state, and that is the only
   route today.
4. **Fit.**
   - There is no new key. Each written value is one the author could have written.
   - It reads the schema's own resolution table.
   - It removes two copies of the same logic.
5. **Second adopter.** A generator for a server-rendered target needs to know which states live on
   the server, to plan its session store.
6. **Cost.**
   - A behaviour change in the terminal for pinned states under a section or page profile, and for
     read caches under a section profile.
   - React behaviour is unchanged, apart from consolidating the code.
   - No format version.
7. **Considered.**
   - (a) Leave the resolution in the renderers and print it from `ess ui load` by calling one of
     them. Rejected: there would be three copies, and `ess-ui` cannot call a renderer.
   - (b) Chosen: resolution in `ess-ui`.

## Decisions

- **P0-D14: How does React key a server state once `scope` is always filled?**
  - (A) A bare key where the scope is its store's default, and `scope:key` otherwise. Nothing
    stored moves for a document that leaves `scope` out. A document that wrote the default scope
    explicitly (`scope: user` on a server state) loses its `user:` prefix.
  - (B) `scope:key` always. Every unwritten server state of every deployed generated app moves
    its key.
  - (C) Do not fill `scope`, as the one exception to "every default".

  **Recommended: (A).**

## Scope

- new: `crates/ui/ess-ui/src/placement.rs` (about 150 lines)
- cited: `crates/ui/ess-ui/src/fill.rs` (write `store`, `scope`, `cache`),
  `crates/ui/ess-ui/src/schema.rs` (profile tables)
- cited: `crates/ui/ess-ui-react/src/emit.rs` (366-445), `templates/runtime/state/server.ts.tmpl`
  (16)
- cited: `crates/ui/ess-ui-tui/src/placement.rs` (82-139), `crates/ui/ess-ui-tui/src/app.rs`
  (2770-2774)
- cited: `crates/ui/ess-ui-check/src/rules.rs` (1392-1433: read the resolved store, keep the
  refusals)
- new tests as above; `schemas/ui/ess-ui.schema.yaml` (1439 note); `CHANGELOG.md`
- Overlap: `g01-app-forks` edits `StateStore`'s storage part of `placement.rs` (179-352). This unit
  edits the resolution part (82-139). Land `g01-app-forks` first.
- Size: L, about 520 changed lines (`ess-ui` about 190, the renderers and checker about 150, tests
  about 180).

## Depends on

- `g03-loader-fills-defaults`
- `g01-app-forks` (same file, so it lands first)
