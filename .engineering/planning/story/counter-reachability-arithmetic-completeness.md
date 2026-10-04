---
format: aep.planning-md/3
id: story:counter-reachability-arithmetic-completeness
kind: story
status: active
title: Counter reachability refuses incomplete arithmetic bounds
refs:
- provider: github
  reference: beyond10x/ess#413
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/counter_limit.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T21:39:27Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-10-03T21:39:27Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

A counter reachability enumeration returns the existing incomplete result whenever any required bound or widest-step arithmetic cannot be represented, while finite ordinary counter/CAS execution and shared related-counter semantics remain correct.

## Fit review

1. Need: the brand-free Counter reproduction in the retained ESS413 revision2 fit/design has Create set revision0, guarded increment1, stale-input and signed-MAX refusals. Measured0.51/0.52 synthesis reports two refusals and incorrectly describes1 as the nearest reachable value below MAX; the short authored Create/Authorize0/Authorize1 path reaches2. These are retained owner observations, not new coordinator executions. Fit/designSHA5916502a20b823b4b4bec949cb0101e77462f8f8c3b3129143946593c3c27f3e; independent reviewSHA59595e768daf1a481a4da1963375167c6187e0e61c3412025fb80b1ab77dbf43.
2. Class: defect in arithmetic completeness, not a new seed capability. Current subject_fact.rs:2524-2554 promises a complete reachable span or None, but chain(checked_add(...)) silently omits unavailable literal bounds and filter_map silently omits unavailable negative magnitudes. Primitive facts.rs checked_add intentionally refuses signed-i64 overflow. Missing required arithmetic cannot establish completeness.
3. Existing idiom: ordinary finite command arrangement works; authored entity setup can separately test extreme states but does not feed generated synthesis. These measured controls and docs/design/authored-entity-state-arrangement.md:105 explain the separate seed gap. No arbitrary reset, larger search budget or weaker zero-refusal gate is required to correct the enumerator.
4. Fit: keep the current Option incomplete result, exact Number arithmetic, existing search budget and shared reachable helper. related_guard.rs consumes the same helper and needs a regression, not a speculative production edit. Preserve all command/input/related precedence and exact faulty-target assertions. All suites still use the current format/initial-state authority.
5. Second adopter: a sequence-number allocator with compare-and-swap increments and signed-limit exhaustion needs the same accurate bounded reachability; tests/counter_limit.rs already models this class.
6. Cost: no source, IR, suite, report or generated public API change. Correct malformed selection output only; preserve unaffected deterministic suites. Production scope is one helper file and its existing counter-limit test file. No primitive Number, CAS grounding, seeds or runtime-adapter changes.
7. Alternatives: change nothing preserves a false completeness claim; raising the bound cannot repair missing arithmetic and cannot enumerate to MAX; production reset changes the modeled domain for its checker. Chosen checked propagation returns an honest existing incomplete/refused result. Explicit synthesis seeds are separately proposed413B/42-43 and are not adopted here.

## Decisions

Accept413A as the previously reviewed bounded correctness correction, supplemental to the unchanged pinned50 ledger. The prior runbook accepted the fit in principle; this artifact settles exact implementation ownership. External412/413 owner may prepare the two-path candidate from the coordinator's current integration commit. Neither304 unit currently has a production writer on subject_fact.rs; integrate the independently reviewed A correction before dispatching304 production so that304 rebases onto it. No concurrent writer to those two paths. Compiler custody remains separately scheduled.

No413B implementation, new seeds/provenance,42/43 adoption, primitive arithmetic change, speculative CAS change, production setter or relaxed refusal gate. Whole413 remains open after A if its seed obligation remains unimplemented. Return any required scope expansion or independently red finite-CAS defect before editing more.

## Acceptance

Failing base tests and corrected tests prove MAX, MIN, padded-bound and widest-negative-magnitude arithmetic never yield a falsely complete set; a finite limit2 CAS fixture executes ordinary create/advance0/advance1, observes2, rejects stale expectation and exhausts at2 without EstablishEntity; and the shared related-counter regression plus existing counter/subject-input/precedence suites preserve healthy and decisive faulty-target behavior.

