---
format: aep.planning-md/3
id: review-result:adversary-ui-tui-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-ui wave unit ui-spec-tui-renderer
relations:
- reviews: story:ui-spec-tui-renderer
revision: 1
---
unit: story:ui-spec-tui-renderer — working tree ~/.local/state/worktree/trees/b10x/ess/ess-ui-tui (base 4f471fc1e, uncommitted crate crates/ui/ess-ui-tui, after correction 1)
verdict: NEEDS-CHANGE
cases: executed 35→48, red 12
origin: introduced 12 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: none

## 1. Diff

`git --no-pager diff --stat` (tracked files, the implementor's, untouched by me):

```
 Cargo.lock | 358 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 Cargo.toml |   1 +
```

Untracked files I added, all test files:

```
crates/ui/ess-ui-tui/tests/adversary_pass2_live.rs
crates/ui/ess-ui-tui/tests/adversary_pass2_placement.rs
crates/ui/ess-ui-tui/tests/adversary_pass2_shell.rs
```

No implementation file touched. After the red runs I ran rustfmt on adversary_pass2_placement.rs; it wrapped one `assert_eq!`. No assertion changed.

## 2. Cases added (each file run alone before the suite)

| case | asserts | now |
|---|---|---|
| live: adv2_a_held_row_patched_while_held_is_counted_once | page 2, tk-06 opened at 0s and patched at 18s: footer "+1 new" | red |
| live: adv2_a_held_row_is_released_with_its_latest_patch | same, then `p`: tk-06 priority urgent, subject kept | red |
| live: adv2_a_search_back_to_page_one_does_not_strand_the_held_row | search from page 2 → page 1: tk-06 shown, or no "+1 new" | red |
| live: adv2_an_event_at_the_reconnect_instant_is_not_lost_to_the_refetch | no-coalesce variant; live and TicketUpdated both at 2s: tk-04 high | red |
| live: adv2_a_coalesced_batch_is_not_lost_to_a_refetch_in_the_same_step | batch opened at 1.0s, reconnect at 1.1/1.2s, one 2s step: tk-04 high | red |
| live: adv2_an_event_that_ends_a_reconnect_refetches | reconnecting then an event, no `live` beat: tickets.Page re-read | red |
| live: adv2_another_tickets_session_status_does_not_show_in_the_header | tk-02 header at 16s does not show tk-01's "ticket_chat reconnecting" | red |
| live: adv2_the_typing_indicator_shows_in_its_own_session | tk-01 at 1s draws "Someone is typing" | red |
| live: adv2_a_script_session_without_the_per_param_does_not_reach_every_ticket | session `{ticket: tk-01}` does not reach tk-02 | red |
| placement: adv2_sign_out_clears_the_session_storage_draft | reply draft gone after sign out and sign in | red |
| placement: adv2_sign_out_clears_memory_drafts | users edit drawer memory draft gone after sign out and sign in | red |
| placement: adv2_empty_ids_do_not_collide_in_the_storage_key | (user "", acct) and (acct, "") do not share a store | red |
| shell: adv2_a_bulk_key_at_zero_selected_rows_runs_nothing | bulk key with 0 selected, or after deselect, runs nothing; with 1 selected it runs | green |

First red output (full logs in part 6):

```
adv2_a_held_row_patched_while_held_is_counted_once panicked at adversary_pass2_live.rs:115:5:
one new row (tk-06) was opened and then patched while held; the footer counts events:
│  Invoices              ││page 2/3 · 5 rows · +2 new
adv2_a_held_row_is_released_with_its_latest_patch panicked at adversary_pass2_live.rs:138:5:
the held rows were replayed newest first, so the 0s open overwrote the 18s patch: {"id": "tk-06", "priority": "normal", "updated_at": "2026-09-30T10:01:00Z", "subject": …}
  left: "normal"  right: "urgent"
adv2_a_search_back_to_page_one_does_not_strand_the_held_row panicked at adversary_pass2_live.rs:158:5:
on page 1 the held row tk-06 is neither shown nor released, and "+1 new" stays:
│  Invoices              ││page 1/3 · 5 rows · +1 new
adv2_an_event_at_the_reconnect_instant_is_not_lost_to_the_refetch panicked at adversary_pass2_live.rs:196:5:
TicketUpdated tk-04 arrived with the reconnect and was dropped because the section was loading   left: "urgent"  right: "high"
adv2_a_coalesced_batch_is_not_lost_to_a_refetch_in_the_same_step panicked at adversary_pass2_live.rs:221:5:
the coalesced TicketUpdated was flushed into a loading section and dropped   left: "urgent"  right: "high"
adv2_an_event_that_ends_a_reconnect_refetches panicked at adversary_pass2_live.rs:245:5:
the channel went reconnecting → live (by an event) and tickets.Page was not re-read
adv2_another_tickets_session_status_does_not_show_in_the_header panicked at adversary_pass2_live.rs:260:5:
tk-02's header shows tk-01's session lifecycle:  Partner portal │ Ticket │ /tickets.detail?id=tk-02   ○ ticket_chat reconnecting
adv2_the_typing_indicator_shows_in_its_own_session panicked at adversary_pass2_live.rs:274:5:
TypingStarted for tk-01 played at 0s and the typing icon is not drawn on tk-01:   (reply section drawn with [ Send ], no icon line)
adv2_a_script_session_without_the_per_param_does_not_reach_every_ticket panicked at adversary_pass2_live.rs:299:5:
a session without ticket_id was delivered to tk-02: ["stray"]
test result: FAILED. 0 passed; 9 failed          (adversary_pass2_live, EXIT=101)

adv2_empty_ids_do_not_collide_in_the_storage_key panicked at adversary_pass2_placement.rs:119:5:
actor (user "", account acct) and actor (user acct, account "") share a storage directory   left: String("first-actor")  right: Null
adv2_sign_out_clears_memory_drafts panicked at adversary_pass2_placement.rs:82:5:
the memory draft typed before sign-out is shown after signing in again:   │name             [previous-actor-name]
adv2_sign_out_clears_the_session_storage_draft panicked at adversary_pass2_placement.rs:55:5:
the reply draft typed before sign-out is shown after signing in again:
││body             [previous-actor-draft]
││✎ [previous-actor-draft          ] (expression)
test result: FAILED. 0 passed; 3 failed          (adversary_pass2_placement, EXIT=101)

adv2_a_bulk_key_at_zero_selected_rows_runs_nothing ... ok     (adversary_pass2_shell, EXIT=0)
```

## 3. Suite run (after all cases existed)

`CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-ui-tui CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p ess-ui-tui --no-fail-fast`

```
unittests src/lib.rs          ok. 4 passed
adversary_pass1_live.rs       ok. 9 passed
adversary_pass1_placement.rs  ok. 3 passed
adversary_pass1_shell.rs      ok. 8 passed
adversary_pass2_live.rs       FAILED. 0 passed; 9 failed
adversary_pass2_placement.rs  FAILED. 0 passed; 3 failed
adversary_pass2_shell.rs      ok. 1 passed
tests/tui.rs                  ok. 11 passed
doc-tests                     ok. 0 passed
EXIT=101
```

Before = 35, from correction 1's reported gate (c1-gate-test.txt: 4 + 9 + 3 + 8 + 11). The same five binaries ran the same counts here. After = 48. Clippy not run (disk brief: one suite run only). `rustfmt --check` on my three files → exit 0.

## 4. Findings (tree above; the crate does not exist at 4f471fc1e, so every finding is introduced)

| # | file:line | verdict | sev | finding | what reaches it |
|---|---|---|---|---|---|
| 1 | src/app.rs:1177 | NEEDS-CHANGE | warning | Paged away, every event for a held id adds 1 to `new_rows` and pushes another payload. One opened-then-patched row reads "+2 new". | example tickets script (tk-06 at 0s and 18s) on any page ≥2; needs >page_size rows (10 by default; tests use 2) |
| 2 | src/app.rs:1240 | NEEDS-CHANGE | blocker | `insert_pending` replays held payloads newest first. The oldest wins the merge, so tk-06 comes back as `normal`, not `urgent`, and stays wrong. | same as 1, then `p` to page 1 |
| 3 | src/app.rs:2439 | CONFIRMED | warning | The search/filter prompt sets `page = 0` (also :2458) without `insert_pending`. On page 1 the held row is never shown and "+1 new" stays until the user presses `p` on page 1. | any filter or search entered while paged away with held rows |
| 4 | src/app.rs:1149 | CONFIRMED | warning | `apply_live` drops an event when the fed section is Loading. With `resume: refetch`, an event due in the same step as the `live` beat (or during read_latency) is lost. The fixture refetch never brings it back. | the example's scripts put `live` and an event at one instant (activity 20s, metrics 55s); on tickets.list, coalesce hides it except in one clock step; a list without coalesce loses it every time |
| 5 | src/app.rs:1049 | CONFIRMED | warning | `flush` sends coalesced payloads through the same guard. A batch whose window closes while a reconnect refetch is loading is discarded whole. | one `advance` spanning an event, a reconnect and the window close; drive's 100ms step makes it rare |
| 6 | src/app.rs:981 | INFEASIBLE | note | An event after `reconnecting` sets status `live` without the refetch that the `live` beat triggers under `resume: refetch`. | no example script ends a reconnect without a `live` beat |
| 7 | src/view.rs:129 | CONFIRMED | warning | `channel_status` is per channel, not per session. tk-02's header shows tk-01's `ticket_chat` connecting/live/reconnecting. | example: tickets.detail tk-02, 15–18s of every 18s cycle |
| 8 | src/app.rs:2609 | NEEDS-CHANGE | warning | `channel.ticket_chat.typing` resolves to `latest["typing"]`, a key no payload has: the prose derived field is dropped by the key filter. "Someone is typing" never shows, even on tk-01. This is the only visible use of the session-scoped signal. | example: tickets.detail tk-01, any time |
| 9 | src/app.rs:1068 | INFEASIBLE | note | A script whose `session` lacks the channel's `per` key fails open and reaches every ticket's page. | nothing found; the example's script names ticket_id |
| 10 | src/placement.rs:205 | NEEDS-CHANGE | blocker | session_storage is cleared only when the store is built. Sign out and sign in again, and the reply draft is still shown and on disk. Schema: session_storage `survives_signout: false`, `clear_on` default `[signout, account_switch]`. | example: type a reply on tk-01, `:sign out`, sign in, open tk-01 |
| 11 | src/app.rs:2227 | NEEDS-CHANGE | blocker | The SignOut branch sets `signed_out` and navigates. Memory state (drafts incl. sensitive ones, selection) stays. The users edit drawer shows the previous name after the next sign-in. Schema: memory `survives_signout: false`. | example: users.list edit drawer, `:sign out`, sign in, `e` |
| 12 | src/placement.rs:318 | INFEASIBLE | note | `segment(Some(""))` is `""` and `Path::join("")` adds no directory, so (user "", account X) and (user X, account "") share one directory. The doc comment at :192 says "no two keys share a directory". | nothing found: needs an actor whose id is the empty string |

Judgement, not made into a case:
- src/app.rs:222: the actor ids behind the storage key are read once, in `with_adapter`. After a sign-in as another actor, or `session.SwitchOrganization`, the store keeps the first actor's directory. A case needs an adapter whose `session.Me` changes, and the fixture adapter's never does. Fixing 10 without re-keying leaves this open. undecided reach, introduced, warning.
- src/lib.rs:100–116: when `drive` panics (including inside `advance`/`deliver`), the hook restores the terminal before the message prints. But the code that reinstalls the previous hook is never reached. After a panic the process keeps the restore hook, which writes LeaveAlternateScreen to stdout on later panics. Checked by reading only: `run` needs a TTY. note.

Fixes I would suggest (not applied):
- 1–2: hold held rows keyed by match field and merge in order (oldest first).
- 3: release held rows wherever `page` becomes 0.
- 4–5: buffer events for a Loading section and apply them once its read is Ready.
- 7: status per session instance.
- 8: compute derived channel fields (typing from a TypingStarted younger than 5s).
- 10–11: on SignOut, clear memory, session_storage and the server_session placements, and re-key the store from the next actor.
- 12: encode an empty segment distinctly, e.g. `%`.

## 5. Attacked, not broken

- A bulk key at zero selected rows, or after the only selection is removed, runs nothing. With one row selected it runs partners.TagPartners.
- Switching tickets through navigation stops the old session's rows: `in_session` holds; tk-02 gets no tk-01 replies (pass-1 case still green).
- Navigation clears coalesce batches, held rows and memory view cache (`go`, app.rs:1307–1316).
- A coalesced batch whose section is paged away at flush is counted, not inserted. Held rows are re-checked against `only_if` when released.
- Degrade chains: graph_editor → collection builds a collection with no reorder, so it needs no no_drag. A declared `refuse` and an unknown fallback both refuse with the node path. No fallback the TUI implements needs another capability it lacks. Checked by reading profile.rs and app.rs:2876.
- Storage segments: `/`, `..` and `%` are encoded, and absent (`=`) differs from every encoded value. Only the empty string collides (finding 12).

## 6. Paths written outside the worktree

- ~/.cache/ess-ui-wave/tui/adv2-live-red.txt
- ~/.cache/ess-ui-wave/tui/adv2-placement-red.txt
- ~/.cache/ess-ui-wave/tui/adv2-shell-run.txt
- ~/.cache/ess-ui-wave/tui/adv2-suite.txt
- ~/.cache/ess-ui-wave/tui/adv2-review.md (this file)
- plus the tests' state dirs under ~/.cache/b10x-target/ess-ui-tui/tmp/ess-ui-tui-adv2/ (CARGO_TARGET_TMPDIR) and build artefacts in the assigned build dir

## 7. Findings block

```findings
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1177
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: while paged away each event for an already-held id increments the new-row count, so one opened-then-patched row reads +2 new
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1240
  category: concurrency
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: held payloads are replayed newest first, so an older event overwrites a newer patch and the released row shows stale data
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 2439
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the filter prompt resets the page to 0 without releasing held rows, leaving +N new on page 1 and the row unshown
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1149
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a live event reaching a section whose resume-refetch read is loading is dropped, so an event at the reconnect instant is lost
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1049
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a coalesced batch flushed while a reconnect refetch is loading is discarded whole
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 981
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an event ending a reconnect flips status to live without the resume-refetch re-read the live beat triggers
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 129
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: channel status is per channel not per session, so another ticket's session lifecycle shows in this ticket's header
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 2609
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the derived channel field typing resolves to a payload key that never exists, so the typing indicator never shows
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 1068
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a script session lacking the per param fails open and is delivered to every session's page
- file: crates/ui/ess-ui-tui/src/placement.rs
  line: 205
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: session_storage is not cleared on sign-out, so the previous session's reply draft is shown after signing in again
- file: crates/ui/ess-ui-tui/src/app.rs
  line: 2227
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: sign-out keeps memory state including sensitive drafts, which reappear after the next sign-in
- file: crates/ui/ess-ui-tui/src/placement.rs
  line: 318
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an empty id segment adds no directory, so two distinct actors can share one storage directory despite the doc comment
```
