---
format: aep.planning-md/3
id: review-result:outside-target-adversary-pass-1
kind: review-result
status: active
title: 'Adversary, pass 1: the conformance target process'
relations:
- reviews: story:a-conformance-run-drives-a-target-process-in-any-language
revision: 1
---
7 failing tests and 5 judgement findings. Commit covered: f51c4f97ec (worktree ess-outside-target).

```
unit: story:a-conformance-run-drives-a-target-process-in-any-language, f51c4f97ec + 2 untracked test files
verdict: CONFIRMED (5 red cases)
cases: executed 48→56, red 5  (48 = the unit's tests, run after my cases existed, mine not selected: lib process 32, CLI process_target 16)
origin: introduced 4 / pre-existing 0 / undecided 1
wrote-outside-worktree: 1 directory: the session scratch directory, subdirectory adv/
needs-coordinator: whether a signalled `ess` has to end the target's process group (the design page doesn't say)
```

**1. `git diff --stat`:** empty. `git status` shows only the two new test files (`??`). No implementation file changed.

**Charter note:** item 13 asked for mutations in the worktree. The adversary procedure forbids mutating files under attack, and another agent is editing this worktree. So I mutated a copy instead (`git archive f51c4f97ec` into `adv/mut`, own target dir), and the copy is restored.

**2. Failing tests** (each ran alone first, red on its first run):
- `crates/edge/ess-cli/tests/adversary_process_target_pass1.rs` — **defect**
  - `a_sigterm_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind` → `processes of the target's group left behind after ess was terminated: [2479536] of [2479527, 2479536]`
  - `a_sigint_to_ess_mid_run_leaves_no_process_of_the_targets_group_behind` → `left: [2479713]`
  - Claim broken: acceptance 5 and brief claim 3; design `:146-147`.
  - What happens: `ess` has no signal handler. It dies, and neither `finish` nor `Drop` runs (`target_process.rs:236-242`, `:311-315`). Because of `process_group(0)` (`:77`), a signal sent to `ess` never reaches the target's group.
  - The test's adapter follows the design and exits when its stdin closes; the server it started in its group is left running, reparented to init.
  - A target busy inside a request ends only when it next reads stdin, so possibly never. I saw this with `--stall-in`: the peer and its grandchild were alive 3 s after `ess` died. The page (`:140-142`) says exactly this for the direct child and says nothing about the group.
- `crates/verify/ess-conformance/tests/adversary_process_target_pass1.rs`
  - `the_text_report_prints_a_targets_implementation_name_without_control_characters` → `left: ["U+001B", "U+000D", "U+001B"]` — **defect**. The handshake `name` (`process/wire.rs:228-239`) is printed raw by `report.rs:669-673`, so a target can print a forged "34 passed" summary line on stdout. Seen on the real CLI too (`^[[31mRED^G`).
  - `the_text_report_prints_a_targets_unsupported_reason_without_control_characters` → `left: ["U+001B", "U+000D", …]` — **hardening**. Same problem for the target's `why` text, printed by `report.rs:684-693`.
  - `serve_ignores_a_request_member_it_does_not_know_in_every_operation` → `left: ["close_periodic (not-a-request)", "observe_periodic (not-a-request)", "fixture_values (not-a-request)"]` — **defect**. Contradicts design `:201-202` and `:993` ("requests are open"). The cause is `deny_unknown_fields` on `periodic.rs:58` and `fixtures.rs:84`. `begin_scenario` with the same extra member is answered.
  - Consequence: if a later `ess` adds an optional member to one of these requests, `serve` (or a Rust adapter built on `process::serve`) exits 2, and every such scenario ends `error`.
  - Three more cases are green: numbers both ways under `arbitrary_precision` off and on, out-of-range numbers in nested answers, and `an_unsupported_command_answer_is_unsupported_and_keeps_the_process`, which kills mutant M2.

**3. Suite run after my cases existed:** lib `process`: 32 passed. CLI `process_target`: 16 passed, both exit 0.

**4. Judgement findings**
| file:line | finding | verdict / origin |
|---|---|---|
| `process/wire.rs:228-239` | name and version have no size bound: a 5 MB name produced a 5 MB report member and a 5 MB stdout line. No test, because the design sets no bound. | CONFIRMED / introduced |
| `process/exact.rs:77-113` | A 16 MB `[0,0,…]` answer cost 625 MB max RSS and about 11 s per answer (debug build). 16 MB of nested `[` took over 30 s per answer: 34 answers were not done after 17 min. Every nesting level re-parses the rest of the line, up to 128 levels. No test, because no bound is specified. | CONFIRMED / introduced |
| `process.rs:557-584` | When every scenario times out, total time = scenarios × `--target-timeout`, plus a restart handshake each. Measured 34.2 s for 34 scenarios at 1 s. At the default 30 s: 17 min for billing, 2 h 30 for 300 scenarios. | CONFIRMED / introduced (stated, not judged) |
| `process.rs:742` | An answer sent before it was asked, carrying the next id, is accepted; acceptance 5 says an unasked answer is an error. Measured: such a target falls out of step and every scenario errors, so it gains no pass. It can't be told apart from a fast answer, so no test. | INFEASIBLE / undecided |
| `process/serve.rs:125-129` | `serve` answers a last line with no `\n`; the `ess` side (`target_process.rs:140-144`) refuses one. Harmless. | CONFIRMED / introduced |

