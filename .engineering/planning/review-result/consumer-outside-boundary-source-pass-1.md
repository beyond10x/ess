---
format: aep.planning-md/1
id: review-result:consumer-outside-boundary-source-pass-1
kind: review-result
status: active
title: Finite outside-boundary mechanism first source examination
relations:
- reviews: task:consumer-accounting-authored-boundaries
revision: 1
---
unit: the finite outside-consumer-boundary mechanism at 00a707e8f76d23e0231508491affc3381daf36df (tree 09ebf5dfd23fa5eb474d5902ca74e290d10f2826), plus the test-only delta below
verdict: NEEDS-CHANGE
cases: executed 202→204, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths
needs-coordinator: the one-line receipt fix is named in part 4 and not applied; 03-package stopped at the bin target, so the 12 integration targets and their 3 retained ignored cases did not run in this pass

# First source examination — outside-consumer-boundary mechanism

Scope: the exact submission `00a707e8` against base `da30abad`, eight files, 3,462 insertions.
Design read whole: `docs/design/consumer-outside-boundary-accounting.md` =
`4b1e3fa0e103073cf849f6953f78418d55dcf20f2f927e59b5366f92883fc48f`, plus
`consumer-accounting-reconciliation.md` and `consumer-accounting-applicability.md` as its two
predecessors, and the implementation contract `s3-mechanism-brief.md`.

Review tree `…/ess-evolution-s3-source-review-1-20260916`, HEAD `00a707e8`, no commit, no store
edit, no stash, no cleanup. Root executed in
`…/ess-evolution-s3-boundary-mechanism-20260916` after adopting only `cases.patch`;
`review-files.sha256` and `execution-files.sha256` are byte-identical (1,548 rows each, `diff -q`
reports no difference), and both carry
`4d04cec0b25d1ff5016b379b120539cc420b428ac77794ca6421ea327a154f76` for the one changed file. Root's
message states 1,549 non-planning files; the two manifests each hold 1,548 newline-terminated rows.
Root producer `72904/86ac5a`, terminal 101.

## 1. `git --no-pager diff --stat`

```
 .../consumer_coverage/outside_boundary_tests.rs    | 64 ++++++++++++++++++++++
 1 file changed, 64 insertions(+)
```

`git status --porcelain`: `M crates/edge/ess-xtask/src/consumer_coverage/outside_boundary_tests.rs`

One path, and it is a test path. No production file was mutated, not even briefly; no authority
JSON, design, baseline or planning file was touched; no existing case, assertion, helper or
visibility was altered; no `#[cfg(test)]` accessor was added to any production module. The 64 lines
are appended below the previous last case under a dated banner.

## 2. The cases I added, and the run of each alone, before the suite

`crates/edge/ess-xtask/src/consumer_coverage/outside_boundary_tests.rs`, both appended.

| Case | Asserts | Now |
| --- | --- | --- |
| `the_proof_counts_only_the_downstream_cases_its_pairs_bind` | the guard receipt's `executed_downstream_cases` equals the number of distinct cases the reviewed pairs' downstream cells bind | **red** |
| `a_planned_outside_boundary_cell_no_reviewed_pair_declares_refuses` | `verify` refuses a plan carrying a 169th outside-boundary cell no reviewed pair declares | green |

### 01-count-case — exit 101, verbatim

