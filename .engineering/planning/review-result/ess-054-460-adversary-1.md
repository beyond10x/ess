---
format: aep.planning-md/3
id: review-result:ess-054-460-adversary-1
kind: review-result
status: active
title: 'Adversary pass 1, #460 specification formats command'
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-460
revision: 1
---
unit: ess-054-460-adv, the uncommitted working tree on base `97271a43d` in `~/.local/state/worktree/trees/b10x/ess/ess-054-460-20261005`
verdict: NEEDS-CHANGE
cases: executed 324→336, red 2
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (part 6)
needs-coordinator: none

**1. `git --no-pager diff --stat`**, tracked files (all 8 are the implementor's; I changed none):
```
 Taskfile.yml | 1 +, ess-cli/src/main.rs | 20, ess-xtask/src/docs.rs | 181, ess-xtask/src/main.rs | 10,
 ess-domain/src/system.rs | 204, ess-domain/tests/identity_changing_updates.rs | 7, cli.md | 17, spec-versions.md | 15
 8 files changed, 400 insertions(+), 55 deletions(-)
```
I added two untracked files, both test files: `crates/edge/ess-cli/tests/adversary_460_pass1.rs` and `crates/edge/ess-xtask/tests/adversary_460_pass1.rs`. I edited no production file.

**2. Cases added.** Each file was run alone before any suite run.

| Case | Asserts | Now |
|---|---|---|
| cli `every_family_the_help_sends_to_the_history_page_is_on_it` | every `F/N` family that `ess specify formats --help` sends to the history page appears on `spec-versions.md` | **red** |
| cli `since_at_or_past_the_newest_prints_nothing_and_succeeds` | `--since ess/23` and `--since ess/99` exit 0; text output empty, json/yaml output `[]` | green |
| cli `since_is_held_to_the_published_pattern` | `ess/01`, `ess/+1`, `ESS/1`, `ess/ 1`, `ess/` each exit 2 with empty stdout | green |
| xtask `two_releases_never_share_a_reference_label` | `1.0.0` and `0.10.0` (also `0.4.61`/`0.46.1`, `1.2.0`/`0.12.0`) get different `[rNN]` labels | **red** |
| xtask `a_hand_edit_of_any_row_is_refused` | `compare` refuses an edit to each of the 23 rows | green |
| xtask `the_base_release_claim_for_an_unreleased_format_is_refused` | the base cell `[0.54.0][r54]` on `ess/23` is refused | green |

The xtask file compiles `src/format_history.rs` in through `#[path]`, so it also runs that module's 6 own tests. Its total is 9.

Red output, captured on the first run of each file (exit 101 both times):
```
every_family_the_help_sends_to_the_history_page_is_on_it panicked at crates/edge/ess-cli/tests/adversary_460_pass1.rs:62:5:
`ess specify formats --help` says the version history page lists these families with their releases, and the page names no version of them
  left: ["ess-ui"]
 right: []
two_releases_never_share_a_reference_label panicked at crates/edge/ess-xtask/tests/adversary_460_pass1.rs:40:9:
1.0.0 and 0.10.0 link the same reference label
  left: Some("r10]")
 right: Some("r10]")
```

**3. Suite run, after the cases existed**

| Command | Result | Exit |
|---|---|---|
| `cargo test -p ess-domain --test format_history --test identity_changing_updates` | 3 + 11 passed | 0 |
| `cargo test -p ess-xtask --bins --test adversary_460_pass1` | 247 passed; adversary file 8 passed, 1 failed | 101 |
| `cargo test -p ess-cli --no-fail-fast --bin ess --test specify_formats --test command_surface --test adversary_460_pass1` | 51 + 5 + 7 passed; adversary file 2 passed, 1 failed | 101 |
| `cargo xtask format-history --check` | byte-identical | 0 |
| `cargo xtask cli-reference --check` | agrees | 0 |
| `cargo xtask diagnostics --check` | byte-identical | 0 |
| `cargo xtask docs` | 85 supported versions, 194 tracked, 75 families | 0 |

Before = 324: these same targets with my two files left out of the count. After = 336.

**4. Findings** (they cover the working tree above)

| file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|
| `crates/edge/ess-cli/src/main.rs:129` | NEEDS-CHANGE / introduced | The help says the history page lists `ess-ui/N` "with their releases". The page names no `ess-ui/` version; `FORMAT_RELEASES` has `("ess-ui", 1, Some("0.47.0"))`. Red case cli:62. | Every `ess specify formats --help`, and the generated `cli.md:457`. Fix: add an `ess-ui/1` row to the page, or drop `ess-ui/N` from the help and regenerate `cli.md`. |
| `crates/edge/ess-xtask/src/format_history.rs:36` | INFEASIBLE / introduced | `reference_label` maps `1.0.0` and `0.10.0` both to `r10`, so the cell would link the 0.10.0 release notes and `undefined_references` would still pass. Red case xtask:40. | Nothing yet; it needs a 1.0.0 or 0.4.61-style release. |
| `crates/specify/ess-domain/src/system.rs:226` | INFEASIBLE / introduced | `release: None` now ships inside the `ess` binary. No gate ties a `None` row to the release commit: the 0.53.0 release flipped `None` to `Some` by hand in `docs.rs` (`7d498fdd1`), and no AGENTS.md step names `FORMAT_HISTORY`. | A 0.54.0 release commit that misses the flip ships `ess/23 unreleased` permanently. Before this change that miss stayed in xtask and could be fixed after the tag. Fix: `cargo xtask release verify` refuses a `None` row. |
| `crates/specify/ess-domain/tests/identity_changing_updates.rs:74-79` | CONFIRMED / introduced | An existing assertion was rewritten: it checked the `("ess", 23, …)` source row, and now checks `FORMAT_HISTORY.iter().any(major == 23)`. The const assertion already guarantees that whenever `SUPPORTED_FORMATS` has 23, so the new check cannot fail on its own. | The suite. The rewrite follows the move, but it is a rewritten existing case, so it should be reported. |

**5. Attacked and could not break**
- **Catalogue drift:** a const assertion (`system.rs:239-256`) fails the build on any row added to one list and not the other, or on order or empty `added`. I read it; I did not mutate it.
- **Releases:** all 23 releases match the base `FORMAT_RELEASES` exactly. Each matches the oldest CHANGELOG section that names it; `ess/1` and `ess/21` are not named in the CHANGELOG.
- **JSON/YAML shape:** keys and order are stable, and `newest` is set on one row only, including under `--since`.
- **Refusal hint:** the hint is in the only producer of the `ess/` header refusal (`system.rs:1168`). The ~40 refusals for a construct below its format each already name the format they need.
- **`--check`:** catches an edit to any of the 23 rows, marker damage, and the base `0.54.0` cell. `docs` reads the release cells itself.
- **Published URL:** `beyond10x.github.io` + `/ess/` + `docs` + `spec-versions` is correct.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-460/adv/`: `env.sh`, `red-xtask.log`, `red-cli.log`, `suite.log`, `suite-cli.log`, `xtask-checks.log`, `base-rel.txt`, `new-rel.txt`, `tmp/`
- `~/.cache/ess-054-460/adv-base-rel.txt`: written there by mistake, then moved into `adv/`; it no longer exists.
- `/dev/shm/ess-054/ess-054-460-adv`: `cargo clean` done, 1.9 GiB removed.
- `~/.cache/b10x-go-cache/ess-054-460-adv`: empty.

I took no worktree lease: the invariants forbid worktree commands.

**7.**
```findings
- file: crates/edge/ess-cli/src/main.rs
  line: 129
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the formats long help (and generated cli.md:457) says the version history page lists ess-ui/N with its release, and spec-versions.md names no ess-ui version
- file: crates/edge/ess-xtask/src/format_history.rs
  line: 36
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: reference_label maps 1.0.0 and 0.10.0 (and 0.4.61/0.46.1) to the same rNN label, so a future release cell would link another release's notes and pass undefined_references
- file: crates/specify/ess-domain/src/system.rs
  line: 226
  category: judgement
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: FORMAT_HISTORY release None now ships in the binary and no gate forces the release commit to flip it, so a missed flip ships "ess/23 unreleased" in 0.54.0 permanently
- file: crates/specify/ess-domain/tests/identity_changing_updates.rs
  line: 74
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: an existing assertion was rewritten into a FORMAT_HISTORY membership check the const assertion already guarantees, so it cannot fail independently
```
