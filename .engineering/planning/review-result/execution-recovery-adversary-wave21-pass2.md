---
format: aep.planning-md/1
id: review-result:execution-recovery-adversary-wave21-pass2
kind: review-result
status: active
title: Execution recovery adversary, wave 21, pass 2
relations:
- reviews: story:review-execution-recovery-implementation
revision: 1
---
unit: story:review-execution-recovery-implementation — worktree ess-execution-recovery-wave21 at HEAD 6a8aff7 (base 2900f628)
verdict: red
cases: executed 646→648, red 2
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: none
needs-coordinator: no

Recorded by the wave-21 coordinator from the aep-drive:adversary 0.8.1 (Opus) report as returned, pass 2, 2026-09-09. Harness accounting: 135,464 sub-agent tokens, 58 tool uses, 37.6 min. Workstation path prefixes removed; nothing else changed.

## 1. `git --no-pager diff --stat`

```
 crates/edge/ess-cli/tests/execution_recovery.rs | 221 ++++++++++++++++++++++++
 1 file changed, 221 insertions(+)
```

One path, a test file. No implementation, docs or model file touched.

## 2. The cases I added (both red now)

**A. `crates/edge/ess-cli/tests/execution_recovery.rs:5592` — `c07_step8_the_authored_comparison_is_skipped_on_the_desired_matches_predicate`**

Two halves, one chart. A `Helm` wrapper (`AuthoringHelm`) that renders `spec.replicas: 3` on top of the fixture's render — the fixture's own render emits only `apiVersion`/`kind`/`metadata`, so every existing case's authored comparison is over four fields the live object carries by construction. Half 1 (control, passes): against the baseline pre-state the apply runs and the post-mutation readback refuses `ObservedDrift`. Half 2 (red): same chart, same live objects, but they already carry the caller-approved desired fingerprints, so `decide` answers `DesiredMatches` and `mod.rs:978` returns before any authored field is looked at.

```
running 1 test
test c07_step8_the_authored_comparison_is_skipped_on_the_desired_matches_predicate ... FAILED

thread '...' panicked at crates/edge/ess-cli/tests/execution_recovery.rs:5623:5:
assertion `left == right` failed: the desired-matches predicate compares the caller's fingerprint and never the authored fields, so a live object that contradicts the chart is reported settled: settled ["api", "checkout", "web", "legacy-c", "legacy-b", "legacy-a"], complete true
  left: None
 right: Some(ObservedDrift)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 105 filtered out; finished in 7.25s
```

**B. `crates/edge/ess-cli/tests/execution_recovery.rs:5648` — `c05_different_bytes_at_a_retained_generation_are_admitted`**

