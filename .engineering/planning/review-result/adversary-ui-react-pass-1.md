---
format: aep.planning-md/3
id: review-result:adversary-ui-react-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-ui wave unit ui-spec-react-renderer
relations:
- reviews: story:ui-spec-react-renderer
revision: 1
---
unit: story:ui-spec-react-renderer, working tree ess-ui-react on c0f777f3c (crate untracked, uncommitted)
verdict: NEEDS-CHANGE
cases: executed 9→22, red 11
origin: introduced 11 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths
needs-coordinator: none

## 1. diff --stat

`git --no-pager diff --stat`: `Cargo.lock | 10`, `Cargo.toml | 1` (the implementor's, not mine). The crate is untracked, so the stat does not list it.
Adversary-added file: `crates/ui/ess-ui-react/tests/adversary_pass1.rs` (one test file). No implementation file touched.

## 2. Cases (tests/adversary_pass1.rs), red when first run alone

| case | asserts | now |
|---|---|---|
| sensitive_state_placed_by_defaults_never_reaches_url_or_browser_storage | sensitive state with no `store` never gets a url/session/local hook | RED: `useUrlState<string>("token_hint", "")` |
| a_sensitive_state_never_falls_back_to_local_storage | `store: server` + `fallback: local_storage` on sensitive state not emitted | RED: `{ fallback: { when: "true", store: "local_storage" } }` |
| a_credential_is_never_placed_in_local_storage | class credential never in browser storage | RED: `useLocalStorageState<string>("vault:pages/secrets.edit/state/api_token", "")` |
| a_page_profile_decides_placement_before_document_defaults | partner portal: thin page `paused` → server_session, fat page `zoom` → local_storage | RED: `useMemoryState<boolean>("portal:pages/activity.feed/state/paused", false)` |
| browser_storage_keys_carry_the_actor | storage keys include the actor | RED: storage runtime never mentions actor |
| url_states_of_two_same_named_nodes_get_distinct_query_keys | distinct url states, distinct query keys | RED: `["picker.mode", "picker.mode"]` |
| a_form_section_state_named_draft_type_checks | tsc passes | RED: TS2451 redeclare `draftValue` / `setDraftValue` |
| a_draft_of_a_document_type_is_typed_as_that_type | draft hook typed `<M.NoteDraft>` | RED: `useSessionStorageState<Record<string, unknown>>` |
| a_per_param_channel_session_reaches_transport_and_fixture_player | `sessionPer` / script `session` read | RED: `sessionPer` declared, never read (1 occurrence) |
| stale_after_turns_sections_stale_without_a_transport_stale_signal | something produces `stale` from staleAfter | RED: `lastLive` written, never read |
| resume_refetch_reads_again_after_a_reconnect | `resume: refetch` has a code path | RED |
| every_live_effect_generates_and_type_checks | patch_row/insert_or_patch+only_if+coalesce/remove_row/replace/refetch/when_paged_away emitted, runtime arms exist, tsc | green |
| constructs_the_example_lacks_render_at_their_paths_and_type_check | tree, conditional selection, selectable columns, chosen chart, popover/fullscreen, tabs with node+action, groups, parts, result, record, grid widget, toggle action; paths match ess_ui both ways; tsc | green |

## 3. Suite

`cargo test -p ess-ui-react --no-fail-fast` → adversary_pass1: 2 passed, 11 failed; partner_portal: 9 passed; EXIT=101. fmt check 0, clippy -D warnings clean.

## 4. Findings (all introduced; tree above)

| # | file:line | verdict | severity | finding | reached by |
|---|---|---|---|---|---|
| 1 | src/emit.rs:239 | NEEDS-CHANGE | blocker | `placement_call` ignores `sensitive` and credential class; schema refusals not applied | any doc using placement_defaults for draft/page_state; ess_ui load accepts it |
| 2 | templates/runtime/state/server.ts.tmpl:14 | NEEDS-CHANGE | blocker | sensitive server state falls back to localStorage | `fallback` on a sensitive state; ess_ui accepts |
| 3 | src/emit.rs:195 | NEEDS-CHANGE | blocker | `store()` skips section/page profile steps of the resolution order; tests/partner_portal.rs:225 pins the wrong answer | the example: activity.feed, workflows.editor |
| 4 | src/emit.rs:229 | NEEDS-CHANGE | warning | storage key `{app}:{path}` lacks actor.user_id/account_id | example: ticket reply draft, theme |
| 5 | src/emit.rs:565 | NEEDS-CHANGE | warning | node/overlay url prefix is the last segment only; same-named nodes share a query key | any doc with two same-named stateful nodes |
| 6 | src/emit.rs:296 | NEEDS-CHANGE | warning | form `draftValue` collides with a state named `draft`; tsc fails | doc with a form-section state named draft |
| 7 | src/emit.rs:289 | NEEDS-CHANGE | warning | draft type hard-coded to `Record<string, unknown>` | every form with a typed draft |
| 8 | templates/runtime/live.ts.tmpl:189 | NEEDS-CHANGE | warning | `session: {per}` ignored by transport URL, channel registry and fixture player | example: ticket_chat plays tk-01 into every ticket |
| 9 | templates/runtime/live.ts.tmpl:240 | NEEDS-CHANGE | warning | `stale_after` never produces stale over SSE/WebSocket | any `stale_after` channel with network sources |
| 10 | templates/runtime/live.ts.tmpl:103 | NEEDS-CHANGE | warning | `resume: refetch` does nothing on reconnect | example: metrics, tickets channels |
| 11 | templates/runtime/core.tsx.tmpl:363 | CONFIRMED | note | section frame and its composite carry the same data-ui-path (nested duplicate), same for overlay frame + body | every section |
