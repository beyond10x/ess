---
format: aep.planning-md/1
id: review-result:error-wire-adversary-pass1
kind: review-result
status: active
title: Review error wire aliases and legacy preservation
relations:
- reviews: story:error-wire-codes
revision: 1
---
unit: gap9 implementation adversary1
verdict: green
cases: 0 new executions; bounded source and retained-evidence review
origin: n/a
wrote-outside-worktree: this assigned private review report only
needs-coordinator: yes — explicit diff/4 routing, schema/catalog integration and delivery
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 d1988bee46568230bf6f65ce2d2fda406afd754798759d8bfa8377359e5079ee, retained as local-evidence:runtime-gaps/publication-replay/snapshots/d1988bee46568230bf6f65ce2d2fda406afd754798759d8bfa8377359e5079ee.md. Source creation recorded at 2026-09-11T05:21:21Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 60c5c7494c3cf5afe882d2289957e48d04d2efc66f39ad89ec3326dd90c0fe75, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/60c5c7494c3cf5afe882d2289957e48d04d2efc66f39ad89ec3326dd90c0fe75-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

No confirmed product defect in the reviewed error-wire implementation. Review covered source admission, raw/admitted/resolved error identity, aliased and fallback transport codes, native Rust/Go escaping, payload schema identity and semantic delta admission. No production or test files were changed and no builds, tests, planning or Git writes were run.

| Surface | Independent assessment |
|---|---|
| Alias and fallback | `ResolvedError::wire_code` returns explicit `naming.wire` or the full qualified error name. This correctly avoids `Naming::wire_or`'s local-name fallback. Two distinct errors may return the same code while remaining separate map entries, error handles, outcome branches and payload types. No reverse lookup from an aliased code is invented. |
| Schema identity | `ess-gen/src/types.rs::MessageSchema::of_error` still keys the schema by `error.name`; Rust/Go error type declarations and error encoders likewise use the semantic name. A code alias is not used as a type, schema key, function identifier or field path. The stale explanatory comment is already a known root-owned documentation correction. |
| String escaping | Rust emits code literals with Rust string Debug formatting. Go now uses JSON string serialization rather than Rust escapes, which produces Go-compatible escapes for the tested NUL/quote/backslash values. Declared wire strings are values only; they do not enter executable identifiers or comments. The author accurately reports emitted-source witnesses, not generated-application execution. No runtime execution claim was inferred from those tests. |
| Source admission | Existing typed `Naming` is strict and the new field is omitted when empty. Nonempty error naming requires source/4 through the supplied `primitive_admission` patch. Legacy absent/empty metadata retains its previous bytes. The pinned reader's retained exit1 confirms unsupported new source is refused. Empty metadata has no behavior and does not require a new source capability. |
| Semantic changes | Wire comparison uses the same qualified fallback as emission; explicitly spelling the previous fallback produces no false wire change. Display and naming-summary changes are distinct from the pre-existing summary change. Subject identity remains the qualified error name. |
| Version integration | Candidate tests and shared patch use diff/3 under the earlier coordinator allocation. Root subsequently assigned new Error naming deltas to diff/4 with the response vocabulary, preserving the older diff/3 capability set. This is a known coordinator routing update, not an implementor defect. Integration must update the supplied minimum-format patch and expected versions together; do not publish these new Error variants as diff/3. |

The root must apply both source/4 routing and error admission, then the updated diff/4 minimum-format routing, alongside the source-only implementation. Without those shared patches the unit's verified admission guarantees are incomplete. Generated schema and public catalog remain coordinator-owned. No release, merged candidate, current consumer migration or successful conformance is claimed here.

Reviewed managed candidate: `wt-05ba20e2a7ca`, base `1fd6ba62c497c1c0fd4354745e2f60e26859c235`. Source-only patch SHA256 `ab2803535fe425c6afcbf1873dbe70263a6600303882210596e8d31ceea8c3a6`; source admission patch `9e3c46728a13ebca53fafdaa7e0c116b18f7f60aa5bb75569a72c7c763ad47bd`; earlier diff/3 patch `f7546b9736ba18f7b63a8b91b753e7e1e43af566c69c515e87ffb22622928a23`. The author report retains 37 distinct passing cases and scoped lint evidence; this review did not rerun them. Findings: none beyond the explicitly known integration obligations above.

```findings
[]
```