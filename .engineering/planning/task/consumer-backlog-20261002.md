---
format: aep.planning-md/3
id: task:consumer-backlog-20261002
kind: task
status: active
title: Process the full consumer-defect backlog in grouped deliveries
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 26
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

GitHub bot API inventory at2026-10-02T17:50Z:48 open issues, no open PR in that response. This is inventory, not a count of unfixed defects. PR386/381/387 merged under required gates; PR387 merge1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036 preserves its exact tested tree with all15 checks green. All nine inherited PRs are reconciled. The one-time personal-account secret exception is consumed. Latest verified release remains0.51.0; no0.52 tag exists in this task's evidence.

Local carrier batch/consumer-runtime-20261002 at819f7c1a1635a28a64140cb558ed7017b65893de holds307/360,318, binding/child-signal corrections and389 source/IR/diff/projection/runtime work. Native/Go/TypeScript one-time observers have shared43 runtime controls and actual WASM evidence. This is not proof of complete admitted-feature support.

The original combined ess-conformance/ess-cli/ess-entity-runtime/billing-realization run ended101:461 test-result summaries,3287 passed,60 failed,11 existing ignored;21 binaries failed. Preserve combined-runtime-examples-packages.log. Its focused correction rerun at9ce0e5930 completed0:26 actual binaries,277 passed/0 failed/0 ignored, covering all21 failed binaries plus regression additions. Evidence: combined-correction-regressions.log/.exit. This is corrected focused evidence, not a corrected whole-package pass.

Stored-field guard and repeated external-control implementations are now reviewed and integrated at9aefe4f40/f227d08fb;6e316e11d corrects #204 to require actual successful execution and view state. Combined native-guards-repeated-integration.log/.exit is0:27 passed/0 failed across5 binaries. The parity binary includes actual33 Billing/34 Oracle scenarios and16 fault variants.

Entity setup correction recovered after worker usage interruption: source22072a9f6, integrated819f7c1a1. Setup plus generated creation originally failed on occupied minted identities (red5/1); corrected recovery11/0, strict scoped Clippy0 and formatting0. Final independent review approved with0 reviewer executions, immutable review SHA2567665a2c0a04499dc7d0d3140c65feae0f6e6b67fa14079e3dc28f8eeef483ac9. Supplied collision refusal is preserved; only minted identities retry before later generated payload slots. Carrier integration entity-setup-integration-corrected.log/.exit is0:26 passed/0 failed across5 binaries. The preceding command named a test in the wrong package and exited101 before compilation; its log is retained separately, not counted as execution evidence. Valid Integer identities remain an explicit unfinished store capability.

verification-report:consumer-interpreter-capability-audit records remaining admitted-feature gaps, with actual fixture evidence for related/absent/supplied/replay/response support and secondary effects. Native set-effect execution is in progress under the recorded design decisions. Related/caller execution is being scoped from actual facts and existing contracts. Other full-support work remains required; missing external authority is never fabricated. Worker usage errors did not discard recovered source or evidence.

The prior runtime-site-build.log passed WASM/browser lab and site, but later tutorial/example changes still need final site validation. Earlier core1891/0 evidence predates the regenerated corpus. Remaining delivery checks include actual capability completion, corrected whole affected-package checks, ci-lint/projection/schema/site, governed evidence reconciliation and one grouped PR gate. No component PR or additional remote full gate has started.

Other backlog:314 behavior merged but issue awaits318 delivery;318/307/360 local pending delivery;347 ESS capability/guidance merged while private consumer integration unverified;319 requires the accepted ess/22 dependency/generation bundle;312 remains draft with fit/design and general isolation/cross-caller coverage incomplete. The complete48-issue and consumer-AEP audit remains required. No reliable full-backlog release ETA is claimed.

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

## Remaining interpreter capability sequence

The operator's all-runtime feature requirement remains the release boundary. verification-report:consumer-interpreter-capability-audit preserves the source-derived inventory atff128cbdb (corrected source SHA256e5f1ef7910169830102d6eaa65faa089693cbd30ad78a56f317e8bbe2a4b163c, reviewer executions0). It identifies eleven reachable admitted-feature groups beyond the Billing/Oracle models; source findings are not represented as measured regressions until an actual case runs.

Do not interpret the implemented command-execution story's narrow Billing acceptance as proof that every admitted command construct is implemented. The current batch authorization covers completing interpreter capabilities under the operator mandate, while every new contract decision must still be explicit. Start with the existing stored-field-guard story, now active: mixed_guard_wrong_state.rs and adversary_mixed_guard_pass1.rs remain its named acceptance. Then group related-row predicates and supplied caller facts; typed value sources and secondary/filtered set effects; explicit absent input and supplied identity/existence; repeated controls and entity setup; retained results and truthful ordered scans. Each slice requires an actual healthy case and discriminating negative controls, review, and integration into the same grouped carrier. Refusal-only ports do not satisfy these requirements.

The affects omission is a source-derived silent-effect risk and has priority for an actual regression: existing set-effects.yaml Invite has secondary effects that execute::take never reads. No source release is declared complete while the admitted effect is omitted. Absolute clock, externally provisioned fixture values and periodic-host/reading evidence remain separate authority seams: missing facts must remain explicit rather than invented. Before implementing a seam, determine whether the existing target request already supplies sufficient facts and record any unresolved decision.

