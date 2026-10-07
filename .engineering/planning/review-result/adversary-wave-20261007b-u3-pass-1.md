---
format: aep.planning-md/3
id: review-result:adversary-wave-20261007b-u3-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 2026-10-07b unit 3 (idle output record without a machine path)
relations:
- reviews: story:an-idle-output-record-carries-no-machine-path
revision: 1
---
```
unit: story:an-idle-output-record-carries-no-machine-path (e2ba76b37f plus one untracked test file)
verdict: red
cases: executed 144→151, red 3
origin: introduced 1, pre-existing 3, undecided 0
wrote-outside-worktree: none
needs-coordinator: yes
```

**1. What I changed.** `git --no-pager diff --stat` is empty. `git status --short` shows a single untracked test file: `?? crates/edge/ess-cli/tests/adversary_idle_record_pass1.rs`. No file under `src/` changed.

`<worktree>` below means `~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b-u3`.

**2. The cases.** All three are in `<worktree>/crates/edge/ess-cli/tests/adversary_idle_record_pass1.rs`. It is a new, standalone test binary that compiles the engine in through `#[path]`, the same way `output_ownership_adversary.rs` does. That brings 4 of the engine's own `xattr_tests` along. Each case ran alone first and was red; output below with the scratch prefix replaced by `<scratch>`:

- **`a_settled_root_keeps_no_binding_in_a_stale_checkpoint_slot`: red.** A run is cut at `before:rename:checkpoint`. The same command then runs again and changes nothing, and `drift()` (the `--check` comparison) returns `[]`.
  `panicked at …adversary_idle_record_pass1.rs:152:5: a settled .ess-output still names this machine's root: [(".../checkpoint-slot-root/.ess-output/state.next", "checkpoint-slot-root hex-encoded"), …, "\"root\":"), …, "\"directory\":")]`
- **`recovering_a_version_two_transaction_settles_a_record_without_the_binding`: red.** A `/2` checkpoint stopped mid-transaction (state `Prepared`) is recovered with `ownership::recover`.
  `panicked at …:186:5: recovery settled a "ess-output-state/2" record carrying the binding: [("<scratch>/…/recovered-root/.ess-output/state.json", "recovered-root hex-encoded"), …]`
- **`adopting_into_a_version_two_root_settles_a_record_without_the_binding`: red.** `probe::adopt` runs into a root whose settled record a `/2` writer left.
  `panicked at …:233:5: adoption settled a "ess-output-state/2" record carrying the binding: [("<scratch>/…/adoption-target/.ess-output/state.json", "adoption-target hex-encoded"), …]`

**3. The full run, after the cases existed.** Command: `TMPDIR=<scratch>/tmp cargo test -p ess-cli --locked --test adversary_idle_record_pass1 --test output_ownership --test output_ownership_adversary --test output_ownership_correction --test ownership_relocation_adversary_p2 --test generate_check --test generate_check_adversary --test requires_release --test cli_binding --test schema_registry_identity --test normalization --no-fail-fast` ended `EXIT=101`.

- The new binary: 4 passed, 3 failed (the three above).
- The other ten binaries: 4+9+10+18+60+11+5+11+7+9 = 144 passed, 0 failed.
- `<before>` = 144, taken from the same run with my binary left out. `<after>` = 151.

**4. Findings, covering e2ba76b37f.**

