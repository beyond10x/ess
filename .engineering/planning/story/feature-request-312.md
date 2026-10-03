---
format: aep.planning-md/3
id: story:feature-request-312
kind: story
status: active
title: Suites state their empty-target assumption and act across callers
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#312
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: docs/design/scenario-initial-state-and-cross-caller-witnesses.md
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T19:07:02Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-10-03T19:07:02Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

Suites either hold on a shared target or say they need an empty one, and some scenario acts on a row as a different caller than arranged it.

## Origin

beyond10x/ess#312, found in the #287 adversary pass; pre-existing on 0.49.0.

## Fit review

The accepted canonical intake record at consumer-backlog commit `acb88e97d3c99587c3b0a501314ce35d73fa4f3b` and its retained continuation classifies this as a defect in suite claims and cross-caller evidence. The suite must declare an empty logical modeled namespace before each scenario and must arrange and act on the same row as different declared callers where the source permits it. No new source keyword or implicit physical database deletion is introduced. Actor/caller authority, source refusal precedence and existing composition restrictions remain intact. The binding implementation contract is `docs/design/scenario-initial-state-and-cross-caller-witnesses.md`.

The coordinator accepted initial-state provenance in the unreleased suite34/35 pair; formats1-33 retain unspecified initial-state semantics and their old bytes. CountReport/2 remains closed. Fresh ordinary/coverage synthesis selects34/35. Historical format-specific tests must explicitly construct legitimate legacy documents without new provenance when testing legacy admission, while current-emission tests assert34/35 and typed Empty. A report/1 writer cannot accept a newly synthesized suite34; modern tests must use exact-suite-bound report/2 without rewriting genuine Failed/Skipped/Unsupported categories.

The current bundle imports the production source, but final combined verification remains incomplete. This continuation reconciles the stale draft mirror with the accepted canonical active story; it does not close312 or turn prior partial evidence into current success.

## Bounded compatibility correction — 2026-10-03

Bounded #312 report compatibility slice, candidate c5682d541bb95bf050b14c748a9acaf56e74ca32, base f5857f8571d4dddbf86210a03af55a3e9b7733db. Root authored three test files after exact pre282 baseline reproduced ten report/1 suite-major precondition failures. No production changes. All 17 original tests retained; two new controls added. Independent review1 assigned to existing Sol reviewer_292; no compiler grant yet, no acceptance or integration claim.

Baseline six binaries: 20 passed/18 failed, exit101. Scoped three binaries: 1pass/4fail,2pass/2fail,4pass/4fail. Corrected:5/5,4/4,10/10, zero ignored, exit0. Actual mixed legacy suite4/report1 and current report2 collector results equal the allcurrent result; deliberate collector stand-ins remain fabricated test input, not target pass claims. Wrong suite digest, count mismatch and contradictory status are refused. Existing manifest1/2/3 and unchanged unsupported-scenario verdict controls preserved. Scoped strict lint and owning fmt/check exit0. Retained exact log and patch hashes in review brief.

Other baseline failures reproduced before282: three absent controls, one aggregate-delta old-format control, and four Go parity freshformat assertions. Go parity did not yet execute Go. Full31target/166failure inventory remains open, none waived. Required final checks still pending.

#282 author correction patch50226754db2dee0d1beec948f24a227b2a0b454dd20c9d3e69771cc9f01de69f delivered source-only before assigned model hit usage limit. Root took verification custody, acquired separate own lease without clearing author's, and began whole interpreted_command_execution tests. Initial attempt correctly refused disk start guard; after own terminal ess-synth cache dry-run/review/clean675files838MiB, actual start free13,608,767,488. No other repository cache touched. Corrected source remains uncommitted/unintegrated until actual results and final review2.

## Remaining mutation report fixture migration

Batch A compatibility continuation, based on fe251b2a4. Report migrationc5682d541 is independently approved and integratedcb6940709; current-suite candidate82803ca32 is author-verified21/21 with independent Go4 still pending resource headroom. Story312 remains open.

Dispatch the existing scope_nested_increment worker for the five test files mutation_external, mutation_external_adversary, mutation_gained_refusals, mutation_skipped_baseline and mutation_unkillable. Exact diagnosisSHAf603e230ee962723a97cc9d8c03d38b7804f3774a952430febb4a3a3f6e0a4e9 establishes40 report/1 precondition failures among50tests before collector assertions. Only test-fixture migration is authorized: actual runs retain their exact report2 categories, coherent explicit stand-ins are re-admitted, intentionally invalid membership reaches its intended refusal, and existing manifest and genuine legacy reader controls remain decisive. No production change, skipped acceptance or issue closure.

