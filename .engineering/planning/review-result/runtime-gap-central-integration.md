---
format: aep.planning-md/1
id: review-result:runtime-gap-central-integration
kind: review-result
status: active
title: Review shared response and predicate format integration
relations:
- reviews: release-plan:runtime-gaps-wave-20260911
revision: 1
---
unit: independent central integration review of runtime gaps 9/10/13 and selection correction
verdict: red — one central typed response admission omission
cases: 0 executed; source-only inspection
origin: n/a
wrote-outside-worktree: local-evidence:runtime-gaps/integration-static-review.md only
needs-coordinator: yes — repair typed response admission and execute the focused regression
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 011264f8e0c0319a005b678113c01d6c485e76ea25fdf6b91382f95b0042ed30, retained as local-evidence:runtime-gaps/publication-replay/snapshots/011264f8e0c0319a005b678113c01d6c485e76ea25fdf6b91382f95b0042ed30.md. Source creation recorded at 2026-09-11T05:53:57Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 5b467b8f9a29fdfb676e8da3e202199e1da2a8417c971c1f0668ebdab446b82c, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/5b467b8f9a29fdfb676e8da3e202199e1da2a8417c971c1f0668ebdab446b82c-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

Reviewed the stable integration source in managed tree `wt-643e4be649fc`, without builds, tests, edits or re-review of owned predicate internals. The finding was reported and acknowledged before correction. Candidate admission.rs SHA256 `635823a397fa48fa4887f88161274172991bf65b23186d06d1fbedd946a4a2f2`; scenario.rs SHA256 `1de685b6165661af0d57385322a97bdcf650865b11d3a4c2b34bb0204f0117b7`; Go runtime.go SHA256 `8798fd1a69f2e5989a34f1551bf6cd671b822730af7e81ad09bd64f96256feeb`.

| ID | Priority | Finding | Required correction |
| --- | --- | --- | --- |
| INT-RESPONSE-1 | P2 | `admission::suite` has no `ExpectResponsePayload` branch. A directly constructed typed suite pinned below major 8 therefore passes this entry even with a response step. Both direct JSON writers call this entry, so they can emit a document that the raw reader subsequently rejects. A public directly constructed `response::Observation` can also bypass its `validate()` through this writer path; its serde `try_from` validation does not protect direct construction. | In central typed suite admission, refuse response steps below major 8 and validate each response observation before serialization/target effects. Add the bounded direct-construction regression for pinned old major and malformed observation under an admitted major, alongside a valid response case. |

Exact trace: `crates/verify/ess-conformance/src/scenario.rs:218` and `:233` call `admission::suite` before pretty/compact serialization. `src/admission.rs:424–523` checks quoted predicates, setup, periodic, reading, selection/accessor and legacy Binary64 shapes but never response steps. In contrast, raw `step_value` at `src/admission.rs:351–362` refuses `expect_response_payload` below major 8, and `src/response.rs:14–58` validates only the serde conversion path. This is a concrete static omission; no runtime counterexample was executed by this review.

Remaining requested integration checks:

- Source format 4 is registered in `ess-domain/src/system.rs`. `Specification::validate` enters `primitive_admission::specification`; that function calls `validate_response_contracts` at line 84 and refuses nonempty error naming below source 4 at lines 149–156. Response completeness and response/generated source admission are reachable through this hook; the absence of direct new calls in spec.rs is not an omission.
- Both ordinary `scenario.rs:156` and coverage `coverage_build.rs:473` fresh selectors combine `response::used_by` OR `quoted_predicate_format::used_by`, selecting /8 and /9 respectively. Existing extended capabilities retain the prior fallback. The shared supported array and raw admission admit 1–9, with coverage required exactly for 5/7/9.
- Raw Rust/Go response old-major refusal survives. The central Rust original predicate hooks remain in selection and view-expectation admission, and the typed quoted guard remains the first `admission::suite` call. Go calls the predicate-version guard in view expectations and first selection predicates. Browser applies its guard only at admitted view predicate positions and still refuses unsupported response/setup vocabulary. Predicate internals were not re-reviewed.
- Rust coverage parent validation includes /9 and requires the parent's version to equal the selected suite's version. Go accepts /9 references, derives actual provenance version in `referenceFor`, and retains 5/7/9 coverage-presence/reader branches. Browser admits its limited 5/9 subset and derives the actual parent/reference version, rather than substituting 5. This is not a claim that the browser implements all /9 vocabulary.
- New error wire/display/naming-summary changes and command response/response-payload changes have minimum ess-diff/4 in `ess-diff/src/change.rs:343`. The supported delta array includes 4; diff generation emits those specific variants for changed metadata/contracts. Existing unrelated changes preserve earlier minimum versions.
- The combined Go primitive branch at runtime.go:6464 gives `responseMode` its dedicated `responsePrimitiveAdmits` check, then checks finite numeric Decimal/Binary64 for ordinary selection, then uses the existing accessor primitive validator. `asNumber` does not accept text. Response-only closed-record/map handling, the separate response observation admission and execution dispatch survive integration.
- Go explicit report/2 refusal remains present for both 8/9 and earlier coverage/extended versions. The two adjacent checks are redundant in structure but preserve their intended behavior; no semantic defect was found there.

The bounded central inspection is complete. Root owns the acknowledged correction, its first focused verification and final integrated generation/schema checks. No additional review panel or broad gate is requested.

```yaml
findings:
  - id: INT-RESPONSE-1
    priority: P2
    file: crates/verify/ess-conformance/src/admission.rs
    line: 424
    title: Direct typed suite admission omits response version and observation validation
    status: unresolved_at_review
    evidence: Static writer-to-admission trace; no execution
```

```findings
- file: crates/verify/ess-conformance/src/admission.rs
  line: 424
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Direct typed suite writers do not enforce response observation validity or the suite8 minimum before serialization.
```