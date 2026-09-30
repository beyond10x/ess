---
format: aep.planning-md/3
id: story:acceptance-runs-as-toolchain-scenarios
kind: story
status: draft
title: A toolchain story's acceptance is scenarios a conformance run decides
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A story about the `ess` toolchain states its acceptance as `ess-scenario/2` files against
`models/toolchain`, and a conformance run against the built `ess` binary decides it: the report's
exit status is the verdict and `aep plan artifact evidence --from <report>` records it. No agent
judges whether an acceptance line holds.

## Acceptance

- A toolchain target adapter runs `models/toolchain-scenarios/**` against the `ess` binary under test
  (each act runs the command the model's `naming.wire` names, in an isolated directory with the
  scenario's fixture sources); `task check` runs it and fails on any non-passing scenario.
- `models/toolchain` moves from `ess/3` to the newest format; `toolchain.specify.Refused` carries the
  diagnostic code and location through error payload sources (ess/19); a view over generated
  artifacts (path and content lines) lets a scenario assert what a generator wrote.
- `story:union-tag-inline-with-fields` has its acceptance rewritten as scenario files (a variant field
  named like the tag is refused with its code and path; two variants with one name are refused; an
  older format refuses the key by name; the Rust target writes `#[serde(tag = "<field>")]`; a model
  without the key compiles to the same bytes), and they are red before the feature and green after.

## Origin

Operator, 2026-09-30: acceptance written in ESS, decided deterministically.
