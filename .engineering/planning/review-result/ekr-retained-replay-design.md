---
format: aep.planning-md/1
id: review-result:ekr-retained-replay-design
kind: review-result
status: active
title: Independent retained-result contract review
relations:
- reviews: story:retained-command-result-replay
revision: 1
---
# Independent review of retained command results

Owners: coordinator — design and decisions; ESS implementor — source and deciding executions; independent reviewer — this read-only finding record.

Verdict: NEEDS-CHANGE on one bounded comparison contract before source dispatch. The command-local relation and selection/effect split are suitable for the EKR adopter; they do not require a generic history query language.

Reviewed ESS source: `16aa8c7617214420d7d7f2108d0a896a5ed14eb0`. Reviewed proposal: `docs/design/retained-command-results.md`, dated 2026-09-22, and `.engineering/waves/ekr-retained-replay-scope.md`. Reviewed EKR accepted requirements: `.engineering/waves/p1-durable-record-decisions.md` and the retained activation/conformance preparation. This pass read source only: no build, compiler invocation, runtime probe, source edit, AEP edit, store access or publication. The numerical consequence below is static inference from the two existing implementations, not a claimed executed failure.

```yaml
- id: ESS-REPLAY-DESIGN-1
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  title: Choose one admitted value and equality contract for retained responses
  contract:
    - docs/design/retained-command-results.md:83
    - docs/design/retained-command-results.md:90
    - docs/design/retained-command-results.md:105
  evidence:
    - crates/specify/ess-primitives/src/node.rs:53
    - crates/verify/ess-conformance/src/response.rs:252
    - crates/verify/ess-conformance/src/response.rs:299
    - crates/verify/ess-conformance/src/go/response.go:295
    - crates/verify/ess-conformance/src/go/response.go:342
  basis: Static inference; no execution in this review.
  consequence: Existing typed response admission allows Decimal, but Rust can erase a decimal difference that Go retains. The proposed exact cross-runner equality is therefore not determined by saying to reuse existing rules.
  correction: Explicitly bound the new replay response contract and define its structural equality before implementation. For this EKR unit, refuse Decimal and Binary64 transitively with a named refusal; preserve old response behavior. Alternatively authorize a common lossless numeric representation and its parity cases explicitly.
- id: ESS-REPLAY-DESIGN-2
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  title: An immediate retry does not distinguish own result from an equal current head
  contract:
    - docs/design/retained-command-results.md:75
    - docs/design/retained-command-results.md:95
    - docs/design/retained-command-results.md:140
  evidence:
    - .engineering/waves/ekr-retained-replay-scope.md:110
    - EKR:.engineering/waves/p1-durable-record-decisions.md:39
    - EKR:.engineering/waves/p1-durable-record-decisions.md:119
  basis: Static observation of the prescribed witness sequence; no execution in this review.
  consequence: Immediately after origin success the current head can equal the original result, so a current-head implementation can pass the generated witness. The design already delegates later-head acceptance to the adopter, but its earlier universal failure claim contradicts that boundary.
  correction: Qualify the generated-witness claim to differing observed results. Keep the explicit later-head real-handler adopter case, with two distinguishable results and retry of the first identity. Likewise, a timestamp resample that returns the same timestamp cannot be detected by value comparison alone.
```

## 1. Deciding comparison correction

The proposed rule compares the complete typed origin response with the replay response (design lines 83–91). The current response observer refuses Binary64 but admits other primitive types, including Decimal (`response.rs:243–255`). Rust's `Node` decodes noninteger JSON numbers through `as_f64` and `Number::new` (`ess-primitives/src/node.rs:53–63`); its existing response comparison uses `Node` equality (`response.rs:299–303`). Go's snapshot preserves JSON number tokens with `UseNumber` (`go/response.go:277–292`) and compares them as exact rational numbers (`295–316`, `342–347`). Thus the responses `{value: 0.1}` and `{value: 0.10000000000000001}` can become the same binary64 value in Rust while remaining different exact rationals in Go. Merely reusing both helpers does not select one meaning. This is a pre-existing helper distinction exposed by the new promised guarantee, not a request to redesign old suite behavior.

Smallest recommended contract for this initial replay feature:

- Recursively admit the existing closed response types except Decimal and Binary64. Reject either through named types, optionals, lists, maps, structs or union arms. This does not remove Decimal from legacy response/event assertions. EKR's retained Seed/Commit result projection needs identities, hashes, integers, timestamps and structured containers, so no generic decimal upgrade is needed to unblock it.
- Integer means the existing signed-64-bit admitted value, compared exactly, including values beyond binary64's exact range. Use `9007199254740992` versus `9007199254740993` as an unequal pair and include the signed bounds. Do not pass Integer through a float or a lossy JSON round trip.
- Compare the complete admitted tree: exact field/key sets, ordered list elements and union tag, then recursive values. Strings, including the existing Timestamp representation, compare as strings. UUID and Bytes retain existing canonical admission. Map iteration order is immaterial.
- Decide optional omission versus explicit null once. My recommendation is to preserve the captured response's field-presence distinction for this **complete result** assertion: omitted then null fails. Existing event mapping deliberately treats them alike (`response.rs:292–297`); that event comparison is not an authority for silently normalizing a retained result. This is a bounded new-step rule and does not change old steps.
- Snapshot an actual successful invocation's complete response before any later target callback can mutate it. Bind the snapshot to its command, origin outcome, original subject identity and local invocation; reject duplicate keys, wrong references, overwrite and missing captures. The synthesized retry should reuse the original resolved input and actor, as the scoping report already says at line 97. That test recipe does **not** redefine EKR's semantic seed matching or make a raw document path an admission authority.

