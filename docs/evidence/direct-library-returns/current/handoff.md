unit: direct-library-return-observations — port reviewed direct returns to current ESS
verdict: green
cases: touched lane executed 2284→2286, red 0 remaining; 3 decisive red regressions retained
origin: n/a
wrote-outside-worktree: see scratch-paths.txt (exact paths)
needs-coordinator: yes — record evidence, commit/publish, release and supply frozen producer identity

## 1. Unit and authority

The active story and assigned brief authorize direct library return declarations and observations on released ESS 0.38, preserving every shipped format. Scope hypotheses were checked against the actual source: source/16 and suites/26–27 already implement the shipped round-three vocabulary; authored scenario/4 was available. Reserved identities are **ess/17**, **ess-scenario/4**, **ess-conformance/28** (ordinary), and **ess-conformance/29** (inventory). Old suite bytes are neither relabeled nor reinterpreted.

`returns: true` declares a successful typed return, without asserting absence of side effects. An authored Act may carry `response: {field: literal}`. The Rust runner checks the exact invocation's returned response against its compiled complete schema and independent literal assertions. It creates no event, stored subject, or manufactured view. Go/TypeScript generation refuses this unsupported observation. Native Binary64 remains refused.

The port preserved the reviewed exact-number, presence, duplicate/order, closed-object and independent 1 MiB/depth128/65,536-member response profile. The new regression found and fixed stale invocation tracking when a newer absent-input command intervened. Independent review then found envelope depth incorrectly subtracting from the direct literal profile; the fix gives only direct expectation field values in suites/28–29 the depth128 local budget and performs typed decoding only after the bounded structural check. Legacy suites and non-response paths retain their original JSON limits.

Authored YAML retains the inherited parser's whole-document recursion limit. The design explicitly separates that transport limit from the exact actual-return and admitted-suite profiles. No new YAML parser, truncated literal, or weaker assertion was introduced. Independent review accepted this distinction and rechecked the original suite-boundary failure green.

## 2. Diff

See `diff-stat.txt` and `status.txt`, captured from Git. The root-owned active story is intentionally excluded from implementor ownership. New code is Rust. The schema was regenerated through xtask; reviewed metadata changes are exactly the three definitions-container hashes affected by the added outcome property. No coverage relationship, consumer profile, or verification guard was weakened. Toolchain source format and public format inventory now include the new reserved identities. Diagnostic census changes reflect the two added validation constructors.

Thirty-four prototype evidence files retain their original Git blob identities under `docs/evidence/direct-library-returns/historical-0.37/`; the containing README marks them as historical prototype evidence. `historical-byte-check.log` is the exact comparison. The earlier tar comparison's differences concerned metadata, not bytes, and are retained separately.

## 3. Decisive red runs

Commands used bounded jobs, debug0 and incremental off, with complete stderr/stdout retained.

- `cargo test -p ess-conformance --test direct_return_port --locked`: `direct-port-red.log`; 0 passed, 1 failed, exit101. The new ess/17 direct return declaration was rejected as unknown `returns` before implementation.
- `cargo test -p ess-conformance --test direct_returns direct_return_admission_tracks_every_command_invocation_form --locked`: `invocation-boundary-red.log`; 0 passed, 1 failed, 16 filtered out, exit101. Admission accepted a direct assertion belonging to an earlier invocation after a different absent-input command.
- `cargo test -p ess-conformance --test direct_returns pure_return_depth_boundary_survives_original_suite_bytes --locked`: `depth-boundary-red.log`; 0 passed, 1 failed, 17 filtered out, exit101. A valid depth128 literal failed original-byte admission with `JSON nesting exceeds 128` at the suite-enveloped value.

The complete original review probe output is retained by the independent reviewer alongside its final report. Its unchanged six contract probes all passed after the correction.

## 4. Green runs and compatibility evidence

Final statuses and runner-derived counts are appended below. Existing successful pre-correction gates remain retained in their original logs rather than overwritten.

