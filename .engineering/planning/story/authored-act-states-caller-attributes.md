---
format: aep.planning-md/3
id: story:authored-act-states-caller-attributes
kind: story
status: draft
title: An authored act states the caller's attribute values
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

An authored `ess-scenario/1` act can state the caller's attribute values, and a command whose
outcome reads a caller attribute runs from an authored scenario against every target.

## Evidence

A consumer (ess 0.52.0, reported unchanged on 0.56.0): a command whose outcome sets a field from
`{caller: account_id}` (the actor declares that attribute) authors with 0 refusals in
`ess verify conform author`, and `ess verify conform run --target interpreted --report-format 2`
reports the scenario `unsupported` with no reason; bisected to the first act whose command reads a
caller attribute. `crates/verify/ess-conformance/src/authored.rs` builds every authored act's
`ScenarioStep::ExecuteCommand` with `caller: std::collections::BTreeMap::new()` (two sites, read
from the source, not yet run), so no act can carry the attribute the command reads.

## Acceptance

- An act accepts a `caller:` map of attribute values, each a literal or a `{$instance: <name>}`
  reference; `ess verify conform author` refuses an attribute the act's actor does not declare,
  and an act whose command reads a caller attribute it does not state, naming both.
- The consumer's shape (neutral names) runs from its authored scenario against the interpreted
  target and the native runner and passes; the generated Go and TypeScript runners pass it too, or
  the gap is reported per target.
- Until an act states it, a scenario that reaches a caller attribute with none supplied is
  reported with a reason naming the attribute, in the text and JSON reports.
- An authored suite with no `caller:` key keeps its bytes; the `ess-scenario/1` schema and its
  reference page describe the new key, and the format consequence of the new key is decided
  explicitly (old-reader test).

## Scope (inferred)

`crates/verify/ess-conformance/src/authored.rs`, the scenario schema under `schemas/`, the
authoring reference under `website/docs/`, tests in `crates/verify/ess-conformance/tests/` and
`crates/edge/ess-cli/tests/`.
