---
format: aep.planning-md/3
id: runbook:ui-live-apps-serial-integration-20261003
kind: runbook
status: draft
title: Five serial ui-live-apps waves, one integration branch and one PR
relations:
- informed_by: epic:ui-live-apps
revision: 12
---
## Authority and delivery

Operator approved the five proposed waves serially on 2026-10-03, requiring all of them on one integration branch and one PR. This supersedes per-wave publication cadence: no unit PRs and no intermediate merge to main. Integration branch: batch/ui-live-apps-complete-20261003, based on origin/main 1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036. No open PR matches this batch; the open #396/#397 type-generation PRs are unrelated. Create one bot-authenticated PR when the batch is reviewable, and update that same PR thereafter.

Sequence: served-store-and-entry (#318); feature-request-282 (guard precedence); related-via-optional-input (#304); related-via-stored-reference (#304); related-guard-behaviour (#319). Each unit gets its own managed checkout, serial implementation and independent adversary review. Integrate verified commits into the branch above. Source compile/test, scope and format requirements remain binding. Record local completion separately from final CI evidence; do not claim implemented on a skipped or absent acceptance lane. The common ess/21 bundle constraint in .engineering/waves/downstream-gaps.md remains unresolved until its actual scope and current ownership are reconciled.

## Recovered server candidates

Previous session candidate and full recovery record remain on unit/served-store-and-entry-public at c12cc427a74b3da022708f9a2ca92496e50e047d, including runbook:ui-live-apps-wave-7-resume, raw-report normalization provenance and the first adversary correction. It is retained as unpublished recovery evidence, not overwritten or silently promoted.

The newer accepted story on current main contains additional structural-identity, component-reachability and fallible-context requirements. Committed candidate e9355b003a8c0153d927fbe89c8597cebc667787 from batch/consumer-served-entry-20261002 was applied without committing onto this integration branch as a starting point. Its original checkout remains untouched. Bot author and committer were verified. Its tests/results are not assumed green here; reconcile its implementation with the earlier candidate's useful regression coverage, generated fixtures, docs, dependencies and all current acceptance before freezing review.

## Custody and resources

Integration checkout: ess-w7-server-public; owner session codex-resume-459ab620-public. Historical raw evidence and logs: `$HOME/.cache/uilab-todo/w7s-codex`. New unit checkouts and their targets will be recorded before dispatch. No shared compile target between trees. Rust is mandatory for committed executable repository code; generated Go output remains an explicit accepted target. Keep scratch outside repositories and outside /tmp; 10 GiB free space is the build floor. Preserve original review evidence privately; public reports normalize workstation prefixes with an explicit note and original digest.

## Cleanup and release audit in progress

Current integration checkout was clean before branch creation. Predecessor ids ess-w4-ui-tui-app, ess-w5-go-behaviour and ess-w6-view-params have no live leases or tracked/untracked changes, but retain ignored target output; the third also retains examples/billing-web/Cargo.lock. They require reviewed recovery proof and evidence preservation before managed cleanup. Original ess-w7-store-entry was already privately archived and removed through managed GC. Do not infer the workspace is globally clean: other sessions own active work and open PRs.

Read-only GitHub release listing shows 0.51.0 published at 2026-10-02T03:20:46Z. Exact tag/check/artifact verification is being performed independently; later merged UI work is not assumed released. No new release is cut as part of this audit.

## Unit dispatch and verified release facts

Recovered candidate committed as aee7c22561865469b09a297fd2881cdf03d941ac with both bot identities verified. #318 unit checkout created through worktree: ess-serial-318-20261003, branch unit/serial-served-entry-20261003, same base. Implementor owns source/tests/docs and scoped dependency resolution, its own target directory and lease; root owns all AEP/CHANGELOG/integration writes. Scratch: `$HOME/.cache/uilab-todo/serial-20261003/318`. Its brief requires measured reconciliation with older test controls, full package counts, strict Clippy and fmt-check. No previous candidate's green is claimed for these bytes.

Independent read-only audit verified ESS 0.51.0 published 2026-10-02T03:20:46Z, annotated tag e86d26acc5b4dc680814d5fac34346c557540310 peeling to 0347ffa222939e3791e574d2dbe42d4b4b02d979. https://github.com/beyond10x/ess/actions/runs/36958141779 succeeded, including four native archives, SHA256SUMS, checksum and Linux binary smoke verification. Exact-tag Gate succeeded in run36958106188. Release-record audit37008746985 succeeded. Assets were verified via metadata and successful release checks, not independently downloaded. No remote0.52.0 tag or release exists. Current main is106 commits beyond0.51.0; merged PR381/386/387 work remains unreleased. Documentation publication was not audited.

Predecessor refresh inspection proved all three retained heads ancestors of advertised main: ess-w4-ui-tui-app at88bd4f3aa, ess-w5-go-behaviour at896696ae7, ess-w6-view-params atd414cfc21. Ignored output is being preserved with managed archives before exact-id retirement; no raw tree deletion or blanket cleaning.

## Cleanup outcome and first measured baseline

The #318 implementor ran the recovered candidate's served_entry suite:25 passed,0 failed,0 ignored, exit0. This baseline precedes added reconciliation controls. Runtime metadata inspection found time0.3.55 requires Rust1.88 while the generated manifest claims1.85; the worker is adding a regression then selecting a compatible pinned version. Disk crossed the10GiB floor after baseline completion; no subsequent compiler children were launched until recovery.

Managed archives preserved every ignored predecessor file. Exact-id GC removed ess-w5-go-behaviour and ess-w6-view-params. GC retained ess-w4-ui-tui-app with `worktree-dirty: tracked, untracked, or ignored files make cleanup unsafe`; its finished record and private archive remain, with ignored target output. Do not force removal. Raw inspection and cleanup evidence is retained in the session scratch.

The exclusively reused predecessor build cache's tmp directory was compressed to `$HOME/.cache/uilab-todo/w7s-codex/predecessor-cache-tmp.tar.gz`, compared against the source with tar (exit0), and SHA256 recorded beside it. Only then was the exact original cache tmp directory removed; no managed tree was manually deleted. Free disk was22GiB on the next check and worker compilation resumed in its own unit target. Archives are retained recovery evidence, not a claim that all workspace state has been cleaned.

## Confirmed merge boundary

Operator answered on 2026-10-03: "Keep the existing ess/21 bundle; hold this PR until the remaining bundle work is ready". Implement the five approved units serially and accumulate them on batch/ui-live-apps-complete-20261003. The single PR remains held/unmergeable until the original full ess/21 bundle is ready. No omitted bundle member is implicitly deferred, and no partial-format release is authorized. This approval resolves the prior ownership-of-format question at the merge boundary; the coordinator owns shared format mechanics only as needed for the five units, preserving all other bundle obligations.

## Exact-main fixture reference and retained cleanup limit

The coordinator created managed ess-serial-baseline-20261003 at1ff305685, built ess-cli with debug0/jobs2/locked in that tree's own target (exit0,4m22s), and synthesized examples/billing and examples/gatepass for Rust and Go to `$HOME/.cache/uilab-todo/serial-20261003/baseline/reference`. All4 references matched their committed main fixture trees with diff-qr excluding .ess-output. The baseline executable is retained privately as baseline/ess-main. Its1.2GiB target was removed with cargo clean; the helper tree was clean, its own lease ended, and managed retirement was reviewed. References are suitable for official output adopt; no fixture copying or ownership bypass is permitted. Current-main comparison tests already exclude .ess-output, so retain official ownership metadata.

The older w4 manager retry still returns worktree-dirty even after the remaining empty target directories were removed; repeated archive replacement preserves each state but did not complete retirement. Keep ess-w4-ui-tui-app retained and report the manager refusal. No manual checkout removal is authorized. w5/w6 cleanup succeeded. Historical standalone consumer rust-target caches under the original session scratch were removed with cargo clean after confirming no session process used them; source/log/report evidence remains. The surrounding find traversal printed missing-directory notices after cargo removed those targets (wrapper exit1); individual cargo removals completed. This is resource cleanup, not a test result.

## Review continuity and comparative proof

The first adversary's unchanged publication copy was recreated through AEP on this integration line as review-result:served-store-and-entry-adversary-pass-1-20261002. It describes original candidate d58db28, not the newer implementation. Its finding was fixed in preserved43fb9a25; the recovered current implementation uses a different module arrangement and now passes recovered controls. Keep this distinction explicit. The final second pass will attack the newly frozen corrected candidate; do not claim the earlier review covers its new code, and do not add a third attack after that pass.

Exact-main CLI synthesized the same served-notes fixture used by the current unit: Rust23 artifacts, Go16 artifacts, both16 generated capabilities and0 obligations/refusals. Both entry paths were absent (explicit filesystem assertions passed). Logs: baseline/notes-{rust,go}.log in assigned scratch. Corrected unit acceptance now reports30 passed,0 failed,0 ignored, including actual generated executable operation, compared with initial25 passed. This establishes new no-handwritten-server behavior against main; final frozen bytes still require the remaining package checks and adversary.

Coordinator inspected the three legacy assertion corrections. Caller/generation/external expressions now pin the fallible try-call and propagation syntax while preserving other error-field assertions. Single-crate feature assertion pins exactly dep:clap/uuid/time. The independent planned-stub reader excludes only the exact asserted dynamic unmet_context helper, bounds literal-source scans to each struct literal, and preserves the four-entry plan/stub equality check. These are accepted companion-seam contract updates, not removed acceptance obligations.

## Completed predecessor cleanup correction

The retained w4 checkout was not merely empty directories. Filesystem inspection found two stale Unix sockets under target/review-boundaries-17/authored-discovery/fixtures/181773-11: socket-config/ess-inputs.yaml and bad/socket. The creating PID181773 was absent and ss -xap showed no active matching sockets. After deleting exactly those disposable socket entries and then empty target directories, the manager's dry run reported ess-w4-ui-tui-app eligible. Exact-id gc apply reported removed; the path was verified absent. This corrects the earlier incomplete empty-directory inference. Managed archives preserve the prior states.

The helper ess-serial-baseline-20261003 was also removed through managed GC after its reference outputs and executable were retained outside the checkout. The three predecessor branches unit/ui-tui-app-generator, unit/go-generated-behaviour and unit/served-view-params each passed merge-base --is-ancestor against origin/main, then git branch -d deleted exactly those branches. No unrelated checkout or branch was removed. All six recovered predecessor worktrees w2 through w7 are now retired; the separately named integration checkout and current serial unit remain active.

## Concurrent remote activity observed

A later read-only remote check during #318 final package testing reports main d0f22461dae326f59664bc9427efad3589641961, after unrelated integer-bounds work landed. Open PR398 is another session's release/0.52.0-20261003, titled publish ESS0.52.0 from merged consumer fixes; PR397 event-roots remains open. No0.52.0 tag was returned by this check. This updates the earlier open-PR snapshot without taking over that release or treating a release PR as publication. Our frozen #318 source still targets recorded main1ff305685; reconcile remote changes before final integration validation. No existing PR matches our serial batch branch.

## Wave1 frozen candidate and final review dispatch

Implementor handed back clean source candidate4a17f69b62f53e80bda3c32cdecd79f06fca1ac0 after coordinator bot commit and author/committer verification. All23 changed files, including6new generated files, were staged; private .ess-output metadata stayed ignored. Raw report SHA256 ed02ece4b4476b57bc9c8d9e21105bdbe1a3fd94e787c206789ec6a9aab45634 is retained in assigned318scratch. Package-after.exit=0; the runner's60 summary groups total404passed0failed1ignored. Before-fix full run already contained the five new controls:404executed,378passed26failed1ignored. Acceptance baseline25→30passed. The single ignored removed-record scenario is pre-existing, not counted as green.

Strict ess-synth Clippy, fmt-check, diff-check, both actual Rust1.85.1 native layouts, Go vet/gofmt, concurrent real-entry invocations and all4fixture byte comparisons exit0. Gatepass-realization executed12cases (7unit+5conformance), exit0, preserving legacy Context compatibility. Local Go is1.27.0; PR CI's specified Go lane remains required before terminal completion. Runtime pin correction is clap4.6.7/uuid1.26.1/time0.3.45; no unrelated dependency upgrade.

Independent served_entry_adversary_final dispatched tests-only in the same managed unit checkout after implementor/coordinator leases ended. It receives source comparison base1ff305685 and frozen4a17f69b, measured counts and raw logs, own lease/target, prospective-case-first instructions, and the remaining second/final review budget. No next-wave implementation begins until this review/correction/integration completes. The coordinator prepared only the #282 dispatch brief. Current source is not yet declared implemented or published.

## Operator cross-session ownership instruction

Operator asked this session to coordinate with Codex session01a0feb8-366c-7ff2-bce6-17aa26249ee3 and explicitly reserved all ess-transports work to a third Claude session. This batch excludes ess-transports; do not change, review-as-owner, merge or release that session's work. Preserve our five-story serial scope and full ess/21 bundle hold. Source corrections and new-unit dispatch are temporarily held while establishing overlap ownership; the tests-only final review completed independently.

Normal cross-thread message and CLI queue routes both refused the named ephemeral session (no rollout; ephemeral submissions unsupported). A coordination message was successfully delivered to the other active ess coordinator thread01a0fbbb-1141-7e33-8cc0-9a6184900bea, asking it to confirm/relay the supplied session and report its scope. Message supplied our branch, checkouts, five stories, held PR boundary, pending Go decoder correction and exclusive Claude ownership of ess-transports. Await actual reply; delivery is not acknowledgement. No release/transport checkout was touched.

## Final review result retained; correction awaiting ownership check

The final tests-only review added166lines and3cases to served_entry; all existing assertions remain. Target output:32passed1failed0ignored, exit101. Actor-alias and static HEAD/document-route boundaries passed in both Rust and Go. Union identity/order passed in Rust; generated Go compilation failed before its assertions. Exact-main CLI reproduced the same failure, and cmp showed byte-identical generated wire.go. The defect is unchanged go/http.rs:730: Optional union payload shape is declared within its presence branch and referenced afterwards.

Immutable publication report recorded verbatim as review-result:served-store-and-entry-adversary-final-20261003; raw SHA256 d7d97ccecd04f94d8eac6ddc89c0a7cddaaf241c48a77cf9172b10468147f057. Explicit findings comparison against served-store-and-entry-adversary-pass-1-20261002 reports0carried,1new(pre-existing Go decoder),1resolved(older candidate memory-domain gating). The default findings selection picked historical scope reviews rather than this attack pair, so the explicit --from/--to report is retained in318scratch. No third attack is authorized; route the one correction after the cross-session scope check, then verify the preserved regressions and complete package tests. Reviewer released its lease; source remains frozen.

Preliminary common-security check on integration ec636c34a3 succeeded, scanned113commits and retained signed receipt. The initial invocation lacked its trusted policy path; the retry supplied the exact policy/key paths from installed hook configuration, without changing policy. This receipt covers that preliminary head only and must be regenerated for final publication.

## Cross-session contact established

Coordinator01a0fbbb confirmed it owns local batch/consumer-runtime-20261002 in ess-backlog-next-20261002: #389 one-time responses, #312 isolation/mixed callers, native typed Store/caller/existence/SubjectField+Increment and RelatedField. It confirmed this session should own318/282/304/319, noted older local318/307/360 copies to reconcile, and stated it does not own releasePR398 or transports390–395. Our later conformance interpreter/synthesis edits must wait for a reviewed shared base or explicit file ownership agreement; bounded ess-synth server correction is disjoint from its current work. Both peers explicitly reserve all ess-transports work to the third Claude session.

The exact operator-supplied session01a0feb8 was reached through the running local app-server's supported turn/start operation after metadata-only thread/read confirmed ephemeral=true, idle, canAcceptDirectInput=true, no parent. No config/model/permission override was used. Delivery returned turn01a0ff05-03cb-71b1-86e2-b0ab74060543 inProgress with no error; full message and response retained in assigned scratch. It carries all ownership boundaries and asks whether that session ownsPR398. This supersedes the earlier delivery failure; acknowledgement still pending.