- The released ESS0.38 CLI rejects the genuine new source17 and exact suite28/29 bytes, exit1: `old-reader-source17.log`, `old-reader-suite28.log`, `old-reader-suite29.log`.
- Genuine released suite26/27 fixtures were produced using the verified official ESS0.38 binary, each with 3 scenarios and no refusals. The new port test admits those exact bytes and regenerates them identically with unchanged absent-input meaning.
- Genuine new direct suite28/29 fixtures each contain 2 scenarios and zero refusals. Exact high integers remain present. AEP generated additional genuine narrow fixtures using this producer; root supplies frozen commit provenance after publication.
- The baseline conformance test binaries executed 1137 passing cases. The baseline command's later rustdoc phase overlapped implementation edits and failed against its stale library, so that command is not claimed wholly green. Final touched runs include successful doc-tests.
- Initial combined touched tests exposed a diagnostic census drift (45→47), and the actual census was corrected. Initial xtask tests exposed the three definitions-container metadata hashes; those values were recomputed through the existing actual schema extractor. These red logs are retained. No expectations for behavior were weakened.

## 5. Deliberate boundaries

No AEP mutations, commits, pushes, release-version changes, or ER production-library behavior changes were made by this implementor. Root owns publication and frozen producer provenance. Full ESS `task check` belongs to CI under AGENTS.md; locally the touched crates, whole conformance suite, formatter/strict workspace Clippy/rustdoc, public site, schema/projection and xtask lanes were run. Existing ignored tests were not changed or newly introduced.

The interpreted-target run of a released absent-input suite was only a diagnostic transport probe and returned unsupported observations. It is not claimed as positive conformance evidence; actual Rust target tests and byte compatibility assertions are the positive evidence.

## 6. Scratch paths and handoff

See `scratch-paths.txt` for exact retained scratch paths and task-owned build-directory boundaries. No build cache or managed tree was removed. The final CLI and managed tree remain available for the coordinator; only this implementor's lease is released at handoff. Independent review owns its separate review cache and report.

### Final rerun results

- `cargo test -p ess-domain -p ess-compiler -p ess-conformance --locked`: executed **2284 → 2286**, exit **0**; 0 failed, 7 unchanged ignored, 266 runner summary lines. `touched-tests-final.log` contains the complete run, including doc-tests. The increase is the two depth-boundary regression cases.
- `cargo test -p ess-conformance --test direct_returns --test direct_return_port --locked`: direct-return cases **17 → 19** and port cases **2 → 2**, exit **0**; no failed, ignored or filtered cases. `depth-boundary-green.log`. The two added cases are in the direct-return binary and are not filtered out.
- `task test-xtask`: executed **297 → 297**, exit **0**; 0 failed, 3 unchanged ignored, 14 runner summaries. No xtask cases were added by the depth correction. `xtask-tests-final.log`.
- `task ci-lint`: exit **0**, including repository formatter, strict workspace/all-targets Clippy, workspace rustdoc, release verification and action checks. `ci-lint-final.log`.
- `task site-build`: exit **0**. `site-build-final.log`.
- `task projection-check`: exit **0**. `projection-check-final.log`; canonical generation, schema, release-format inventory and documentation projections are current.
- `cargo +1.85.0 check -p ess-domain -p ess-compiler -p ess-conformance --locked`: exit **0**. `msrv-check-final.log`. Two unchanged compiler dead-code warnings remain on MSRV; current-toolchain strict Clippy is clean.
- `cargo build -p ess-cli --bin ess --locked`: exit **0**. `build-cli-depth-final.log`.
- Final CLI toolchain validate, compile and author: all exit **0**; 7 valid model files, 1 authored scenario, 0 refusals. `toolchain-*-final.log`.
- Final genuine suite28/29 generation: each exit **0**, 2 scenarios, 0 refusals. Both exact-byte `cmp` checks against the previously delivered fixtures exit **0**. `produce-suite{28,29}-final.log` and `final-sha256.txt`.
- Independent unchanged six-probe review recheck: **6 passed**, no outstanding findings. Coordinator retained the review report and original red probe separately.

The three decisive red outputs follow verbatim (compilation headers omitted; complete original logs retained).

