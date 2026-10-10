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

## 5. Close — 2026-10-08

| unit | unit commit | merge | adversary passes | outcome |
|---|---|---|---|---|
| U2 the "before" query | `7c0e3221d0` | `3f0e8e1539` | 2 (1 blocker, 1 warning; then the coordinator ran the pass-2 cases under the disk floor) | one query in `synthesize/precedence.rs`; the synthesis helpers and `authored::not_taken` filter it; no synthesized byte moves; two authored acts the interpreter could never pass are now refused (CHANGELOG) |
| U3 `subject_fact.rs` | `109682195b` | `32296a1e92` | 1 (two notes outside the file) | `selects`, `first_read` and `leaves_external` read the plan; no byte moves |
| U5 `row_set.rs` | `2f30e603c1` | `e5c06df154` | 1, then a correction the coordinator verified | `input_for` and `consistent` read the plan; the three surviving `row_set` mutants are caught |
| U4 `related_guard.rs` | `c5bde8ef59` | `13f688be95` | 1, then a correction the coordinator verified | `selects`, `orders_present_related_refusal`, `leaves_external` read the plan in the model's format; the two surviving mutants are caught; one external witness can change (CHANGELOG) |

The open adversary cases are committed as `#[ignore]` tests naming their owner:
`adversary_selection_plan_w5_u3_pass1_open.rs` (3, `story:existence-family-sends-read-selection-plan`) and
`adversary_selection_plan_w5_u4_pass1_open.rs` (6: 4 pre-existing, `story:related-guard-wrong-state-beside-external`;
2 test-seam only). The coordinator ran U4's six against the base `related_guard.rs`: all six fail there too.

Not run under the disk rule (DEC-20261008-29, `/` under 40 G): the adversaries' mutant and base
scratch builds for U3 and U5. The implementors' mutant runs in their own trees stand.

`story:synthesis-reads-selection-plan`: every helper Acceptance bullet 2 names now reads the plan.
