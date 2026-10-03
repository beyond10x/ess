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
- confidence: cited
  path: crates/verify/ess-conformance/tests/support_nested_response/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/support_nested_response/foreign.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/support_nested_response/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/support_nested_response/values.rs
- confidence: inferred
  path: docs/design/nested-response-observations.md
revision: 14
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

## Frozen reader and repository red, 2026-10-03

Implementation owner preserved an actual pre-amendment reader binary linked against runtime046db8a680 before production changes. Binary SHA256663b007524dba832bae388425bdf59bca6ca414c5ea5c3e524065a6ad05a23ac; source/dependency diff against that checkpoint is empty. It admits original healthy suite39380b6a20ad97e89587879d08280c8b7bcd51c932cacd6e060dd968c868e0b9 under suite34 with1scenario, exit0. Exact source/lock/build log/binary/hash manifest are privately retained under ess-nested-response-probe-20261003/frozen-reader. The frozen copy will not be rebuilt after authority changes. No new-envelope rejection has been claimed before a real generated nested suite exists.

In managed ess-nested-response-observations-20261003, repository test binary nested_response_observations now runs3tests against unchanged production:1passes (healthy and generated-sibling control),2fail independently because wrong response and wrong event each return Passed. Terminal101; target/backlog-input/nested-response-red.log and exit retained. Production implementation started only after that observed red. Owner branch fix/nested-response-observations-20261003 begins at5c5aeaf795, exclusive synthesis cache and live lease. No full workspace build or remote gate was started. All four runtime matrices and independent implementation review remain pending.

## Disk coordination and frozen nested candidate

On the integrator's disk coordination request, root inspected only its own completed outputs and removed 21 exact reviewed compiler-cache directories from the runtime carrier. First 15 directories total 1.05 GiB and six additional old fixture target directories bring the measured apparent-byte total to 1.24 GiB. No managed tree, source, fixture source, logs, evidence, frozen reader, active servers/synthesis cache or another owner's output was removed. Exact path/byte/process-check manifests remain in carrier target/backlog-input/disposable-cleanup-20261003.* and disposable-cleanup-2.*. Both original frozen handoff hashes verify unchanged after cleanup. Shared available space continues changing with other sessions; root reports only observed availability, not exclusive headroom.

Own new compiler starts require at least 12 GiB while release398 verifies; existing work is not killed.293 actual repository regression red completed0passed/2failed with terminal101 before production edits; implementation continues without builds during the resource hold.292 has prepared Rust regressions only and waits for a cache. Those source trees and evidence remain retained.

Nested response owner freezes13source/test files against5c5aeaf795 as patch9e87ddaef39cd06666bd962763e18a8fe633d5f037a2fb45c4058a16d23631be. Updated owner report8c622ff51baa2fa9dc4a420a342cdb1405197cc2f4ce531603cea72660cfd689 records actual native/Go/strict-TypeScript matrices, old-reader rejection and earlier actualWASM success. Exact frozen-source final all-target ess-conformance/ess-synth Clippy exits0, log7b9d847b9d7735c1b7ca93736b79c2a1091fec72292c31dfb4cacbdd10a17204. Final post-structural-correction warm WASM remains queued until disk threshold permits. This is owner evidence, not an independent rerun or approval. Root source inspection and independent adversary review are underway; no nested commit/import/publication yet. Added source scopes are path.rs plus support_nested_response/{admission,foreign,values}.rs and the originally planned WASM test; TypeScript runtime delta only wraps an existing diagnostic call for its style gate.

## Independent frozen-source review

Independent reviewer approves the exact frozen 13-file patch 9e87ddaef39cd06666bd962763e18a8fe633d5f037a2fb45c4058a16d23631be with no findings. Immutable report SHA256 791952b2d62dd4c59387eace5e957c303126a47ff7177675c25a7c3997489261 is recorded as review-result:consumer-nested-response-implementation-pass1. Review executions are zero cargo builds and zero generated-runtime/test-binary executions. One isolated installed-Node regex check disproved a proposed terminal-newline admission mismatch; that hypothesis was withdrawn and its unexecuted Rust test remains only in private scratch. No reviewer test or production edits remain in the source tree.

The report cited the earlier owner report d081106ee23776fbc929d7461739b24c0aeb74d8c049e541af1ff359c0963c14; root separately reconciles the owner's final report 8c622ff51baa2fa9dc4a420a342cdb1405197cc2f4ce531603cea72660cfd689 against the same unchanged frozen source. Strict all-target Clippy and repository formatter succeeded. The final post-correction WASM recheck remains queued because shared available disk is below the temporary 12 GiB build-start floor. Prior WASM success is retained, not mislabelled as this final rerun. Source approval does not remove that remaining verification requirement. No commit, integration, release or browser-product completion is claimed.
