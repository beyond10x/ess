---
format: aep.planning-md/3
id: story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in
kind: story
status: active
title: A running terminal app forks into a copy that shares no mutable state with it, keeps file-placed state in memory when asked, and says when its next event is due
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:18:56Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:18:57Z", actor: "human:timo", revision: 5}
---
## Need

A program that explores UI states branches at every state. It takes each offered step on a copy of
the state and keeps the original for the next step. Today an app cannot be copied, and two apps in
one process are not independent.

- `ess_ui_tui::App` (`crates/ui/ess-ui-tui/src/app.rs:433-489`) has no `Clone`. The fields in the
  way:
  - `doc: Document` by value (`:435`). A derived `Clone` would copy the whole document on every
    fork.
  - `adapter: Box<dyn DataAdapter>` (`:438`). The trait has no way to copy an adapter
    (`data.rs:53-63`).
  - `store: StateStore` (`:441`), which derives only `Debug` (`placement.rs:179-188`).
  - The `RefCell` drawing caches (`:477-488`). These clone without trouble.
- The store keeps local and session storage as files named `<store>.yaml` under
  `<state_dir>/<app>/<user>/<account>/`.
  - It reads them on every access (`placement.rs:339-344`) and writes them on every change
    (`:346-351`).
  - It deletes `session_storage.yaml` when it rekeys or signs out (`:231`, `:240`).
  - So two apps over one `Options::state_dir` (`app.rs:27-31`) share one storage, and a copy that
    wrote there would change its origin.
- `FixtureAdapter` (`data.rs:82-97`) derives `Debug` and `Default` but not `Clone`. All its fields
  hold `Clone` types (`FixtureEffects` is `Clone`, `crates/ui/ess-ui/src/binding.rs:122`). It
  holds the command log and the rows the model's effects changed.
- `App` does not say when the next thing is due. The terminal's player keeps when each script
  entry is due (`live.rs:93-140`), but nothing exposes it, so a caller cannot offer an `advance`
  step to the right instant.

The requester's caller explores thousands to millions of states per journey, so the cost per fork
decides whether this is usable. The requester measured, in process: open 1.2 ms and draw 0.4 ms.
This is the base of report item 1. The design is `g01-state-design`, and the `ess-ui-test`
session is built on it in `g01-session-actions`.

## Outcome

- `App::fork_with(&self, adapter: Box<dyn DataAdapter>) -> Result<App, TuiError>` returns an app
  equal to `self` in every field except the adapter, which the caller supplies as a copy of its
  own. Nothing mutable is shared afterwards.
- `App::doc` becomes `Rc<Document>`, and the plan and options are shared the same way. A fork
  copies the mutable state only, as `g01-state-design` lists in its Forks section.
- `Options` gains an in-memory storage mode, `Options::in_memory()`. In that mode the store keeps
  local and session storage as maps of its own and never touches the file system. `StateStore`
  becomes `Clone`.
- `fork_with` refuses an app whose store keeps files, naming why: the fork would share its
  origin's storage.
- `FixtureAdapter` derives `Clone`.
- `App::next_due(&self) -> Option<Duration>` gives the next instant at which any of these happens:
  a script entry plays, a read is answered, a poll runs, or a coalesced batch is delivered. It
  reads a new `Player::next_at` in `live.rs`.
- File mode, `Options::new(state_dir)` and every existing behaviour are unchanged.

## Acceptance

New test file `crates/ui/ess-ui-tui/tests/fork.rs`. Each test is expected under 3 s and is timed;
none may run over 60 s, and the crate's test step stays within 120 s. The file joins the crate's
single test binary of `story:the-test-step-meets-the-test-budget` if that has landed; otherwise it
is a new file, which that story moves.

1. `a_fork_and_its_origin_move_apart`. Red today: `fork_with` does not exist.
   - Setup: `examples/partner-portal/ui.yaml` opens in memory mode with a `FixtureAdapter`, and
     the app is forked.
   - On the fork: type into the partner list's search, move the clock by 5 s, and open an
     overlay.
   - The origin's `rows`, `regions`, `location`, `section_state` and drawn buffer are byte-equal
     to their values before the fork. The fork's differ.
2. `a_fork_draws_what_its_origin_draws`. Right after the fork, both draw equal `TestBackend`
   buffers and equal `regions()`. After the same key sequence on both, they are still equal. This
   holds for five named key sequences on `examples/partner-portal/ui.yaml`, and one on a document
   with a channel script.
3. `memory_storage_writes_no_file`. The test writes a minimal document with one `local_storage`
   preference and one `session_storage` draft, each changed by an action that `sets` it.
   - It runs the app in memory mode, with `HOME` and `TMPDIR` set to two empty directories of the
     test, and lists both directories before and after: neither gains an entry.
   - A file-mode control over a third directory writes both files there.
