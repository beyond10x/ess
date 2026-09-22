---
format: aep.planning-md/1
id: epic:independently-provisioned-conformance
kind: epic
status: draft
title: Conformance inputs are independent of the tested implementation
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Conformance against an independently provisioned implementation uses typed inputs fixed before execution and honest assertions about implementation-generated output. A runtime never derives an expected value by copying the tested post-state.

## Acceptance

- Typed fixture values are independently provided, admitted and frozen before target activity.
- Synthesized and authored scenarios preserve their source-declared guards and equality assertions.
- Implementations in Rust and emitted Go/TypeScript agree, including refusal and mutation cases.
- New persisted semantics have explicit format admission and old-reader compatibility tests; fixture-free behavior stays unchanged.

## Scope

Conformance input and observation semantics, their source declarations, runtimes and validation. No application implementation, stimulus builder or integration credentials are included.
