---
format: aep.planning-md/2
id: review-result:consumer-outside-boundary-source-pass-2
kind: review-result
status: active
title: Finite outside-boundary mechanism source examination pass 2
relations:
- reviews: task:consumer-accounting-authored-boundaries
revision: 1
---
unit: the corrected finite outside-consumer-boundary accounting mechanism at `cc8c7f020275f9b3c22d69a8d330f18a312f02f7`, tree `e835ae55c52067679190b77a320c49675a9ee571`, plus four appended test cases; whole-mechanism base `da30abadcc3555b554a600216f88a4352738afdc`, correction parent `00a707e8f76d23e0231508491affc3381daf36df`
verdict: CONFIRMED — one `note`, no blocker, no NEEDS-CHANGE, no red case
cases: executed 204→208, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths
needs-coordinator: the disposition of the §4 note, and the still-absent S4/S5/S13 and authority inputs §6 names

---

## 1. Diffstat — proof of what was touched

The review worktree `ess-evolution-s3-source-review-2-20260916` has since been retired, so this is
read from the retained patch and the two execution manifests rather than from a live `git diff`.
Both are in this directory.

```
 .../consumer_coverage/outside_boundary_tests.rs    | 190 +++++++++++++++++++++
 1 file changed, 190 insertions(+)
```

`cases.patch`: one `+++` path, `crates/edge/ess-xtask/src/consumer_coverage/outside_boundary_tests.rs`;
190 added lines (175 non-blank), **0 deleted and 0 modified**. Every added line sits below the
pass-1 block at line 1177. **Every path in that diff is a test file**, and there is no second path —
no production file, no visibility, no authority, no manifest, no fixture on disk. The three fixture
mutations the cases need happen inside the cases, on the `Parts` value each builds for itself.

Correspondence between what I reviewed and what root executed, verified here rather than assumed:

| Check | Result |
| --- | --- |
| `review-files.sha256` vs `execution-files.sha256` | byte-identical, 1,548 paths each (`cmp -s`, exit 0) |
| `outside_boundary_tests.rs` in both manifests | `3a31a686f6fd8dbd5f61e13223f5330923dce56a108b4525566af2c70161145a` — the hash READY.md recorded before anything ran |
| the other seven mechanism files vs `cc8c7f02` | `account.rs`, `aggregate.rs`, `enforce.rs`, `mod.rs`, `outside_boundary.rs`, `reconciliation.rs`, `rust.rs` — all seven equal, by `git show cc8c7f02:<path> \| sha256sum` against the executed manifest |
| `docs/design/consumer-outside-boundary-accounting.md` in the executed manifest | `4b1e3fa0e103073cf849f6953f78418d55dcf20f2f927e59b5366f92883fc48f`, the accepted digest |