Keep honest extreme-state refusals after A; A does not promise zero refusals or extreme-state reachability. Preserve unchanged suite bytes where output is unaffected, current fresh provenance and old-format refusals. Required exact controls, strict scoped lint, owning formatting, affected tests and independent whole-unit review precede serial integration. Initial production author grant is source-only; no compiler lane or implementation acceptance is inferred from design approval.

## Production implementation boundary

The expected arithmetic baseline is measured red at required MAX/MIN window bounds (3pass/2fail), while the full counter_limit baseline now passes17/17 on unchanged production after a test-only hash-formatting compile correction. Source freeze2b7d64232621c327024b418597767a6e198154e162cdf5efc8c8b8c7bca45d47a preserves all arithmetic tests and original production bytes. Retain both failed logs; the initial compiler error was not a finite-CAS semantic verdict.

The external owner may now implement only the approved arithmetic-completeness correction in subject_fact.rs and scoped regression assertions in counter_limit.rs. Every required negation/padded-bound computation must propagate None on arithmetic failure rather than omit a bound and report a partial reachable set as complete. Preserve numeric semantics, search budgets, honest extreme-state refusal, shared-related behavior, finite-CAS arrangement/outcomes and canonical unaffected bytes. No seed capability, additional format, production state setter or unrelated CAS/input/guard change. Full affected proof and independent whole-unit review remain required before serial integration. Source edits may proceed while root verifies the disjoint authored/aggregate test unit; compiler custody is separately granted.

## Focused treatment and remaining integration checks

Frozen production source bbc9d2f51fb3a3f7fbadb539e833143c2e8a8c36d08a3433566fb9514c14833e passed all5arithmetic regressions (126unrelated libtests filtered) and unfiltered counter_limit17/17 with0failed/ignored. Raw logs15cf0fe051ff7fbb9ee5af3f7e5504eca36a64d2691608a150fbe7599b832280 and530c97bf8ed192d29aa8182ae877d0b34f57eb7f43e45650925b4c1752e6ff35. Source hashes unchanged; ordinary-two6bf91d8f and shared-related-three8f56c982 canonical bytes match the baseline. Original2arithmetic reds and initial test-only formatter compilation failure remain retained. Full affected-package, strict lint and independent review remain required; no integration yet. Refresh the worker base after reviewed fixture corrections and the separately diagnosed missing-token generated-runner defect, to avoid rerunning a knowingly stale full-package baseline. Seed413B remains separate/outside50 and no42/43allocation or implementation is authorized by this arithmetic correction.

## Refreshed owning-package validation underway

The unit is fast-forwarded from0beee60c6 to reviewed integration2255315a48a947f363d46cb3c691a57b4879f3b1, retaining exactly two source paths and unchanged treatment bytes. Refreshed patchSHAa9f78c08bdf51955d61dd471578929d0c8877ea7eb4430c5809be7fe6c844d78; subject_fact.rs SHA cb3b9fb0d1b4acd7aabf8a43f813e3920133f480ce0fd55dc2d88eef08d9dc20; counter_limit.rs SHA26e00ff65a9ab39c453bd60ad2da24433e567cc61115a0fc094ae65b923cbc44. Root consumed the refresh and actual tool prerequisite records. Rust1.98.1, Go, Node, TypeScript, Prettier, Task and installed Node type definitions are present; TMP is outside Git and Go cache is unit-exclusive. Fresh preflight free17044844544bytes exceeds the unchanged12884901888 floor.

The sole ESS compiler lane is assigned for strict all-target Clippy, owning package and repository formatting, then the full unfiltered ess-conformance package with --no-fail-fast, no required-runtime skips and a7200second command bound. Lint is starting; no terminal package result is claimed. An earlier grant was not executed before a separate cleanup task ended; the owner explicitly confirmed no refresh/compiler had occurred and resumed the same grant. Final independent whole-unit review and source integration remain due. Optional-input production remains dependent on this completion;412 and transport execution remain held.

