---
format: aep.planning-md/3
id: story:browser-response-conformance
kind: story
status: draft
title: Support response contracts in browser conformance products
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Browser conformance products preserve and support the same admitted response authority as native and generated runtimes. Navigation of declarations remains visibly distinct from executing assertions against an implementation.

## Evidence and fit

Required internal coverage gap under the operator's all-features mandate. Source inspection in the runtime carrier finds crates/verify/ess-conformance/src/web.rs:308 refuses every response::used_by suite. tests/response_payload.rs:218 pins this blanket refusal. The existing player explicitly states at web.rs:288 that replay navigates declarations and emits no qualifying execution evidence. Existing standalone wasm32 execution of the Rust Runner therefore proves runtime portability, not completion of the browser product.

The read-only nested-response scope report has SHA256 2bd62ad5b85e0001b5cd6bb45889d3463bfea378448dd849f8a421dc3264c371. No actual browser probe or product implementation is claimed by that report.

## Acceptance

1. Preserve original suite bytes, parent/coverage authority and every response declaration through actual browser loading, navigation, selection and reset; never invent observed response values.
2. Replace the blanket refusal only with a demonstrated supported route. Execute a response-bearing suite through a real browser target bridge and compare healthy, wrong-response and wrong-event verdicts with native/Go/TypeScript. Independent stale-invocation, optional-presence and exact-integer controls remain required.
3. Keep declaration playback separate from execution results. A standalone WASM test or removal of an admission guard is insufficient.
4. Exercise existing browser fidelity and coverage tests on the integrated candidate; original-byte admission must reject forged or historical unsupported authority before target callbacks.

## Decisions

Accept the required product gap for design and implementation in the full backlog; it is not a permanent exclusion from all-runtime support. No new authored source syntax is proposed. The browser target/runner bridge design and exact production scope must be established from an actual red-capable browser probe before implementation. Coordinate persisted authority with nested-response-observations and the single held ess/21 bundle. This draft reserves no format version and authorizes no production edits.

## Scope

Cited declaration/admission seams: crates/verify/ess-conformance/src/web.rs, src/web_replay.rs, assets/player.js, assets/coverage-player.js and tests/response_payload.rs. Cited browser verification seams: crates/edge/ess-cli/tests/replay_fidelity_browser.rs and tests/coverage_browser.rs. Inferred real target-bridge surface remains unassessed; determine it before dispatch. Native interpret/**, caller synthesis and transport publication are separately owned.
