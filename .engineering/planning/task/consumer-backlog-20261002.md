---
format: aep.planning-md/3
id: task:consumer-backlog-20261002
kind: task
status: active
title: Process the full consumer-defect backlog in grouped deliveries
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T08:40:21Z", actor: "human:timo", revision: 2, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T08:40:21Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Every consumer defect or gap in the ESS GitHub issue backlog and the AEP store has an evidence-backed disposition, and every actionable defect is delivered in a coherent batch under the required gates.

## Acceptance

The reconciliation accounts for every open GitHub issue and every nonterminal consumer-facing story, links duplicates and existing implementations, and separates released fixes, merged fixes, queued candidates, declined requests with a demonstrated idiom, and concrete unresolved blockers. Actionable defects have regression evidence and a grouped pull request; queued or blocked work is never reported as shipped.

## Authority and execution

Operator direction on 2026-10-02: process the full consumer-defect backlog from AEP and GitHub today, ship fixes fast in groups, one managed worktree and one pull request per related batch. Prepare locally before expensive remote gates; keep one new batch in CI while the next is prepared. The request authorizes this continuing work, not fabricated approvals or bypassing required gates.

The initial source set is 82 open GitHub issues and 161 nonterminal stories in a 923-artifact AEP store. Recheck both sources before the completion audit. Counts are inventory, not completed work.

## Current evidence

GitHub bot API inventory at 2026-10-02T17:50Z: 48 open issues, no open PR in that response. This is an inventory, not a count of unfixed defects. No reliable full-backlog release ETA is claimed; the operator requires all actionable backlog work and complete runtime feature support.

PR386, PR381 and PR387 merged under required gates. PR387 merge1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036 preserves its exact tested tree with all15 checks green. All nine inherited PRs are resolved through merged carriers or evidence-backed consolidation; no source branches or managed trees were deleted. The one-time personal-account secret exception is consumed. All later GitHub writes use the bot App. Release0.51.0 remains the latest verified release; no0.52 tag is claimed.

The local carrier batch/consumer-runtime-20261002 now has81d14bf05 plus the frozen12-file native Interpreter checkpoint ac0800023e84f2be71177d1c86a1c57cc8578cd5714047d57ff37c39f727ead4 under combined validation. It includes307/360,318, binding/child-signal corrections and389 source/IR/diff/projection support; native/Go/TypeScript observers share43 runtime controls and actual WASM execution. Producer17/0 and isolated state guard1/0 passed. The source feature is not declared delivered while its combined checks and grouped PR gate remain outstanding.

The first full conformance/synth run completed: conformance2277passed/39failed/11existingignored across313 summaries; synth407passed/0failed/1existingignored across62 summaries. Failures were retained, not hidden: outdated port refusal/inventory expectations, missing standalone Go modules, report/2 unsupported exit/count expectations, obsolete native held-state refusal, Go formatting and a missing local ESS_TYPES_NODE path. Corrections are committed348e444df; actual15-binary rerun and final two-binary correction passed every previously failing area. All28 TypeScript parity tests ran green with the configured Node types. The future-version diagnostic correction retains exact requested/supported versions and provenance path without advising an equally unsupported Rust runner. Independent review consumer-runtime-integration-corrections approved; reviewer executions0.

Explicit example source facts and complete caller migration are integrated81d14bf05 from botcommit af9a9311b51e9b2a808843f502dd5b3af427defc. Oracle stores contact. Billing requires caller-supplied issued_at, preserving every33/34 scenario identity and fault assertion. Initial independent review reproduced an unchanged missing-timestamp caller0/1; correction31conformance+38lowering+7CLI tests passed, strictlint passed, and source review approved. No interpreter-invented stored timestamp was substituted. All115 original/caller paths are committed. The browser lab and public site build passed runtime-site-build.log, exit0, including actual WASM and browser script checks.

Native owner final focused18-target run reports99passed/0failed/0ignored. It includes exact per-scenario agreement for33Billing+34Oracle scenarios and all16 fault variants, typed ordinary responses, stateful guards, views/paging/aggregates, supplied delivery facts and relative elapsed observations. Actual refusals now carry consistency tokens so unchanged-state snapshots can be checked. Missing required aggregate keys refuse instead of becoming invented nulls. Ten-file independent review approved; the facts author's own two files and integration were reviewed separately by the coordinator. Absolute-time and periodic-host observations require explicit facts; no such missing authority is invented. One additional explicit unscripted-obligation acceptance regression is being finished separately.

Carrier combined cargo test for ess-conformance, ess-cli, ess-entity-runtime and billing-realization is running in combined-runtime-examples-packages.log. Full domain/compiler/diff/generator evidence remains1891/0 with4existingignored. After integrated tests: ci-lint, projection/schema checks, remaining acceptance evidence and one grouped PR gate. No intermediate component PR or new remote full gate has been started.

