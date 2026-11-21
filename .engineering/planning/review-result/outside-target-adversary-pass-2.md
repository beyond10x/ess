---
format: aep.planning-md/3
id: review-result:outside-target-adversary-pass-2
kind: review-result
status: active
title: 'Adversary, pass 2: the fixes of pass 1 in the conformance target process'
relations:
- reviews: story:a-conformance-run-drives-a-target-process-in-any-language
revision: 1
---
I wrote 5 new test cases: 1 is red (a defect). The other 4 are green, and one of those kills a mutant the unit's own tests let through. I also have 4 judgement findings.

```
unit: story:a-conformance-run-drives-a-target-process-in-any-language, eb18d0e5fb + my 2 untracked test files (worktree ess-outside-target)
verdict: CONFIRMED (1 red case)
cases: executed 73→78, red 1  (73 = the unit's tests, run after my cases existed with my two files not selected: lib process 48, conformance pass1 6, CLI process_target 17, CLI pass1 2)
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: 1 directory, adv2/ under the session scratch directory (scripts and logs, 232 KB left)
needs-coordinator: whether a signalled ess should re-raise the signal instead of exiting 128+n (judgement finding 1)
```

**1. Diff.** I changed no tracked file; my only additions are the two untracked test files. The worktree also shows changes to `crates/verify/ess-conformance/tests/adversary_process_target_pass1.rs` and `crates/edge/ess-cli/tests/process_target_guide.rs`. Those are not mine; another agent made them.

