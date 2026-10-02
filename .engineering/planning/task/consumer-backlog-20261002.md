---
format: aep.planning-md/3
id: task:consumer-backlog-20261002
kind: task
status: active
title: Process the full consumer-defect backlog in grouped deliveries
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 31
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

Latest verified remote inventory remains2026-10-02T17:50Z:48 open issues and no open PR; inventory is not a count of unfixed defects. Inherited PRs386/381/387 merged under required gates, all nine inherited PRs reconciled. PR387 merge1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036 preserves its tested tree with15 green checks. Latest verified release0.51.0; no next release tag or current batch PR has been created. The one-time personal-account secret exception is consumed.

Carrier batch/consumer-runtime-20261002 nowee6962765749d25e019fa819c7b0019799aad464 holds307/360,318 and389 plus actual native stored guards, repeated external controls, validated setup/collision correction, absent-input execution, present-related predicates and filtered/secondary set effects. Latest bot commits819f7c1a1,b4a98ae06,c88fa782f,ee6962765 preserve the reviewed source units; only conflicting adjacent module declarations needed reconciliation. Bot commit correctly refused while that conflict was unresolved; root then preserved both declarations and committed through the same bot tool.

Original combined4-package run remains retained exit101 (3287pass/60fail/11ignored;21 failed binaries). All21 failed binaries were included in corrected focused rerun at9ce0e5930:26 binaries277/0/0, exit0. No corrected full-package result is claimed. Subsequent exact combined checkpoints: native guards/repeated27/0; setup integration26/0; absent controls30/0; absent/related integration30/0. Latest set-effects-combined-integration.log/.exit at ee6962765 is0,47 tests passed/0failed/0ignored across6 binaries (absent9,setup6,related7,parity3,effects10 and independent-effects12). Parity includes33 Billing/34 Oracle and16 actual fault variants. Earlier native/Go/TS one-time43 controls and actual WASM evidence are preserved; none proves complete admitted-feature coverage.

Two independent reviews found actual defects and drove measured corrections: setup collision red5/1 to recovery11/0, and excluded-primary set filter red0/1 to scoped22/0. Both final bounded reviews approved; original needs-revision records and fixed outcomes are retained. All independent reviews named here executed0 tests; owner/coordinator execution logs carry the counts. Scoped strict lint/formatting passed for the committed units. Whole affected-package and final site checks remain pending after current source integration.

Active next ownership: scope_boolean implements lawful typed Store identities in a new tree fromee6962765; server_corrections implements invocation-local typed caller context from the same base; scope_aggregate implements accepted-redesigned312 in ess-backlog-initial-state-20261002 from4bc5660e4. Each owns an isolated tree/cache. Root owns AEP, shared integration, release documentation and subsequent sequencing.312 reuses the unreleased suite34–35 allocation, declaring an empty logical scenario namespace and adding same-row cross-caller evidence; it does not authorize physical global resets. Actual312 provenance0/2 and UUID mutant0/1 baselines are retained by its owner; no treatment completion is claimed.

Remaining native capabilities include supplied/existing-identity execution, retained results, remaining typed value sources and truthful ordered scans, alongside completion of typed identities/caller context. Source-derived audit and8 actual fixture reports are retained; every incomplete capability remains part of the operator's full-support release bar. Absolute-clock/periodic/provisioned-fixture authority cannot be fabricated. History/1 string identities are an explicit wire limitation, not permission to guess typed identity values.

Other full-backlog requirements remain:314 behavior merged but issue awaits318 delivery;318/307/360 local pending;347 private consumer integration unverified;319 requires the accepted ess/22 dependency/generation bundle;312 now active, not completed. Full48-issue and consumer-AEP completion audit still required. After implementation, run final affected-package/projection/schema/ci-lint/site checks, one grouped PR gate, then exact release requirements. No reliable full-backlog release ETA is claimed.

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

## Explicit absent-input execution continuation

Coordinator owns the measured absent-input gap from capability-absent.json: all3 source-built scenarios Unsupported. Existing compiler fixture absent-input.yaml and conformance absent_input.rs distinguish no document from an empty map and contain decisive wrong-answer mutants. Implement in the current managed carrier: dedicated interpret command-completion helper, narrow execute.rs absent-branch selection and removal of whole-command refusal for InputAbsent, parent absent callback and common result completion, additive actual-target tests in absent_input.rs. No new vocabulary or source version; this executes the admitted ess/16 construct.

