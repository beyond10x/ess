unit: story:direct-library-return-observations — independent-review corrections
verdict: green
cases: direct_returns14→16; conformance before correction801→803 passed; independent adversary7→7 passed
origin: n/a
wrote-outside-worktree: EVIDENCE_CACHE/extension evidence/build paths below
needs-coordinator: no patch; record reviewer findings/corrections and publish through bot path

Two independently reproduced false passes are corrected in the direct-return feature.

1. Named struct child presence was dropped while building authority and ignored during recursive value validation. `typed_fields::direct_response_declarations` now preserves it; the original declaration producer retains its previous serialized bytes. The DirectResponse validation profile enforces child presence recursively using the same helper as top-level fields. NullWhenAbsent and OmittedWhenAbsent have both admitted positive values and rejected negative values, in literal authority and actual runner returns.
2. Opaque Json values bypassed the direct resource traversal. They now receive recursive depth128, collection65,536 and1MiB aggregate checks for both literal authority and actual observations. Every Node variant is covered in the closed traversal. Existing selection and mapped-response profiles are unchanged.

Changed during this correction: src/typed_fields.rs, src/direct_response.rs, src/selection.rs and tests/direct_returns.rs in ess-conformance. No dependencies, public API versions, schema declarations or production ER behavior changed.

Red tests were written/run before implementation and failed on the claimed comparisons:

- direct-return-presence-red.log: `pure_return_nested_presence_is_preserved_and_checked`,1 failed, expected false but true for missing null_when_absent child; exit101.
- direct-return-json-red.log: `pure_return_json_obeys_the_same_recursive_resource_bounds`,1 failed, expected false but true for65,537-element Json array; exit101.

Green checks, each exit0:

- `cargo test -p ess-conformance --test direct_returns --locked -j2`:16 passed,0 failed/ignored. direct-return-json-green.log. Includes14 original cases, the presence regression with real Runner controls, and Json resource cases at/over the bounds.
- `cargo test -p ess-conformance --locked -j2`:803 passed,0 failed,2 pre-existing ignored. json-conformance-tests.log. Prior intermediate presence-only regression802 passed is retained separately.
- `cargo clippy -p ess-domain -p ess-compiler -p ess-conformance --all-targets --locked -j2 -- -D warnings`: json-clippy.log.
- `cargo +1.85.0 check -p ess-conformance --locked --offline -j2`: corrections-msrv-check.log; only existing compiler dead-code warnings.
- `cargo build -p ess-cli --locked -j2`: json-cli-build.log. Current binary `EVIDENCE_CACHE/extension/build/debug/ess` includes both corrections.
- `cargo fmt -p ess-conformance -- --check`, `git diff --check`: exit0.
- Store/executor reviewer reran its seven independent probes, all passed: its evidence/ess-adversary-seven-recheck.log. This result is reported by that reviewer; its separate report owns the execution evidence.

All commands used the existing explicitly assigned cache build root, incremental off, debug0 and2 jobs; MSRV uses the existing msrv-build sibling. Previous required task ci-lint passed before these internal corrections and remains in ci-lint.log; final touched Clippy, tests and formatting were repeated after them.

External evidence added:

- EVIDENCE_CACHE/extension/corrections-handoff.md
- EVIDENCE_CACHE/extension/direct-return-presence-red.log
- EVIDENCE_CACHE/extension/direct-return-presence-green.log
- EVIDENCE_CACHE/extension/presence-clippy.log
- EVIDENCE_CACHE/extension/presence-conformance-tests.log
- EVIDENCE_CACHE/extension/presence-cli-build.log
- EVIDENCE_CACHE/extension/direct-return-json-red.log
- EVIDENCE_CACHE/extension/direct-return-json-green.log
- EVIDENCE_CACHE/extension/json-clippy.log
- EVIDENCE_CACHE/extension/json-conformance-tests.log
- EVIDENCE_CACHE/extension/json-cli-build.log
- EVIDENCE_CACHE/extension/corrections-msrv-check.log

The separate bounded ER core/shared-checker adversary pass and all its test-only probe/fixture paths are documented at `EVIDENCE_CACHE/extension/er-review/report.md`.
