---
format: aep.planning-md/3
id: story:a-wire-name-in-a-path-segment-refuses-slash-and-dot-segments
kind: story
status: draft
title: A wire name that a generated path segment reads refuses '/', '.' and '..'
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by the fit review of https://github.com/beyond10x/ess/issues/493: a wire name containing `/`
or `..` passes `ess specify validate` and is copied into the OpenAPI path as it is, for example
`/grant/views/../.well-known/demo-configuration`. A wire name that a path segment reads can then
address another route.

## Acceptance

- `ess specify validate` refuses a wire name that a generated path segment reads when it contains
  `/`, or is `.` or `..`, naming the declaration and the wire name.
- The OpenAPI projection, the Rust and Go servers and every other target that builds a path from a
  wire name are covered by that refusal; a test per target shows the refused name never reaches a
  generated path.
- Existing examples and suites validate unchanged.