The implementation can then prove parity with a small shared vector set rather than guessing equality separately in the Rust and Go runners. A broader Decimal capability is a legitimate alternative, but it needs its own explicit lossless numeric contract; this review does not authorize it.

## 2. Selection and effects are coherently separated

The current local shape check requires every ordinary state branch to own an input-bound effect subject (`command/subject_state.rs:111–125`), while refusal validation forbids effect-bearing errors (`command.rs:1431–1457`). A compiler-owned common selection subject for the unique default error resolves that actual contradiction without inventing a preserving effect. The proposed finite-complement bound is appropriate for the new admission; preserve legacy open-default behavior, whose existing fallback is at `subject_state.rs:223–249`.

The synthesis change is substantive. `prepare_state_input` currently reads `outcome.subject` and returns the first successful state witness (`synthesize.rs:2206–2244`). A subjectless refusal therefore needs explicit common-selection arrangement, and Proposed/Rejected/Stale need separate observed controls. Do not merely admit the declaration while retaining a first-default-only witness. `observe_subject_state` also currently takes the selected effect subject (`2255–2277`); the new selection authority must reach both arrangement and observation. The design requires those corrections explicitly.

## 3. Create and move replay use the right independent identity

The current model derives create identity from an emitted event and move identity from command input (`command.rs:661–686`). The proposed replay reuses that origin authority and prohibits a second subject/instance declaration. This is suitable for both EKR Commit and document-only Seed: capture Seed's original emitted revision identity before the retry, and select that retained row afterward. Reading identity from the retry's response or current head would let the implementation choose its own comparison subject.

A replay's external eligibility text must select the dedicated replay strategy. Current synthesis emits `ConfigureExternalOutcome` whenever `TestStrategy::InjectFault` is selected (`synthesize.rs:1408–1414`), so the new resolved strategy must bypass that route deliberately. The design's no-fault-injection sequence and prohibition on target-side expected-response caching are concrete and sufficient. EKR's parsed Seed input, trusted context and authority-anchor comparison remain kernel responsibilities; the document path alone is not equality authority (design lines 68–71; EKR durable decisions lines 23–28).

Complete observation remains an adoption prerequisite, not a reason to invent a new CLI verb. Existing preservation synthesis requires immediate identity views covering every declared field and state (`synthesize/subject_fact.rs:362–423`). The report correctly identifies that a current-head-only Revision view is insufficient for the older captured identity. The proposed result snapshot supplements subject snapshots rather than replacing them.

## 4. Native outputs, formats and unsupported readers

The source confirms the stated format availability: ESS through 6 (`ess-domain/src/system.rs:53`), suites through 11 (`ess-conformance/src/scenario.rs:366`) and semantic diff through 5 (`ess-diff/src/delta.rs:13`). Allocations ess/7, suites/12 and /13, and ess-diff/6 are free at this source. No ess-ir/2 is needed. Optional resolved fields must be omitted for legacy inputs so the proposed old-byte fixtures remain meaningful.

Native response generation is currently event-mapping dependent (`ess-synth/src/rust/items.rs:420–426`); expanding that specifically to replay and its referenced origin is required even when neither maps a response to an event. The existing runner's `expect_response_payload` explicitly requires an event from that same invocation (`runner.rs:1197–1215`), so a new silent-result assertion must not pretend that step already covers it.

The design correctly requires TypeScript/browser readers to implement the new vocabulary or reject the envelope before callbacks. Preserve the separate negative case of new replay steps mislabeled as an old envelope; a version list alone does not prove that refusal. Likewise, existing `SnapshotSubject` is keyed by view and overwrites on recapture (`runner.rs:1836–1844`, `1878–1881`); the new immutable result-snapshot requirement must have its own admission/runtime check rather than inheriting that behavior accidentally. This is an implementation control already implied by the design, not an additional design blocker.

## Dispatch conclusion

Resolve the new comparison contract and qualify the immediate-witness claim, then dispatch the scoped source unit. No other design blocker was identified in this bounded pass. The execution gate must still demonstrate both native runners, per-state refusal controls, complete original-identity observation, snapshot integrity and old/new reader behavior; this static review establishes none of those runtime results.

Coordinator disposition, 2026-09-22: both findings accepted. The coordinator verified that EKR SeedResultV1 and CommitReceiptV1 transitively need no Decimal/Binary64 and selected the bounded refusal, exact i64/string/container comparison and presence-sensitive optional rule above, with Rust/Go parity vectors. The later-head claim will be qualified as recommended. This records the decision delivered to the reviewer; the coordinator will apply the design amendment separately. No source or normative repository document was edited by this review.