So the executed tree differs from the examined source in exactly one file, and that file differs by
exactly these four cases. Root executed in its own tree
(`…/ess-evolution-s3-boundary-mechanism-20260916`, per every log's `Compiling` line); the manifest
equality above is what binds that tree to the one I read, not the path.

## 2. The four cases I added

All four were written before anything was executed, and all four were declared **predicted GREEN**
in `READY.md` §3 before root held them. **There is no red output to quote, because none was
predicted and none occurred.** These are coverage of decisive controls the accepted design names
and the suite did not assert; they are not defect claims, and I do not present them as one.

`<before>` = 204 has two independent sources and neither is a pre-emptive suite run: the pass-1
coordinator disposition's `main 204 passed`, and each focused run below reporting `207 filtered out`
against 1 selected. `<after>` = 208, the bin target's own count in `package.log:215`.

| Case | Asserts | Now |
| --- | --- | --- |
| `the_accounting_5_candidate_reader_refuses_both_relabellings_and_a_malformed_digest` | a v4 candidate relabelled `/5` refuses, a v5 candidate relabelled `/4` refuses, and four malformed `outside_boundary_sha256` values (non-hex, 64-char uppercase, empty, 65-char) each refuse | green |
| `an_absent_ambiguous_or_foreign_entrypoint_owner_refuses_at_g2` | two packages carrying the reviewed entrypoint refuse; none carrying it refuses; one unambiguous owner that is not `consumer_package` refuses | green |
| `a_resolvable_model_package_outside_the_workspace_refuses_at_g3` | a package present in `resolve.nodes` and absent from `workspace_members`, whose obligation owner matches, refuses on the membership clause and **not** on G1 | green |
| `an_outside_boundary_cell_naming_another_declared_boundary_refuses_in_verify` | a planned cell whose `boundary` names another *declared* boundary refuses in `verify`, which reads cells through `plan_cells_v5` rather than `read_plan_v5` | green |

What each one covers, and why it was not already covered:

- **C1** — design line 270 requires that relabelling refuses in both directions. The plan layer had
  both (`outside_boundary_tests.rs:501-507`); the candidate layer had neither.
  `account::read_candidates_v5` (`account.rs:228`) had one caller, `mod.rs:596`, and no test, so its
  only bespoke check — 64 lowercase hexadecimal characters, `account.rs:231-238` — could have been
  deleted without changing a test.
- **C2** — design G2 bullet 2 requires exactly one inventory package to carry the entrypoint and
  that package to be `consumer_package`. The existing G2 case rotates digests, which refuse one
  comparison later at `outside_boundary.rs:457-476`; the ownership branch at
  `outside_boundary.rs:443-456` had no case.
- **C3** — design G3 requires `model_package` to be a workspace package. The existing G3 case uses
  `fixture-other`, which *is* a member, so it stops at the owner comparison at
  `outside_boundary.rs:594`; the membership clause at `outside_boundary.rs:601-606` had none.
- **C4** — `verify` consumes `plan["cells"]` through `plan_cells_v5` (`outside_boundary.rs:555`),
  not through `read_plan_v5`, so the reader's check at `enforce.rs:2314-2317` is not what protects
  the guard. `verify`'s own check at `outside_boundary.rs:558-562` had no case; the appended pass-1
  case covers the count comparison at `outside_boundary.rs:695`, a different branch.

Each case, run alone before the suite. Command, then its own output and exit, verbatim from this
directory's `.log` and `.exit` files.

```
cargo test --locked --offline -p ess-xtask --bin ess-xtask -- --exact --nocapture \
  consumer_coverage::outside_boundary_tests::<case>
```

| Case | `test result` line | exit |
| --- | --- | --- |
| C1 | `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 207 filtered out; finished in 0.09s` | 0 |
| C2 | `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 207 filtered out; finished in 0.19s` | 0 |
| C3 | `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 207 filtered out; finished in 0.07s` | 0 |
| C4 | `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 207 filtered out; finished in 0.07s` | 0 |

Each selected 1 and filtered 207: the filter matched, and the count moved 204→208. No case was
added and left unselected.

## 3. The suite, after the cases in §2 existed

```
cargo test --locked --offline -p ess-xtask      exit 0   package.log / package.exit
cargo clippy --locked --offline -p ess-xtask --all-targets -- -D warnings   exit 0   clippy.log / clippy.exit
cargo fmt -p ess-xtask -- --check               exit 0   fmt.log (0 bytes) / fmt.exit
```

`package.log`, 13 test targets, all executed:

```
running 208 tests
test result: ok. 208 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 53.26s
```

Totalled across all 13 targets: **278 passed, 0 failed, 3 ignored**. `clippy.log` is two lines,
`Compiling` and `Finished`, with no warning or error; `fmt.log` is empty.

**Ignored-case disclosure.** The 3 ignored are all in `tests/host_paths_adversary_6.rs`
(`package.log:318-325`), which reports `0 passed; 0 failed; 3 ignored`. Each carries its own reason
naming `story:the-unread-tree-bullet-is-read-whole`. These are the same three pre-existing ignored
cases the pass-1 correction disclosed; they concern an unread-tree bullet parse, not this mechanism,
and nothing in this pass changed an ignored case, a test selection or an ignore reason.

## 4. Judgement findings

One, covering commit `cc8c7f02`. It does not hold the unit.

| | |
| --- | --- |
| **finding** | `enforce.rs:2582` — the conservation check in `qualify_v5` cannot fail |
| **what was measured** | `counts.values().sum::<usize>() != plan.cells.len()`. `counts` is seeded with the six closed kinds, `kind_v5` returns one of exactly those six, and each cell increments exactly one, so the sum equals `plan.cells.len()` by construction. The second disjunct, `counts["OutsideConsumerBoundary"] != plan.pending_outside_boundary_pairs`, is already enforced by `read_plan_v5` at `enforce.rs:2335`, which `qualify_v5` calls first at `enforce.rs:2522`. |
| **what reaches it** | `mod.rs:761`, `check_at` → `qualify_v5`, on the ordinary `task consumer-check` path. The line is reached; it is the *refusal* that is unreachable. |
| **verdict / origin / severity** | CONFIRMED / introduced / `note` |

**This is disclosure, not a change request.** The conservation the design requires at line 63-68 is
genuinely enforced, at `enforce.rs:2332` (`models.len() * profiles.len() == plan.cells.len()`), and
the separate outside-boundary count is enforced at `enforce.rs:2335` and `enforce.rs:2583`. Nothing
is unchecked. Removing or tightening the redundant line is optional cleanup for whoever next owns
this file; it is **not** a prerequisite for this unit, and it must not be recorded as required
implementation, a new review unit or an expanded acceptance condition.

It is not expressible as a failing case: asserting that a branch is unreachable needs a mutation of
the file under review, which the charter forbids and which I did not perform.

## 5. What I attacked and could not break

One line each; none produced a case.

- **The authority shape.** All 19 boundary and pair fields are checked non-blank or as 64-lowercase-hex; ordering is strict-ascending on `consumer` and on `(model, consumer)`; the 3/168 counts, the used-by-at-least-one-pair rule, the undeclared-boundary refusal and the duplicate-key deserializer all hold.
- **G1.** Traverses `deps` over every `dep_kinds` entry without filtering by kind, refuses an absent `resolve`, an ambiguous consumer package, a node missing from the graph and a dependency declaring no kind; records `traversed_nodes` and `resolve_sha256` as the design requires.
- **G2.** All six design bullets are compared, and the entrypoint-ownership branch is now covered by C2.
- **G3.** All four design clauses are compared, and the workspace-membership clause is now covered by C3.
- **G4.** Requires exactly one planned cell at the exact four-part downstream coordinate carrying executed cases, all of them in this run's executed union; a `BaselineUnknown`, `AggregateClosure`, `SchemaDocumentMetadata` or `OutsideConsumerBoundary` cell at that coordinate yields `None` and refuses.
- **The private token.** `VerifiedOutsideBoundary` has private fields, no `Deserialize`, no `From<Value>`, no public and no test-only constructor; `verify` is its only producer. A hand-written `outside-boundary-proof.json` constructs nothing. The forged-proof control is discharged by the type.
- **Plan ordering.** The 168 cells are inserted after the aggregate cases and before the replacement match, the stale check and the unaccounted sweep — the order design line 302-307 requires.
- **The old formats.** The only pre-v5 edits in `enforce.rs` are two `#[allow(dead_code)]` attributes, one `#[cfg(test)]` accessor and one dispatch arm in `aggregate_cells`; the single deleted line in the whole mechanism diff is a Clippy `reason` string. `DispositionV2` gained no variant.
- **The correction itself.** `executed_downstream_cases` now counts the unique cases the reviewed pairs' downstream cells bind, collected while G4 checks them; `executed` is used only as a membership test.
- **The design's own factual claims**, checked read-only and every one matched: `spec.rs` = `c4a73f9c…f332a977` with zero `ess_compiler` occurrences and its three items at lines 151/219/266; baseline `3dd8dff5…374a47de`; the G1 reading replayed at 91 nodes from `ess-domain` (compiler not reached) and 107 from `ess-compiler` (domain reached), `ess-primitives` the only workspace dependency; 56 models / 168 pairs / 3 consumers, all 168 classified `UNPROVED_NO_ALLOWED_DISPOSITION`; 1,813 frozen models, 47 absent (141 pairs) and 9 changed (27 pairs) with all 27 frozen shapes equal to the recorded `old_shape`; all three profile hashes equal their frozen group hashes with owner `ess-domain` and follow-up `epic:qualify-initial-consumer-baseline`; 2,061 loader claims and **zero** at G4's seven compiler profiles.

## 6. Integration state — unchanged by this pass, and not this pass's to close

`crates/edge/ess-xtask/src/consumer_coverage/reviewed-outside-boundary.json` does not exist in this
tree, and neither does `reviewed-reconciliation.json` or `reviewed-aggregate-closures.json`. Both
`plan_extraction` (`mod.rs:579`) and `check_at` (`mod.rs:745`) read the outside-boundary authority by
path, so the production path cannot run today. With 2,061 loader claims and zero at the seven
compiler profiles, G4 would refuse every one of the 168 pairs as things stand.

This matches the accepted design's complete-input precondition and the brief exactly. **No
production pair is claimed qualified**; the mechanism's behaviour is established by fixture only.
The missing S4/S5/S13 evidence, the complete authorities and both real ordinary-gate experiments
(made-reachable, body-mutation) remain root-owned, and this examination neither discharges them nor
adds to them.

## 7. Scope of this pass

This is SOURCE PASS 2 OF 2 and it closes here. It records the final source examination and nothing
else: no source commit, no cleanup, no production change, no manifest, no authority, no planning or
root file was written, no third examination is proposed, and no prerequisite is expanded. Full ESS
acceptance, the complete matrix and S3 qualification are not claimed and are not this report's to
claim. Surviving issues go to root disposition.

## 8. Paths written outside the worktree

Five, all under the assigned scratch directory
`home-path:sha256:9e2f12589b5ba5f4101cc79d70716a4ccabd2256b35ed9fdc6db4791f898ba74`:

- `READY.md`
- `report.md` (this file)
- `g1-replication.py` — the G1 traversal replay in §5
- `design-facts.py`, `downstream-facts.py` — the count and digest checks in §5

No `/tmp`. Everything else in this directory — the four `.log`/`.exit` pairs, `package.*`,
`clippy.*`, `fmt.*`, `cases.patch`, the two `.sha256` manifests, `cli.*`, `finalize.*` — was written
by root, not by me.

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs
  line: 2582
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the v5 qualified conservation check cannot fail, because counts is seeded with the six closed kinds and each cell increments exactly one, and the real conservation is already enforced at enforce.rs:2332 and enforce.rs:2335
```