Storage evidence is retained in ignored task-owned scratch. To preserve the8GiB floor, root removed217 completed core test executables earlier, then373 exact completed conformance/synth executable artifacts listed in completed-runtime-test-binaries.txt, reclaiming15GiB. No source, managed tree, active-worker cache, library artifacts or retained log was deleted. The second cleanup occurred after all root test processes had completed and before the current combined run.

Read-only issue audit pins carrier81d14bf05/main1ff305685:314's behavior portion is merged but full issue awaits318;318,307 and360 are locally implemented pending delivery;347's ESS capability/guidance is merged while private consumer integration is unverified;319 still needs the accepted ess/22 dependency/generation bundle;312 remains draft with fit/design and general isolation/cross-caller coverage incomplete. These substantive remaining items are not closed by389. All48 issues and every remaining consumer AEP artifact still require their evidence-backed dispositions and actionable implementation.

## Completion boundary

Implementation proceeds on unblocked items while external publication is blocked. A published branch is queued, not released. Design decisions and external dependencies require explicit linked blockers, with their exact question and next action. No item is deferred merely to make the inventory smaller.

## Synthesis batch execution

Managed tree ess-backlog-synthesis-20261002, branch batch/consumer-synthesis-20261002, base 28aeddddfc86c4a92c48aba98d5d70091ba4219d. Source fix #309 committed as c28e3bdaeb914a29f72273a1b8a8095009bdf174 after exact hashes matched the 300-lane test report; independent source review found no concrete defect and executed no tests. Source package evidence is local-evidence:ess-backlog-synthesis-20261002/target/backlog-input/309-report.md.

Next accepted units #342 and #317 execute sequentially in this same tree, preserving one worktree/PR per related batch as directed. Their cited scopes overlap synthesize.rs, so no concurrent source edits. An implementation worker owns source; coordinator alone owns AEP, commits and publication. Test scratch remains outside Git, build output in task-local target; bounded package verification retains logs and avoids accumulating all test executables. No source change while its verification is live.

Authorized continuing work: unit fixes, local tests, bot commits, grouped publication and integration only after required gates. No remote rerun for an unchanged secret-policy failure. Release remains subject to the repository's exact-commit requirements. Other scoped issues #288 and #308 are not yet implementation assignments.

## PR consolidation policy

