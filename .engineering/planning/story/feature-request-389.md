---
format: aep.planning-md/3
id: story:feature-request-389
kind: story
status: active
title: One-time response disclosure is declared and checked
tags:
- fast-lane
refs:
- provider: github
  reference: beyond10x/ess#389
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-gen
- confidence: cited
  path: crates/generate/ess-openapi
- confidence: cited
  path: crates/generate/ess-synth
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-diff
- confidence: cited
  path: docs/design/one-time-response-values.md
- confidence: cited
  path: schemas/generated
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T13:57:12Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"approval":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T13:57:12Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"approval":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Make one-time response disclosure an explicit contract with validation and executable redisclosure checks. The operator placed389 on the fast lane on2026-10-02.

## Consumer request

## Problem

ESS can declare a typed command response, but cannot declare that a particular generated value may be disclosed only by its originating successful invocation and must never be returned again.

A generic example is issuing or rotating an API credential: return the fresh plaintext secret to the caller once, retain only a verifier, and provide no later operation that reveals that same plaintext value. Each successful rotation may disclose a new secret once; it must not disclose the previous one.

Leaving the secret out of declared entity fields, events and views expresses part of the intended design, but does not make one-time disclosure an explicit contract or generate checks against subsequent disclosure.

## Current evidence

Checked ESS 0.51.0 and the current main source schema:

- `RawCommandSpec.response` uses the shared `Field` definition, which has no one-time disclosure property and disallows additional properties.
- ESS 0.39.0 introduced direct returns (`returns: true`, source `ess/17`) and authored response assertions. These support observing a returned value, but do not declare or check its one-time disclosure.

A response declaration such as this can describe the value's type:

```yaml
response:
  - {name: secret, type: String}
```

It cannot express the additional temporal and non-disclosure constraint. No matching issue was found in the existing issue search.

## Requested capability

Introduce a supported way to declare a one-time response value, with validation and conformance support. For illustration only, a possible syntax would be:

```yaml
response:
  - {name: secret, type: String, once: true}
```

The syntax is a proposal, not a requirement. The contract should specify:

1. Which successful outcome originates and returns the value.
2. That the plaintext value cannot flow into declared persistent fields, events, views or other responses; retaining a derived verifier is a separate permitted operation.
3. What subsequent calls, retries and concurrent requests may return. In particular, define the interaction with retained-result `replays:` and what happens after a lost response.
4. How rotation starts a new one-time disclosure without exposing an earlier value.

## Validation and conformance expectations

- Reject declared flows that contradict the disclosure restriction, or report unsupported analysis explicitly.
- Capture the originating response and check applicable subsequent declared reads, commands and retries for redisclosure, including relevant actor changes and rotation.
- Include a deliberately faulty target that returns the original secret again, so the generated checks demonstrably catch the violation.
- State the coverage boundary: finite black-box scenarios do not prove absence of plaintext in implementation storage, logs, undeclared endpoints, or every possible future execution. Identify implementation-owned checks and any unsupported restart/concurrency cases rather than implying a complete proof.

The request is for an explicit disclosure contract and meaningful executable checks, not a claim that `{generated: true}` proves secure generation or hashing.


## Delivery decision

Prioritize immediately. Bind the detailed design before implementation, including at-most-once semantics after lost responses, rejected full-result replay, fresh rotation, forbidden declared flows, named unsupported cases and finite observation limits. Use the next source major ess/21 for this fast-lane contract; the previously planned, unshipped coordinated syntax bundle moves together to ess/22. This changes delivery order and version allocation, not that bundle's accepted behavior. Older acceptance/review records remain historical.

## Fit review

Existing shared Field has no disclosure property; direct-response assertions observe only the immediately preceding result. Retained-result replay promises complete equality and therefore conflicts with one-time plaintext. A narrowly typed outcome annotation reuses the existing response declaration and identifies the successful origin. Source/IR/suite versioning and every projection/runtime must preserve or explicitly refuse the new meaning. Binding design and independent review are in preparation.

## Mandatory runtime parity, operator correction 2026-10-02

The operator explicitly requires every supported conformance runtime to support every admitted feature. For389, native Rust, Go, TypeScript and any browser/WASM conformance path must execute the complete new contract before release. The earlier proposed Go/TypeScript refusal-only first delivery is rejected and superseded.

