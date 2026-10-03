---
format: aep.planning-md/3
id: review-result:consumer-293-design-pass1
kind: review-result
status: active
title: Explorer design requires separate external arrangement authority
relations:
- reviews: story:feature-request-293
revision: 1
---
needs-revision

Independent source review of the #293 fit and implementation design. The underlying Optional/unknown-instance need is supported; one binding design correction is required before implementation approval. No new feature exclusion is recommended.

Reviewed immutable artifacts:

- `fit-proposal.md`: SHA256 86bf244ceb13f4211ad083fe156d29f23993665719756cb2e200ccf57eb5a7af.
- `implementation-design.md`: SHA256 50f0e84dfde4f5ba2060646d5cb2d4c632070c17a6dcf41bc872cf8c6a9f21ad.
- Source HEAD: 5c5aeaf795a46aacfd3709e04f630d83d8a6a837.
- Go explorer source blob: 027e44e43aa1e0fdb81becbe132284642536cdc9.
- TypeScript explorer source blob: bc67bd9bd2241bc1ef68550dcfdd1aec5cc59957.

Reviewer execution count: **0** builds, tests, target executions or probes. File/hash inspection and source reading only. The companion's executed results are author evidence, not independently rerun evidence. No production or AEP edits. This report contains no private credentials or consumer-specific source.

```findings
[
  {
    "file": "implementation-design.md",
    "line": 40,
    "category": "external-selection-and-replay-authority",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The design redirects a selected external subject-requiring branch to unknown-instance and deduplicates equivalent redirected choices, but does not retain arrangement identity separately from the expected final outcome. Both current ports use the same outcome object to decide ConfigureExternalOutcome, select replay choices, and assert the target's returned outcome. For a command with a subjectless ordinary branch and an external supplied update, a missing identity must allow the ordinary answer without arrangement and the unknown-instance answer after arranging that external update. Replacing the external choice with the not-found outcome loses the arrangement; retaining the external outcome makes perform expect the wrong returned outcome. Specify an internal choice carrying original arrangement identity plus resolved expected outcome, thread it through choices/perform/trace/replay/shrink/unarrangeable and reach accounting, and deduplicate only choices whose arrangement obligations and expected semantics are equivalent. Add actual compiled Go/TS controls for this case and two external alternatives redirecting to the same unknown-instance result. Do not solve it by excluding external combinations."
  }
]
```

## Evidence for the finding

The proposed sequence at supplement lines 37–40 correctly retains early input refusal and delays unknown-instance routing until an actual subject-requiring alternative is selected. The missing part is how that alternative is represented afterward. This is a design gap, not a claim that an unimplemented patch already fails.

Current Go source `crates/verify/ess-conformance/src/go/explore.go`:

- `exploreDecision` around 888 stores outcome objects, including an array of external outcome objects.
- `exploreChoices` around 1100 filters and returns those objects using their external outcome names and prior arrangement state.
- Lines 1726–1743 choose one object, infer whether to arrange from that object's condition, put its name into `step.external`, then pass that same object to `explorePerform`.
- `explorePerform` around 1272 compares the actual returned outcome with that object's name; its subject/effects drive the expected model update.
- Replay lines 1616–1624 reconstruct the selected choice by testing its external condition/name against `step.external`.
- Reach accounting at 1750 records the selected object's name, so after separating identities it must report the actual expected terminal outcome without claiming an external outcome was returned merely because it was arranged.

TypeScript source `crates/verify/ess-conformance/src/ts/explore.ts` has the same coupling in `Decision` around 675, `choices` around 846, the main loop at 1417 onward, `perform` around 989, and replay at 1320–1327.

The required distinction follows existing native source control flow: `interpret/execute.rs::select` chooses the actually forced eligible external branch; `take` at approximately 1118 looks up that selected branch's supplied row; a missing row routes through `unknown_instance` at 1245. That function either executes the declared creating missing-row branch or returns the declared no-effect refusal. It does not change which external cause was arranged. `ess-domain/src/command/outcome_shapes.rs::validate_command` at 443 onward permits a missing-instance branch when a supplied move/update/delete exists; it does not require every accepting alternative to name that subject. Source validation of the proposed concrete fixture is still required before it counts as executed acceptance evidence.

