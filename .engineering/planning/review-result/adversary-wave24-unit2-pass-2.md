---
format: aep.planning-md/1
id: review-result:adversary-wave24-unit2-pass-2
kind: review-result
status: active
title: Adversary pass 2 against planning store carries workstation paths
relations:
- reviews: story:planning-store-carries-workstation-paths
revision: 1
---
# Adversary pass 2 against `story:planning-store-carries-workstation-paths`

Worktree `wt-5cf844fd8563`, uncommitted over base `bd722fa964bd225b9755f272e22b45fab334449f`.
Verdict **NEEDS-CHANGE**. Cases 175 → 178, 3 red. Budget exhausted: two passes, two NEEDS-CHANGE.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| F1 | `host_paths.rs:1262` | CONFIRMED / introduced / **blocker** | `host_paths_adversary_6.rs:83`, exit 101. `documented_unread_trees` returns `[(".engineering/", true, 60, 32806)]` for the corrected bullet **and** for one carrying the falsified superlative whose reasons contradict its own head (`1 file and 2 lines`). The parse stops at the bullet's first colon. The bullet states two counts before it and 87, 0.27%, 28730, 59-of-60, 99.7%, 32163-of-65838 after it. | The unit shipped that document until correction round 1. The doc at `:1262` names that edit as the reason the parse was widened — "every number in the bullet was unread prose. One of those numbers was wrong." The number that was wrong is in the region still unread. |
| F2 | `host_paths.rs:1244` | INFEASIBLE / introduced / warning | `host_paths_adversary_6.rs:121`, exit 101. A `- `-marked bullet between the anchors yields 1 claim where `* ` yields 2. No panic. | The adversary built the document. `UNREAD_TREE_SECTION`'s doc says "every bullet between the anchors is read" — false for any marker but `* `. Markdown-legal, 6 instances elsewhere under `crates/`, none in this file. |
| F3 | `host_paths.rs:1244` | INFEASIBLE / introduced / note | `host_paths_adversary_6.rs:158`, exit 101. A continuation line whose trimmed text equals the closing anchor closes the section there; later bullets dropped, `claimed.is_empty()` does not fire, run green. | The adversary built it. A second closing anchor is the pre-correction truncation reached through the anchor instead of the indentation. |
| F4 | `host_paths.rs:1503` | NEEDS-CHANGE / introduced / warning | No case. The count comparison is inline in the `#[test]` body, so it is not a function, not in `TRANSCRIBED`, and no adversarial target can drive it. Its only execution is the one production run against true values. | This is also why the coordinator's `1 files, 1 lines` fixtures in `host_paths_adversary_5.rs` survive: `documented_unread_trees_in` discards the counts and the parse validates nothing. |
| F5 | `host_paths.rs:56` | NEEDS-CHANGE / introduced / warning | The bullet pins `60 files, 32806 lines` by exact equality. 49 of the 60 carrying files are `review-result` documents; 49 of 122 tracked review-results (40%) carry the class. | **Measured live by the coordinator, and it is not a forecast.** Worktree at base: 60 carrying files. Main checkout with this session's uncommitted store writes: **64**. The lane is stale by four files before the unit merges. The trigger is a store write and the remedy is an edit to `host_paths.rs`; the protocol reserves the first to the coordinator and forbids the second to it. |
| F6 | `host_paths_adversary_5.rs:224` | CONFIRMED / introduced / note | No case. `the_journal_holds_the_largest_share_…` is vacuous twice over: `unread_bullet` breaks at the first blank doc line, so its window is `host_paths.rs:56–61`, and `largest single share` is at `:71`. The `return` always fires. Even with a reaching window the string is now a **disavowal**, so the case would fire on a correct document. | Answers the brief's question. The head counts are pinned by the detector comparison; the superlative and the other five numbers are pinned by nothing (F1). |
| F7 | `ess-xtask/src/consumer_coverage/metadata_tests.rs:253` | CONFIRMED / pre-existing / warning | Reproduced twice, deterministically. Plain `cargo test -p ess-xtask --locked` → `128 passed; 1 failed`, `current_compiled_provider_executes_one_guard_and_binds_its_opaque_proof_to_this_run`, "unsupported measured compiled profile DEBUG: Some(String(\"true\"))". With only the profile knobs pinned it moves to "NUM_JOBS: Some(String(\"20\"))". With the Taskfile's full `env` line → `129 passed; 0 failed`. | The case rejects any invocation but the Taskfile's exact `env`. The per-crate command the unit briefs prescribe is red against it. |

## The coordinator was wrong about F7

I reported that unit 2's `--bin` red "did not reproduce" and offered two readings: it never existed,
or it is order-dependent. Both are wrong. It is environment-dependent and deterministic. Both my
runs used the Taskfile's pinned `env`, which is the one environment where it passes, so I measured a
different build from the one the unit measured and read the difference as the unit's error. The
unit's number was right. What its report was missing was the case name and the environment, which is
why two re-runs could not settle it.

## Attacked and could not break

- The five prose numbers are all true today: journal 87 carrying lines / 32,163 findings;
  `authored-discovery-source-pass1.md` 28,730 lines (widest); 59 of 60 files; 65,838 total findings;
  99.7%. Measured with the lane's own transcribed detector. The adversary removed its own passing
  case rather than leave one in the tree; the measurement stands here.
- `line.trim() == *opening` in both spellings: `impl PartialEq<String> for &str` compares
  `self[..] == other[..]`. Byte equality both times. Reasoned, not run.
- `transcription_drift`'s converse check: exact today — all 36 column-zero `fn` lines end in `{`, all
  12 `#[test]` attributes sit immediately above their `fn`. Blind to a rustfmt-wrapped signature, to
  `pub`/`const`/`async fn`, to a `fn` inside a `mod`, and to a second attribute between `#[test]` and
  `fn` (which would produce a false red). Not drivable — `lane_functions` is private and
  `lane_source()` reads a fixed path — so no case. Reasoned, not run.
- Bullet grammar: an em-dash inside the tree name, two backtick pairs, a missing colon, reordered
  fields, extra fields, a leading `+`, `32,806`, an underscore, a singular `1 file`, an en dash, a
  space before the colon — all parse correctly or panic loudly. Fail-safe.
- Section boundaries: a second **opening** anchor panics; a closing anchor before the opening one is
  ignored; an empty section panics. Only the second **closing** anchor is silent (F3).
- The module summary line is scoped to the four trees and true of them. Pass 1's F1 is closed.