Use shared healthy and adversarial fixtures to verify capture, retries, fresh rotation, actor changes, recursive leak checks including keys, authored timelines, admission, bounds, value-free diagnostics and verdict/count equivalence. Inventory all supported execution surfaces and test each; do not assume a shared library proves an unexecuted integration path. Intrinsic finite observation limits, including unavailable restart/concurrency primitives, remain explicit and identical across runtimes. A target's inability to provide an observation is distinct from a runtime omitting the newly admitted vocabulary, and must never become a passing check.

Full runtime parity is a release acceptance gate. Prioritization changes order, not completeness.

## Accepted design and execution scope

The coordinator accepts docs/design/one-time-response-values.md at SHA256 fbc81cf73e2b0bc11a2ab4258137d2a4121fed669892e81e8e602c02b827de3c and example c9b42b8d1bee19683508589fe9fec7900cdab25267ab7592af417f7b22aa3273 after independent review by scope_aggregate (approve, no unresolved findings; own executions 0). The example is proposed source21 syntax, not yet validated by an implementation.

Implementation is authorized under the operator's fast-lane instruction and explicit complete-runtime mandate. This includes closing Go/TypeScript prerequisite gaps for suites28/29 (direct responses),30/31 (delivery context),32/33 (structured values), followed by34/35. No language-specific refusal-only release. Browser visualization is not execution evidence; actual WASM adapters receive execution controls. Shared serialized suites, scenario IDs, callback traces, counts, fixed codes and redaction are acceptance requirements.

The frozen design is binding acceptance, including every applicable actor, outcome-local flow ownership, independent event logs observed through their finite deadlines, constrained newtype authority, exact-field exemption, all prior values after rotation, authored timelines, admission before callbacks, bounded capture, and value-free diagnostics. Unsupported intrinsic restart/concurrency obligations remain explicit and equal; they do not excuse a missing runtime port.

Work proceeds in dependent stages within this story: closed typed source/IR/suite/admission and immutable shared vectors first, then independent Go/TypeScript ports from that exact contract commit, native execution and integration. Each stage retains red/green evidence and receives independent review; the story stays active until complete validation and integration. Source21/diff13/suite34/35 allocation and source22 reallocation remain as already recorded.

## Prerequisite port sequencing

