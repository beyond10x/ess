---
format: aep.planning-md/3
id: review-result:bot-release-publication-adversary-2
kind: review-result
status: active
title: Release preparation corrected tag-history proof
relations:
- reviews: story:bot-release-publication
revision: 1
---
unit: ESS bot-release publisher preparation correction at d09cd00ba0ece15bf043798ac622d32c9f94eb3b with author correction hashes f894ea6b and b6221897
verdict: nothing found
cases: executed 1→1, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 30 paths
needs-coordinator: none

```text
 .../tests/adversary_release_preparation.rs         | 174 +++++++++++++++++++++
 1 file changed, 174 insertions(+)
```

This remains the complete adversary-owned delta. The author correction changed only `.github/workflows/release-record.yml` and the existing test module in `crates/edge/ess-xtask/src/main.rs` relative to pass 1; those bytes were copied from the author tree under coordinator authorization and were not authored in this pass.

The standalone tests-only patch SHA-256 is `e7bcef7d35d81be8703ed544c1c4e4189516944dcce21c7fe837ef483a8523ab` (`<assigned-scratch>/publisher-review/adversary-test.patch`).

The final author diff is:

```text
 .github/workflows/release-record.yml    | 17 +++++--
 .github/workflows/release.yml           | 53 ++++++++++-----------
 AGENTS.md                               | 34 +++++++++----
 crates/edge/ess-xtask/src/main.rs       | 55 +++++++++++++++++----
 crates/edge/ess-xtask/tests/ci_lanes.rs | 84 +++++++++++++++++++++++++++++++++
 5 files changed, 190 insertions(+), 53 deletions(-)
```

Final source SHA-256 values:

```text
f894ea6bd3f76c95c7db7cc3cc877f4bd81d8c3c597f3e75f7649ec5ccefa2e5  .github/workflows/release-record.yml
13be9b09d600c6c627937834000ca3981249917c40945e775bc085cce899038b  .github/workflows/release.yml
15be8b6e0e0b2dce1fdb6da4bee344da6decc736c66a7653ce17825214ee0b40  AGENTS.md
b62218975f26f4930abaa9add716a8767e1ef385cd2a23097f3322929010973a  crates/edge/ess-xtask/src/main.rs
23748b5a0d4c5fb6457183ec554b068e5c46da7caba4de7e94e2ccf3715af8a0  crates/edge/ess-xtask/tests/ci_lanes.rs
45076b556c696f54a6f6262479224c029a4bc09c83cb12434f7ea8a25b9e78c6  crates/edge/ess-xtask/tests/adversary_release_preparation.rs
```

## 2. Existing adversary case after correction

The author correction adds `fetch-depth: 0` and `fetch-tags: true` to the release-record checkout. The previously red real-Git case is green without changing its assertions:

```console
CARGO_TARGET_DIR=<assigned-build>/target TMPDIR=<assigned-build>/tmp CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-xtask --test adversary_release_preparation --locked -- --exact release_record_fetches_historical_tag_objects_before_checking_off_main --nocapture
```

```text
   Compiling ess-xtask v0.52.0 (<review-tree>/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 6.67s
     Running tests/adversary_release_preparation.rs (<assigned-build>/target/debug/deps/adversary_release_preparation-1fe6a147f3708751)

running 1 test
test release_record_fetches_historical_tag_objects_before_checking_off_main ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

```

Exit status: 0. Exact raw-output SHA-256: `301f33f060ec305bebaa0b57b8393e7acf34ecdb8eab1cb925ae7e2fd4122274`.

## 3. Affected unit and checks

The author-added unit assertion for both checkout settings is green:

```console
CARGO_TARGET_DIR=<assigned-build>/target TMPDIR=<assigned-build>/tmp CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-xtask --bin ess-xtask --locked -- --exact tests::the_release_record_is_checked_after_publication_and_failed_preparation --nocapture
```

```text
   Compiling ess-xtask v0.52.0 (<review-tree>/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 9.23s
     Running unittests src/main.rs (<assigned-build>/target/debug/deps/ess_xtask-d02f7e78a20aceea)

running 1 test
test tests::the_release_record_is_checked_after_publication_and_failed_preparation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 224 filtered out; finished in 0.00s

```

Exit status: 0. Exact raw-output SHA-256: `95e4de6514b9d8f6a2c21b78ee16175da4a4bba0af1dd668cf289617eca51765`.

`rustfmt --edition 2021 --check` passed for `src/main.rs` and the adversary test; its empty raw output has SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Strict scoped clippy passed:

```console
CARGO_TARGET_DIR=<assigned-build>/target TMPDIR=<assigned-build>/tmp CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
```

```text
   Compiling ess-xtask v0.52.0 (<review-tree>/crates/edge/ess-xtask)
    Finished `dev` profile [unoptimized] target(s) in 9.16s
```

Exit status: 0. Exact raw-output SHA-256: `69780dfd80b6017cbcd40643708cb51874e8e5d86c1922d8ac386889eb099929`.

The pass-1 frozen author suite remains the broad local result: 365 passed with 3 documented ignores when the adversary case was deselected. This bounded correction pass did not rerun the full package suite or a hosted workflow.

## 4. Findings

The pass-1 finding at `.github/workflows/release-record.yml:45` no longer reproduces. The unchanged case now proves the record checkout receives enough history and tag objects to peel the remote-advertised off-main tag before `tags_off_main` evaluates reachability.

## 5. Attacked and not broken

- The correction is limited to two checkout inputs and matching assertions in an existing unit test.
- `ref: main` and `persist-credentials: false` remain present.
- The real Git fixture still proves the negative shallow/tagless condition before it checks the corrected workflow configuration.
- No broader new attack was performed, as directed.

Untested boundaries remain unchanged from pass 1: no hosted workflow, live token permission, bot publication, release creation, artifact download, GitHub Release asset inventory, or remote API response was exercised. No credentials were read and no public write occurred.

## 6. Paths written outside the worktree

Publication-safe aliases are used here. The exact absolute inventory is retained in `<assigned-scratch>/publisher-review/external-paths.raw.txt`. It contains the 19 pass-1 paths plus these 10 pass-2 paths:

- `<assigned-scratch>/publisher-review/correction-receipt.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-case-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-case.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-unit-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-unit.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-rustfmt.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-clippy-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-clippy.raw.txt`
- `<assigned-scratch>/publisher-review/pass2-freeze.raw.txt`
- `<assigned-scratch>/publisher-review/report.public.pass2.md`
- `<assigned-scratch>/publisher-review/adversary-test.patch`

The assigned build directories remain `<assigned-build>`, `<assigned-build>/target`, and `<assigned-build>/tmp`; no cleanup was performed.

```findings
[]
```
