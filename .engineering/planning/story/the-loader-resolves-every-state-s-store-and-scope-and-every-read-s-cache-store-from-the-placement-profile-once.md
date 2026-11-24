---
format: aep.planning-md/3
id: story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once
kind: story
status: active
title: The loader resolves every state's store and scope and every read's cache store from the placement profile, once, and a server state's stored key follows its scope
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test
- depends_on: story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:07Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:09Z", actor: "human:timo", revision: 5}
---
## Need

Three defaults of `ess-ui/1` are not constants. Each is resolved from the document's placement
profile:

- `State.store`, by `PlacementProfile.resolution` (`schemas/ui/ess-ui.schema.yaml:1439`,
  `:1563-1570`);
- `State.scope`, by the resolved store (`:1442`);
- `Reads.cache`, `{store: {resolve: {class: view_cache}}}` (`:1291`).

Today three crates each resolve the store, and they disagree:

- **React** resolves it in `crates/ui/ess-ui-react/src/emit.rs:366-445`. For a `pinned` state
  with no `store`, it skips the section and page profiles (`:375-380`).
- **The terminal** resolves it in `crates/ui/ess-ui-tui/src/placement.rs:82-126`, and **the
  checker** in `crates/ui/ess-ui-check/src/rules.rs:1392-1433`. Both ignore `pinned`, so a section
  or page profile moves a pinned state.
  - The schema's `pinned` step has `effect: profile_ignored` (`:1565`).
  - The state's note says "no profile may move it" (`:1440`).
- Each resolver takes an explicit `store` first (`placement.rs:83-85`). So once the loader writes
  the resolved store, all three agree without any edit to them.
- **`Reads.cache`'s default cannot be read into `Document` as written.** `ReadCache.store` is a
  `Store` and has no `resolve` form (`crates/ui/ess-ui/src/model.rs:3007`). The fill therefore
  writes the resolved `Store`.
- **React keys a server state by its scope only when `scope` is written:**
  `const remoteKey = options.scope ? \`${options.scope}:${key}\` : key;`
  (`templates/runtime/state/server.ts.tmpl:16`). Once `scope` is always filled, every server state
  would get a prefix. One corpus document writes the default scope on a server state:
  `examples/partner-portal/ui.yaml:234` (`store: server, scope: user`). So "bare where the scope is
  the default" would move that state's stored key from `user:layout` to `layout` (design finding
  D10).

## Outcome

- **Resolution in `ess-ui`.** The placement pass (`crates/ui/ess-ui/src/fill/placement.rs`, the
  second pass) resolves placement in the schema's order:
  1. the explicit store;
  2. `pinned`, which ignores every profile;
  3. the section profile;
  4. the page profile;
  5. `placement_defaults`;
  6. the document profile.

  It reads the profiles' tables from the embedded schema (`PlacementProfile.profiles`). It writes,
  in this order:
  - onto every `State`, the `store` it resolves to;
  - then the `scope` the by-store table gives;
  - onto every `Reads`, `cache: {store: <Store>}`, resolved for the class `view_cache` in that
    read's section and page.

  Where resolution yields nothing (a `hybrid` document with no default for a class,
  `unresolved: refuse`), nothing is written, and `ess ui check` refuses it as it does today, with
  the same message and path. The placement pass writes the rows of the `HELD` list of
  `fill/placement.rs` where they resolve. The list stays, so the constant pass never writes an
  unresolved `store`.
- **The renderers' and the checker's resolvers stay.** Each takes the written store at its step
  1. No code of theirs is deleted here: the cleanup is the draft follow-up
  `g03-no-renderer-fallback`, which is not part of P0.
- **Pinned states.** The terminal and the checker now place a pinned state where the schema says,
  through the filled `store`, with no edit to the terminal.
- **A server state's key follows its scope, not who wrote it** (P0-D14 B, settled by the
  coordinator). React's emitter (`emit.rs:609`) passes `scope` to `useServerState` only where the
  resolved scope differs from its store's default in the by-store table (`schema:1442`; `server`
  gives `user`), whether the author wrote it or the fill did. `Store::default_scope()` in
  `crates/ui/ess-ui/src/model.rs` returns that default, read from the schema through `schema.rs`.
  - A server state with no `scope`, or with `scope: user`, gives a bare key.
  - A server state with `scope: account` gives `account:<key>`, as today.
  - So one document, filled or written out, gives one set of stored keys, and a document read back
    from `ess-ui-resolved/1` keys its states as the first load did.
