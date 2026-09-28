unit: story:direct-library-return-observations — Direct library return observations
verdict: green
cases: direct-return lane executed 1 → 14; initial red 1; complete touched-crate lane after initial feature and after resource correction 1,738 → 1,739 passing
origin: n/a
wrote-outside-worktree: EVIDENCE_CACHE/extension (explicit coordinator build/evidence location)
needs-coordinator: no patch; coordinator owns independent review, AEP evidence and bot publication

## Unit and scope

Implement actual successful library return assertions without fabricated publication, persisted subjects or last-result views. New source ess/16 `returns: true` requires the command's nonempty typed response; authored ess-scenario/4 acts accept `response: {field: literal}`. Empty response maps request a complete shape check. Generated returning outcomes automatically check response shape. Event/state effects remain independent declarations.

The confirmed integration seam is `SemanticCommandResult.response`; the adapter receives only invocation inputs. Source version and strict outcome parsing, authored version admission, suite vocabulary admission, generation emitters and Rust runner were inspected before implementation. No baseline path provided a direct authored response claim. Existing mapping observations depend on event fields and cannot express this feature. The initial test proved the old source parser rejected the new declaration.

## Changed files

Managed tree: `WORKTREE_ROOT/ess/er-library-conformance`, base `472d35fbe3109ad47f89a65df44b60c1f14acd55`, branch `feat/library-return-conformance`.

The tracked diff is 24 files, 320 additions / 53 deletions; new files add 698 lines (direct_response.rs 232, direct_returns.rs 422, design document 44). The coordinator's AEP changes are separate and were not edited by this worker.

- ess-domain: command.rs adds the typed outcome member, conversion and validation; system.rs admits source16; entity.rs and outcome_group.rs initialize the new field in test constructors.
- ess-compiler: ir.rs persists nondefault returning outcomes; resolve.rs carries authority. Legacy false fields are omitted, preserving old bytes.
- ess-conformance: new direct_response.rs admits finite typed schemas and independent literals, closes return shape, and compares actual values. scenario.rs, admission.rs and coverage modules admit ordinary26 / inventory27. authored.rs compiles scenario4 literals and auto shape checks. synthesize.rs generates shape checks. runner.rs observes the exact last command result. selection.rs introduces a separate direct-response resource profile while preserving prior limits. Go/TS emitters explicitly refuse unsupported new observations.
- Test constructors/exhaustive matches and future-version refusals updated. New direct_returns.rs covers correct/wrong/absent/extra/nested/ordered/duplicate/exact-integer values, undeclared coordinates, old versions, legacy bytes, generated shapes, coverage/report2 association, unsupported generators, Binary64 refusal and resource limits.
- schemas/generated/ess.schema.json regenerated; docs/design/direct-library-returns.md added; diagnostic code inventory and CHANGELOG updated.

## Red evidence

Initial command: `cargo test -p ess-conformance --test direct_returns --locked -j2` with the assigned cache target, incremental off and dev debug zero. Full output: `direct-return-red.log`. Exit 101.