Required acceptance is all50 original tests plus the integrated genuine legacy mixed-reader control, scoped strict lint, owning formatting, and independent whole-unit review on the frozen candidate before serial integration. Root owns planning and integration; source-only preparation begins while the sole ESS compiler lane is reserved for the preceding Go review and fresh12GiB start guard. The bounded brief is retained outside the planning store; this record selects the already accepted story and its existing cited test-directory scope.

## Current-suite compatibility integration

Candidate82803ca32c941eae3eceae6a262cd449b6ccca88 is integrated c166ffea49aff0d0a2bb2b7f79e4f160c487b4f0 after independent review-result:current-suite-compatibility-312-20261003-r1 approved with no findings. Actual author21/21 includes four generated-Go tests (12.66s), zero ignored/filtered, strict scoped Clippy and owning format0. Reviewer independently ran exact supplied native17/17, inspected and hash-verified author Go4 evidence, and did not recompile or rerun Go. That redundant rerun was withdrawn after repeated disk-floor refusals; no refusal is counted as execution. The original author actual Go acceptance remains the evidence, not a skip. Exact author logSHA9a4fb5010fd6e83181eedefb5c9c5279258987f691ab8c3ad48e4d3460a44b41; independent reviewSHA8a50f44a57e566143bc65742231ca6dde3cef1380186eb39f99848c02ab431a2. Public review is immutable; genuine legacy26/27 and current34/35 controls remain distinct.

Five-file remaining mutation migration is source-prepared only, all50 tests preserved. PatchSHA20eed6695ca47f932b85c95541a9d1344f9f8e87f975a483908c677f65e146aa before formatting/execution; no green claim. The current-suite target is handed exclusively to that unit after retaining and verifying all three reviewed binaries. Root retired only terminal282 dev cache after retained546-test log and corrected14-test binary were hash-verified: Cargo dry-run/clean923files635.9MiB. Source/logs remain. Fresh disk subsequently exceeded21GB, permitting the external412 baseline to recheck its conditional start grant. Story312 and final bundle verification remain open.

## Response field parser inventory correction

Read-only diagnosisSHA832bdcc36f225ff517972e211e745f1ced18043103a27875f42c9bd81d1d6673 identifies the exact two new valid published-pattern sites behind underscore_field_names.rs:153-157 observing17 instead of15: src/go/response.go:525 and src/ts/response.ts:1138. They validate nested response roots, declarations and mapping members. Existing stale-regex rejection and exact count equality must remain. Author scope_nested_increment is assigned only that test file's explicit inventory/comment and15-to17 correction, with the independent copied-field307 control as a two-file verification unit. The retained original fullrun is the measured red baseline; actual complete test binaries, strict scoped lint, owning format and independent whole-unit review remain required. No regex/parser implementation change or skipped acceptance is authorized.

## Legacy runner entry-point migration scope

The retained full-run baseline (SHA39076a45b7c1c2ec1883b78230ec1b24818e367352c2e498e189fcd601e73d78) measured execution.rs0passed/12failed and faults.rs7passed/13failed. Every failing entry stops at Runner::try_run's UnsupportedReportFormat before target execution. Root inspected runner.rs:345-373: the legacy run/try_run entry refuses every suite major>=5; current freshly synthesized suites are34. git diff226af8bfe..502bf1a75 is empty for runner.rs and these two test files, so their relevant source is unchanged from that measured baseline.

Next bounded source assignment is only execution.rs and faults.rs: admit the exact fresh or intentionally mutated suite through AdmittedSuite, run it through run_admitted, and inspect the actual report/diagnostics. Preserve custom Runner clock/configuration, all32 tests, deterministic whole-report equality, exact diagnostic and faulty-target assertions. The faults helper should use the admitted path for every current suite rather than only Retry. This changes test callers, never the production legacy refusal; existing genuine suite4/report1 controls remain authority for old readers. Do not relabel a fresh suite as legacy, fabricate results, filter failures or loosen counts. If exact admission or real execution exposes another defect, retain the result and return to root before widening scope.

Required acceptance: both full binaries32/32 with zero ignored, strict scoped lint, owning formatting/diff, and independent whole-unit review before integration. All original source/helper and byte-determinism obligations stay intact. This records a ready bounded correction under the existing accepted312 test scope; root dispatches an available existing worker and serial compiler custody separately. It is not implementation evidence.

## Reviewed remaining migration integration

Five-file report migration9203115b533eb9cb00530d0c436b1320ea4ef8e9 is integrated502bf1a75b471742c2680c00e552f7b84d80be66 after immutable review-result:remaining-report-compatibility-312-20261003-r1 approved. Author50/50 plus genuinelegacy1/1; reviewer independently50/50 plus all10 sibling tests, each terminal0/zero ignored. Exact reviewed patchSHA0e4cbdf33d04259aaa921cca01d4d68d0c8c967e1494ff45169e1a3840fac31e and independent logSHAbedb6ceb929b0da0ea3d87fe3da42c5f88b136bf9ea7e235d91cb3b70c3e0f90 retained.