## Final review probe conditions

The independent review's prepared command plan bbcc6204c71465c548f9d4d2349033f909e95121082fe5496b615c4d26153601 retains one new public API MAX/MIN arithmetic case and omits repeated author controls covered by the full package run. The coordinator inspected the plan and unchanged probe96346e1e50da2c3297a19f17ca154efb40eb2fdabae1d3cc5f994921ab767436. Execution is conditionally authorized only after the full package command terminates successfully, required runtime execution is accounted for, the two source hashes still match the refreshed freeze, and newly resolved owner-built rlibs are hashed in a retained command inventory. Bounds: rustc compile120seconds, exactly independent_extreme_counter_refusals_do_not_claim_a_complete_nearest_value180seconds, total300seconds. Fresh available disk must be at least12884901888bytes before each start; retain the existing isolated environment and external scratch. A failed condition holds execution; there is no cleanup grant, repeated author control, extra case, source change or substitute acceptance. The package is still running when this decision is recorded; no success or probe execution is claimed.

## Full package result and compatibility prerequisite

The refreshed full package command terminated101 after1128.553625seconds:337result summaries,2609passed,2failed,8ignored,0filtered. Terminal rawlogSHAb4ff23325419aaf72d5b84c5247c5b1bc28a007106612e1c80c469ad171c1b0b. Both failures are in retained_replay; actual generated Go reported153pass events,33fail events,0skips. Seven existing ignored integration cases plus one ignored doc example are explicitly retained; no required-runtime skip notice was observed. Strict all-target lint, owning format and task fmt-check passed. The two arithmetic source files and patch remain unchanged and both finite canonical hashes match baseline, but the package remains red. The independent probe's success condition is unmet and execution is held; no final approval or integration is claimed.

Root now owns task:retained-replay-fixture-consistency under story:feature-request-312 to resolve the fixture/token contract mismatch supported by source diagnosis. This is separate from the two arithmetic edits; causal treatment proof remains due. Refresh this unit after the separately reviewed fixture correction and satisfy the whole-package and final independent-review requirements. The execution lane was returned on process termination; capacity remains below the required floor. The user's compiler-child inventory remains read-only with no cleanup authorization.

## First whole review recorded

Independent first whole review4dc84d7740cbb73bfd6fbfa36378ce235ecdb9ccb254fa283c197e057f7a1cab is now review-result:arithmetic-completeness-413a-20261004-r1. Its publication-safe normalizationf08be95432362c1cf1218688669097d8c34c103d39911a6ef00e68be3297037f removes only the private scratch prefix and clarifies that the owning external coordinator ran the author checks. Both acceptance blockers remain confirmed with origin undecided in the immutable review; no arithmetic implementation defect was found. Disposition escalates the package failures to task:retained-replay-fixture-consistency. Review1 is closed incomplete. Exactly one final whole review remains after the correction and full package verification; no new review budget or repeated author controls are created. The independent MAX/MIN probe remains unexecuted.

## Build artifact custody correction before final verification

The original full-run target was unexpectedly absent at the renewed archive preflight. No complete archive or copied old review-link closure exists; do not claim those binaries, target-local fixtures or old rlibs retained. Preflight-refusal1bb2f7569907f254359735017612bc51277d60c9e4ddde82708f2edb7e303372, surviving-evidence5a7d22f3d81e0953cb3ec14288be8ffa4cce265f522df5a196343216699bb44d and target-absence-bound92e78c8c1e9685f535d88dfef1dcb806b3244d8596ef2502bcedd416b2349451 retain the observations. Owner and root performed no original-target cleanup; actor is unknown.

The source two-path freeze, original complete package log, first review, full target manifest, externally retained failed Go fixtures and baseline executable survive and were reverified. First review stays immutable and incomplete; no replayed proof or review-budget reset follows from artifact loss. After the independently accepted fixture integrates, rebuild corrected source, run the required full package, and bind final review's still-unexecuted public MAX/MIN probe to fresh artifacts only. The separate fixture author's required36Rust tests and strict lint now pass; its independent review remains pending.