```text
running 1 test
test pure_return_correct_literal_compiles_without_events ... FAILED
direct-return source parses: Error("unknown field `returns`, expected one of ...")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Resource regression was first run against the inherited selection limits. Full output: `direct-return-resource-red.log`. Exit 101.

```text
running 1 test
test pure_return_payloads_have_an_independent_lossless_resource_bound ... FAILED
[Refusal { origin: "large.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(library.api)), name: AuthoredName("pure-return") }), cause: Unreadable { detail: "response literal value: resource" } }]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out
```

## Green evidence

All commands used `CARGO_TARGET_DIR=EVIDENCE_CACHE/extension/build CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`; cargo used `-j2`, Task used `CARGO_BUILD_JOBS=2`.

- `cargo test -p ess-conformance --test direct_returns --locked -j2`: initial new lane executed 1 → 14; exit0, `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. `direct-return-resource-green.log` retains full output. This lane has no ignored/skipped cases.
- `cargo test -p ess-domain -p ess-compiler -p ess-conformance --locked -j2`: initial completed implementation 1,738 passed → final 1,739 passed, exit0. Aggregated from 175 actual cargo summary lines in `targeted-tests.log` and `targeted-final-tests.log`; nine pre-existing ignored tests remain and zero failures. A pristine whole-crate baseline count was not measured, so no such count is inferred.
- `task ci-lint`: exit0, `ci-lint.log`. Includes repository fmt-check, strict all-target workspace Clippy, rustdoc, release consistency and action checks. Earlier standalone Clippy failure in `targeted-clippy.log` was corrected; final strict workspace Clippy is green.
- `cargo +1.85.0 check -p ess-domain -p ess-compiler -p ess-conformance --locked --offline -j2`: exit0 with separate `msrv-build` cache; two existing resolve.rs const-function dead-code warnings; `msrv-check.log`. Cargo manifests and lockfile unchanged.
- `cargo xtask schema`: exit0; `schema.log`, regenerated schema 111,514 bytes.
- `cargo build -p ess-cli --locked -j2`: exit0; `cli-final-build.log`. CLI `EVIDENCE_CACHE/extension/build/debug/ess` includes final resource fix.
- `git diff --check` and unchanged Cargo manifest/lock checks: exit0.
- CLI generated exact suite26 twice: `cmp` exit0. SHA256 `e5521c88da53e788d31f174de24467fe7147f399746d95dfd31612eff5222bf2`.
- Exact baseline `PINNED_ESS_0_37 --version` reports0.37.0; its `verify conform run --suite direct-return-suite26.json --target interpreted` exits1: `UnsupportedSuiteVersion: execution readers admit suite majors 1–25`. Its `specify validate --path old-reader-source.yaml` exits1 with unknown `returns`. Logs named old-reader-0.37-suite/source.log. Both refusals happen before target execution. Historical0.36 reader was additionally checked in old-reader-suite/source.log.

## Deliberate boundaries

No production ER changes, dependency changes, MSRV change, invented events/subjects/views, Binary64 relaxation, Go/TS executable additions, AEP edits, commit or publication. CI owns the whole ESS gate; targeted packages and required ci-lint were executed locally per AGENTS. Source16 / scenario4 / suites26–27 are intentional format additions; old source/IR/suite bytes retain their established omission rules.

The resource class was bounded separately: direct typed responses allow at most 1 MiB serialized return bytes, depth128 and65,536 members per collection, checked fully; authored authority also at most1 MiB. The regression covers both long exact string assertions and larger ordered sequences. Existing selection/mapping observations retain4 KiB/64-item limits. No payload truncation.

## All external paths

- EVIDENCE_CACHE/extension/build/
- EVIDENCE_CACHE/extension/msrv-build/
- EVIDENCE_CACHE/extension/handoff.md
- EVIDENCE_CACHE/extension/ci-lint.log
- EVIDENCE_CACHE/extension/cli-build.log
- EVIDENCE_CACHE/extension/cli-final-build.log
- EVIDENCE_CACHE/extension/cli-regeneration.log
- EVIDENCE_CACHE/extension/cli-synthesis.log
- EVIDENCE_CACHE/extension/direct-return-build.log
- EVIDENCE_CACHE/extension/direct-return-cases.log
- EVIDENCE_CACHE/extension/direct-return-red.log
- EVIDENCE_CACHE/extension/direct-return-resource-green.log
- EVIDENCE_CACHE/extension/direct-return-resource-red.log
- EVIDENCE_CACHE/extension/fmt-check.log
- EVIDENCE_CACHE/extension/msrv-check.log
- EVIDENCE_CACHE/extension/old-reader-0.37-source.log
- EVIDENCE_CACHE/extension/old-reader-0.37-suite.log
- EVIDENCE_CACHE/extension/old-reader-source.log
- EVIDENCE_CACHE/extension/old-reader-suite.log
- EVIDENCE_CACHE/extension/schema.log
- EVIDENCE_CACHE/extension/targeted-clippy.log
- EVIDENCE_CACHE/extension/targeted-final-tests.log
- EVIDENCE_CACHE/extension/targeted-tests.log
- EVIDENCE_CACHE/extension/old-reader-source.yaml
- EVIDENCE_CACHE/extension/direct-return-suite26-regenerated.json
- EVIDENCE_CACHE/extension/direct-return-suite26.json