Two-file candidate37f600cdf579d462c0009c17aa76e7d51ce72355 is integrated4ecd55469 after review-result:copied-field-and-response-inventory-20261003-r1 approved. Both author and reviewer native8/8+3/3 passed, strict scoped lint and the author's exact-file format invocation passed. Independent logSHAfe8b7fe8c0296367a8b904c00f26c841d9c60b1636a2050dfbbd088029bfdc57; reviewed patchSHAfbe9afaa4c2ed770d10ddef34a2f7323cb1ce1e2429ce2194f7fd626aba4801e. The exact3 promoted/rolled-back/finished interpreter witnesses and whole native report pass; all copy/branch mutants remain. Audited parser inventory is13field+4fact=17, stale patterns0. No production code changed in these compatibility units.

Latest bot fetch confirms main e07f55a9b13000a6931d842d5534d906b6e9202d is an ancestor. Integrated repository task fmt-check on4ecd55469 with Rust1.98.1 FAILED: task exit201, cargo-fmt child exit1. Its sole source difference is subject_guard_copied_field.rs import ordering, because the unit's standalone rustfmt invocation did not match the repository package formatting settings. The root prematurely recorded a pass before inspecting the terminal result: evidence20261003T205407Z-000-aa6d70315aec is an inaccurate assertion, retained unchanged and explicitly superseded by the corrective evidence record. It cannot satisfy acceptance. The earlier prose claiming integrated formatter success is replaced by this correction. Repository formatting remains pending, and the original failed log remains unchanged.

The repository formatter intentionally excludes byte-pinned generated projections. This observed import-order failure is separate from the earlier cargo-fmt-all generated Billing/Gatepass differences; neither is rewritten as a pass. All remaining baseline failures and final bundle checks stay open.

## Fresh suite feature provenance migration scope

Next bounded compatibility unit: only crates/verify/ess-conformance/tests/current_time_guard.rs, field_presence.rs, field_presence_synthesized.rs and pre_execution_fixtures.rs. The retained baseline39076a45b7c1c2ec1883b78230ec1b24818e367352c2e498e189fcd601e73d78 reaches six stale assertions: fresh34 versus now-offset26, ineffective26-to24 replacement, two fresh34 versus presence24 expectations, and fixture ordinary34/coverage35 versus18/19. Source remains unchanged since that baseline. None establishes a production defect or licenses relaxing admission.

Preserve all34 existing tests (14 current_time_guard,6 field_presence,1 field_presence_synthesized,13 pre_execution_fixtures), all clocks, boundary targets, exact value/presence policies, fixture isolation, mutated provider refusals, Go/TypeScript runtime checks and parent lineage assertions. Current generated suites must assert their exact current major and explicit Empty initial-state provenance, then actually admit/execute where the original control requires it. Pin genuine historical26/27 now-offset,24/25 presence and18/19 fixture compatibility with valid legacy envelopes; legacy negative cases must remove unsupported new provenance only in an explicitly labelled compatibility fixture, mutate the actual asserted header, and fail for the intended unsupported vocabulary rather than an unrelated envelope precondition. Never relabel a current suite as accepted historical evidence, use numeric version ordering as compatibility, or soften exact faulty-target assertions. Existing early-format constants remain historical vocabulary introductions, not the current suite selector.

Acceptance is the complete four binaries, zero failed/ignored, strict scoped lint, owning package/repository formatting, source diff and independent whole-unit review. Preserve actual emitted Go and TypeScript execution with ESS_TYPES_NODE set to the installed definitions; unsupported/skipped required execution cannot pass. Stop and report any real semantics/admission/target defect or required scope beyond these four files before editing more. This is a ready scope under the existing accepted312 compatibility work, not measured completion. Root assigns compiler custody separately after external412's exact two-case run is terminal and the fresh12GiB floor passes.

## Runner migration and corrected formatting integration

Candidate6d5886b1df775386592f0afc2c02aedc390ab041 is integrated60b6c95c1 after independent review-result:legacy-runner-entry-migration-312-20261003-r1 approved with no findings. Author and independent retained-binary execution both passed execution12/12 and faults20/20, zero ignored. Scoped strict lint and owning package format passed. Exact patchSHA735af560af714e1761cc9549c8cb766f5953870a76eb6622e9168a208809274d; reviewer logsSHA55125b4b6ed7277e77e94aed0be27c9ed897eec761078f73ad10a98b3ea7f1f3 and b8d217b37dd4555dd752e3fe8d5953af78810a63e8eb625485a6656ce7099168. Production runner refusal and all custom clocks/fault controls/determinism assertions stay intact.