- **One stored key moves.** Of the documents that load today, only the partner portal writes a
  server state's default scope: `layout` (`examples/partner-portal/ui.yaml:234`,
  `store: server, scope: user`). Its key moves once, from `user:layout` to `layout`, and a value
  saved under the old key reads as the default after the upgrade. `positions` (`:630`,
  `scope: account`) keeps `account:positions`.
- **Before and after:**

  | case | terminal before | terminal after | React before | React after |
  |---|---|---|---|---|
  | pinned state, no `store`, under a section or page profile | the profile's store | the store the schema gives (`placement_defaults` or the document profile) | the schema's store | unchanged |
  | server state, no `scope` | not keyed by scope | unchanged | key `layout` | key `layout` |
  | server state, `scope: user` written | not keyed by scope | unchanged | key `user:layout` | key `layout` |
  | server state, `scope: account` written | not keyed by scope | unchanged | key `account:<key>` | unchanged |

- **Read caches.** The terminal keeps deciding its read cache by the document-level `view_cache`
  class (`app.rs:2769-2771`). Honouring each read's own cache store is a terminal defect outside
  P0, in `g03-terminal-follows-defaults`.
- **Refusals.** The refusals (sensitive states in `url` or browser storage, credentials outside
  `memory` or `server_session`) stay in `ess-ui-check`.

## Acceptance

New test files:
- `crates/ui/ess-ui/tests/placement.rs`;
- `crates/ui/ess-ui-tui/tests/placement_filled.rs`;
- `crates/ui/ess-ui-react/tests/server_keys.rs`.

Each test is expected under 3 s, or 10 s where it runs the generated runtime, and is timed; none may
run over 60 s, and each crate's test step stays within 120 s. Each file joins its crate's single
test binary of `story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a
new file, which that story moves.

1. `the_loader_resolves_every_store_and_scope`. Red today. The document is `hybrid`, with
   `placement_defaults`, a `thin` page, a `fat` section and states of every class. Among them is a
   `pinned` state with no `store` under the `fat` section.
   - In `load_resolved`'s tree, every `State` holds the `store` and `scope` that the test's
     expected table gives. The pinned state takes `placement_defaults`, not the section's profile.
   - Every `Reads` holds `cache.store` as the table gives.
2. `an_unresolvable_class_is_left_unfilled`. A `hybrid` document with no default for a class has
   no `store` on that state, and `ess ui check` prints the message and path it prints today.
3. `a_pinned_state_is_not_moved_by_a_section_profile` (`ess-ui-tui`). Red today. In the terminal,
   `StateStore::store_of` for the pinned state of test 1 is the store the tree holds.
4. `the_renderers_place_where_the_loader_placed`. For every state of every document of the UI
   corpus, React's emitted placement and the terminal's `StateStore::store_of` equal the tree's
   `store`.
5. `server_keys_follow_the_scope` (`ess-ui-react`). The emitted `useServerState` call:
   - for a server state without `scope` passes no `scope`, so the key stays bare;
   - for `scope: user` written passes no `scope`, so the key is bare;
   - for `scope: account` written passes `scope: "account"`;
   - a document that writes `scope: user` on a server state and the same document without it
     emit equal sources.
6. `the_partner_portal_keys_move_only_for_layout` (`ess-ui-react`). Red today. A text pin of every
   `useServerState` call emitted for `examples/partner-portal`, written before the change from
   today's emitter: after the change every call keeps its `scope` argument except `layout`'s, which
   passed `scope: "user"` and passes none. The test names the one moved key, `user:layout` ->
   `layout`.
7. Existing tests: the test sets of `ess-ui-check`, `ess-ui-tui` and `ess-ui-react` pass with no
   expectation changed (no existing test pins a server state's `scope` argument). Any other move
   stops the unit and is reported.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui -p ess-ui-react --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui -p ess-ui-tui -p ess-ui-react -p ess-ui-check -p ess-ui-docs --locked`;
