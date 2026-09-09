---
format: aep.planning-md/1
id: review-result:execution-recovery-adversary-wave21-pass1
kind: review-result
status: active
title: Execution recovery adversary, wave 21, pass 1
relations:
- reviews: story:review-execution-recovery-implementation
revision: 1
---
unit: story:review-execution-recovery-implementation — worktree ess-execution-recovery-wave21 at HEAD 4f9dfba (base 2900f628)
verdict: red
cases: executed 632→634, red 2
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: 1 path (the assigned TMPDIR)
needs-coordinator: no

Recorded by the wave-21 coordinator from the aep-drive:adversary 0.8.1 (Opus) report as returned, pass 1, 2026-09-09. Harness accounting: 152,100 sub-agent tokens, 64 tool uses, 33.0 min. Workstation path prefixes removed; nothing else changed.

## 1. `git --no-pager diff --stat`

```
 crates/edge/ess-cli/tests/execution_recovery.rs | 150 +++++++++++++++++++++++-
 1 file changed, 146 insertions(+), 4 deletions(-)
```

One path, a test file. (4 deleted lines = `cargo fmt -p ess-cli` rewrapping the `use` list I added `HelmIdentity` to. No existing case altered.)

## 2. Cases added — `crates/edge/ess-cli/tests/execution_recovery.rs`

| line | asserts | now |
|---|---|---|
| 4565 `an_exact_reviewed_repair_pre_state_admits_the_apply_from_baseline` | `observe::decide` on a permit whose `repair_from` **is** the observed drifted pre-state returns `Predicate::ApplyFromBaseline` | red |
| 4597 `the_engine_performs_the_apply_an_exact_reviewed_repair_snapshot_authorizes` | same decision through production `execute`: run 1 refuses `ObservedDrift`, the caller installs the snapshot the invocation itself journalled as a new authority revision (+ quiescence for the retained claim), run 2 must apply | red |

Red output, that filter alone, before the suite:

```
$ cargo test -p ess-cli --test execution_recovery -- --exact \
    an_exact_reviewed_repair_pre_state_admits_the_apply_from_baseline \
    the_engine_performs_the_apply_an_exact_reviewed_repair_snapshot_authorizes
running 2 tests
test an_exact_reviewed_repair_pre_state_admits_the_apply_from_baseline ... FAILED
test the_engine_performs_the_apply_an_exact_reviewed_repair_snapshot_authorizes ... FAILED

---- an_exact_reviewed_repair_pre_state_admits_the_apply_from_baseline stdout ----
thread '...' panicked at crates/edge/ess-cli/tests/execution_recovery.rs:4582:25:
drift refuses unless a new caller authority revision supplies the exact independently reviewed repair_from snapshot (docs/design/review-execution-recovery.md:459), and this one supplied it: ObservedDrift the observed pre-state equals the reviewed repair snapshot; this decision is the caller's to record as a new authority revision

---- the_engine_performs_the_apply_an_exact_reviewed_repair_snapshot_authorizes stdout ----
thread '...' panicked at crates/edge/ess-cli/tests/execution_recovery.rs:4654:5:
an exact independently reviewed repair_from pre-state authorizes the apply (docs/design/review-execution-recovery.md:459): selected: api, checkout, web, legacy-c, legacy-b, legacy-a
settled: (none)
unresolved: api
incomplete execution evidence — ObservedDrift: the observed pre-state equals the reviewed repair snapshot; this decision is the caller's to record as a new authority revision

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 92 filtered out
```

The refusal text is the finding in one line: it tells the caller to record the decision as a new authority revision, and case 2 records it as a new authority revision, and the answer does not change.

## 3. Package gate, after the cases exist

| command | exit | result |
|---|---|---|
| `cargo fmt -p ess-cli -p ess-kubernetes -p ess-xtask -- --check` | 0 | clean |
| `cargo clippy --locked -p … --all-targets -- -D warnings` | 0 | clean |
| `cargo test --locked -p ess-cli -p ess-kubernetes -p ess-xtask` | **101** | stops at `execution_recovery`: `test result: FAILED. 92 passed; 2 failed` |
| `cargo test --locked --no-fail-fast -p …` | **101** | 57 lanes, **executed 634, passed 632, failed 2** |
| `cargo xtask support --check` | 0 | source-support block agrees |
| `cargo xtask generate --check` | 0 | clean |

`executed 632` before = the `--no-fail-fast` total minus my two, and it equals the `632` the implementing commit reported. Only my two lanes are red; the other 56 lanes are untouched.

## 4. Findings

Commit covered: `4f9dfba`, worktree `ess-execution-recovery-wave21`.

