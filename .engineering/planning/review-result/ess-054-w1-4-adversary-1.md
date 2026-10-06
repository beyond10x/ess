---
format: aep.planning-md/3
id: review-result:ess-054-w1-4-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W1-4 adoption docs and generate --check
tags:
- ess-0.54.0
relations:
- reviews: story:ess-generate-check
- reviews: story:feature-request-433
revision: 1
---
unit: W1-4, range 9ffa94d66a..f9ae59c50b, worktree `~/.local/state/worktree/trees/b10x/ess/ess-054-w1-4` plus one untracked adversary test file
verdict: CONFIRMED (3 red cases: `--check` misses `.ess-output` drift; it passes a tree that regeneration refuses; one refusal names the wrong path)
cases: executed 63→73, red 3
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (see part 6)
needs-coordinator: none

**1. Diff stat.** `git --no-pager diff --stat` is empty. `git status --short` shows one path, and it is a test file:
```
?? crates/edge/ess-cli/tests/generate_check_adversary.rs
```
I wrote a second test file for the mutant probe, `crates/edge/ess-cli/tests/zz_adv1_mutant_probe.rs`, and removed it straight after that run. I changed no implementation file.

**2. Cases added** in `crates/edge/ess-cli/tests/generate_check_adversary.rs`. It includes `src/output_ownership/mod.rs` through `#[path]`, as `output_ownership.rs` does, so the 4 `xattr_tests` also run in this binary (10 tests in all). Captured red run of this target alone (`cargo test -p ess-cli --test generate_check_adversary`, EXIT=101):

| case | asserts | now |
|---|---|---|
| `check_passing_means_regeneration_leaves_the_tree_unchanged_with_a_stale_record` | if `--check` exits 0, the same command without `--check` changes no file under `--out` | RED: `--check exited Some(0) (stderr: ), and then the same command without --check rewrote [".ess-output/state.json"]` |
| `check_passing_means_regeneration_succeeds_without_the_record` | if `--check` exits 0, regeneration succeeds | RED: `--check exited Some(0) (stderr: ), and then the same command without --check refused: error: unowned output destination: openapi/email-service.yaml; adopt exact generated reference bytes explicitly` |
| `a_refusal_of_the_ownership_record_names_its_path_relative_to_the_output_root` | a `user.ess_test` attribute on `.ess-output/state.json` is refused naming `.ess-output/state.json` | RED: `error: ess 0.53.0 refused output state.json: output has extended metadata outside the ordinary snapshot contract: user.ess_test` |
| `check_against_an_absent_root_creates_nothing` | `--out absent/deeper --check` exits 1, says `is missing`, creates nothing | green |
| `drift_refuses_an_interrupted_operation_in_place` | `drift` on a Staging root refuses, points to `recover`, writes nothing | green |
| `drift_on_a_copied_interrupted_root_names_the_recorded_root` | `drift` on a copy of a Staging root refuses with `was recorded at <a>` and writes nothing | green; red against the mutant below |

Mutant probe, run on a mutated copy in scratch and never on the tree: `mod.rs:453` `if relocating || !matches!(payload.checkpoint, Checkpoint::Idle { .. })` changed to `if relocating`. My copied-root case went red: `pending generated output at …/b; run 'ess generate output recover --ownership-root …/b'`. That refusal tells the user to recover at the copy, which `relocate` says is impossible. Before this case, no test in the repository called `ownership::drift` (grep), so nothing covered that branch.

**3. Suite run**, made after the cases existed:
`cargo test -p ess-cli --locked --offline --no-fail-fast --test generate_check --test output_ownership --test generate_check_adversary` → EXIT=101
- generate_check: 9 passed
- generate_check_adversary: 7 passed, 3 failed (the three above)
- output_ownership: 54 passed

The before count of 63 is 9 + 54, taken from the implementer's `final.log`.

**4. Findings** (tree f9ae59c50b plus my test file)