Existing suite28–33 typed contracts are already frozen in main1ff305685. Start independent prerequisite ports from this identical committed native authority while the new389 contract is being implemented; their exact feature semantics and invalid-document rules remain unchanged. Go owns src/go/** and its new Rust parity integration binary; TypeScript owns src/ts/** and its own new binary. Shared admission API edits remain with the Go owner. Workers cannot change shared native DTOs, fixtures or expected meanings. The new389 ports still wait for the independently reviewed source21/suite34/35 contract commit and immutable vectors. This parallelizes implementation of existing contracts without guessing new policy.

Include existing runtime report-status parity: native unsupported target capability and ordinary target errors must have identical unsupported/error status and counts in generated runners. Preserve explicitly skipped cases as skipped. Test this with actual callbacks and serialized report/2; do not equate skipped or failed with another terminal status.

## Shared acceptance gates reproduce runtime omissions

Shared gates now require every supported suite major and every native scenario step/value/expectation in TypeScript execution. The old28–33 exclusions and named missing-step allowance are removed. Native Unsupported is compared as unsupported, not rewritten to skipped. Existing three feature-generation refusals are replaced by successful-generation requirements; separate port tests must establish real execution.

Before any port integration, the unchanged-runtime controls are red: runtime_suite_admission plus typescript_suite_versions filtered every_ execute0 passed/3 failed/0 ignored, exit101. The failures identify Go and TS suite28–33 refusals and all five missing TypeScript tags. Feature emission filtered preserve executes2 passed/3 failed/0 ignored, exit101, naming the three current emitter refusals. Raw logs: runtime-parity-gates-red.log and runtime-feature-emission-red.log in ess-backlog-next-20261002/target/backlog-input, each with an exit file. task fmt-check and git diff --check pass. These are intentional failing acceptance controls, not finished implementation or release evidence.

## Interpreted target prerequisites

The full runtime mandate exposed generic Interpreted target gaps already owned by existing stories: interpreted-eventual-views, interpreted-bindings-and-unmet-obligations, interpreted-scenario-supplied-facts and interpreted-trust-gate (all draftrev3 at audit). Do not duplicate their work in new intake or mark them delivered from a native Runner test using another target. Their retained acceptance compares actual billing/oracle scenarios and the fault matrix, including eventual lag and unsupplied external obligations.

Issue389's accepted native stage must exercise its actual interpreted issuance/rotation/read/event path and account for these dependencies. The existing product target's missing implementation is not an intrinsic model impossibility or a consumer adapter limitation. These stories remain unfinished until their named acceptance executes; the parity audit is not passing evidence. Exploratory model-subset gaps remain separately owned by accepted221/223 and the recorded explorer inventory, not silently counted as conformance support.

## Versioned generated report status semantics

The existing go-scenario-status/1 producer profile explicitly forbids error and unsupported categories; TypeScript also emits that profile. Correcting terminal classifications under the same profile would silently change persisted meaning. Allocate go-scenario-status/2 for both generated runtimes, carrying all five existing report/2 categories: passed, failed, error, unsupported and skipped. Keep /1 parsing and its three-category restriction unchanged; unknown profiles refuse. The report/2 envelope already carries these five categories and an explicit versioned producer_profile, so its envelope stays /2. Native rust-scenario-status/1 remains unchanged. Skipped is never substituted for an unsupported observation, and target execution errors never become assertion failures.

Independent bounded decision review by scope_aggregate: coherent, no contradiction; older readers must refuse the new profile. Root owns shared counts.rs, count reader regression tests and documentation/CLI producer expectations. Go/TypeScript workers emit the new profile and execute actual status parity controls. Test new-profile round trips, legacy profile restrictions and bytes, exact counts/outcomes, unknown profile rejection and inconclusive/failing execution status. This is a dependent acceptance contract within389, not a claim that runtime ports are complete.

## Shared profile reader verification

Shared producer-profile reader committed as30e9e84e7891f793c0c89d4695acf2fac88bf28e. New-profile regression tests against the prior reader:0passed2failed0ignored, exit101. Final unchanged tests plus existing count-report controls:11passed0failed0ignored, exit0. Scoped strict Clippy across conformance/CLI libraries, ess binary and count_reports target passed; task fmt-check and git diff --check passed. Logs and exit files are generated-profile-v2-{red,final,clippy,fmt} in ess-backlog-next-20261002/target/backlog-input. Independent source review by scope_boolean approved with no findings and own executions0. Generated Go/TypeScript actual producer parity is still the port workers' dependent acceptance, not established by these reader tests alone.

## Runtime review corrections

Independent coordinator review, 2026-10-02. Own build/test executions for these review findings: 0; findings are source inspections, with deciding controls assigned to implementors.

The Go prerequisite port incorrectly used aggregate scenario Failed status to decide whether an assertion should continue. A preceding contradiction can leave the aggregate Failed after a later ordinary target error; native execution stops at that error. The worker accepted the finding and is adding an actual callback trace covering contradiction, later observation error, and a subsequent delivery which must not run. Flow and joined status must remain separate.

Go unresolved structured-instance values were also classified as Failed by resolveAll, where native reports Error before the invocation. The worker accepted the finding and is adding a live native-versus-Go control. The TypeScript worker independently found and corrected the same resolution category and an older clock-reading callback-error classification. Final commits and independent review remain pending.

The initial one-time contract admission fixture manifest lacked constrained-newtype authority vectors and source regressions for nested forbidden flow, retained replay, opaque reading and null input. The coordinator requested these before contract freeze; the worker reports focused source10/0 and admission6/0 after adding them. These counts are implementation evidence, not coordinator executions, and do not establish disclosure execution or full runtime parity.

Coordinator shared test migration now preserves precise report/2 Error and Unsupported categories rather than comparing normalized Failed/Skipped values. CLI live-producer fixture expectations and projection policy work are uncommitted and awaiting integrated validation. No new PR, tag or release is published from this unfinished group.