```
   Compiling ess-xtask v0.24.0 (home-path:sha256:05be5d66897ea0cbcf37fb6b118003d09b29da0fd7ac5df1365f72b60d00801a)
    Finished `test` profile [unoptimized] target(s) in 9.63s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-933e37d58b3a5a7c)

running 1 test

thread 'consumer_coverage::outside_boundary_tests::the_proof_counts_only_the_downstream_cases_its_pairs_bind' (2440591) panicked at crates/edge/ess-xtask/src/consumer_coverage/outside_boundary_tests.rs:1146:5:
assertion `left == right` failed: the receipt counts this run's whole executed union, not the downstream cases its pairs bind: {"aggregate_qualified_rows":0,"authority_sha256":"8e730c51f0b108d26cc7423aeadd76aaecf6f98d65c2709dbbfa4eef0626f621","baseline_sha256":"3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de","boundaries":3,"claim_limit":"No behavior was executed at these consumers and none is claimed. OutsideConsumerBoundary is neither support nor refusal, extends no baseline, and qualifies no aggregate row; the retained downstream obligation stays with the consumer that reaches the model.","dependency":{"boundaries":[{"consumer":"authored-admission","consumer_package":"fixture-domain","traversed_nodes":2,"unreachable_packages":["fixture-compiler"]},{"consumer":"authored-assembly","consumer_package":"fixture-domain","traversed_nodes":2,"unreachable_packages":["fixture-compiler"]},{"consumer":"authored-validation","consumer_package":"fixture-domain","traversed_nodes":2,"unreachable_packages":["fixture-compiler"]}],"resolve_sha256":"34ebd949ec28b0ad0a5978a8ca4333327b4144aa9e135cd603af7deecdcb8176","resolved_nodes":4},"downstream_consumers":{"cli-binding-process-execution":56,"cli-binding-resolution":56,"cli-binding-rust-emission":56},"executed_downstream_cases":2,"format":"ess-consumer-outside-boundary-proof/1","models":56,"pairs":168,"provider_profile_sha256":"0f51cf0d94a70f309f1ecc59e1f8544f199ef9995ffda4921dd21b0ca330eec8","qualified_outside_boundary_cells":168,"source_sha256":"1c01e76fa2dff3b0510be169bc1d504de71d0c9a11c2a3356d8164c53026c5ba"}
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test consumer_coverage::outside_boundary_tests::the_proof_counts_only_the_downstream_cases_its_pairs_bind ... FAILED

failures:

failures:
    consumer_coverage::outside_boundary_tests::the_proof_counts_only_the_downstream_cases_its_pairs_bind

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 203 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

It failed for the reason claimed, and the reason is legible in the receipt it printed: the guard
consumed one downstream case (`downstream_consumers` totals 168 pair rows over three consumers, all
bound to the single fixture case) and the receipt reports `"executed_downstream_cases":2`, the size
of the executed union the case handed in.

### 02-extra-pair-case — exit 0, verbatim

```
   Compiling ess-xtask v0.24.0 (home-path:sha256:05be5d66897ea0cbcf37fb6b118003d09b29da0fd7ac5df1365f72b60d00801a)
    Finished `test` profile [unoptimized] target(s) in 9.67s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-933e37d58b3a5a7c)

running 1 test
test consumer_coverage::outside_boundary_tests::a_planned_outside_boundary_cell_no_reviewed_pair_declares_refuses ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 203 filtered out; finished in 0.07s
```

Green, and that is the result: `outside_boundary.rs:693-695` is live. It was untested before, and it
is the only thing between the execution plan and an outside-boundary exception no review covers.

## 3. The suite, after both cases existed

`03-package`, exit 101:

```
running 204 tests
…
failures:
    consumer_coverage::outside_boundary_tests::the_proof_counts_only_the_downstream_cases_its_pairs_bind

test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 54.22s

error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

The single failure is the new count case, at the same `:1146` assertion and the same `left: 2 /
right: 1`. Every one of the 203 other cases in the bin target passed, including the mechanism's own
18 controls and the whole retained v1–v4 suite. 202 → 204 is the two cases I added.

**Cargo stopped at the bin target.** The 12 integration targets did not run in this pass, so the
three ignored cases were not re-observed here. Their standing is unchanged and comes from
`s3-mechanism/coordinator/09-package.log:312-319`: `tests/host_paths_adversary_6.rs`, 3 ignored, 0
passed, all three against `story:the-unread-tree-bullet-is-read-whole`
(`a_bullet_between_the_anchors_is_dropped_unless_it_is_spelt_with_an_asterisk`,
`a_closing_anchor_inside_a_bullet_truncates_the_list_without_a_panic`,
`the_parse_cannot_tell_the_corrected_bullet_from_the_one_the_correction_removed`). They are
pre-existing, concern an unread-tree bullet parser, and are untouched by this mechanism. Every other
integration target in that log reports `0 ignored`.

`04-clippy`, exit 0 — `Finished dev profile [unoptimized] target(s) in 7.29s`, no diagnostic.
`05-fmt`, exit 0 — empty output. Both ran with the new cases present.

## 4. Judgement findings

### F1 — `executed_downstream_cases` reports the whole run's executed union

| | |
| --- | --- |
| what was measured | `outside_boundary.rs:715` writes `"executed_downstream_cases":executed.len()`. Red at `outside_boundary_tests.rs:1146`, `left: 2 / right: 1`, exit 101 (`01-count-case`), reproduced in `03-package` |
| what reaches it | every ordinary `task consumer-check` run. `mod.rs:746-756` passes `executed_union` — `mod.rs:727-731`, the union of legacy and model cases — as `executed`. `implementation-result.md` §5.4 records 2,061 current loader claims; G4's own consumption is at most 56 cases, one per model |

`verify`'s G4 loop (`outside_boundary.rs:656-682`) is the only consumer of an executed case in this
module, and it consumes exactly the `cases` of the planned downstream cells the 168 reviewed pairs
name. The receipt names that quantity and carries a different one. On the real matrix the field will
read 2,061+ where the evidence it describes is at most 56 — an over-count of roughly 40×, in the
direction that makes the proof look stronger than it is.

Bounded reachability and severity. The field is written once and read nowhere: `rg
executed_downstream_cases` returns exactly one hit, in `outside_boundary.rs`. It is not compared in
`qualify_v5`, not summed, not a guard input. It therefore changes no refusal and gates nothing; the
consequence is confined to the two persisted artifacts a human audits — `outside-boundary-proof.json`
(`mod.rs:757-760`) and `qualified-cells.json.outside_boundary_proof` (`enforce.rs:2601`). Severity
`warning`: it should be corrected before the mechanism produces a receipt anybody reads, and it does
not hold the unit.

