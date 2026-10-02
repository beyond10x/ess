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
revision: 6
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
