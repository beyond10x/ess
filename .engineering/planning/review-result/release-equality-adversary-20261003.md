---
format: aep.planning-md/3
id: review-result:release-equality-adversary-20261003
kind: review-result
status: active
title: Bounded release equality correction adversary
relations:
- reviews: task:release-0-52-0-20261003
revision: 1
---
unit: release equality correction b4b74139b1f71212677a280dddfa525ad32c6eb6 plus tests-only working tree
verdict: nothing found
cases: executed 10→12, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 14 files in assigned scratch
needs-coordinator: preserve tests and require final integration/release gates

1. git --no-pager diff --stat

```
 crates/generate/ess-gen/tests/integer_bounds.rs | 91 +++++++++++++++++++++++++
 1 file changed, 91 insertions(+)
```

2. Added cases, written before any test execution

Both cases are in crates/generate/ess-gen/tests/integer_bounds.rs. Existing cases and implementation are unchanged. The 10-case before count is from the implementor's reported green target; this pass did not run a baseline suite before writing its cases.

adversary_three_equalities_match_conjunction_at_integer_boundaries exercises the real compiler, JSON Schema projection and jsonschema validator. All six declaration orders of three equalities are checked through both required struct fields and integer newtypes. Contradictory signed-endpoint and negative/zero/positive sets must reject; repeated equalities at i64::MIN, i64::MAX and 2 must accept exactly their value. There are 60 projected schemas and 360 validator assertions. Green on first execution.

Command: cargo test -p ess-gen --locked --offline --test integer_bounds adversary_three_equalities_match_conjunction_at_integer_boundaries -- --exact

```
   Compiling ess-gen v0.51.0 ($WORKTREE/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 0.94s
     Running tests/integer_bounds.rs (target/debug/deps/integer_bounds-1c9c87243d146bce)

running 1 test
test adversary_three_equalities_match_conjunction_at_integer_boundaries ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.24s

```
Exit status: 0.

adversary_optional_three_equalities_preserve_presence_in_every_order checks all six orders of i64::MIN, 0 and i64::MAX equalities. Twelve projected schemas cover both omission-based and null_when_absent Optional fields. Absence/null remains accepted and each constrained integer is rejected (48 validator assertions). Green on first execution.

Command: cargo test -p ess-gen --locked --offline --test integer_bounds adversary_optional_three_equalities_preserve_presence_in_every_order -- --exact

```
    Finished `test` profile [unoptimized] target(s) in 0.11s
     Running tests/integer_bounds.rs (target/debug/deps/integer_bounds-1c9c87243d146bce)

running 1 test
test adversary_optional_three_equalities_preserve_presence_in_every_order ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.04s

```
Exit status: 0. There is no red test output.

3. Target suite, run after both individual cases

Command: cargo test -p ess-gen --locked --offline --test integer_bounds

```
    Finished `test` profile [unoptimized] target(s) in 0.10s
     Running tests/integer_bounds.rs (target/debug/deps/integer_bounds-1c9c87243d146bce)

running 12 tests
test bounds_and_constants_become_keywords ... ok
test a_strict_comparison_and_the_tighter_bound_win ... ok
test equality_on_a_field_that_may_be_null_does_not_become_const ... ok
test invariants_no_keyword_says_exactly_stay_annotations ... ok
test a_newtype_of_integer_carries_its_own_bounds ... ok
test a_validator_refuses_what_the_invariants_refuse ... ok
test contradictory_optional_equalities_keep_null_and_absence ... ok
test equality_intersects_bounds_and_preserves_consistent_values ... ok
test contradictory_equalities_preserve_every_constraint_in_either_order ... ok
test contradictory_newtype_equalities_reject_both_values_in_either_order ... ok
test adversary_optional_three_equalities_preserve_presence_in_every_order ... ok
test adversary_three_equalities_match_conjunction_at_integer_boundaries ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

```
Exit status: 0.

All cargo commands used CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=2, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, RUSTC_WRAPPER= and TMPDIR=$SCRATCH. No CARGO_TARGET_DIR was set; the assigned checkout's target was used. Observed free space stayed above the 10 GiB floor.

Additional checks: strict Clippy for this test target passed. The first task fmt-check identified only formatting of the two new permutation arrays; rustfmt was applied to the test file, leaving exactly 91 additions and no deletions. The final task fmt-check passed. No full-package or workspace test run was claimed by this adversary.

Clippy command: cargo clippy -p ess-gen --locked --offline --test integer_bounds -- -D warnings

```
    Checking ess-gen v0.51.0 ($WORKTREE/crates/generate/ess-gen)
    Finished `dev` profile [unoptimized] target(s) in 0.29s
```
Exit status: 0.

Final formatting command: task fmt-check

```
task: [fmt-check] cargo fmt --package billing-realization --package ess-cli --package ess-cli-contract --package ess-cli-project --package ess-compiler --package ess-composition --package ess-conformance --package ess-diff --package ess-deployment --package ess-domain --package ess-entity-runtime --package ess-gen --package ess-kubernetes --package ess-openapi --package ess-primitives --package ess-realization --package ess-service-contract --package ess-synth --package ess-ui --package ess-ui-check --package ess-ui-docs --package ess-ui-react --package ess-ui-test --package ess-ui-tui --package ess-xtask --package gatepass-realization --package infra-analyze --package infra-compiler --package infra-domain --package infra-project --package infra-spec --package schema-contract -- --check
```
Exit status: 0.

4. Findings

Nothing found in this bounded pass. No judgement findings, approval or completion claim. No base execution or source mutation was needed because no failure was found.

5. Attack scope

The complete base-to-candidate diff and canonical release correctness blocker were read before cases were written. Both integer_bounds and newtype_integer_bounds reach apply_bound from named type projection. The probes compile valid ess/20 authored models and consume the projected schema in a real validator.

Could not break conjunction under six permutations, repeated equality, a third distinct equality or signed integer endpoints.
Could not break required/newtype satisfiable positive controls or Optional presence preservation.
The existing 10 target cases, including equality/range controls, remained green. Unrelated transport behavior, other projection targets and complete release readiness were not assessed.

6. Every file written outside the worktree

Assigned scratch directory: $SCRATCH

- $SCRATCH/equality.log
- $SCRATCH/equality.exit
- $SCRATCH/optional.log
- $SCRATCH/optional.exit
- $SCRATCH/suite.log
- $SCRATCH/suite.exit
- $SCRATCH/fmt.log
- $SCRATCH/fmt.exit
- $SCRATCH/clippy.log
- $SCRATCH/clippy.exit
- $SCRATCH/fmt-final.log
- $SCRATCH/fmt-final.exit
- $SCRATCH/report.raw.md
- $SCRATCH/report.public.md

The publication-safe copy substitutes $SCRATCH for the assigned scratch prefix and $WORKTREE for the assigned managed tree prefix; those are the only log/path normalizations. The raw report preserves recorded command output. Worktree lease maintenance used session equality-adversary-20261003; only that lease is released at handoff. No AEP mutation, commit, push or cleanup was performed.

```findings
[]
```