Operator steering: reconcile existing PRs and reduce full gate count. The nine-open-PR inventory is being reduced to two existing carriers (#381 UI and #386 generated servers), plus one later synthesis/conformance PR. PRs 383/384 were already consolidated into386; 375/380 now closed after exact-ancestry/complete-patch-equivalence proof of preservation in377. Remaining UI sources344/345/368/369/377 remain open until unique source and regressions are preserved in a published381candidate. See task:consumer-ui-consolidation-20261002 for exactheads and overlap/knownfailures.

Prepare and review each combined batch locally before publication; do not push component branches for separate full gates. Run one full gate on each final combined candidate, changing that head only for demonstrated fixes. Existing green component gates are not evidence of combined behavior. Merge current main before the final gate where needed so release reuse can match tested trees. Do not rewrite or squash history to evade the policy scan; only the secret owner can resolve that baseline blocker. No unchanged retries while it remains.

## Generated-server next local pass

PR386's exact candidate f0b220099 has a green ESS Gate and remains blocked only on remote common security. Continue accepted source corrections316 (creation identity in Rust/Go),379 (Web/Clap existence target seams; Go already implemented),385 (canonical guidance) sequentially in managed ess-backlog-servers-20261002. No component PRs or pushes: retain one server carrier and publish only after the complete next batch is locally verified/reviewed. Original store/entry requirement318 remains in scope of the full backlog and must not disappear when314's behavior portion is reported.

One worker owns source, coordinator owns AEP/commits/publication. Reuse the task's warm target with bounded focused tests; preserve 8GiB free-space floor. No large archive/full-workspace run while UI and synthesis lanes are active. All executable source is Rust; existing generated Go is exercised through Rust harnesses under repository conventions.

## Next remote gate consolidation

To satisfy the operator's explicit gate-cost constraint, combine the independently reviewed six-fix conformance batch (309/342/317/288/308/298) and UI365 after each finishes local checks. They have no conflicting source edits. Preserve their separate managed branches, bot commits, immutable reviews, regression baselines and affected-package logs, then merge into batch/consumer-synthesis-20261002 for one consumer-fix PR. No UI-only PR or component gate is needed. Current serverPR387 remains the only full gate in flight; integrate its tested main merge before publishing the next candidate. Final combined ci-lint/projection/site validation must describe actual combined bytes; unchanged per-package source can retain its measured package evidence with exact hash proof. Any new integration conflict or behavior change requires relevant checks. Larger served-entry318 and browser startup repairs remain distinct later work, not silently added to this frozen candidate.

## Reuse PR387 for the corrected combined candidate

The full gate on387 failed one stale service-contract expected string caused by385's deliberate contract wording change. Correct the consumer expectation locally and retain the measured failure. Since this necessitates a new candidate gate, fold the already reviewed/local-green conformance and UI365 commits into the existing batch/consumer-servers-20261002 PR387 carrier after final combined checks, rather than open another PR. Preserve482609 as an ancestor; no force update or source retirement. Rewrite387's title/body for all final included fixes and actual validation. This supersedes the earlier separate next-PR sequencing and saves one full gate. Browser and318 remain outside this combined candidate.

## Corrected combined PR387 published

PR387 now carries exact head ba5d657b72913b29ecc19e9a6d2c2a111f083793, preserving the previous482609 head and main b4da64e as ancestors. Its rewritten title/body covers ten issue fixes (316,379,385,309,342,317,288,308,298,365). Signed common evidence inspected96 commits without findings; remote common Security and privacy and documentation-source checks succeeded. The full correctness run37006394198 remains in progress; there is no merge or release claim. Browser, served-entry318 and related-field307/360 work remain outside this frozen remote candidate. No unchanged-head rerun or separate synthesis/UI PR was created.

## Remote planning-validator compatibility correction

PR387 run37006394257 failed at exact ba5d657 during planning validation: AEP0.62.0 rejects transition executor fields written by local AEP0.68.0, then reports derivative undeclared references for the refused files. Local validation used0.68.0 and passed, so it did not reproduce CI's parser. Preserve attribution metadata; do not strip records or suppress validation.

Accepted correction: pin .github/workflows/planning.yml to released AEP0.68.0 commit6d7a44d3607d2d9a6ffdf0a165993c546c43d0db, matching the version that writes this store. Remote tag0.68.0 resolves to that commit. Validate the candidate with that release, run workflow checks/ci-lint, and retain the store's existing history and evidence. This changes the repository CI dependency only, not the installed CLI or store format. Let the currently running correctness gate complete before publishing so any further candidate failures can be grouped into the same correction; no unchanged-head rerun can repair the old parser.

## Full browser validation and carrier decision

Frozen source committed2220bce9b9 retains patch SHA2568e46d087f26a7bbbddd72364a01610d869af5191c1036fedb4aae7bd99b67733. Full cargo test -p ess-cli --locked --no-fail-fast completed2026-10-02T12:42:44Z with exit0:109 test binaries plus doc-test result,904 passed/0 failed/0 ignored. Log target/backlog-input/browser-full-cli.log in the managed browser tree. Focused identical-test baseline17pass/6fail to23pass/0fail, counter/lock mutants, strict all-target Clippy, formatting and independent no-findings review remain retained.

Because correcting PR387's obsolete planning parser already requires a new exact head and full gate, include these now-reviewed browser fixes in that correction. This avoids a separate browser PR/full gate while keeping unfinished318 and307/360 source out. The still-running prior-head remote correctness shards will be read before publication so any additional concrete failure can be corrected together. No planning or correctness check is waived.

## Combined carrier published, 2026-10-02

PR387 now publishes exact head ad4506162678a6ea9c299d04e0c12160fd35155a, combining ten GitHub fixes, three reviewed browser-startup fixes and the planning-validator correction. Signed common evidence covers105 commits. The official AEP0.68.0 release archive checksum was verified before validating this store; the workflow pins its exact release commit6d7a44d3607d2d9a6ffdf0a165993c546c43d0db. The former AEP0.62 parser rejected executor/correlation metadata, so the fix preserves that metadata and updates the parser.

Current runs: CI37009794833, planning37009794856, site37009794976, common37009791641. Planning validation, site build and common security have passed on this head; full CI remains pending. Earlier ba5d657b72 full Gate success does not substitute for this head. The carrier stays frozen during checks. Subsequent318,307,360 and binding-unused-event work integrates separately on batch/consumer-runtime-20261002 for one combined package validation and PR.

## Source version allocation, 2026-10-02

The operator explicitly prioritized issue389 for the fast lane. The coordinator allocates the next unshipped source major, ess/21, to its one-time response disclosure contract. The previously planned coordinated downstream syntax bundle moves together to ess/22; its accepted behavior, dependency ordering and requirement to ship as one bundle are unchanged. Prior mentions of ess/21 in this artifact record the earlier allocation, not the current implementation target. No released source/IR/suite meaning is rewritten by this planning change.

This allocation must be reflected in binding designs and compatibility tests before implementation. The389 design independently names its new IR and ordinary/coverage suite versions from actual current source; those numbers are not inferred from the source-format number.

## Mandatory runtime parity, operator correction 2026-10-02

The operator explicitly requires every supported conformance runtime to support every admitted feature. For389, native Rust, Go, TypeScript and any browser/WASM conformance path must execute the complete new contract before release. The earlier proposed Go/TypeScript refusal-only first delivery is rejected and superseded.

Use shared healthy and adversarial fixtures to verify capture, retries, fresh rotation, actor changes, recursive leak checks including keys, authored timelines, admission, bounds, value-free diagnostics and verdict/count equivalence. Inventory all supported execution surfaces and test each; do not assume a shared library proves an unexecuted integration path. Intrinsic finite observation limits, including unavailable restart/concurrency primitives, remain explicit and identical across runtimes. A target's inability to provide an observation is distinct from a runtime omitting the newly admitted vocabulary, and must never become a passing check.

Full runtime parity is a release acceptance gate. Prioritization changes order, not completeness.

## Timestamp explorer integration regression

The explicit Billing Timestamp input migration exposed a regression in the existing implemented story:concurrent-explorer-runner and story:explorers-record-view-reads acceptance. The actual combined CLI run fails go_and_typescript_write_equal_bytes_from_the_same_seed_over_examples_billing because IssueInvoice is absent from the command inventory, and the_explorers_read_every_view_and_write_rows_only_on_an_answered_read because no OutstandingInvoices row is ever read. Native source audit identifies both generated primitive planners omitting Timestamp, before any target callback; intake SHA2562627cc97f00d7b7d97ae9c72b2eec8240cfddf6b96e07d65b4e728992d83c51f, own additional executions0.

Correction within this active consumer-runtime batch: scope_aggregate owns the generated Go and TypeScript explorer templates and focused regression paths in its managed TS tree. Add matching deterministic seeded Timestamp scenario-input generation, including named Timestamp wrappers. Preserve existing command-inventory, actual-row and byte-identical-seed assertions; no target-owned clock, arbitrary stored value, skipped command or changed expected inventory substitutes for the feature. The current native witness generator already supplies valid Timestamp inputs, so this is a generated explorer capability repair, not new source syntax or a source-major allocation. Exact changed-path scope and hashes accompany the frozen handoff. All committed executable code remains Rust apart from established generated-language templates exercised by Rust harnesses.

Root remains the only AEP writer and owns integration. The existing browser one-line compatibility correction stays a separate patch. Compile work pauses near the8GiB disk floor; current root run stays live and source-frozen. Final affected checks and independent review precede publication in the grouped PR. Existing implemented story statuses do not imply this newly exposed regression is already repaired.

## Combined validation correction pass

Combined runtime/examples validation completed with exit101 in ess-backlog-next-20261002/target/backlog-input/combined-runtime-examples-packages.log. It reported21 failed test binaries. Exact failures are retained; no full-pass claim is made. They include derived example pins/corpus, strict report/2 categories/exits, legacy HTML metadata compatibility, missing Timestamp explorer generation, obsolete Interpreter unsupported expectations, a mutation fixture missing explicit issuance input, new Oracle contact-retarget mutation inventory, and tutorial format/file-count output.

Carrierff128cbdb commits the exact12 native Interpreter files from reviewed00ecf91c6b3a98eee27372e914d16f89df2fa83d, verified by Git object hashes. Carrier01ce4b82f integrates native obligation controls20ee6382, Rust historical producer compatibility7d29598a and reviewed21-file migration3014907d18de801849c74ed10274f33348a2eb60. The latter's author reports64 focused tests green, strict Clippy and formatting exit0; independent source review approved with0 reviewer executions and immutable JSON SHA25662294c42ac1506dff244c2435fc5ace73fbbdd74f9c9c9260875197708842d97. The carrier already supplies the required explicit Go profile/2 semantic plan.

Remaining corrections are bounded continuations of the accepted runtime batch: native worker owns CLI capability expectations plus mutation audit assumptions, Go worker owns exact go_conformance count categories, TS worker owns Timestamp planners, coordinator owns legacy browser patch, mutation-survivor input, tutorial output and state/view expectations. No component PR or remote gate is started. Final combined checks and new site validation after changed snippets remain required. Earlier core package counts predate the regenerated example corpus and are not final-byte evidence.

After the combined process handle11050 returned terminal101, coordinator removed only368 completed task-owned test executables (13.71GiB), listed in combined-finished-test-binaries.txt, checking regular executable/non-symlink status and fuser inactivity. No source, logs, libraries or worker cache was deleted; observed free space afterward30GiB.
