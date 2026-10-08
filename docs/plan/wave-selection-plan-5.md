# Wave 5 of `epic:one-selection-plan` — synthesis asks the plan which branches answer first

> **Status: approved 2026-10-08 by the operator** under wave 4's approval (option A: "the two waves
> after it named in the operator's option A (synthesis, then the overlap witnesses)"), and the plan
> approved in the session the same day. Written on `integrate/selection-plan` `53d5345df2` (waves 2-4
> plus `main` at https://github.com/beyond10x/ess/pull/495). Skill: `aep:implementing` 0.20.1, wave
> mode.

## 1. Pre-flight — measured 2026-10-08

| fact | evidence |
|---|---|
| root filesystem 67 G free | `df -h /`, after `cargo clean` in the PR 488 tree |
| `synthesize.rs` and `synthesize/` are free of the ess controller | conductor, after https://github.com/beyond10x/ess/pull/495 merged (`86b493d7f`) |
| the synthesis bytes pin passes on this base | `synthesis_suite_bytes_table` 2 passed, after re-pinning three lines main moved (`ff92194804`) |

## 2. Units

| unit | story unit | surface | after |
|---|---|---|---|
| U2 | `story:synthesis-reads-selection-plan` unit 2: the one "before" query | `src/synthesize/precedence.rs` (new), `src/synthesize.rs` helpers, `src/authored.rs` `not_taken`, a phase-exchange test | — |
| U3 | unit 3 | `src/synthesize/subject_fact.rs` | U2 |
| U4 | unit 4 | `src/synthesize/related_guard.rs`, `related_guard/stored.rs` | U2 |
| U5 | unit 5 | `src/synthesize/row_set.rs` | U2 |

U2 runs alone. U3-U5 touch different files and run in parallel once U2 is merged; each gets its own
brief. Every unit's gate is the synthesis bytes pin unchanged plus the ignored
`adversary_244b_window_free_bytes` run once. Paths are under `crates/verify/ess-conformance/`.

## 3. Unit records

| unit | managed worktree id | branch | build directory | scratch root | stage |
|---|---|---|---|---|---|
| integration | `ess-selection-plan` | `integrate/selection-plan` | its own `target/` | — | at `53d5345df2` |
| U2 | `ess-selection-plan-w5-u2` | `unit/selection-plan-w5-u2-query` | its own `target/` | `~/.cache/ess-selection-plan/w5-u2-scratch` | to dispatch |

Dispatch types: `aep:implementor`, then `aep:adversary`, at most two passes each.

## 4. Commits this wave makes

The opening store commit (this page); one commit per unit; the merges into
`integrate/selection-plan`; the closing store commit. Delivery to `main` is the PR that carries waves
3 and 4, or its successor, merged through the App on a green `Gate`. No tag, no release.