| # | file:line | what breaks | case | verdict | origin |
|---|---|---|---|---|---|
| 1 | `crates/edge/ess-cli/src/output_ownership/mod.rs:101-155` (`drift`) | Only file bytes are compared, never the ledger. A stale `.ess-output/state.json` (projections committed, record not) passes `--check`, then `ess generate` rewrites it. The git idiom this replaces in the guide would catch that. The guide says to commit `.ess-output` and that `--check` exits 0 "when the tree is current". Reached by the documented workflow. | stale_record | CONFIRMED | introduced |
| 2 | same function | With `.ess-output` absent (e.g. `git add generated/*` skips dot directories), `--check` exits 0 and the reconcile step it prescribes refuses with `unowned output destination`. | without_the_record | CONFIRMED | introduced |
| 3 | `crates/edge/ess-cli/src/output_ownership/filesystem.rs:366-377` (`image`) | `relative` is relative to whatever directory `image` was given. For the state record it prints `state.json`, and in `adopt` it is relative to the reference root. This contradicts the new guide text ("names the file, relative to the output root") and story 433's acceptance. Fix: add the context at the callers that know the root, or give `.ess-output/` its own context in `read_state_at`. | record_refusal | CONFIRMED | introduced |
| 4 | `filesystem.rs:371` | Refusal-text change. Every refusal raised inside `image`/`snapshot` now starts `error: ess <version> refused output <rel>: ` before the old sentence. Substring matches still work. Anchored or exact matches, and golden stderr files, break, and the embedded version changes the text on every release. The incompatible-type refusal now names the path twice. No in-repo consumer matches the old prefix (grep). Story 433 accepted the stderr change. | none (judgement) | CONFIRMED | introduced |

Suggested fixes, which I did not apply. For 1, compare the ledger `--check` would write (the `plan` → `transaction.before != after` test that `publish` uses at `mod.rs:1097-1106`) and report `.ess-output/state.json` as drifted. For 2, call that drift or refuse it, or document it as accepted.

**5. Attacked and could not break**
- Does `--check` write? `Locks::acquire` only `flock`s existing directories. An absent root is not created. A pending root, in place or copied, is refused with nothing written.
- Relocation bypass: it applies only to Idle records. Pending in place is stopped by `ensure_idle`; pending copied is stopped by `relocate`.
- Mode change: `publish` keeps the file's existing mode when the bytes match (`mod.rs:876-888`, `:984-991`), so `--check` ignoring mode agrees with `publish`.
- An extra unrecorded file in `--out`: `publish` leaves it alone, so not reporting it is consistent.
- Stale `.ess-output` entries for owners outside the selected `--kind` are ignored, matching `publish`.
- `requires = "out"` and exit 1 on drift: covered by the unit's own tests, which are real assertions.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-wave/W1-4/adv1/red-cases.log`
- `~/.cache/ess-054-wave/W1-4/adv1/mutant-probe.log`
- `~/.cache/ess-054-wave/W1-4/adv1/suite.log`
- `~/.cache/ess-054-wave/W1-4/adv1/mutant/output_ownership/` (mutated copy, 4 files)
- `~/.cache/ess-054-wave/W1-4/adv1/tmp/` (TMPDIR, now empty)

The build dir `/dev/shm/ess-054/W1-4` was built into and then removed with `cargo clean`, as the adversary brief says. The session lease `ess-054-w1-4-adv1` was acquired and released.

**7. Findings block**
```findings
- file: crates/edge/ess-cli/src/output_ownership/mod.rs
  line: 101
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'ess generate --check exits 0 on a stale committed .ess-output/state.json, and the same command without --check then rewrites it, so CI is green on a tree regeneration changes'
- file: crates/edge/ess-cli/src/output_ownership/mod.rs
  line: 101
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'with .ess-output absent, --check exits 0 while the reconcile step it prescribes refuses every file as an unowned output destination'
- file: crates/edge/ess-cli/src/output_ownership/filesystem.rs
  line: 371
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the new refusal context names paths relative to the directory image() was handed, so a refused state record reads "refused output state.json", contradicting the guide and story 433 acceptance of an output-root-relative path'
- file: crates/edge/ess-cli/src/output_ownership/filesystem.rs
  line: 371
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'every image-originated refusal now starts "ess <version> refused output <rel>: " before the old sentence, which breaks anchored or exact stderr matches and golden files on every release and doubles the path in the incompatible-type refusal'
```
