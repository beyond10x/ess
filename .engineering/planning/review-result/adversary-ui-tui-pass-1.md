---
format: aep.planning-md/3
id: review-result:adversary-ui-tui-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-ui wave unit ui-spec-tui-renderer
relations:
- reviews: story:ui-spec-tui-renderer
revision: 1
---
unit: story:ui-spec-tui-renderer — working tree ~/.local/state/worktree/trees/b10x/ess/ess-ui-tui (base 4f471fc1e, uncommitted crate crates/ui/ess-ui-tui)
verdict: NEEDS-CHANGE
cases: executed 13→33, red 12
origin: introduced 14 / pre-existing 0 / undecided 0
wrote-outside-worktree: 7 paths (part 6)
needs-coordinator: the acceptance line "a document without the needed `degrades` is refused" contradicts the schema's Degrades.rule (fallbacks[0] applies before refuse); pick one

## 1. Diff

`git --no-pager diff --stat` (tracked files, both the implementor's and not touched by me):

```
 Cargo.lock | 357 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 Cargo.toml |   1 +
```

Untracked files I added, all test files:

```
?? crates/ui/ess-ui-tui/tests/adversary_pass1_live.rs
?? crates/ui/ess-ui-tui/tests/adversary_pass1_placement.rs
?? crates/ui/ess-ui-tui/tests/adversary_pass1_shell.rs
```

No implementation file was touched. After the first run I made lint-only edits to my own test files (rustfmt, backticks in doc comments, a binding renamed for clippy::similar_names). Assertions did not change.

## 2. Cases added (each file first run alone, before the suite)

| case | asserts | now |
|---|---|---|
| live: adv1_a_param_scoped_channel_does_not_leak_into_another_tickets_conversation | tk-02's conversation receives none of tk-01's ticket_chat replies | red |
| live: adv1_only_if_matches_params_drops_a_row_outside_the_search_filter | with q=portal, TicketOpened tk-06 is not inserted | red |
| live: adv1_count_new_while_paged_away_does_not_shift_the_page | on page 2, a new row shows "+1 new" and page 2 keeps its rows | red |
| live: adv1_coalesce_batches_a_burst_into_one_render | two events 300ms apart under coalesce 500ms: no render shows only one of them | red |
| live: adv1_resume_refetch_rereads_the_fed_section_after_a_reconnect | reconnecting→live on a `resume: refetch` channel re-reads tickets.Page | red |
| live: adv1_returning_to_a_page_rereads_its_memory_view_cache | leaving tickets.list and going back (backspace) re-reads it | red |
| live: adv1_stale_after_alone_marks_the_metric_stale | at 46s the metric is stale because of stale_after alone | green |
| live: adv1_a_collection_fed_by_a_stale_channel_is_marked | activity.feed list title carries stale past 60s | green |
| live: adv1_remove_row_patch_row_and_refetch_have_their_effects | each effect, on a document variant | green |
| placement: adv1_storage_files_are_namespaced_by_actor | the draft file path is keyed by us-01 and org-01 | red |
| placement: adv1_no_sensitive_value_reaches_a_state_file | password (users edit), api_key, sign-in email and password are in no state file | green |
| placement: adv1_session_storage_is_empty_at_the_start_of_a_run | the previous run's draft is gone and not shown | green |
| shell: adv1_a_missing_degrades_entry_takes_the_capability_tables_first_fallback | a graph_editor without degrades runs as a collection (schema rule) | red |
| shell: adv1_the_account_menu_is_drawn_and_reachable_by_keyboard | "Sign out" is drawn, or `:switch` opens Switch organization | red |
| shell: adv1_bulk_actions_are_offered_once_rows_are_selected | "Add tag" is shown once a row is selected | red |
| shell: adv1_table_columns_stay_aligned_with_wide_characters | a CJK name does not shift the email column | red |
| shell: adv1_tiny_terminals_and_resizes_never_panic | every page, overlay and the palette, at 9 sizes, with unicode labels | red |
| shell: adv1_a_refusal_inside_a_board_widget_names_the_widget | `no_charts: refuse` on a board widget names …/board…/trend | green |
| shell: adv1_a_failing_fixture_read_shows_failed_and_retries | missing placeholder fixture: failed, "R retries", R → loading → failed | green |
| shell: adv1_empty_result_and_last_page_edges | n past the last page stays on 2/2; search with no match gives Empty + message | green |

Red output as first captured (files run alone; full logs are in the part 6 paths):

```
adv1_a_param_scoped_channel_does_not_leak_into_another_tickets_conversation panicked at adversary_pass1_live.rs:238:5:
tk-02's conversation shows tk-01's replies: ["One more question about the logout URL."]
adv1_coalesce_batches_a_burst_into_one_render panicked at adversary_pass1_live.rs:120:5:
at 1100ms one render shows tk-04 patched (true) and tk-02 patched (false): the 500ms coalesce window did not batch the burst
adv1_only_if_matches_params_drops_a_row_outside_the_search_filter panicked at adversary_pass1_live.rs:152:5:
a live row that does not match q=portal was inserted  left: ["tk-06", "tk-04"]  right: ["tk-04"]
adv1_resume_refetch_rereads_the_fed_section_after_a_reconnect panicked at adversary_pass1_live.rs:201:5:
the tickets channel reconnected with `resume: refetch` and tickets.Page was not re-read
adv1_returning_to_a_page_rereads_its_memory_view_cache panicked at adversary_pass1_live.rs:218:5:
tickets.list was shown again from a cache the schema places in memory (lost on unmount)
adv1_count_new_while_paged_away_does_not_shift_the_page panicked at adversary_pass1_live.rs:176:5:
page 2 shifted under the reader instead of counting the new row:
test result: FAILED. 3 passed; 6 failed          (adversary_pass1_live, EXIT=101)

adv1_storage_files_are_namespaced_by_actor panicked at adversary_pass1_placement.rs:115:5:
the draft file `portal/session_storage.yaml` is keyed by app only, not by actor.user_id and actor.account_id
test result: FAILED. 2 passed; 1 failed          (adversary_pass1_placement, EXIT=101)

adv1_a_missing_degrades_entry_takes_the_capability_tables_first_fallback panicked at adversary_pass1_shell.rs:80:9:
refused although the schema's capability table supplies `collection`: pages/workflows.editor/sections/editor: the tui renderer lacks `no_graph_editor` and this node declares no fallback for it; add `degrades: {no_graph_editor: <collection>}
adv1_table_columns_stay_aligned_with_wide_characters panicked at adversary_pass1_shell.rs:217:5:
the email column of the wide-name row is shifted:
││  name             email                   roles
││  東 京 都 渋 谷 区 パ ー ト ナ ー 株 式 会 社   dana@example.com        #admin
││  Lee Sample       lee@example.com         #finance
adv1_bulk_actions_are_offered_once_rows_are_selected panicked at adversary_pass1_shell.rs:67:5:
the bulk action is dropped silently:  (row shows ✓, no "Add tag" anywhere)
adv1_the_account_menu_is_drawn_and_reachable_by_keyboard panicked at adversary_pass1_shell.rs:51:5:
the account menu region renders nothing and its actions have no key:
adv1_tiny_terminals_and_resizes_never_panic panicked at ratatui-0.29.0/src/buffer/buffer.rs:253:13:
index outside of buffer: the area is Rect { x: 0, y: 0, width: 20, height: 2 } but index is (0, 2)
panicked: [ "<every page> / palette at 1x1", "… / palette at 2x2", "… / palette at 20x2", … ]   (20x5, 20x3, 10x4 do not panic)
test result: FAILED. 3 passed; 5 failed          (adversary_pass1_shell, EXIT=101)
```

## 3. Suite run (after all cases existed, lint-only edits applied)

`CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-ui-tui CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p ess-ui-tui --no-fail-fast`

```
unittests src/lib.rs          ok. 3 passed
adversary_pass1_live.rs       FAILED. 3 passed; 6 failed
adversary_pass1_placement.rs  FAILED. 2 passed; 1 failed
adversary_pass1_shell.rs      FAILED. 3 passed; 5 failed
tests/tui.rs                  ok. 10 passed
doc-tests                     ok. 0 passed
EXIT=101
```

Before = 13, from the same run with my three files left out: 3 unit tests + 10 in tui.rs. That matches the implementor's gate-test.txt. After = 33.
`cargo clippy -p ess-ui-tui --all-targets -- -D warnings` → exit 0. `cargo fmt -p ess-ui-tui -- --check` → exit 0.

## 4. Findings (tree above; origin: the crate does not exist at 4f471fc1e, so every finding is introduced)

| # | file:line | verdict | sev | finding | what reaches it |
|---|---|---|---|---|---|
| 1 | src/app.rs:962 | NEEDS-CHANGE | blocker | `deliver` ignores the channel's `scope: param` / `session: {per: ticket_id}` and the script's `session: {ticket_id: tk-01}`. tk-01's replies and typing indicator show on every open ticket. | example: open tk-02 from the tickets list and wait 3s |
| 2 | src/app.rs:1003 | CONFIRMED | warning | `passes` treats a param the payload lacks as matching, so `q` (search) never filters live rows. | example: tickets.list, search "portal", 0s event |
| 3 | src/app.rs:2417 | CONFIRMED | warning | `count_new` still inserts at index 0, so page 2 shifts and "+1 new" is shown as well. | example: tickets.list on page ≥2 with a live insert |
| 4 | src/app.rs:932 | CONFIRMED | warning | `Live.coalesce` is never read; each event renders on its own. | example: tickets.list declares coalesce: 500ms |
| 5 | src/app.rs:935 | CONFIRMED | warning | A lifecycle beat only sets status; `resume: refetch` and schema `stale → refreshing` on reconnect never re-read. | example: metrics and tickets have resume: refetch; metrics reconnects at 55s |
| 6 | src/app.rs:1024 | CONFIRMED | warning | The read cache is never evicted on navigation, although view_cache resolves to memory ("lost on unmount"). A page you return to shows its old rows with no re-read. | any back or re-open of a page |
| 7 | src/placement.rs:178 | CONFIRMED | warning | The storage dir is `<state>/<app>` only; the schema keys session/local storage by [origin, actor.user_id, actor.account_id]. | every storage write; matters with >1 actor per machine |
| 8 | src/profile.rs:181 | NEEDS-CHANGE | warning | A missing degrades entry is refused. Schema Degrades.rule applies `capabilities.$c.fallbacks[0]` first, so only no_file_upload can refuse by default. tui.rs:183 asserts refusal for `chart: pie` without degrades (schema: table). The story acceptance states the refusal. | any document relying on the schema default |
| 9 | src/view.rs:76 | CONFIRMED | warning | The `account_menu` region is neither drawn nor keyed. Unreachable by keyboard: switch_org overlay, session.SignOut, and so auth.sign_in. | example shell `app` |
| 10 | src/view.rs:750 | CONFIRMED | warning | `bulk_actions` are never rendered or keyed. Selection with space works but leads nowhere. | example partners.list "Add tag" |
| 11 | src/view.rs:1526 | CONFIRMED | warning | pad, truncate and the column widths (view.rs:785) count chars, not cells, so wide characters shift columns and cut lines late. | any CJK or emoji value |
| 12 | src/view.rs:521 | CONFIRMED | warning | The palette popup sits at `area.y + 1` without clamping; height ≤2 with `:` open panics in ratatui's Clear. `run` has no panic guard, so the terminal stays raw. | terminal ≤2 rows with the palette open; 20x5 is fine |
| 13 | tests/tui.rs:301 | CONFIRMED | warning | `every_page_and_overlay_of_the_example_renders` cannot fail: the top bar makes text non-empty, and the hints line always contains "esc" while an overlay is open. An overlay whose body renders nothing passes. (By reading; no mutant run.) | the suite's only "everything renders" check |
| 14 | tests/tui.rs:138 | CONFIRMED | note | `a_stale_channel_marks_its_metric` passes on the script's explicit `lifecycle: stale` at 50s, not on stale_after (reconnecting 15s + 30s = 45s). A mutant ignoring stale_after stays green. adv1_stale_after_alone_marks_the_metric_stale covers it (green). | n/a |

Fixes I would suggest (not applied):
- 1: key script and channel instances by session params.
- 2: compare search params against row text, as the fixture read does.
- 3: skip the insert when count_new applies.
- 4: buffer events until the coalesce window closes.
- 5: re-read fed sections on reconnect.
- 6: drop section cache entries on `go`.
- 7: join the actor ids from session.Me into the state dir.
- 9 and 10: render the region and bulk bar, and give them keys.
- 11: use unicode-width.
- 12: clamp the popup to the frame.

## 5. Attacked, not broken

- Sensitive values never reach a state file: users edit password, settings api_key, sign-in email and password.
- Session storage is cleared at start, and the old draft is not shown.
- remove_row, patch_row and refetch effects behave as declared.
- stale_after marks the metric, and the collection fed by activity, stale.
- A refusal inside a board widget names the widget path.
- A missing placeholder fixture shows failed with "R retries", and R goes failed → loading → failed.
- Paging past the last page stays on the last page; a search with no match gives Empty plus the message.
- 20x5, 20x3, 10x4, 3x30, 200x5 and 27x6 render every page and overlay with long unicode labels without panicking.

## 6. Paths written outside the worktree

- ~/.cache/ess-ui-wave/tui/adv1-live-red.txt
- ~/.cache/ess-ui-wave/tui/adv1-adversary_pass1_placement-red.txt
- ~/.cache/ess-ui-wave/tui/adv1-adversary_pass1_shell-red.txt
- ~/.cache/ess-ui-wave/tui/adv1-suite.txt
- ~/.cache/ess-ui-wave/tui/adv1-clippy.txt
- ~/.cache/ess-ui-wave/tui/adv1-review.md (this file)
- ~/.cache/b10x-target/ess-ui-tui/tmp/ess-ui-tui-adv1/ (the tests' state dirs, via CARGO_TARGET_TMPDIR) plus build artefacts in the assigned build dir

## 7. Findings block

```findings
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 962
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: live delivery ignores channel scope param and session per ticket_id, so one ticket's chat replies and typing state appear on every other ticket's page
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1003
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: only_if matches(params) passes any param the payload lacks, so the search filter q never drops live rows
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 2417
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: when_paged_away count_new still inserts the row, shifting the page under the reader
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 932
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: Live.coalesce is never read; a burst renders event by event
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 935
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a reconnect never re-reads fed sections although the channel declares resume refetch
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1024
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the read cache survives navigation although view_cache resolves to memory, so a page revisited is not re-read
- file: crates/ui/ess-ui-tui/src/placement.rs
  line: 178
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: storage files are namespaced by app only, not by actor.user_id and actor.account_id as the schema keys them
- file: crates/ui/ess-ui-tui/src/profile.rs
  line: 181
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a missing degrades entry is refused where the schema rule applies the capability table's first fallback, and the acceptance line agrees with the code, not the schema
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 76
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the account_menu region is not drawn and has no key, so switch_org, sign out and the sign-in page are unreachable by keyboard
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 750
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: collection bulk_actions are silently dropped, neither rendered nor bound to a key
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 1526
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: column padding and truncation count chars not terminal cells, so wide characters misalign table columns
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 521
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the palette popup is placed below the frame on terminals of two rows or fewer and ratatui panics
- file: crates/ui/ess-ui-tui/tests/tui.rs
  line: 301
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the every-page-and-overlay test asserts text that the top bar and hints line always supply, so it cannot fail for an empty render
- file: crates/ui/ess-ui-tui/tests/tui.rs
  line: 138
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the stale test is satisfied by the script's explicit stale beat and would stay green with stale_after ignored
```