Keep omitted and empty input distinct. Resolve only the declared InputAbsent outcome before field validation; when none exists answer undeclared without events or state changes. Preserve actor grants, command identity, typed declared error, consistency and no event fabrication. Normal supplied-input commands continue normal selection despite their absent sibling. The caller-attribute evaluator remains separate pending coordinated context completion. Root owns these sections while scope_boolean owns related selection/admission and server_corrections owns effect/value execution; imports are sequential and no source changes happen during root builds.

## Resumed runtime implementation evidence

Carrier b4a98ae06 implements explicit absent-input execution with a shared command result completion helper. Actual source-built3-scenario regression was0/1 test with Unsupported before correction, then passed. Final owner5-binary validation30/0 covers absent_input9, entity_setup6, repeated_controls7, runtime_parity3 and scenario_facts5; strict scoped Clippy0/fmt0. Added direct control originally used an invalid ScenarioId and failed before production; corrected fixture and final evidence are distinct retained logs. Independent review0 executions approved exact patch8801f6d10deddddc176ee01bb8c60b66225e450bd9fc6f1a5f48858ea3b76add; original JSON SHAefb5fc92c19ff8473d4e6da2d3c8bf64d315f60f1b403608eae1d5cc16ccc17b is preserved verbatim inside mechanical AEP Markdown adapter review-result:consumer-interpreted-absent-input.

Present-related guards sourceee50829daaaeaa8112f868aa830248ace21d3da5 integrated asc88fa782f. Identical-final-test baseline5pass/8fail to13/0 plus neighbor43/0, strictlint/fmt0; root independent source review0 executions recorded consumer-interpreted-related-guards. Root combined absent-related-integration.log/.exit then passed0:30 tests across5 binaries, covering absent9, newrelated7, parity3, stored5 and existingrelated6. Exact text identity lookup, typed actual row fields/state, decoys, Unknown facts and precedence are observed. ExistingInstance combinations/nontext identities/caller context remain incomplete.

Set effects first review consumer-interpreted-set-effects-pass1 found an introduced excluded-primary bug: filter evaluation happened before primary exclusion. Implementor reproduced actual admitted primary-only case0/1 and corrected exclusion order; pass2 patch eb077670d83fe82a7683c903f21f0128c3c1edbf748ad4c3088da22c4fbd81d1 is frozen with owner22/0 and strictlint/fmt0 for final independent review. No integration or resolved-finding claim until that review and merge evidence. Original four-file patch/report and failing run remain retained.

Feature312 is active revision22, accepted-redesigned after its seven-question fit assessment. Existing caller/singleton coverage is retained; remaining general same-row and initial-state requirements are explicit. Owner scope_aggregate uses managed ess-backlog-initial-state-20261002, branch batch/consumer-initial-state-20261002 from4bc5660e4, exclusive synthesis cache, source/design/provenance/synthesis/generated reader/display scopes only. Reuse the unreleased suite34–35 pair; no additional source/suite major allocated. Native interpreter ownership stays separate. Binding design precedes production edits; no312 implementation-pass claim exists yet.

Storage incident: worker observed free space3GiB while other workspace jobs were active, below8GiB floor; its Clippy had already completed when polled. Root fresh observation recovered to14GiB without root reclamation, then14–15GiB on later checks. Pause new builds below floor and retain this observation; do not claim the floor was continuously maintained. No source, managed tree or evidence was deleted.

## Typed interpreter identity continuation

The measured Integer setup refusal is admitted-feature work under the full-runtime mandate. scope_boolean's read-only scope atc88fa782f identifies Store String keys, setup's text-only guard, view Node::Text reconstruction and related/effect keys. Proceed only from the integrated reviewed set-effects candidate. Owner creates a managed tree and uses its idle servers cache; root preserves sole AEP/release-document ownership.

Adopt BTreeMap keys of Node with existing lawful Eq/Ord, preserving exact numbers and type distinctions; never stringify or parse guessed JSON. Generic public Store::instances() must enumerate every row and return typed identity Nodes. Add explicitly named text_instances() as a documented compatibility projection; retain instance(entity,&str) as text-only convenience and add instance_typed(entity,&Node). This is an explicit public Rust source-compatibility change, to be noted in the next0.x minor release; silently skipping nontext rows under the old generic iterator is forbidden. No serialized format is added.

