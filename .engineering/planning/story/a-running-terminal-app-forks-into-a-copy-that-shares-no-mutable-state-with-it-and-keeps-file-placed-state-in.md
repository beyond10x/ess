---
format: aep.planning-md/3
id: story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in
kind: story
status: draft
title: A running terminal app forks into a copy that shares no mutable state with it, and keeps file-placed state in memory when asked
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A program that explores UI states branches at every state: it takes each offered step on a copy of
the state and keeps the original for the next step. Today an app cannot be copied, and two apps in
one process are not independent.

- `ess_ui_tui::App` (`crates/ui/ess-ui-tui/src/app.rs:433-489`) has no `Clone`. Three fields stand
  in the way:
  - `adapter: Box<dyn DataAdapter>` (`:438`). The trait has no way to copy an adapter
    (`data.rs:53-63`).
  - `store: StateStore` (`:441`), which derives only `Debug` (`placement.rs:179-188`).
  - The `RefCell` drawing caches (`:477-488`). These clone without trouble.
- The store keeps local and session storage as files named `<store>.yaml` under
  `<state_dir>/<app>/<user>/<account>/`. It reads them on every access (`placement.rs:339-344`)
  and writes them on every change (`:346-351`). It deletes `session_storage.yaml` when it rekeys or
  signs out (`:231`, `:240`). Two apps over one `Options::state_dir` (`app.rs:27-31`) therefore
  share one storage. A copy that wrote there would change its origin.
- `FixtureAdapter` (`data.rs:82-97`) derives `Debug` and `Default` but not `Clone`. Its fields all
  hold `Clone` types (`FixtureEffects` is `Clone`, `crates/ui/ess-ui/src/binding.rs:122`). It holds
  the command log and the rows the model's effects changed.

This is the base of report item 1. The design is `g01-state-design`, and the `ess-ui-test` session
is built on it in `g01-session-actions`.

## Outcome

- `App::fork_with(&self, adapter: Box<dyn DataAdapter>) -> Result<App, TuiError>` returns an app
  equal to `self` in every field except the adapter, which the caller supplies as a copy of its own.
  Nothing mutable is shared afterwards.
- `Options` gains an in-memory storage mode (for example `Options::in_memory()`). In that mode the
  store keeps local and session storage as maps of its own and never touches the file system.
  `StateStore` becomes `Clone`.
- `fork_with` refuses an app whose store keeps files, naming why: the fork would share its
  origin's storage.
- `FixtureAdapter` derives `Clone`.
- The file mode, `Options::new(state_dir)` and every existing behaviour are unchanged.

## Acceptance

- New test file `crates/ui/ess-ui-tui/tests/fork.rs` (the crate has one test binary per file today;
  consolidation belongs to `story:the-test-step-meets-the-test-budget`). Time each test: each is
  expected under 3 s, none may run over 60 s, and the file's step stays within 120 s.
  1. `a_fork_and_its_origin_move_apart`. Red today: `fork_with` does not exist.
     - Setup: `examples/partner-portal/ui.yaml` opens in memory mode with a `FixtureAdapter`, and the
       app is forked.
     - On the fork: type into the partner list's search, move the clock by 5 s, and open an
       overlay.
     - The origin's `rows`, `regions`, `location`, `section_state` and drawn buffer are
       byte-equal to their values before the fork. The fork's differ.
  2. `a_fork_draws_what_its_origin_draws`. Right after the fork, both draw equal `TestBackend`
     buffers and equal `regions()`. After the same key sequence on both, they are still equal.
     This holds over at least five key sequences on `examples/partner-portal/ui.yaml` and one
     document with a channel script.
  3. `memory_storage_writes_no_file`. The test writes a minimal document with one
     `local_storage` preference and one `session_storage` draft, each changed by an action that
     `sets` it. In memory mode `state_dir` is never created. The same run in file mode writes both
     files (a control).
  4. `a_file_backed_app_does_not_fork`. `fork_with` on an app opened with `Options::new(dir)`
     returns `Err`, and the message names the shared storage directory.
  5. `a_forked_store_signs_out_alone`. Signing out on the fork clears its session storage and
     memory. The origin still holds its own.
- `cargo clippy -p ess-ui-tui --all-targets --locked -- -D warnings` and
  `cargo test -p ess-ui-tui --locked` pass. No other crate's test changes.
- The `App` and `Options` rustdoc says what a fork shares (nothing) and why file mode refuses.
  `CHANGELOG.md` names `App::fork_with` and the memory mode under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 1. The full review is in `g01-state-design`. This unit is its first code step.

1. **Need.** A copy of a running UI state. The reproduction is above: nothing can copy an `App`,
   and two apps over one directory share storage.
2. **Class.** This is a *gap*.
3. **Already expressible?** No. A caller could build a second app over another `state_dir` and
   replay every key, which is the cost the request removes.
4. **Fit.** No authored surface is added and no format changes. It reuses the store's own
   placement rules: memory mode changes where storage lives, not what survives a sign-out
   (`placement.rs:222-245`).
5. **Second adopter.** A property test of the terminal renderer that compares two branches from
   one state.
6. **Cost.**
   - New public Rust API in `ess-ui-tui`. It is additive, so existing callers compile unchanged
     unless they build `Options` with a struct literal: a new field breaks such literals. The
     constructors avoid that.
   - No diagnostic, format or generated byte moves.
7. **Considered.**
   - (a) A `fork()` method on the `DataAdapter` trait. Rejected for now: the runner holds its
     adapter behind an `Rc` that it reads too (`runner.rs:29-53`), so the caller must build the
     copy itself.
   - (b) Copying the storage files into a fresh directory per fork. Rejected: it costs a file copy
     per branch and leaves directories behind.
   - (c) Chosen: `fork_with` plus memory mode.

## Decisions

None of its own; it follows P0-D1 and P0-D2 (`g01-state-design`). If P0-D2 is answered (B), this
unit is still needed.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs`: `Options` 26-50, `App` 434-489, a new `fork_with` after
  `with_adapter` (510-524) and `bound` (526-602)
- cited: `crates/ui/ess-ui-tui/src/placement.rs`: `StateStore` 179-352 (`new`, `rekey`, `sign_out`,
  `file`, `write_file`)
- cited: `crates/ui/ess-ui-tui/src/data.rs`: `FixtureAdapter` 82-97 (derive `Clone`)
- cited: `crates/ui/ess-ui-tui/src/lib.rs` (exports)
- new: `crates/ui/ess-ui-tui/tests/fork.rs`; `CHANGELOG.md`
- Overlap: active stories edit `app.rs`, among them
  `story:a-form-groups-own-action-runs-from-its-key`,
  `story:the-terminal-pages-a-server-read-through-its-pager-state` and
  `story:the-terminal-evaluates-every-expression-form-the-schema-defines`. None of them touches
  `Options`, the `App` struct or `placement.rs`.
- Size: M, about 300 changed lines (code about 120, tests about 180).

## Depends on

Nothing. It can be built at once, beside `g01-state-design` and `g03-loader-fills-defaults`.