**5. Attacks that found nothing**
- Protocol faults: 3 and 10 000 unasked lines, an early answer, two answers on one line, trailing garbage, a line of 16 MiB + 1, and `ok:{}` for commands all ended error or failed, never passed. A line of exactly 16 MiB is accepted.
- Fault text: every fault and refusal named only kind, id and byte offset. `serve` stderr names kind, line and byte only.
- Numbers: 17 edge tokens and nested maps came back by value and spelling with the feature off and on; `1e400`, `-1e400` and a 601-character number gave number-out-of-range.
- `serve`: garbage, EOF mid-line, non-UTF-8, an id of `1e400`, a negative id and a huge line all exit 2 without panic. EPIPE gives `serve.io` exit 2; a closed stdout gives exit 0.
- Command line: every refusal exits 2 and leaves an existing `--report-out` untouched. Argv with spaces, quotes, `--`, `-x`, an empty string and `--target billing` arrives verbatim. `--target-dir` resolves a relative program.
- Determinism: nothing in the report depends on wall time, pids or the environment (code reading; the existing test covers it).
- Serialized bytes: unit 1 only added `Serialize` to types that had none. The four byte pins are green: 2, 1, 1 and 17 passed.

**6. Mutations** (applied to the copy, then restored; `cmp` printed `COPY RESTORED`)
- M1, skip the identity check after a restart (`process.rs:599`): lib `a_restart_naming_another_implementation_fails_every_remaining_scenario` went red; CLI stayed green.
- M2, `unsupported` on `execute_command` treated as `unavailable` (`process.rs:576`): all 48 unit tests stayed green. My green case went red.
- M3, no group kill before the reap (`target_process.rs:213`): lib stayed green; 9 CLI tests went red, and `an_answer_past_the_time_limit_is_an_error` hung (`ess` blocked in wait) until the 600 s timeout.

**7. Commands** (each with `RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`):
- `cargo test -p ess-conformance --locked --test adversary_process_target_pass1`
- `cargo test -p ess-conformance -p ess-entity-runtime --locked --test adversary_process_target_pass1 numbers` (feature on)
- `cargo test -p ess-cli --locked --test adversary_process_target_pass1`

**8. Leftover processes:** `ps -eo pid,args | grep -E "<session id>|ess-conformance-peer"` printed nothing. I killed one orphaned `sleep 100000` and three probes myself.

**9. Written outside the worktree:** only `adv/` under the scratch directory: probe scripts and logs, `mut/` (141 MB copy) and `mut-target/` (7.9 GB build directory; delete it when done).

```findings
- {file: crates/edge/ess-cli/src/target_process.rs, line: 77, category: acceptance, severity: warning, verdict: CONFIRMED, origin: introduced, message: "SIGTERM or SIGINT to ess mid-run leaves the target's process group running, because no handler signals the group out of reach of ess's signals"}
- {file: crates/verify/ess-conformance/src/process/wire.rs, line: 228, category: boundary, severity: warning, verdict: CONFIRMED, origin: introduced, message: "the handshake implementation name reaches the text report raw, with escape sequences and line feeds that forge the summary"}
- {file: crates/verify/ess-conformance/src/report.rs, line: 684, category: boundary, severity: note, verdict: CONFIRMED, origin: introduced, message: "a process target's unsupported why is printed raw to the terminal"}
- {file: crates/verify/ess-conformance/src/process/serve.rs, line: 217, category: contract-drift, severity: warning, verdict: CONFIRMED, origin: introduced, message: "serve refuses an unknown request member in close_periodic, observe_periodic scope and fixture_values contract, though requests are open"}
- {file: crates/verify/ess-conformance/src/process/wire.rs, line: 235, category: judgement, severity: note, verdict: CONFIRMED, origin: introduced, message: "implementation name and version are unbounded; a 5 MB name lands in the report and on stdout"}
- {file: crates/verify/ess-conformance/src/process/exact.rs, line: 77, category: judgement, severity: note, verdict: CONFIRMED, origin: introduced, message: "a 16 MiB answer costs about 625 MB and seconds to refuse; nesting re-parses the line per level"}
- {file: crates/verify/ess-conformance/src/process.rs, line: 557, category: judgement, severity: note, verdict: CONFIRMED, origin: introduced, message: "all-timeout run time is scenarios times target-timeout: 17 min for billing at the default"}
- {file: crates/verify/ess-conformance/src/process.rs, line: 742, category: judgement, severity: note, verdict: INFEASIBLE, origin: undecided, message: "an answer written before its request with the next id is accepted"}
- {file: crates/verify/ess-conformance/src/process/serve.rs, line: 125, category: judgement, severity: note, verdict: CONFIRMED, origin: introduced, message: "serve answers an unterminated last line that the ess side would refuse"}
```