Import-only correctionb6069134fe2a63f0d2fe423eb715c35f39ddae7c is integrated0e431c8e8d3281f1b84e8a465a1738f966bfa53f after final review-result:copied-field-and-response-inventory-20261003-r2 approved. Parent37f600c owns the actual11-test and lint evidence; no new native run is claimed for import-only bytes. Reviewer required one report-format correction because findings were an object instead of the AEP-required array; original report preserved, corrected publicationSHA138d1e7ac86c87cd7654a0a36753adf00142f06be38b491132fe42d798eaa37c. Integrated repository task fmt-check was then executed and its terminal0 inspected before evidence recording. This clears the actual import-format defect; it does not retroactively validate the inaccurate aa6d record or erase the prior201 failure. All remaining suite compatibility, actual browser and final bundle gates remain open.

## Emitted reader fixture migration preparation

Prepared next bounded unit after the four-file feature provenance migration: generated_docs.rs (5 tests), leaf_payloads_go.rs (4), runtime_suite_admission.rs (2), total11 existing tests. Original full-run39076a45b7c1c2ec1883b78230ec1b24818e367352c2e498e189fcd601e73d78 already reaches six failures: both generated_docs runtimes reject fabricatedmajor34 documents missing required Empty initial state; two leaf tests expect26/27 instead of fresh34/35; one old-leaf rejection reaches the wrong provenance refusal; runtime_suite_admission relabels fresh Empty provenance into unsupported historical envelopes. These are observed fixture construction/version symptoms, not permission to alter production admission.

Acceptance must retain actual Go and TypeScript readers at every currently registered supported major, future-major rejection, report-format environment gates, README/emitter/runtime agreement, real leaf target and mutant parity, and ordinary/coverage lineage. Construct version-correct explicitly labelled compatibility fixtures and assert initial-state presence exactly where admitted; exercise intentional malformed metadata separately before any target callback. Current synthesis remains current; genuine historical fixtures remain labelled historical. Old leaf-vocabulary faults must reach their intended validation reason. No production file, shared helper, emitted fixture source or runtime behavior edit is preauthorized; return any required scope expansion to root. Source unchanged226af..a87372e36 is to be verified before dispatch. Existing scope is preparation only, no execution/integration completion.

## TypeScript version parity migration preparation

A later bounded test-only unit is typescript_suite_versions.rs (29 tests) and typescript_adversary_rp1.rs (4), total33. The retained full-run baseline records26 stale pinned-version assertions in the first file and1 in the second; these are distinct from TS2688 environment failures in other binaries. Both existing Case harnesses mix newly synthesized ordinary/coverage suites with genuine hand-authored historical documents. Current fresh suites must pin34/35 and typed Empty provenance; do not rewrite the historical authored12/14/26 or other literal fixtures into current suites.

Keep all actual target modes, native-versus-TypeScript exact per-scenario verdict equality, complete nonempty reports, exact healthy/all-fault expectations, clocks, concurrent request/caller state, and full parent lineage. Preserve explicit historical format controls through labelled valid legacy fixtures or unchanged literal documents, with actual reader execution where currently required. Do not blanket-replace every numeric major, delete the Case version assertion, use a greater-than threshold, remove older tests, or treat skipped toolchain execution as success. Toolchain and installed Node type definitions must be present before acceptance. Any genuine target/runtime discrepancy is measured and returned to root as its own scope; production TypeScript templates and shared target resources are not authorized in this unit. Required acceptance is both complete binaries33existing tests, strict scoped lint, owning formatting and independent review. This section is preparation only; dispatch/compiler custody follows active compatibility units.

## Emitted reader fixture integration

Candidate803677f9c9c7532b8b30d3e83a68b9604ad32dbb is integrated0845bbf5b485faddd68540ad6e71b0646cd65015 after independent whole-unit review-result:emitted-reader-fixtures-312-20261003-r1 approved with findings[]. Exact three-file patchSHA648405d4cadc94180c1d22d09ae308b3e270f70394f9146d597a0b16443b0c7c; integration source matches reviewed bytes. Author11/11 actualGo/TypeScript (5+4+2), strict scopedClippy and packageformat passed. Author logSHAadec5b012c6ad063201b3e514fc6458221dd6ba664c13bc1f0697674f148eec3. Reviewer independently audited source and exact raw execution/binary evidence; its own redundant execution was refused before start at11978960896bytes and is not claimed as a run. Public reviewSHA9834b1345c26b1edc5dc29f528c9311b30bfcd53194fd103a183816d75b65b9d.

Both author/reviewer leases ended and all processes were terminal before root transferred the exclusive warm target to the separately prepared TypeScript33 unit. The root reclaimed only its own unused reproducible pinned AsyncAPI dependency directory, preserving package/lock/integrity/install evidence for exact restoration before391 actual validation; no active/foreign cache was changed. Transport412/414 owns the next bounded six-case-plus-allocator execution grant, then TypeScript33 follows. This integration closes this correction only; actual remaining conformance, browser and final bundle gates still remain.