Current completed correction evidence: carrier374294a81 implements reviewed concurrent Go/TS Timestamp drawing, retaining one RNG draw and exact histories; author focused1/0 plus four unchanged CLI tests4/0, independent review0 executions. Carrierfe48502f5 pins full33 CLI Billing execution and32 generated mutation scenarios against the reference, including all mutant verdicts/killers; author32/0, independent review0 executions. Coordinator native view/snapshot lanes29/0, browser12/0, mutation CLI9/0, tutorial8/0, scoped two-test strictClippy0. Tutorial network-dependent npm blocks retain their documented existing skips; no network execution is claimed. Whole affected-package and final site/gate evidence remain pending.

## Native hook continuation ownership

Continuation within the accepted supplied-facts/trust work and the operator's full-support mandate, based on carrier9ce0e5930. Native owner server_corrections continues stored-field guards in execute.rs and the completed-command consistency-token seam in interpret.rs, with actual stored/mixed/adversary controls. Go owner scope_boolean creates a new managed tree for repeated external controls: forced-state representation, exact-command consumption and configure callbacks in interpret.rs plus a dedicated regression. TS owner scope_aggregate creates a new managed tree for explicit upstream entity setup: new setup.rs, minimal parent delegate/import, and dedicated actual authored-route tests. Each worker preserves its prior tree, uses only its own idle task cache, bounded builds, and the8GiB free-space floor.

These are coordinated slices of one accepted runtime unit, not independent whole-file rewrites. Parent integration is sequential at the coordinator; changed parent sections are disjoint and any Store visibility changes require coordination with the native owner before editing. Root alone writes AEP and publishes. No source release, full capability completion or additional remote gate is claimed from these assignments. Invalid/missing authority must remain explicit; actual repeated invocations and durable read-visible entity state are the acceptance, not acknowledgements or refusal-only implementations.

## Interpreter set-effect execution decisions

The coordinator approves the bounded implementation proposal in ess-backlog-one-time-response-20261002/target/backlog-input/interpreted-set-effects-design.md SHA2567c11f6f6beaf63c588831053fa61f6097cb457147f6f0a880cf4ed07113349a4 under the operator's complete-runtime mandate. This fills execution of existing admitted constructs, not a new format or public target API.

Count applied eligible rows, including an update that assigns the same value already held. Evidence: the existing independent Desk::note at crates/verify/ess-conformance/tests/set_effects.rs223–244 increments for each matching update regardless of byte equality. Nonmatches and moves outside from-states do not count. Add an explicit equal-value regression so this implementation follows the executable reference rather than inventing a different count interpretation. The design's phrase rows changed is read as rows receiving the declared set action.

Use pre-outcome row and primary-subject snapshots for selection, then apply overlapping entry writes in declaration order as the interpreter's deterministic implementation policy. The design explicitly requires pre-outcome subject values and excludes promises about physical row-update order; this policy must not be advertised as a new cross-runtime timing guarantee. Preserve same-entity subject exclusion, include the actual subject identity, and retain compiler limits on permitted write sources. Add explicit overlap/snapshot controls, documenting this internal choice; if an existing conformance assertion contradicts it, stop that case and report the measured conflict instead of weakening the assertion.

Ownership: server_corrections implements private execute/set_effects.rs, narrow execute.rs take/emit/filter/value fallback glue and dedicated/additive set_effects tests. No parent target/facts/repeated-control/setup/view rewrites. TS setup correction owns fresh-creation identity collision handling and coordinates that seam before any native generated-value edit. Required source-built14-scenario execution, existing11 handwritten mutants, #288 identity/reversed operands, changed counts, missing facts and touched-row invariants remain in the acceptance. No-refusal-only completion, no claim that other remaining native features are solved, no new remote gate.

## Related-row and caller-fact continuation

Read-only scope proposal by scope_boolean at carrier819f7c1a1, own executions0, identifies execute.rs related_absent/select/interpretable and execute/subject.rs typed facts as the minimal present-related guard seam. The existing tests/fixtures/related-guard-sign-in.yaml provides the actual healthy model. Implement this first in a new managed tree based on the carrier: new execute/related.rs, narrow selection/admission glue in execute.rs, bounded reuse of typed subject facts, new interpreted_related_guards.rs, and the obsolete Unsupported expectation in related_guard.rs. Native set-effects ownership of take/emit/value remains separate; no caller-value plumbing in this slice.

Preserve the existing cross-record-and-stored-field-guards design precedence, actual addressed-row identity, typed stored fields/state and input namespaces, Kleene conjunction, missing-fact refusal and declaration order. Require missing/matching/nonmatching related rows and decoys, state checks, absent Optional defined/not-defined, input-branch precedence and multi-conjunct negatives. ExistingInstance remains a separately recorded incomplete combination until actual identity/existence execution is implemented. No source grammar broadening to generic IR Subject lookup is authorized by this interpreter repair.

Caller completion follows the native set-effects freeze because its request-local typed context crosses take/emit/value. SemanticCommandRequest::caller must reach selection and value execution without being persisted or merged into input. Validate attributes against the named actor, preserve legacy ordinary fields named caller, retain public execute/execute_generating semantics as no supplied context, and require actual caller_values NOTES creation/owner/nonowner scenarios plus Boolean attributes and consecutive different callers. Guard-only implementation is not completion. Missing authority remains explicit and malformed values must fail before mutation. This is the accepted runtime task's feature completion, not a new source contract; root alone writes AEP and publishes the combined PR.