**2. Cases**
- `crates/verify/ess-conformance/tests/adversary_process_target_pass2.rs`
  - `a_restart_naming_the_escape_text_of_the_first_starts_control_character_is_another_implementation` is **red** (defect, small). It breaks the claim that a restarted target naming a different implementation is refused (`process.rs:404`, checked at `:626`). The handshake name is stored escaped (`process/wire.rs:242-246`), but a backslash is left as it is. So `billing`+ESC and the literal text `billing\u{1b}` both become `billing\u{1b}`. The restart is admitted, and the report names one string for two different names. Output when it ran alone: `panicked at …pass2.rs:136:5: a restart naming `billing\u{1b}` (backslash, u, braces) after a first start naming `billing` + ESC is admitted as the same implementation`. Both control assertions (same name twice admitted, a different name refused) passed. It is red with `arbitrary_precision` off and on. What reaches it: only a target that changes its own name between restarts, which gains it nothing. The fix is to escape `\` too, or compare the raw names.
  - `the_single_pass_reader_reads_generated_answers_as_the_reader_it_replaced` is green. It compares 3,000 lines from a fixed-seed generator plus 7 fixed lines against the old reader, restated in the test. The comparison runs through the public answer path and checks the value, the refusal kind and the byte offset. The corpus covers duplicates at every depth, lone surrogates in keys and values, refused numbers, depths 120–133, 100 000 `[`, trailing garbage and every whitespace form. All four answer kinds appear at least 20 times. Green with the feature off and on.
- `crates/edge/ess-cli/tests/adversary_process_target_pass2.rs`, all green:
  - `a_target_leader_that_left_its_process_group_is_still_ended_with_the_run` kills mutant (a).
  - `a_sigterm_during_the_end_grace_exits_143_kills_the_target_and_leaves_an_earlier_report_as_it_was`
  - `a_sigint_and_a_sigterm_during_the_handshake_exit_130_and_kill_the_target`
  - Each wait is bounded (120 s, 60 s plus 10 s, 30 s plus 10 s) and the reason is in the test's comment. A `Drop` kills ess and every pid the target recorded, red or green. The targets are `/bin/sh` and `/usr/bin/perl`.

**3. Suite after my cases.** lib `process` 48 passed; conformance pass1 6 passed; CLI process_target 17 passed; CLI pass1 2 passed; CLI pass2 3 passed; conformance pass2 1 passed, 1 failed (the case above). Clippy with `-D warnings` and rustfmt are clean on both my files.

**4. Judgement findings** (on eb18d0e5fb)
- `crates/edge/ess-cli/src/target_process.rs:78`: ess exits with status 128+n instead of dying of the signal. When bash gets Ctrl-C and its child *exits* 130 rather than dying of SIGINT, it assumes the child handled the interrupt and carries on. So a shell loop over `--target process` runs no longer stops on Ctrl-C; before this commit it did. I wrote no test because the design asks for 128+n. Note / CONFIRMED / introduced.
- `target_process.rs:60`: only the first signal is handled; later ones are swallowed. If the reap in `end_on` blocks (a leader that cannot be killed: EPERM after it runs a setuid program that changes its real uid, or one stuck in D state), ess ignores SIGINT, SIGTERM and SIGHUP for good; before this commit it died. I could not build a reachable case. Note / INFEASIBLE / introduced.
- `crates/edge/ess-cli/src/main.rs:5082`: `--report-out` is written with `fs::write`, which truncates and then writes. A signal landing mid-write leaves a truncated file in place of an earlier report. The old default signal action did the same, and the window is microseconds, so no deterministic test. Note / CONFIRMED / pre-existing.
- `crates/verify/ess-conformance/src/report.rs:456`: `quote_name` leaves `"`, `\`, `=` and `, ` as they are. A target's row member name like `a = 1, b` reads as a second field in a diagnostic, and the literal text `\u001b` prints the same as an escaped ESC. This stays within one line, with no terminal control. Note / CONFIRMED / introduced.
- Not raised, only stated: in `exact.rs:456`, the cost of `request` grows with (number of unknown members × request size). Measured on a debug `serve`: a 1 MiB `close_periodic` with 256 unknown members took 12.07 s; 13 MiB with 200 unknown took 126.45 s.

**5. Found nothing**
- Signals:
  - The target's signal state matches the environment (`/proc` shows no blocked signals, and the same ignore mask as perl started from the shell).
  - The handler is installed only by `Program::spawn`, so other `ess` commands keep their old signal behaviour.
  - A signal during the handshake gives 130; one during the end grace gives 143; two quick signals give the first one's status.
  - No report is written, and an earlier report file is left untouched.
  - Mutant (e) survives, but a deterministic test cannot catch that race.
- Escaping:
  - C0, DEL, C1, U+2028/2029 and the bidi overrides and isolates are all escaped.
  - Lone surrogates and invalid UTF-8 are refused.
  - U+200E/F, U+061C, U+FEFF, U+00AD and tag characters can only reorder or hide text on their own line; none can forge another line.
  - The identity bound counts raw bytes, so a stored name is at most 1536 bytes.
  - The 4096-byte cut lands on a character boundary, before escaping, and its "bytes more" count is correct.
  - `observation`, `detail`, `actor` and `why` all go through `target_text`.
- Open requests in `serve` (probed with real `serve`):
  - Names containing a backtick, a quote, a newline or the text "unknown field" are dropped correctly.
  - 256 unknown members are answered; 257 are refused, as documented.
  - The probe only matters for types that deny unknown fields: `SemanticCommandRequest` simply ignores them, so 5000 data objects holding the same key were answered.
  - The only flatten type that is reachable (`RawField`) cannot cause a wrong drop, because the object nested inside it knows only names it knows too.
  - A request carrying an unknown `x` and also a key `x`+NUL is refused (`exact.rs:506`); nothing realistic sends that.
- Byte stability: no suite or example contains a C1, U+2028/2029 or bidi character, raw or as a JSON or YAML escape. The report/2 summary carries no diagnostic text.

**6. Mutants** (on a copy of HEAD, with its own target dir)
- (a) drop the leader SIGKILL: every unit test stays green. My leader test goes red: ess hangs until the 120 s bound.
- (b) apply the 256-byte bound after escaping: lib `a_handshake_name_or_version_past_256_bytes_is_refused_without_quoting_it` goes red.
- (c) drop the member from the first object holding the name: lib `a_request_member_unknown_in_one_object_is_dropped_there_and_kept_where_it_is_known` goes red.
- (d) exit 1 instead of 128+n: CLI `a_sighup_…` goes red, and so do my two signal tests.
- (e) no lock around spawn and registration: everything stays green; no deterministic test is possible.

**7. Byte comparisons**
- f51c4f97ec vs HEAD: suites billing, gatepass and oracle-fixture × targets billing, oracle-fixture and `interpreted --path examples/<suite>` × text, json and yaml × report formats 1 and 2. I compared stdout, exit code and `--report-out`: 54 identical, 0 different.
- HEAD built-in vs HEAD through `serve`: all 3 suites × billing and oracle-fixture × 3 formats: 18 identical, 0 different.

**8. Commands and cleanup**
- Tests (each prefixed with `CARGO_INCREMENTAL=0 RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`):
  - `cargo test --locked -p ess-conformance --test adversary_process_target_pass2`
  - `cargo test --locked -p ess-conformance -p ess-entity-runtime --test adversary_process_target_pass2` (feature on)
  - `cargo test --locked -p ess-cli --test adversary_process_target_pass2`
- `ps -eo pid,args | grep -E "ess-conformance-peer|<session id>"` printed nothing.
- Deleted: `adv2/old`, `adv2/old-target`, `adv2/mut`, `adv2/mut-target`, the `ess-head` binary and the probe and comparison output directories. What remains in the session scratch directory under `adv2/` is scripts and logs, 232 KB.
- `git status --short`: my two `??` files plus the other agents' edits listed in part 1.

```findings
- {file: crates/verify/ess-conformance/src/process.rs, line: 626, category: boundary, severity: warning, verdict: CONFIRMED, origin: introduced, message: "the restart identity check compares escaped names, and the escape keeps backslashes, so a restart naming the literal text of an escaped control character is admitted as the same implementation"}
- {file: crates/edge/ess-cli/src/target_process.rs, line: 78, category: judgement, severity: note, verdict: CONFIRMED, origin: introduced, message: "exiting 128+n instead of re-raising the signal makes a parent bash treat Ctrl-C as handled, so a shell loop over process-target runs no longer stops"}
- {file: crates/edge/ess-cli/src/target_process.rs, line: 60, category: judgement, severity: note, verdict: INFEASIBLE, origin: introduced, message: "after the first signal every later SIGINT, SIGTERM or SIGHUP is swallowed, so a reap that blocks on an unkillable leader leaves ess ignoring termination"}
- {file: crates/edge/ess-cli/src/main.rs, line: 5082, category: judgement, severity: note, verdict: CONFIRMED, origin: pre-existing, message: "the report is written in place with fs::write, so a signal mid-write leaves a truncated file where an earlier report was"}
- {file: crates/verify/ess-conformance/src/report.rs, line: 456, category: judgement, severity: note, verdict: CONFIRMED, origin: introduced, message: "quote_name prints quotes, backslashes, '=' and ', ' bare, so a target's member name can fake a field boundary or an escape within a diagnostic line"}
```