| # | file:line | measured | what reaches it | verdict / origin |
|---|---|---|---|---|
| F1 | `mod.rs:1258` | A stale `state.next` holding the pending `/3` checkpoint (path, device, inode) outlives every run that changes nothing; `drift()` never looks at it | Ctrl-C or a kill between the `state.next` fsync and its rename. That cut is one of the design's own fault points, and the guide says to commit `.ess-output` | CONFIRMED / pre-existing (base: `state.json` leaked too). Fix: clear a leftover `state.next` on the no-change return, or have `--check` report it |
| F2 | `mod.rs:1713` | Recovering a `/2` transaction settles as a `/2` record with `root` and `directory` | `ess generate output recover` after a crash under 0.55.0 | CONFIRMED / pre-existing. The design page chooses this ("Recovery and adoption keep the version they read"), but it goes against the story's outcome. Fix: `payload.upgrade()` before the closing checkpoint |
| F3 | `mod.rs:444` | Adopting into a `/2` root writes a new `/2` settled record bound to this machine | `ess generate output adopt` adding an owner to a root first enrolled by 0.55.0 | CONFIRMED / pre-existing. Same cause and fix as F2 |
| F4 | `mod.rs:1248` | The new rule that an edited owned file refuses is only checked once, while the record is read (`settled()`, `mod.rs:567`). `plan()` then reads the file again as the "before" copy and never compares it with the record. An edit made in between is overwritten, and its backup is deleted at cleanup | An editor saving during `ess generate`. There is no hook to inject an edit there, so I did not run it | INFEASIBLE / introduced. Never run: there is no hook to inject the edit, and I could not show anyone reaches that window. Base repaired edits in the same folder on purpose. Fix: `plan()` refuses a selected owned file whose bytes differ from the record |

The decision you need to make: F2 and F3 count as pre-existing under the origin rule, but they fall inside #484's outcome. Do they go back to this unit or into a story of their own?

For the changelog: 0.55.0 refuses a settled `/3` record before writing anything, in both write and `--check` mode, with `error: invalid output state: missing field 'root' at line 1 column 918` (exit 1). A CI job pinned to 0.55.0 will fail from the first `/3` commit, with a message that reads like corruption.

**5. Attacked and not broken.** Except where marked, these are from reading the code plus the existing tests.
- Ownership: an owned path that is a symlink, a directory or a file with extra hard links refuses (`filesystem.rs` `snapshot`). Case-only renames refuse through the ASCII case folding in `aliases()`. Differences in file mode only are no-ops (`mod.rs:1028`).
- A `/2` record in the folder it recorded, with an edited owned file, refuses under option B: an Idle record skips the binding check (`mod.rs:562`).
- Machine paths:
  - A mid-transaction record run from another folder refuses before writing (the unit's own test).
  - Nested and symlinked roots are refused by `discovery()` and `absolute()`.
  - Owner keys, blob and stage names, the producer value and the `.ess-output-init-*` checkpoint carry no path.
  - 0.55.0 against a `/3` record: refused, nothing written (run against the installed binary).
- The unit's "old-reader" test only exercises the new reader. Its claim about 0.55.0 matches what I observed, so I did not raise it as a finding.

**6. Paths written outside the worktree:** none. Scratch is all under `<worktree>/target/wave-scratch/u3-adv/` (`tmp/`, `old-reader/`, `build.log`, `suite.log`). The existing adversary binaries left their usual fixtures in `tmp/`.

```findings
[
  {"file": "crates/edge/ess-cli/src/output_ownership/mod.rs", "line": 1258, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "after a cut before the Staging checkpoint's rename, state.next keeps the pending /3 checkpoint with root and directory; unchanged runs return before clearing it and --check never reports it, so a committed .ess-output names the machine path"},
  {"file": "crates/edge/ess-cli/src/output_ownership/mod.rs", "line": 1713, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "recovering a /2 pending transaction settles a /2 record that still carries root and directory, because recovery keeps the format it read"},
  {"file": "crates/edge/ess-cli/src/output_ownership/mod.rs", "line": 444, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "adopting into a root with a settled /2 record writes a fresh /2 settled checkpoint bound to this machine's root instead of /3"},
  {"file": "crates/edge/ess-cli/src/output_ownership/mod.rs", "line": 1248, "category": "concurrency", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "the option-B refusal of an edited owned file runs only at read time (mod.rs:567) and plan() captures preimages without comparing them to the ledger, so an edit between the two is overwritten and its backup deleted"}
]
```