Migrate actual lookups, creation collision handling, updates/deletes, invariants, setup, view projection, related guards and filtered/secondary effects. Preserve complete setup validation (including Null refusal), original typed values, scenario authority, atomic duplicate refusal and snapshot visibility. Changes owned: interpret/execute.rs Store/identity/key plumbing; interpret/setup.rs; interpret/views.rs; execute/related.rs; execute/set_effects.rs key plumbing; linearize.rs compatibility seam; dedicated interpreted_typed_identities.rs and precise affected existing tests. Caller evaluator and existing/upsert selection remain other sequential work and must not be rewritten in this slice.

History/1 subject_key and rows fields are intrinsically strings. Keep existing text history behavior and make unsupported nontext adaptation explicit; do not guess Node values from strings or claim typed history parity. This is a wire limitation, not permission for a false linearizability verdict.

Correct one source-audit premise before implementation: mint is not UUID-only. It uses a UUID fast path, then witness::fields with Distinction::further(tick) for other types (current native execute.rs1211–1243). Constrained wrappers take this typed witness path. Preserve it and verify actual generated numeric identity creation; a finite exhausted witness domain can still return bounded NoValue honestly. Do not document a nonexistent blanket generator limitation.

Acceptance uses admitted models with Integer, Boolean, constrained and structured supplied/setup identities; exact adjacent integers above2^53; equal numeric representations colliding and different Node types remaining distinct where admitted; typed related/filter/effect/view selection with decoys; invalid types/constraints/Null and duplicate requests leaving unchanged state; generated numeric and UUID collision controls; complete typed iteration and explicit text convenience; text-history tests plus honest nontext refusal; scenario reset and eventual setup visibility. Same actual test bytes must go red before source correction, then green, followed by strict scoped lint/fmt and independent review. No componentPR or new remote gate.

## Invocation-local caller execution continuation

Adopt native owner brief interpreted-caller-implementation-brief.md SHA25687ec900fbf6b5956c19cf9f4cd489c9962e8f15f4f71edfdeb400dd652e8ca12, grounded in existing caller-values design and actual request fields. server_corrections creates a managed tree from integratedee6962765 and owns a new private execute/caller.rs, narrow context entry/admission in interpret.rs/command.rs, early and later selection facts, subject/related caller overlays, Work value/error context and dedicated/additive caller tests. Public execute/execute_generating stay available with no caller context. Store/identity plumbing remains scope_boolean's disjoint subsection ownership; root integrates sequentially.

Validate exact supplied attributes against the named granted actor before consuming controls or mutating state. Nonempty caller values with no named actor are an invalid request: TargetError::Unavailable through the existing request error path, not a newly unsupported language feature. Never choose an arbitrary actor. Missing caller facts actually needed by the selected branch remain explicit Unsupported/Undecidable; no supplied caller should be required merely because another unrelated command declares attributes. Actor grants remain NotGranted before execution. Empty/no caller does not invent any authenticated fact.

Keep caller invocation-local and separate from input and stored row namespaces; a legacy declared field named caller shadows the spelling at its own input/stored root. Preserve Boolean/numeric/temporal values and presence/ordering semantics. Copy admitted CallerAttribute leaves into sets, events, errors and nested structures without regenerating or borrowing same-named command input. Binding work never implicitly inherits initiating caller credentials. Common command completion is reused, not duplicated.

Require the actual existing NOTES suite and independent handwritten mutants, swapped roles, owner/nonowner/Boolean guards, exact stored/event/error caller values, nested and optional values, wrong/missing/extra attributes and actor-free nonempty request refusal before mutation, stale context after actor changes or caller=None, and ordinary input/stored caller-member shadowing. Supplied-identity/upsert dependencies remain explicit until implemented. Same tests red then green; neighbors include stored/related guards, effects, absent input, repeated controls, setup and bindings. Strict scoped lint/fmt, frozen hashes and independent review before commit; one final grouped remote gate. All committed executable source Rust, own idle served-entry cache only, externalTMPDIR bounded jobs and8GiB floor.