4. `a_file_backed_app_does_not_fork`. `fork_with` on an app opened with `Options::new(dir)` returns
   `Err`, and the message names the shared storage directory.
5. `a_forked_store_signs_out_alone`. Signing out on the fork clears its session storage and memory.
   The origin still holds its own.
6. `a_fork_shares_the_document`. After a fork, `Rc::ptr_eq` holds for the two apps' documents.
7. `next_due_is_the_next_scripted_instant`. With a channel script whose next entry is at 2 s,
   `next_due` is 2 s. After `advance(2 s)` the entry has played. With no script, no pending read
   and no poll, `next_due` is `None`.
8. `the_cost_of_open_draw_and_fork`. On `examples/partner-portal/ui.yaml` in memory mode, the test
   times 200 opens, 200 draws and 200 forks of the home page state. It prints one line,
   `cost: open=<µs> draw=<µs> fork=<µs>`, with the median of each in microseconds.
   - The first measurement, taken by the implementer on this host, is pinned in the test as
     `OPEN_CEILING_US`, `DRAW_CEILING_US` and `FORK_CEILING_US`, each three times the measured
     median. The report names the measured values.
   - The test fails when a median exceeds its ceiling, and it ends within 20 s.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui-tui --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui-tui --locked`. No test of another crate changes.

Also:
- The rustdoc of `App` and `Options` says what a fork shares (the document, the plan and the
  options, all immutable) and why file mode refuses.
- `CHANGELOG.md` names `App::fork_with`, `App::next_due` and the memory mode under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 1. The full review is in `g01-state-design`. This unit is its first code step.

1. **Need.** A copy of a running UI state, made cheaply enough for millions of states. The
   reproduction is above: nothing can copy an `App`, and two apps over one directory share
   storage.
2. **Class.** It is a *gap*.
3. **Already expressible?** No. A caller could build a second app over another `state_dir` and
   replay every key, which is the cost the request removes.
4. **Fit.** No authored surface is added and no format changes. It reuses the store's own
   placement rules: memory mode changes where storage lives, not what survives a sign-out
   (`placement.rs:222-245`).
5. **Second adopter.** A property test of the terminal renderer that compares two branches from
   one state.
6. **Cost.**
   - New public Rust API in `ess-ui-tui`. It is additive. Existing callers compile unchanged
     unless they build `Options` with a struct literal, which a new field breaks; the
     constructors avoid that.
   - `App::doc` changes type inside the crate. It is `pub(crate)`, so no caller outside sees it.
   - No diagnostic, format or generated byte moves.
7. **Considered.**
   - (a) A `fork()` method on the `DataAdapter` trait. Rejected for now: the runner holds its
     adapter behind an `Rc` that it reads too (`runner.rs:29-53`), so the caller must build the
     copy itself.
   - (b) Copying the storage files into a fresh directory per fork. Rejected: it costs a file copy
     per branch and leaves directories behind.
   - (c) A derived `Clone` that copies the document. Rejected: the cost would grow with the
     document on every fork.
   - (d) Chosen: `fork_with` over a shared document, plus memory mode.

## Decisions

None of its own; it follows P0-D1 and P0-D2 (`g01-state-design`). If P0-D2 is answered (B), this
unit is still needed.

## Scope

- `crates/ui/ess-ui-tui/src/app.rs`:
  - `Options` 26-50, and `App` 434-489 (`doc` and `plan` behind `Rc`);
  - a new `fork_with` after `with_adapter` (510-524) and `bound` (526-602);
  - `next_due` beside `advance` (798).
- `crates/ui/ess-ui-tui/src/placement.rs`: `StateStore` 179-352 (`new`, `rekey`, `sign_out`,
  `file`, `write_file`).
- `crates/ui/ess-ui-tui/src/data.rs`: `FixtureAdapter` 82-97 (derive `Clone`).
- `crates/ui/ess-ui-tui/src/live.rs`: `Player::next_at`, 93-140.
- `crates/ui/ess-ui-tui/src/lib.rs`: exports.
- new: `crates/ui/ess-ui-tui/tests/fork.rs`.
- `CHANGELOG.md`.
- Overlap: active stories edit `app.rs`, among them
  `story:a-form-groups-own-action-runs-from-its-key`,
  `story:the-terminal-pages-a-server-read-through-its-pager-state` and
  `story:the-terminal-evaluates-every-expression-form-the-schema-defines`. None of them touches
  `Options`, the `App` struct, `next_due` or `placement.rs`.
- Size: M, about 400 changed lines: code about 170, tests about 230.

## Depends on

Nothing. Round 1, beside `g01-state-design` and `g03-loader-fills-defaults`, with which it shares
no file. `g03-actor-from-session` (round 2) edits `app.rs` after it.