```text
running 1 test
test direct_returns_use_fresh_source_and_suite_versions ... FAILED

failures:

---- direct_returns_use_fresh_source_and_suite_versions stdout ----

thread 'direct_returns_use_fresh_source_and_suite_versions' (528844) panicked at crates/verify/ess-conformance/tests/direct_return_port.rs:11:6:
the new direct-return declaration parses: Error("unknown field `returns`, expected one of `name`, `when`, `when_subject`, `when_subject_state`, `when_state_changes`, `external`, `wrong_state`, `unknown_instance`, `input_absent`, `existing_instance`, `refuses`, `creates`, `moves`, `updates`, `preserves`, `deletes`, `into`, `accepts`, `replays`, `instance`, `instances`, `affects`, `emits`, `payload`, `sets`, `error`, `summary`, `refs`")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    direct_returns_use_fresh_source_and_suite_versions

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test direct_return_port`
```

```text
running 1 test
test direct_return_admission_tracks_every_command_invocation_form ... FAILED

failures:

---- direct_return_admission_tracks_every_command_invocation_form stdout ----

thread 'direct_return_admission_tracks_every_command_invocation_form' (873539) panicked at crates/verify/ess-conformance/tests/direct_returns.rs:329:14:
a return assertion cannot name an earlier invocation: AdmittedSuite { original: "{\n  \"provenance\": {\n    \"suite_version\": \"ess-conformance/28\",\n    \"system\": \"library\",\n    \"specification_version\": \"v1\",\n    \"spec_digest\": \"9d019b9a7e2bee688694147d28de70214b091071aa2d31296444b61c9efddd89\",\n    \"contract_digest\": \"67b69043e64eaa413446f4643d162bb4306a0ce32385767c94e1065a952c82ce\"\n  },\n  \"scenarios\": {\n    \"library.api/authored/pure-return\": {\n      \"purpose\": \"The actual return contains the declared literal and ordered values.\",\n      \"steps\": [\n        {\n          \"step\": \"execute_command\",\n          \"command\": \"library.api.Read\"\n        },\n        {\n          \"step\": \"expect_outcome\",\n          \"outcome\": {\n            \"command\": \"library.api.Read\",\n            \"outcome\": \"returned\"\n          }\n        },\n        {\n          \"step\": \"execute_command_without_input\",\n          \"command\": \"library.api.Other\"\n        },\n        {\n          \"step\": \"expect_direct_response\",\n          \"response\": {\n            \"command\": \"library.api.Read\",\n            \"outcome\": {\n              \"command\": \"library.api.Read\",\n              \"outcome\": \"returned\"\n            },\n            \"fields\": [\n              {\n                \"name\": \"value\",\n                \"type\": \"String\"\n              },\n              {\n                \"name\": \"sequence\",\n                \"type\": \"List<Integer>\"\n              },\n              {\n                \"name\": \"item\",\n                \"type\": \"library.api.Item\"\n              }\n            ],\n            \"declarations\": {\n              \"library.api.Item\": {\n                \"kind\": \"struct\",\n                \"fields\": [\n                  {\n                    \"name\": \"label\",\n                    \"type\": \"String\"\n                  },\n                  {\n                    \"name\": \"ordinal\",\n                    \"type\": \"Integer\"\n                  }\n                ]\n              }\n            },\n            \"expected\": {\n              \"item\": {\n                \"label\": \"nested\",\n                \"ordinal\": 9007199254740993\n              },\n              \"sequence\": [\n                1.0,\n                2.0,\n                2.0\n              ],\n              \"value\": \"actual\"\n            }\n          }\n        }\n      ],\n      \"source\": [\n        {\n          \"kind\": \"domain\",\n          \"name\": \"library.api\"\n        },\n        {\n          \"kind\": \"type\",\n          \"name\": \"library.api.Item\"\n        },\n        {\n          \"kind\": \"command\",\n          \"name\": \"library.api.Read\"\n        },\n        {\n          \"kind\": \"outcome\",\n          \"name\": {\n            \"command\": \"library.api.Read\",\n            \"outcome\": \"returned\"\n          }\n        }\n      ]\n    }\n  }\n}\n", suite: ConformanceSuite { provenance: SuiteProvenance { suite_version: SuiteFormat(Version(28)), system: "library", specification_version: "v1", spec_digest: SpecDigest("9d019b9a7e2bee688694147d28de70214b091071aa2d31296444b61c9efddd89"), contract_digest: SpecDigest("67b69043e64eaa413446f4643d162bb4306a0ce32385767c94e1065a952c82ce"), component: None }, scenarios: {Authored { domain: DomainRef(QualifiedName(library.api)), name: AuthoredName("pure-return") }: ConformanceScenario { purpose: ScenarioPurpose("The actual return contains the declared literal and ordered values."), steps: [ExecuteCommand { command: CommandRef(QualifiedName(library.api.Read)), actor: None, caller: {}, input: {} }, ExpectOutcome { outcome: OutcomeRef { command: CommandRef(QualifiedName(library.api.Read)), outcome: OutcomeName(returned) } }, ExecuteCommandWithoutInput { command: CommandRef(QualifiedName(library.api.Other)), actor: None, caller: {} }, ExpectDirectResponse { response: Observation { command: CommandRef(QualifiedName(library.api.Read)), outcome: Some(OutcomeRef { command: CommandRef(QualifiedName(library.api.Read)), outcome: OutcomeName(returned) }), fields: [Field { name: "value", type_ref: Primitive(String), naming: Naming { wire: None, display: None, summary: None, code: None, presence: None } }, Field { name: "sequence", type_ref: List(Primitive(Integer)), naming: Naming { wire: None, display: None, summary: None, code: None, presence: None } }, Field { name: "item", type_ref: Named(QualifiedName(library.api.Item)), naming: Naming { wire: None, display: None, summary: None, code: None, presence: None } }], declarations: {QualifiedName(library.api.Item): Struct { fields: [Field { name: "label", type_ref: Primitive(String), naming: Naming { wire: None, display: None, summary: None, code: None, presence: None } }, Field { name: "ordinal", type_ref: Primitive(Integer), naming: Naming { wire: None, display: None, summary: None, code: None, presence: None } }] }}, expected: {"item": Map({"label": Text("nested"), "ordinal": Number(Number(Exact { units: 9007199254740993, scale: 0, binary: 9007199254740992.0 }))}), "sequence": Seq([Number(Number(Exact { units: 1, scale: 0, binary: 1.0 })), Number(Number(Exact { units: 2, scale: 0, binary: 2.0 })), Number(Number(Exact { units: 2, scale: 0, binary: 2.0 }))]), "value": Text("actual")} } }], source: {Domain { name: DomainRef(QualifiedName(library.api)) }, Type { name: DeclaredTypeRef(QualifiedName(library.api.Item)) }, Command { name: CommandRef(QualifiedName(library.api.Read)) }, Outcome { name: OutcomeRef { command: CommandRef(QualifiedName(library.api.Read)), outcome: OutcomeName(returned) } }} }} }, digest: "sha256:9c44947b94b2ce9467d44667f5625e6ec95b97032569b9d11bb2de7e7ce386af", coverage: None }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    direct_return_admission_tracks_every_command_invocation_form

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-conformance --test direct_returns`
```

```text
running 1 test
test pure_return_depth_boundary_survives_original_suite_bytes ... FAILED

failures:

---- pure_return_depth_boundary_survives_original_suite_bytes stdout ----

thread 'pure_return_depth_boundary_survives_original_suite_bytes' (1161210) panicked at crates/verify/ess-conformance/tests/direct_returns.rs:601:37:
depth-128 literal survives its suite envelope: AdmissionError { issues: [AdmissionIssue { reason: "InvalidDocument", path: "$suite.scenarios.library.api.Read/outcome/returned.steps[2].response.expected.value[0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0]", detail: "JSON nesting exceeds 128" }] }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    pure_return_depth_boundary_survives_original_suite_bytes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-conformance --test direct_returns`
```