| ID | file:line | category | severity | verdict | origin | finding |
|---|---|---|---|---|---|---|
| F1 | `crates/edge/ess-cli/src/recovery/observe.rs:399` | acceptance | blocker | CONFIRMED | introduced | `drift()` returns a `Refusal` on **both** arms, so an exact independently reviewed `repair_from` pre-state refuses exactly like an unreviewed one. `docs/design/review-execution-recovery.md:459` — "Other drift refuses **unless** a new caller authority revision supplies the exact independently reviewed `repair_from` snapshot" — and C07 predicate 2 (`:260`) and predicate 4 (`:268`) and C02 (`:28`). `observe::repair_admits` (`observe.rs:423`) is the function that decides this and has **no production caller** (`grep -rn repair_admits crates/` → one definition, zero uses). **Measured:** `execution_recovery.rs:4582` and `:4654`, exit 101. **What reaches it:** `repair_from` is a declared, validated field of the caller-provisioned authority registry (`model.rs:746`, validated `:774`); the engine path is `execute → Run::operation` (`mod.rs:938`) `→ observe::decide` (`observe.rs:342` apply, `:394` retirement) `→ drift`; and the refusal string the operator is shown (`observe.rs:405-409`) instructs them to do the exact thing case 2 does. **Named fix (not applied):** consult `repair_admits` before returning `drift(...)` in both `decide` and `decide_retirement`, returning `ApplyFromBaseline`/`RemoveBaseline`; ownership/incarnation is already decided upstream at `observe.rs:314`. |
| F2 | `crates/edge/ess-cli/tests/execution_recovery.rs:3886` | judgement | warning | CONFIRMED | introduced | `r20_manual_drift_refuses_implicit_repair_and_an_exact_repair_snapshot_admits_it` never sets `repair_from` — it asserts only that the refusal **string contains** `"repair_from"`. The half its own name promises is unwritten, and it is why F1 shipped green. `repair_from: Some(_)` appears nowhere in the whole suite. |
| F3 | `crates/edge/ess-cli/tests/execution_recovery.rs:3511` | judgement | warning | INFEASIBLE | introduced | `Scenario::grant_quiescence` installs the decision but never **removes the old lock**, which C08 (`:296`) states as a required step of the caller's procedure. So every quiescence-lane case (R16, R21, R26, R29 resume, C08) drives `perform` down `retained.is_some() → published = false` (`mod.rs:852-857`) — the resuming invocation mutates while the store's only exclusion names a dead predecessor — and the path C08 actually describes (lock removed → resuming invocation publishes and releases its own claim) is never executed. INFEASIBLE rather than a blocker: I could not build the concurrent second mutator that would make the missing exclusion bite, because the driver has no full-engine mode (R28 lane is claim/observe only), so I cannot show anybody reaches two simultaneous quiescence-admitted runs. |
| F4 | `crates/edge/ess-cli/tests/execution_recovery.rs:4228` | judgement | note | CONFIRMED | introduced | The secret-containment case's own comment says the sentinel goes into "the two places a caller's own bytes can carry one: the protected kubeconfig's credential, **and the desired document's secret slot**" — only the kubeconfig is seeded. And `walk()` scans `scenario.root` only, while `FixturePlatform::private` (`support/fake_recovery.rs:1157`) returns a path under `std::env::temp_dir()`, so the private values/chart directory the engine writes is outside the scan — the contract's "temporary serialized observations" dimension (`:535`) is unmeasured. I did **not** demonstrate a leak there; this is a coverage claim, not a defect claim. |

Also seen, not raised as rows: `a_stopped_journal_is_closed_and_a_short_completion_is_refused` (`:970`) never publishes a `Completed` — the short-completion refusal it is named for is real (`journal.rs:762-773`) but untested here.

## 5. Attacked and could not break

- **Engine authenticity (C13):** `FixturePlatform` is injected below the `Platform`/`Helm` traits; the in-process families do run production `recovery::execute` — not a re-implementation. Line of attack (1) is answered.
- **Journal grammar R18 refusals:** sequence zero not `Opened`, second `Opened`, gap, torn bytes, wrong nonce/epoch, bad predecessor digest, entry after terminal, duplicated terminal — all refuse in `read_history` (`journal.rs:645-706`); the `.stage-` filter matches what `publish_bytes` writes. No red case found.
- **Claim theft:** no PID, age, timeout or lease anywhere in `journal.rs`; `release_claim` re-reads and compares identity first.
- **Chart admission:** hooks, keep policy, `generateName`, cross-namespace, unknown kind, duplicate address, empty render all refuse in `admit_document`/`admit_rendered`; dependencies are excluded by the exact five-file projection.
- **`Completed` accounting:** a short **and** an over-long count both refuse (`check_operation_grammar`). A `Completed` whose count matches `selected.len()` but whose journal carries no per-operation records is admitted as `Closed` — I could not show any producer of such a journal, so I did not raise it.
- **Docs (attack 8):** `--authority` claims on the four pages match `main.rs:946,1851`; `xtask support --check` exit 0.

## 6. Paths written outside the worktree

- the assigned `TMPDIR` for this unit — 2.6G, cargo scratch plus `ess-fixture-private-*` directories the fixture platform creates under `temp_dir()`. Reused, not created by me; not cleaned.

Everything else went to the assigned in-worktree scratch: `target/review-boundaries-21/scratch/u3-adversary-suite.log`, `u3-suite-nofailfast.log`, `u3-support.log`, `u3-generate.log`.

Lease `U3-adversary` released with `worktree hook session-end`.

## 7. findings

```findings
- file: crates/edge/ess-cli/src/recovery/observe.rs
  line: 399
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'drift() refuses on both arms so an exact independently reviewed repair_from pre-state never admits the apply or the uninstall the contract says it authorizes at docs/design/review-execution-recovery.md:459, and observe::repair_admits has no production caller'
- file: crates/edge/ess-cli/tests/execution_recovery.rs
  line: 3886
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the R20 case is named for the exact-repair-snapshot half it never writes, asserting only that the refusal string mentions repair_from, and repair_from is never set to Some anywhere in the suite'
- file: crates/edge/ess-cli/tests/execution_recovery.rs
  line: 3511
  category: judgement
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: 'grant_quiescence omits C08 step "removes the old lock", so every quiescence lane exercises a resuming invocation that mutates while holding no claim of its own and the documented procedure is never executed; the concurrent second mutator that would make it bite cannot be built because the driver has no full-engine mode'
- file: crates/edge/ess-cli/tests/execution_recovery.rs
  line: 4228
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'the secret-containment case seeds only the kubeconfig despite its comment naming two inputs, and its walk covers scenario.root while the private values directory is created under temp_dir, so the temporary-serialized-observations dimension is unmeasured'
```