- `task test-xtask`.

Also:
- The note of `State.store` (`schema:1439`) says the loader writes the resolved store.
  `website/docs/reference/ess-ui.md` is regenerated by `cargo run -q -p ess-cli -- ui docs`.
- `CHANGELOG.md`, under Changed: "The terminal and `ess ui check` place a `pinned` state without a
  `store` where the schema says, ignoring a section or page profile, as a generated React project
  already did."
- `CHANGELOG.md`, under Changed: "**Breaking for a generated React project**: a server state that
  writes its store's default scope is stored under its bare key; the partner portal's `layout`
  moves from `user:layout` to `layout`, and a value saved under the old key is not read."

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** An own generator gets each state's placement without re-implementing the profile
   table. The stock renderers stop disagreeing on `pinned`.
2. **Class.** It is a *gap* for the resolved document. For `pinned` in the terminal and the
   checker, it is a *defect*.
3. **Already expressible?** An author can write `store` on every state, and that is the only route
   today.
4. **Fit.**
   - No new key. Each written value is one the author could have written.
   - It reads the schema's own resolution table.
5. **Second adopter.** A generator for a server-rendered target needs to know which states live on
   the server, to plan its session store.
6. **Cost.**
   - The terminal and the checker move for pinned states under a section or page profile.
   - React moves one stored key in the corpus, the partner portal's `layout`, once: a value saved
     under `user:layout` reads as the default after the upgrade.
   - In return, one digest means one set of stored keys, and an own generator that copies the
     stock key rule reads it from the resolved document alone.
   - No format version.
7. **Considered.**
   - (a) Print the resolution from `ess ui load` by calling a renderer. Rejected: `ess-ui` cannot
     call a renderer.
   - (b) Delete the three resolvers here. Rejected: not needed for item 3 (scope finding S6). It
     is the draft follow-up.
   - (c) Chosen: resolution written into the tree, with the resolvers left as they are.

## Decisions

- **P0-D14: How does React key a server state once `scope` is always filled?**
  - (A) The loader records which keys it filled, and React passes `scope` only where the author
    wrote it. No stored key moves. Inconsistent: `Document` derives `PartialEq`, so the record
    either breaks the round trip of `ess-ui-resolved/1` or hides a real difference, and one digest
    then means two sets of stored keys.
  - (B) A bare key wherever the scope equals its store's default, written or not. The partner
    portal's `layout` moves from `user:layout` to `layout` once. Equal digests mean equal keys.
  - (C) Do not fill `scope`, the one exception to "every default". Not taken.

  **Settled: (B)**, chosen by the coordinator on 2026-11-24.

## Scope

- `crates/ui/ess-ui/src/fill/placement.rs`: the pass, which writes its `HELD` rows; the list stays.
- `crates/ui/ess-ui/src/schema.rs`: a reader of `PlacementProfile.profiles`, `resolution` and the
  by-store scope table.
- `crates/ui/ess-ui/src/model.rs`: `Store::default_scope()`.
- `crates/ui/ess-ui-react/src/emit.rs`: the `useServerState` call (`:609`), `scope` only where it
  differs from the store's default.
- `schemas/ui/ess-ui.schema.yaml`: the note at 1439.
- `website/docs/reference/ess-ui.md`: generated.
- new: the three test files named above.
- `CHANGELOG.md`.
- Not changed: `crates/ui/ess-ui-tui/src/placement.rs`, `app.rs`,
  `crates/ui/ess-ui-check/src/rules.rs`.
- Size: M, about 450 changed lines: `ess-ui` about 200, React about 20, tests about 230.

## Depends on

- `g03-loader-fills-defaults`: `fill/placement.rs` and `Loaded`.
- `g03-model-defaults`, which edits the schema and the generated reference in round 2.
- `g03-actor-from-session`, which edits `emit.rs` in round 2.

Round 3, beside `g01-session-actions`, with which it shares no file.
`g03-defaults-that-move-behaviour` lands after it, in round 4, and shares the schema and the
generated reference.
