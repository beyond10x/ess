---
format: aep.planning-md/3
id: story:nested-response-observations
kind: story
status: active
title: Observe nested response mappings across every conformance runtime
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/tests/nested_response_wasm.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/response.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/response.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/response/path.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/response.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/nested_response_observations.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_payload.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/runtime_parity_go_28_35.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/runtime_parity_typescript_28_35.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/support_nested_response/mod.rs
- confidence: inferred
  path: docs/design/nested-response-observations.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T01:32:21Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-03T01:32:21Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
---
## Outcome

Every source-admitted nested ResponseField destination has a synthesized same-invocation relationship observation with the same healthy/fault verdict on native, actual generated Go, TypeScript and WASM. Browser product execution remains separately required under browser-response-conformance.

## Fit review

1. Need: event.packet.value mapped from response.value currently passes when the two actual values differ. Existing source grammar admits Struct/ResponseField combinations; no new authored syntax is needed. Brand-free probe model08abfdf512a339ef4470636addb97fed52fc13c4c946d9469a8e237fbb229875 establishes the need independently of a consumer repository.
2. Class: conformance coverage defect. response.rs::Observation::of collects only immediate ResponseField mappings, while the compiler resolves nested ones. Shape-valid unequal values evade every synthesized check.
3. Existing expression: ess/14 source compiles, synthesizes1scenario with0refusals/0notes. Real Runner healthy37/37, wrong-event37/38 and wrong-response38/37 all pass3checks; receipt=null independently fails ESS-CF-PAYLOAD. Both probe runs exit101 at NESTED_RESPONSE_RELATIONSHIP_GAP. This is executed evidence, not only source inference.
4. Composition: reuse exact invocation/outcome/event association, existing response value/presence validation and whole-value equality. Add explicit destination segment arrays and separate closed structural authority; preserve ordinary generated/input/literal siblings. Native/Go/TypeScript/WASM parity is required. Browser execution and broader unsupported value profiles remain tracked gaps, not successful cells.
5. Generality: nested invoice receipt fields and nested account registration results both require agreement with the actual response independently of one consumer's names.
6. Cost: amend held unreleased suite34/35 authority; no source/report version or authored key. Seven production files, focused shared Rust-driven fixtures, closed old-reader refusal and exact common byte accounting. Preserve all nested-less historical bytes and feature gates.
7. Alternatives: change nothing leaves a measured false-green; dotted keys reinterpret old authority; passing complete roots to the current response-value validator adds unrelated sibling refusals; asserted path edges do not independently establish declared traversal. Choose complete structural root declarations with checked paths and a dedicated structural-only validator.

## Decisions

Accept, redesigned under the operator's standing full-backlog/all-features authorization. Binding design docs/design/nested-response-observations.md SHA28cf71a68e09bfa8cd8d59bf0b558cf4f4e26b2e983a1912055c003b07e9ecbb supersedes earlier proposal details. The optional closed nested block owns name/type-only structural roots/members, kind-specific declarations and checked segment mappings. Reject extra metadata before projection. Keep response-value authority separate and compare overlapping declarations by representation.

Amend held unreleased suite34/35; raw nested-key presence under older majors refuses before callbacks, including null/empty values. Preserve nested-less bytes and historical behavior. A frozen real pre-amendment reader must reject the added unknown field under34/35. No new suite pair, report version, component PR or independent source21 release.

For nested observations use the explicitly specified common compact UTF-8 whole-observation byte profile across every runtime, with kind-specific member order, UTF-8 lexical map ordering, active-member omission, short/control escaping, no HTML/U+2028 escaping, and response-field presence. Do not reuse incompatible legacy Go marshal or TypeScript marshalShape. Legacy nested-less accounting stays unchanged.

Limits:2..33segments,256totalrelationships,4096unique nominal names across both schemas,1MiB whole canonical observation. Exact structural closure, duplicate/prefix/conflict rejection, nominal terminal assignability, cycle-safe/memoized traversal and source wrapper grammar apply. Unknown/malformed intermediate runtime ancestors fail; only terminal Optional absence/null follows existing policy. Never use expectations as target response authority.

## Acceptance

Preserve measured probe healthy and two independent mismatch controls plus malformed generated sibling. Run actual native, generated Go, TypeScript and WASM on shared authority and compare verdicts/check codes/callback counts. Cover mixed flat/nested leaves, repeated sources, multiple roots, paths beyond3segments, Optional/newtype ancestors, absent/null/scalar ancestors, terminal presence, complete aggregate leaves, exact integers and stale invocation/mutable result controls.

Reject forged root/member/source/type/declaration/metadata, duplicate/prefix/overlapping paths, unsupported intermediate collection/union traversal, conversion on a leaf or ancestor, null/empty authority, unknown/duplicate original keys and old-major authority before any target callback. Pin exact path, relationship, declaration and canonical-byte boundaries, including declaration overlap, graph sharing/cycles and admitted string escaping. Preserve original coverage parents, provenance, selected identity and report counts. Source32-ancestor admitted/33-ancestor refused fixtures establish path off-by-one.

Structural-only Binary64/non-String-map siblings must not create a new response-profile refusal. Existing whole-model Binary64 or actual response-value limitations remain explicit required backlog gaps; do not report their refusal as support. Full browser execution remains separately required.

## Scope and delivery

Seven production files: response.rs, new response/path.rs, admission.rs, go/response.go, go/runtime.go, ts/response.ts and ts/runtime.ts under crates/verify/ess-conformance/src. Shared focused Rust tests and actual generated/WASM harnesses as recorded in machine-readable scope. No interpreter, caller synthesize.rs, transport or shared-branch edits. Root owns AEP, binding design and integration. Implementation uses an isolated managed tree and exclusive synthesis cache, jobs1/debug0/incremental0/external TMPDIR with8GiB floor. New executable code is Rust; existing Go/TypeScript embedded-runtime assets retain the repository exception. Freeze exact candidate for independent implementation review before bot commit/import. All delivery remains in the ONE held bundle.

## Evidence

Probe report dab907a603c09b09ccc609eafca4f7c0d421c15a529314fcadd865dde9099c36 and evidence manifest d3e18af47b894327cf8b0be4390a81f97b87324c674433e10e7051472c7af245 retain original source, suite, reports and terminal101 runs in private ess-nested-response-probe-20261003. Suite39380b6a20ad97e89587879d08280c8b7bcd51c932cacd6e060dd968c868e0b9; healthy/wrong-value reports byte-identical46abcbfcc10041229ffd646bed23998a3a36aac6c58308a3483103e080a30c44. Actual generated-sibling fault disproves an inert checker.

Proposal327839ef111a6a7ee7f4f94e8ea65ba5ea8baa305c5086dd19b9f6cde6b27474 is retained as historical input. Independent design pass1 found two parity blockers, both corrected in binding document. review-result:consumer-nested-response-design-pass2 approves exact28cf71... with findings[] and reviewer executions0. Design approval is not implementation or release proof. Native response integration046db8a680 passed192 focused tests but does not fix this observer gap.
