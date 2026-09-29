---
format: aep.planning-md/3
id: epic:generated-determined-behaviour
kind: epic
status: active
title: Synthesis generates what the specification fully determines
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:20:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:20:01Z", actor: "human:timo", revision: 3}
---
## Outcome

`ess generate synthesize --target rust` generates every part of an implementation the
specification fully determines, so an implementor writes only what the specification cannot
determine. A downstream specification with 118 commands and 47 views needed about 5,100
hand-written lines after 0.44.0 that only restated declared guards, effects, sets and payload
sources; after this epic it needs its genuinely undetermined items only (on the order of 9).

## Decision (operator, 2026-09-29)

The synthesis principle "generated code is structural, never behavioural" becomes: **what the
specification fully determines is generated; what it cannot determine is an obligation.** A
command whose every outcome is expressible gets a generated behaviour written against generated
ports; any command with an outcome the generator cannot express stays a whole obligation, as today.
Storage is a port the implementor provides, never a generated store.

The generated behaviour follows the conformance interpreter's semantics
(`crates/verify/ess-conformance/src/interpret/execute.rs`) and the one precedence order in
`docs/design/cross-record-and-stored-field-guards.md`, because the generated suite holds a target to
them.

## Stories

1. Generated behaviours for fully declared commands, over storage and context ports.
2. A generated invariant check on each entity's data.
3. Actor grants as generated data.
4. A transport-free entry point in the server crate.
5. The synthesis guide, PLAN.md and the principle state the new rule and that storage is a port.