Origin `introduced`, checked rather than inferred: `git show
da30abadcc3555b554a600216f88a4352738afdc:crates/edge/ess-xtask/src/consumer_coverage/outside_boundary.rs`
exits non-zero — the module does not exist at the base, so the defect cannot reproduce there.

**The fix, named and not applied.** In `verify`, collect the cases G4 actually validates. The loop
already holds them: at `outside_boundary.rs:676`, after the membership check passes, extend a
`BTreeSet<String>` with `cases`, and at `:715` report that set's length instead of `executed.len()`.
One added binding, one changed expression, no signature change, no guard change. My case then goes
green on its own assertion. I did not apply it; hard rule 1.

### F2 — the implementation result misattributes the three ignored cases

| | |
| --- | --- |
| what was measured | `s3-mechanism/implementation-result.md` §2 places the three ignored cases in `tests/internal_names.rs`. `s3-mechanism/coordinator/09-package.log:321-331` reports `tests/internal_names.rs` as 6 passed, 0 ignored; the three ignored cases are in `tests/host_paths_adversary_6.rs` at `:312-319` |
| what reaches it | any reader following the submission's own disclosure to those cases, including the next examination pass |

Severity `note`. The disclosure itself is honest and the count is right; only the file name is
wrong. The code is unaffected. Verdict CONFIRMED, origin `introduced` — the document is this unit's.

## 5. What I attacked and could not break

One line each. Nothing below produced a case, and that is the result.

- `plan_v5` against `plan_versioned` + `plan_v4`: same widening, same admitted insertion order, same duplicate/stale/unaccounted sweeps, `PreferExactCurrentBehavior` still built from behavior cells only.
- `read_plan_v5` against `read_plan_versioned` + `validate_model_partition`: every v2 comparison present, four added (reconciliation format, outside-boundary digest, pair count, duplicate requirement rows); none dropped.
- `qualify_v5` against `qualify_v4`: strict superset; the token, the authority digest and the cell set are all bound to the plan.
- `resolve_v2` against `resolve`, `verify_reconciliation_v2` against `verify_reconciliation`: line-for-line ports over separate types; the fourth claim kind matches no external-target proof.
- `validate_authority`: all 10 boundary fields and all 9 pair fields checked; the design's five named refusals each have their own `bail!`.
- G1's traversal over every `deps` entry, G2's six comparisons, G3's four, G4's four: each design bullet has a line, and G4 accepts `Supported` or `Refused` only.
- `rust.rs` `obligation_owner`: every id in `out` is `rust:<decl>` or `rust:<decl>/…`, every `<decl>` is inserted by `declare`, so the map is total — confirmed against the real candidate's `/field/` and `/variant/` ids.
- Real-data premises: `profiles.json` gives the three authored profiles exactly one entrypoint each, and all seven downstream profiles exist as `model-consumer`.
- Freshness: `check` refuses a non-fresh output directory and `check_at` calls `run` itself, so `cargo-resolve.json` cannot be pre-planted; G1 is not satisfiable by a file on disk.
- The forged-proof control: `VerifiedOutsideBoundary` has private fields, no `Deserialize`, no public and no test-only constructor; I could not reach `qualify_v5` without `verify`.

Guards with a `bail!` and still no case, read as correct and not probed: pair and boundary-row
ordering, a boundary row no pair uses, two rows sharing one profile, `validate_exact`'s
wildcard/`default` rejection, `validate_digest`'s lowercase-only rule,
`workspace.contains(model_package)`, empty `dep_kinds`. Recorded so a later pass knows they are
untested rather than sound.

Not attacked and not claimed: the two real ordinary-gate experiments, whole-matrix qualification,
and the four missing authority inputs. Those are root's and remain open.

## 6. Paths written outside the worktree

Two, both mine:

- `home-path:sha256:42aa014d6f394b19b8db93c435237c2fa65f05b91f213192a73486dcad2dbd2c`
- `home-path:sha256:f110531b33124fb7b0d2d4c52d372732f0bf1513ad09b4774c4949303ec1f76f`

No `/tmp`, no build directory, no scratch fixture copy, no shared target directory. Every other file
in this directory was written by root.

## 7. Findings

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/outside_boundary.rs
  line: 715
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the guard receipt's executed_downstream_cases reports the whole run's executed union rather than the downstream cases the reviewed pairs bind, so a real run will record 2,061+ where the evidence is at most 56.
- file: .ess-evolution/waves/0010-opus-accounting/s3-mechanism/implementation-result.md
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the disclosure places the three retained ignored cases in tests/internal_names.rs, which 09-package.log records as 6 passed and 0 ignored; they are in tests/host_paths_adversary_6.rs.
```