A clean run leaves retained evidence (`Opened.registry` = generation G, digest D). The registry is republished at the **same** generation with different bytes (a new authority revision, both immutable archives rewritten, so `read_registry`'s archive check still passes). The case first asserts the condition really holds (same generation, different digest), then asserts the second invocation refuses.

```
running 1 test
test c05_different_bytes_at_a_retained_generation_are_admitted ... FAILED

thread '...' panicked at crates/edge/ess-cli/tests/execution_recovery.rs:5682:5:
assertion `left == right` failed: different bytes for an already retained generation are rejected: settled ["api", "checkout", "web", "legacy-c", "legacy-b", "legacy-a"], complete true
  left: None
 right: Some(AuthorityMismatch)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 105 filtered out; finished in 8.53s
```

Disclosure on order: the first version of case B changed the bytes by setting `may_create: true` on the permits and went red with `InvalidInput` — the model refuses `may_create` beside a baseline (`model.rs:767`), a legitimate refusal but not the one I claim. I replaced the byte-edit with the revision bump and re-ran; the output above is that run. A later edit corrected two binding line citations in the new doc comments (`:250`, `:223`); comments only, and both cases were re-run red afterwards.

## 3. The suite, after the cases existed

```
cargo test --locked -p ess-cli -p ess-kubernetes -p ess-xtask          → exit 101
  execution_recovery: test result: FAILED. 104 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 101.51s
cargo test --locked --no-fail-fast -p ess-cli -p ess-kubernetes -p ess-xtask
  57 lanes, passed 646 / failed 2 / executed 648, exit 101
  error: 1 target failed: `-p ess-cli --test execution_recovery`
cargo fmt -p ess-cli -p ess-kubernetes -p ess-xtask -- --check          → exit 0
cargo clippy --locked -p ess-cli -p ess-kubernetes -p ess-xtask --all-targets -- -D warnings → exit 0
cargo xtask support --check                                            → exit 0
cargo xtask generate --check                                           → exit 0
```

`executed 646` before is the implementing state's own number (commit 6a8aff7 body: "632 -> 646 executed, exit 0 (57 lanes)"); 648 after is the `--no-fail-fast` run above, same 57 lanes.

## 4. Findings

| ID | file:line | category | severity | verdict | origin | finding |
|---|---|---|---|---|---|---|
| F1 | `crates/edge/ess-cli/src/recovery/mod.rs:978` | acceptance | blocker | CONFIRMED | introduced | **measured:** case A, `:5623`, `left: None right: Some(ObservedDrift)`, all six operations settled, `complete true`. **reaches it:** any ordinary rerun/restart against a target already at the desired fingerprints — the R21 lane, the R15/R25 restarts and every repeated `reconcile`; a chart that authors a spec field is the production case, the fixture's field-free render is the artificial one. Binding `:250` ("The existing authored-field and caller-approved complete live-projection comparisons apply to the projection being compared") and `:223` ("Thus a caller fingerprint cannot silently override a differing authored image, selector, replica count or secret reference"). `observe::authored_holds` is wired at exactly one site, `mod.rs:1108`, the post-mutation readback; the pre-state `decide` path takes `observe::snapshot` (`mod.rs:957`), which discards the live objects, so predicate 1 and predicate 2 both compare digests only. Named fix: have `operation()` use `observe::observe` and run `authored_holds(&prepared.rendered, &live)` against the projection being compared before returning on `DesiredMatches`. |
| F2 | `crates/edge/ess-cli/src/recovery/authority.rs:587` | contract-drift | blocker | CONFIRMED | introduced | **measured:** case B, `:5684`, `left: None right: Some(AuthorityMismatch)`, six operations settled under bytes the retained evidence never named. **reaches it:** a caller republishing an authority revision without bumping the registry generation — `read_registry` compares the snapshot only with `registry-history/<generation>.json`, which the same publication rewrites, and `authority::recheck` compares only within one invocation. Binding `:151` is one sentence with two halves: "Reject a generation older than retained execution evidence, **or different bytes for an already retained generation**." `admit_generation(retained: Index, current: Index)` takes no digest and its only caller (`mod.rs:805`) passes two generations; the doc comment states only the first half. Named fix: pass the whole `RegistryRef` and refuse `current.generation == retained.generation && current.digest != retained.digest`. |
| F3 | `crates/edge/ess-cli/tests/execution_recovery.rs:5319` | mutant | warning | CONFIRMED | introduced | **measured:** the R19 process lane's launch assertion is `text.contains("calls: \n") || text.contains("calls: ")`; the second disjunct is satisfied by `"calls: apply api"`, so the assertion cannot fail while the driver prints a call list at all. **reaches it:** an implementation that launched despite the expired observation keeps this lane green — the sibling lanes at `:5351` and `:5397` use the strict forms (`trim_end().ends_with("calls:")`, `contains("calls: \n")`), so the strict form was available and one lane lost it. Fix: use the `:5351` form. Not made into a case of my own: the strict assertion is green today, so writing it proves nothing about the defect. |
| F4 | `docs/design/review-execution-recovery.md:296` | judgement | note | INFEASIBLE | introduced | The brief asked me to read C08 myself on F3-of-pass-1. **The implementor's reading holds in part.** C08 carries no refusal code for "decision installed, lock still present", and the decision is what it makes authoritative — so manufacturing a refusal was the right call. But `:293` ("The next invocation may read and report current observations while an old claim remains, but it cannot mutate or report completed recovery") and `:296`'s four-step order (archive → install decision → **remove the lock** → re-enable execution) together mean the state `a_retained_lock_and_the_callers_decision_decide_different_things:5442` asserts success in — mutating with the predecessor's lock on disk — is one the binding's procedure never reaches, because execution is disabled until the lock is gone. INFEASIBLE rather than a case: I can build the state, and I cannot show the documented workflow produces it. The residue is that the correction's commit message justifies it with "the adversary's own repair case depends on an invocation proceeding under one while the archived claim is still on disk" — the repair case runs after `grant_quiescence`, which removes the lock; the *archived* copy is not the lock, and the two should not be conflated in the record. |

## 5. Attacked, could not break

- `observe::authored_holds` itself (brief 1): one-directional recursion is correct — an extra live field does not refuse, a differing authored field does (case A's control half proves it), and `live_projection` excludes uid/resourceVersion/generation/status while `PresentObject` records uid and resourceVersion separately. Deployment, StatefulSet and Service all go through the identical path.
- `authority::admit_generation` (brief 2): the backwards-generation floor holds against every retained `Opened`; equal generation admits, older refuses. Only the bytes half is missing (F2).
- The full-engine driver (brief 3): `recovery_driver.rs:263` calls the production `ess_cli::recovery::execute` with an injected `FixturePlatform` — not a second implementation. The synthetic target is a file read back by a second process after the first exits (`:5272`), so the effect really outlives the journal.
- `observe::repair_admits` (brief 5): `ReleaseSnapshot` equality is whole, and the Helm identity inside it carries `storage_uid` and the ownership marker, so a snapshot reviewed for another release or another Helm identity cannot compare equal; foreign ownership and a competing occupant are decided outside both branches (`observe.rs:339`, `:375`).
- The orphan-function test (brief 6): I scanned all 63 module-level `pub fn` with the test's own matching rule — no function is kept alive by a comment-only or a string-literal match, and every single-caller function's one hit is real code. I did not mutate the tree to watch it go red; hard rule 1 has no scratch exception and a copied-tree replica would have been testing my replica.
- Secret containment (brief 8): live object content never leaves memory — `ReleaseSnapshot` persists digests, uid and resourceVersion only, and no refusal message interpolates object content. A third sentinel inside an observed object has nowhere to land, so I wrote no case for it.
- R28 with two engines at the sync barrier and at claim publication (brief 4): not attempted — the driver's fault injection has no barrier hook for those two points and building one would have meant rewriting shared fixture behaviour other cases depend on.

## 6. Paths written outside the worktree

None. Scratch used: `target/review-boundaries-21/scratch/suite-after.txt` (the `--no-fail-fast` suite log), inside the worktree and inside the assigned scratch directory.

```findings
- file: crates/edge/ess-cli/src/recovery/mod.rs
  line: 978
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'C07 step 8 is wired only into the post-mutation readback, so the DesiredMatches predicate returns settled for a live object whose authored chart fields it contradicts, which is the exact override step 8 exists to prevent'
- file: crates/edge/ess-cli/src/recovery/authority.rs
  line: 587
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: 'admit_generation implements only the first half of the binding at :151 — different bytes for an already retained generation are admitted, and the registry generation archive cannot catch it because the same publication rewrites it'
- file: crates/edge/ess-cli/tests/execution_recovery.rs
  line: 5319
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the R19 process lane asserts contains("calls: ") as a disjunct, which any call list satisfies, so a launch past the expired observation would keep the lane green'
- file: docs/design/review-execution-recovery.md
  line: 296
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: 'C08 carries no refusal for a resuming engine under a decision with the lock present so the implementor was right not to manufacture one, but the case at :5442 asserts success in a state the binding says is execution-disabled, and the commit message justifies it by conflating the archived claim with the retained lock'
```
