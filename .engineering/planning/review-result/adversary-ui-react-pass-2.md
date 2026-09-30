---
format: aep.planning-md/3
id: review-result:adversary-ui-react-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-ui wave unit ui-spec-react-renderer
relations:
- reviews: story:ui-spec-react-renderer
revision: 1
---
unit: story:ui-spec-react-renderer, working tree ess-ui-react on c0f777f3c after correction 1 (crate untracked, uncommitted)
verdict: NEEDS-CHANGE
cases: executed 25→38, red 11
origin: introduced 11 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths
needs-coordinator: none

## 1. diff --stat

`git --no-pager diff --stat`: `Cargo.lock | 10`, `Cargo.toml | 1` (the implementor's, not mine). The crate is untracked, so the stat does not list it.
Adversary-added file: `crates/ui/ess-ui-react/tests/adversary_pass2.rs`, and nothing else. No implementation file touched.

## 2. Cases (tests/adversary_pass2.rs), each run alone before the suite (log: adv2-red.log)

Runtime cases compile the generated runtime to CommonJS with `tsc` and run it under `node` against a hook-level React stub (state slots, effects flushed on demand, contexts, static tree walk). The stub and scripts go into the scratch project, not into generated output.

| case | asserts | red output when run alone |
|---|---|---|
| two_actors_never_share_a_storage_key | `storageKey` differs for {u\|1, acme} and {u, 1\|acme} | `share the storage key portal:pages/tickets.detail/sections/reply/draft\|u\|1\|acme` |
| an_actor_change_never_renders_the_previous_actors_stored_value | first render after A→B does not return A's session_storage value | `the first render for actor B returns actor A's value "A's private reply"` |
| a_server_state_on_its_local_fallback_does_not_keep_the_previous_actors_value | `useServerState` + local_storage fallback: after A→B and effects, value is not A's | `after the effects ran, actor B still holds actor A's value "A's layout"` |
| a_stale_channel_stays_stale_across_failed_retries | example `activity` over SSE: once stale, no return to reconnecting before live | `stale reconnecting reconnecting stale reconnecting reconnecting stale …` |
| live_again_clears_the_stale_timer | recovery before stale_after, then 120s pass: still live | green |
| a_reconnect_refetches_once_per_transition | `tickets` (resume: refetch): 0 refetches on first connect, then 1 per outage (2 failed retries each) | green |
| a_websocket_from_last_seen_reconnect_carries_the_last_seen_position | example `ticket_chat`: the reconnect URL or its frames carry the last seen id m-7 | `url ws://h/channels/ticket_chat?ticket_id=tk-01, frames []` |
| an_sse_session_reconnect_keeps_the_session_and_the_cursor_apart | SSE + session.per + from_last_seen: reconnect URL has ticket_id=tk-1 and last_event_id=m-7 | `reconnect URL http://h/channels/chat?ticket_id=tk-1?last_event_id=m-7` |
| a_session_channel_on_a_page_without_its_per_value_is_refused | page with no ticket_id anywhere, session channel in header and section: generation refuses | `useChannels(["chat"], { chat: evaluate("params.ticket_id", { params: __params }) })` |
| the_page_and_its_section_key_a_session_channel_on_the_same_value | session value from `state.current`: page and section evaluate to the same value | `the page keys chat on undefined (scope { params: __params }), the section on "tk-1"` |
| runtime_paths_are_unique_across_the_rows_of_a_collection | SectionFrame + Collection, 2 rows: no data-ui-path rendered twice | `[["…/patched/columns/title",2],["…/patched/row_actions/open",2]]` |
| collection_rows_render_their_row_key_path | `…/sections/patched/rows/r-1` and `/r-2` are rendered | `no pages/rows.board/sections/patched/rows/r-1 among [...]` |
| a_live_match_reaches_the_collection_that_keys_its_rows | `live.match: key` reaches the generated `<Collection>` | `<Collection columns={[…]} ` (no "key") |

## 3. Suite

`cargo test -p ess-ui-react --no-fail-fast` (log: adv2-suite.log): lib 0; adversary_pass1 13 passed; adversary_pass2 2 passed, 11 failed; partner_portal 12 passed; `EXIT=101`. `cargo fmt -p ess-ui-react -- --check` exit 0; `cargo clippy -p ess-ui-react --all-targets -- -D warnings` clean.

## 4. Findings (all introduced; tree above)

| # | file:line | verdict | severity | finding | reached by |
|---|---|---|---|---|---|
| 1 | templates/runtime/state/storage.ts.tmpl:29 | NEEDS-CHANGE | warning | the value lives in `useState` and is reloaded only in an effect, so the render after an actor change returns (and paints) the previous actor's value | example: `switch_org` overlay changes actor.account_id while `tickets.detail` reply draft (session_storage) is mounted |
| 2 | templates/runtime/state/remote.ts.tmpl:18 | NEEDS-CHANGE | warning | on the local fallback, a new key with nothing stored leaves the old value in place: the previous actor's value stays indefinitely and is saved under the new actor on the next set | example `overview.layout` (server, fallback local_storage) after `switch_org`; today `useActor` returns `may: () => true` (data.ts), so the fallback condition is false until grants are real |
| 3 | templates/runtime/state/storage.ts.tmpl:13 | INFEASIBLE | note | `${key}\|${user}\|${account}` is ambiguous when an id contains `\|`; two actors then share drafts and preferences | nothing found: ids come from `session.Me` and none observed contains `\|`; fix is an unambiguous encoding (JSON array) |
| 4 | templates/runtime/live.ts.tmpl:258 | NEEDS-CHANGE | warning | each retry reports `reconnecting`, which overwrites `stale`; the timer re-arms at 0 ms, so stale sections flip to fresh and back on every retry | example `activity` (stale_after 60s, backoff 1s..30s): once per retry, every 30 s, for the whole outage |
| 5 | templates/runtime/live.ts.tmpl:150 | NEEDS-CHANGE | warning | the WebSocket client ignores `resume: from_last_seen`: no cursor in URL or first frame, no refetch, so events sent during the outage are lost | example `ticket_chat` (both-way, from_last_seen) |
| 6 | templates/runtime/live.ts.tmpl:107 | NEEDS-CHANGE | warning | the SSE resume query is appended with a second `?`, so for a session channel `ticket_id` becomes `tk-1?last_event_id=m-7` and the cursor is lost | any server_to_client channel with `session.per` and `from_last_seen` (the schema's Channel example shape, one-way); none in the partner portal |
| 7 | src/emit.rs:1580 | NEEDS-CHANGE | warning | with no read param and no page param named `per`, the session falls back to `params.<per>`, which is undefined; every such page shares one `null` instance and the network URL subscribes `?ticket_id=`. No refusal | constructed document; every example page using `ticket_chat` has `params.id` |
| 8 | src/emit.rs:1955 | NEEDS-CHANGE | warning | the page evaluates the section read's session expression against `{ params: __params }` only, the section against its full scope; a session keyed on state or selection gives the header and page `channel.*` values a second, sessionless instance | constructed (master-detail keyed by `state.current`); the example keys on `params.id` and is unaffected |
| 9 | templates/runtime/fields.tsx.tmpl:59 | NEEDS-CHANGE | warning | column cells and row actions render their static path in every row, so each path appears once per row at runtime | every collection with 2 or more rows in the example (tickets.list, partners.list …) |
| 10 | templates/runtime/composites/collection.tsx.tmpl:150 | NEEDS-CHANGE | blocker | rows get no `data-ui-path`; the schema addresses runtime rows as `<collection>/rows/<row key>` (NodePath.syntax.containers) and the acceptance statement requires a path on every rendered node | every collection in the example |
| 11 | templates/runtime/composites/collection.tsx.tmpl:75 | NEEDS-CHANGE | warning | the row key is `row.id`, else the list index; the schema's key is `live.match` first, then `id`, and never an index; the generator does not pass `match` to the collection | any live collection with `match:` (pass-1 EFFECTS shape); rows without `id` fall back to indices |

Fix directions (not applied): 1 — keep `[actorKey, value]` in state and return initial/loaded when the key differs, during render; 2 — reset to `initial` when nothing is stored for the new key; 3 — `JSON.stringify([key, user, account])`; 4 — do not demote `stale` to `reconnecting` (keep stale until live); 5 — send the last event id on reconnect (URL param or first frame) or refetch; 6 — build the URL with `URL`/`URLSearchParams`; 7 — refuse at generation, naming channel, page and per; 8 — evaluate the page-level session against the same scope (or read it from the section); 9–11 — pass the collection its path and row-key field, and render `…/rows/<key>` on each row with cell and action paths under it.

## 5. Attacked, could not break

| target | how |
|---|---|
| resolution order: explicit, pinned (skips section/page), section, page, document defaults, document profile, else refusal | read emit.rs:244-272 against schema PlacementProfile.resolution.order; profile reset per section (1606/1719), page (1892/1962), shell (2122). No new case; partner_portal covers section-over-page and pinned |
| URL key encoding | segment pattern `^[A-Za-z0-9_.-]+$` admits no query-special character, and url.ts sets keys through URLSearchParams |
| `__` internals vs names with leading underscores | state locals are `camel(name)+"Value"` (ts.rs:99), so no local starts with `__` |
| clash suffix determinism | `local_pair` / `component_name` walk BTreeMap / section order |
| stale timer cleared on live again | green case live_again_clears_the_stale_timer |
| reconnect storms | green case a_reconnect_refetches_once_per_transition: exactly one refetch per down→live transition, none per retry |

## 6. Paths written outside the worktree

- `~/.cache/b10x-target/ess-ui-react/` (build dir; test binary and incremental output)
- `~/.cache/b10x-target/ess-ui-react/tmp/ess-ui-react-adv2/` (13 generated scratch projects with the React stub, `adv-out/`, `*.cjs`)
- `~/.cache/ess-ui-wave/react/adv2-red.log`
- `~/.cache/ess-ui-wave/react/adv2-suite.log`
- `~/.cache/ess-ui-wave/react/adv2-review.md`

## 7. Findings block

```findings
- file: crates/ui/ess-ui-react/templates/runtime/state/storage.ts.tmpl
  line: 29
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the render after an actor change returns the previous actor's stored value because the reload runs only in an effect
- file: crates/ui/ess-ui-react/templates/runtime/state/remote.ts.tmpl
  line: 18
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: on the local_storage fallback an actor with nothing stored keeps the previous actor's value indefinitely
- file: crates/ui/ess-ui-react/templates/runtime/state/storage.ts.tmpl
  line: 13
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the pipe-joined storage key is ambiguous when a user or account id contains a pipe
- file: crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl
  line: 258
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: every failed retry demotes a stale channel to reconnecting, so stale sections flicker fresh on each retry
- file: crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl
  line: 150
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the WebSocket client reconnects a from_last_seen channel with no cursor and no refetch, losing events sent during the outage
- file: crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl
  line: 107
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the SSE resume cursor is appended with a second question mark and merges into the session param
- file: crates/ui/ess-ui-react/src/emit.rs
  line: 1580
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a session channel on a page with no per value generates an undefined session shared by all such pages instead of a refusal
- file: crates/ui/ess-ui-react/src/emit.rs
  line: 1955
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the page evaluates a session expression against params only while the section uses its full scope, so they key different channel instances
- file: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
  line: 59
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: column and row-action paths are rendered once per row, so runtime data-ui-path values repeat
- file: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
  line: 150
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: collection rows render no data-ui-path, so no row is addressable as collection path plus rows plus row key
- file: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
  line: 75
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the row key ignores live.match and falls back to the list index, which the schema's row addressing forbids
```
