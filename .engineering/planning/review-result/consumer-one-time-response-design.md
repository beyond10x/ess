---
format: aep.planning-md/3
id: review-result:consumer-one-time-response-design
kind: review-result
status: archived
title: Independent one-time disclosure and full runtime parity design review
relations:
- reviews: story:feature-request-389
revision: 2
transitions:
- {from: "active", to: "archived", at: "2026-10-02T13:59:32Z", actor: "human:timo", revision: 2, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
approve

Independent design review of issue #389, 2026-10-02. This approves the frozen design for implementation; it is not implementation, test, security-proof or release approval.

Reviewed tree: `ess-backlog-one-time-response-20261002`, base `1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036`.

- `docs/design/one-time-response-values.md`: SHA256 `fbc81cf73e2b0bc11a2ab4258137d2a4121fed669892e81e8e602c02b827de3c`.
- `docs/design/one-time-response-values.example.yaml`: SHA256 `c9b42b8d1bee19683508589fe9fec7900cdab25267ab7592af417f7b22aa3273`.
- Authority: https://github.com/beyond10x/ess/issues/389 and canonical `story:feature-request-389`, including the operator's mandatory full-runtime parity correction.

Own execution count: **0**. Read-only design/source review and hash verification; no validation, builds, tests, mutations or source/AEP edits. The example explicitly targets unimplemented source21, so it was not represented as a validated specification. The ess:hardening design-review method was applied to contracts and proposed acceptance, not as an executed mutation audit.

## Findings JSON

```json
{
  "verdict": "approve",
  "design_sha256": "fbc81cf73e2b0bc11a2ab4258137d2a4121fed669892e81e8e602c02b827de3c",
  "own_execution_count": 0,
  "findings": []
}
```

No unresolved substantive finding in the frozen design. The earlier draft was amended during review; approval applies only to the hashes above.

## Reviewed behavior and corrected counterexamples

1. **Outcome ownership and direct flows.** Lines 11–24, 64–80 bind the policy to the exact successful outcome and require typed String/newtype validation. A marked outcome mapping its response into an event is refused; an unmarked alternative sharing the command's response declaration does not automatically inherit that prohibition. Nested mappings and opaque transformations receive explicit treatment. Existing nested response reads are real source syntax (`crates/specify/ess-domain/src/command/value_expression.rs:324`); checks must walk resolved expressions rather than spelling. Static refusal does not pretend to analyze arbitrary implementation code.

2. **At-most-once, retries and rotation.** Lines 39–60 retain consumption after lost response, reject full-result replay, require fresh origins and explicitly bound finite evidence. This agrees with complete typed retained-result comparison in `crates/verify/ess-conformance/src/target.rs:607` and `docs/design/retained-command-results.md:58-69`. No replay redaction loophole remains. Lost receipt, restarts and concurrency remain implementation-owned checks with explicit unsupported coverage; a serial retry is not mislabeled a lost-response experiment.

3. **Actor identity.** Lines 106–109 now include every declared actor, including equal-permission distinct actors. The original permission-equivalence grouping could miss an implementation redisclosing only to a second authorized actor; that counterexample now has an explicit required mutant at lines 310–315. Actor-specific views need particular care: `SemanticViewRequest` at `target.rs:848` carries no actor field. A preceding actor-bearing command does not establish actor-bearing read authority. If a required actor-read observation lacks a real seam, lines 142–151 require a named unsupported cell equally across runtimes, never an ambient-authority assumption or a pass.

4. **Publication leaks and deadlines.** Lines 114–140 now require independent event-log observation in addition to direct events, including refusal publications, retained history and delayed leaks. `ConformanceTarget::observe_events` at `target.rs:207-215` may return before its deadline; direct events explicitly exclude some bound consequences at `:602-605`. A target returning no direct event while leaking through its publication log would have escaped the original draft. The revised contract catches it and also requires the clean-early-event/later-leak mutant. Repeated scans continue through the arranged closing boundary; unavailable completion evidence is unsupported. Equal payloads or optional sequence/correlation values (`target.rs:695-707`) cannot justify dropping observations. Duplicate clean occurrences remain healthy.

5. **Capture authority, scanning and non-disclosure.** Lines 90–112, 153–182 require actual captured responses, exact-field exemptions, all older values across rotations, recursive strings and keys, bounded owned capture state, admission before callbacks and value-free diagnostics even before a malformed origin can be captured. Limits and unsupported observations cannot silently truncate or pass. Failure text, recorded evidence and target-error text are explicitly in scope. Static storage restrictions plus bounded observable channels do not imply that hidden persistence, logs or encoded values were proved safe (lines 185–188).

6. **Fresh envelopes and old bytes.** Lines 190–214 allocate source21, ordinary34, coverage35 and diff13, preserve old unannotated canonical bytes, refuse old pins/readers, and move the earlier coordinated syntax bundle together to source22. A concrete omitted-empty IR policy is consistent with the existing unenveloped IR; no invented ess-ir/2 is proposed. The superseded historical source21 allocation is not represented as already delivered.

7. **Full runtime parity and usability.** Lines 226–300 require the same admitted suite bytes, scenario identities, counts, fixed codes, redaction and healthy/mutant behavior in every execution path. Existing prerequisite gaps are concrete: Go generation refuses direct response/delivery context/structured values at `src/go/mod.rs:50-52`, TypeScript at `src/ts/mod.rs:71-73`, and Go's admitted maximum is27 at `src/go/mod.rs:362`. The design requires these ports to be completed before release, not hidden behind new refusals. Constrained newtypes also need real authority: `src/typed_fields.rs:39-41` currently refuses constrained/reading types, and design lines 266–274 expressly require executable shared constraints. Browser replay remains truthful nonexecution (`src/web.rs:268-288`); actual browser/WASM adapter execution needs shared fixture evidence. A visualization or report translator cannot stand in for a tested runtime.

## Required evidence before implementation approval

The frozen design's acceptance remains binding: meaningful same-test red/green for redisclosure and all observer channels; log-only and delayed-log mutants; equal-permission actors; marked/unmarked alternative flow controls; malformed-origin and target-error redaction; exact-field/multiple-value/old-rotation controls; bounds and forged-policy admission; constrained-newtype vectors; legacy byte/pin controls; authored traces; and actual cross-runtime/path fixture counts and verdicts. None has been claimed executed by this review. No design section was left unread.
