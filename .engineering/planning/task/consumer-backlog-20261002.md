---
format: aep.planning-md/3
id: task:consumer-backlog-20261002
kind: task
status: active
title: Process the full consumer-defect backlog in grouped deliveries
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 14
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

PR386 merged as fa08de5bd816b3cc46694ec4d19d20d13e128764 and consolidated UI PR381 merged as b4da64e38b770fe74103409fe1fef7ae6ca214f4 after required green gates and exact tree checks. The approved policy secret refresh cleared the scan-limit blocker. The one-time personal gh secret exception is consumed; all subsequent GitHub writes use the bot App.

All nine original PRs are resolved: the two carriers merged and seven absorbed PRs closed with preservation proof. No source branches/worktrees were deleted. The open GitHub issue inventory is58, down from82; a fresh refresh found no new or changed open issue bodies. This count does not imply completion of all related AEP acceptance.

Server follow-up PR387 publishes exact4826099161bec53d2da65996958b97cb0c96165a, grouping316/379/385 and fixture/347 documentation. Site, security, planning and preliminary lanes passed; full Gate is running. The npm11 direct-root Git dependency admission problem is resolved using official npm12.2 with allow-git=root unchanged; no exception was taken.

Conformance batch locally commits309,342,317,288,308,298. Headf863ee87b5fd6295e086c6134d104025983441d9 includes current main and the canonical planning reconciliation through56b48d26c. Full affected packages domain/conformance/synth are running on those bytes; no remote PR yet. Boolean final paired tests execute138 on both sides, with11 baseline failures and zero treatment failures.

UI365 has passed six affected package checks, lint, browser lab and site validation on its prior frozen snapshot. Review found additional live-event/filter, numeric-parity and source-switch/live composition defects, now under measured correction before publication. Full store/entry318 remains active implementation and is not represented by the earlier behavior-only fixes.

Release0.51.0 remains the latest verified public release with four archives and SHA256SUMS. No0.52release is claimed. Historical execution sections below retain earlier observations; this section is the current checkpoint.

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
