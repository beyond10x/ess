# ESS PR 182 targeted CI correction

Base: `9d488e266b7ffd910f46bdce730957a8b555a0c3`, managed worktree
`WORKTREE_ROOT/ess/er-library-conformance`.
No commit, push, external write or AEP mutation was performed by this worker.
The worker lease `codex-er-ess-ci-fix-20260929` is released.

## Changes

- `crates/edge/ess-xtask/src/docs.rs`: record ess/16, ess-scenario/4 and
  ess-conformance/26–27 as supported but unreleased (`None`). No release is invented.
- `website/docs/reference/formats.md` and `spec-versions.md`: document the four new
  formats, direct-return semantics, observations and compatibility/refusal boundaries.
- `crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json`: update
  only the three definitions-container shape hashes. The six exact relationships,
  source guard, profiles, classifications and all ordinary descendant obligations remain intact.
- `models/toolchain/domains/specify.yaml`: align SpecificationFormat with the actual
  `SUPPORTED_FORMATS` list by adding ess/16. The existing closed-set equality test is unchanged.

The schema change being reviewed is exactly RawOutcome's boolean `returns` property,
already present in the feature commit. A temporary, removed Rust diagnostic test called
the existing metadata fixture and wire extractor to observe the new definitions hash:
`c3f7d43d185da8f440147d53951f81b00a1984dea9fd11d089989d242695c6f9`.
The final unmodified metadata tests independently validate it against the compiled provider.
Production Rust behavior, generated schema, suite bytes and the ER specification are unchanged.

## Red evidence

All paths below are relative to `EVIDENCE_CACHE/er-ess-extension/`.

- `ci-job-108770974510.log`: read-only GitHub job log for run 36372297273, job 108770974510.
- `projection-red.log`: exit201 (`task` wrapping exit1), missing FORMAT_RELEASES entries.
- `xtask-red.log`: exit201 (`task` wrapping exit101), 193 passed / 11 failed: missing
  format metadata and stale definitions-container hashes.
- `ci-current-wire-digest.log`: existing extractor's actual current digest.
- `xtask-model-enum-red.log`: after fixing the early unit failures, 204 unit tests pass;
  the later integration test finds the missing ess/16 model variant.
- `xtask-aep-concurrency-red.log` and `projection-aep-concurrency-red.log`: a later run
  correctly refused source authority changing during the coordinator's AEP mutation.
  The coordinator then completed that mutation and kept the tree stable for final checks.

## Final checks

All final checks below exited 0. Native Cargo checks used
`CARGO_TARGET_DIR=EVIDENCE_CACHE/er-ess-extension/ci-build`, jobs 2, incremental off,
and debug 0. The xtask task supplies its required Rust 1.98.1/lld/native profile.

| Check | Evidence |
| --- | --- |
| `task test-xtask` — 297 passed, 0 failed, 3 pre-existing ignored tests across 14 suites | `xtask-green.log` |
| `task projection-check` — projections/schema/changelog current; all 51 supported formats accounted for | `projection-green.log` |
| `cargo clippy -p ess-xtask --all-targets --locked -- -D warnings` | `ci-clippy-green.log` |
| `task fmt-check` | `ci-fmt-green.log` |
| `task site-build` — browser lab and Docusaurus | `site-build-green.log` |
| retained CLI `specify validate --path models/toolchain` — valid 7 files | `ci-toolchain-validate.log` |
| retained CLI `specify compile --path models/toolchain --out <cache>/ci-toolchain-ir.json` — 98 declarations | `ci-toolchain-compile.log` |
| retained CLI `verify conform author --path models/toolchain --scenarios models/toolchain-scenarios` — 1 scenario, 0 refusals | `ci-toolchain-author.log` |
| `git diff --check` | clean |

The retained CLI is `EVIDENCE_CACHE/er-ess-tools/ess-9d488e266`.
For `projection-check` and Clippy the profile explicitly sets Rust 1.98.1,
`RUSTFLAGS=-C link-arg=-fuse-ld=lld` and empty Cargo/Rustc wrappers, matching `test-xtask`.
`site-build` uses its prescribed example-local target path with jobs 2; it copies the
Wasm artifact from that path. No complete ESS gate was run; CI owns it.

Generic `cargo fmt --all --check` reports pre-existing formatting differences in byte-pinned
generated dependencies. `Taskfile.yml` explicitly excludes those artifacts in `fmt-check`,
which passed. No generated source was formatted or changed.

## Exact patch

`ci-fix.patch` contains only the five correction files (37 added / 4 removed lines).
SHA256: `d301df8999221e774780c9284f50bae483cf59fe5c71b51235d93a030027215b`.
The coordinator's AEP updates are outside this worker's patch.