Suggested internal representation is a pair such as `{ arrangement: optional original external handle, expected: resolved outcome }`. This is internal metadata, not a public callback or format change. A trace already has the external arrangement name; replay must match against the pair's arrangement field, then recompute/check its expected terminal outcome from current source/model. The ordinary no-arrangement choice remains distinct even when its final result matches a redirected external choice. Two distinct external arrangements remain distinguishable for unarrangeable reporting and the existing forced-state semantics. Go and TypeScript must share order and PRNG consumption.

## Reviewed portions without additional blocking findings

**Optional draws and absence.** A separate presence-bearing draw result is appropriate: existing Go readPath plus exploreUndetermined and TypeScript undefined/NO_RECORD distinguish different concepts today. The proposed recursive wrapper keeps unsupported inner reasons and existing mode/depth boundaries. Carrying metadata is feasible through `ResolvedField.naming.presence` (`ess-compiler/src/ir.rs:296` and `docs/design/wire-presence-json-prefix.md`, section 2). Default no-policy absence may be either missing or null under existing conformance presence matching (`go/runtime.go::presenceAdmits` around 8331 and the TypeScript optional comparison around 5314), so choosing a documented canonical spelling is not by itself a new source rule. Explicit policies must remain enforced.

The requirement to treat deliberately omitted Optional input as known absence is necessary. The current Go `exploreValue` returns Undetermined when a path is missing; TypeScript `valueOf` returns undefined. The implementation must complete the proposal's typed read helper all the way through payload comparisons, stored model fields, views and invariants, not just input drawing. In particular a known absence must not become an unconstrained expected field which `exploreAgree` skips. The proposal already requires Optional-copy and omission fixtures; those tests must assert healthy absent outputs and detect a fabricated present value. This remains implementation acceptance work, not an additional design blocker.

**Unknown-instance source semantics.** Refusal and creation are genuinely different source-admitted forms. Domain validation at `outcome_shapes.rs:116` forbids effects/emits on a noncreating missing-instance branch; lines 290–350 admit a creating missing-instance branch paired with a supplied update/move using the same identity. Native `unknown_instance` distinguishes these forms. The design correctly avoids introducing a new refusal-only restriction and calls for actual creation effects through the existing effect machinery. Its effect-preserving coverage must include this admitted create-or-update form, not only the original refusal probe.

**Draws and serial replay.** The dedicated fresh-ID collision check is necessary because current drawStep always requests an existing subject. Keeping exact field paths and observed creation-index references for present Optional identities matches existing nested replay behavior. The stronger same-command/existence/semantic-fault shrink witness is necessary: current `exploreShrink` around 1838 and TypeScript `shrink` around 1531 accept merely the same failure kind. The proposal explicitly calls for correcting that for this feature.

**Concurrent shared code.** Both ports draw prefix and workload through the same draw helper. TypeScript `runConcurrent` around 2342–2372 draws all workload inputs against the prefix model, before executing the concurrent calls. Therefore “fresh” there proves absence only in that prefix/draw snapshot; it cannot promise the row stays absent until a later concurrent invocation. An upsert can create it between calls, and later calls can validly update it. The proposal correctly keeps concurrent outcome judgment at the native history boundary. Implementation tests must preserve that distinction and must not apply the serial replay absence fence to reject a legal concurrent race or reinterpret an actual recorded outcome. This is a clarification of existing concurrent semantics, not grounds to exclude upserts.

## Required revision and rereview boundary

Amend the supplement with the arrangement/expected-outcome choice contract, exact affected internal seams, deduplication rule and actual test cases. Preserve the existing Optional, creation and concurrent coverage commitments. No format/API expansion or feature exclusion is needed. A delta rereview of that revised design is sufficient before implementation authorization; production review will still require actual compiled fixture and generated-runtime evidence.

This is an AEP fence-format rendering of the immutable review ecb2ed60d49410ea008b027bafd2a95e1715aec4bcda282be301bbc490d1a356. The original remains unchanged; no new review execution is implied.
The schema origin is rendered as introduced, identifying the omission in this proposed design. The original prose origin and source evidence remain in the raw report and the review narrative above; this does not claim an executed production defect.
