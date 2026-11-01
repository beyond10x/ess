---
format: aep.planning-md/3
id: story:bound-response-root-follows-the-published-result
kind: story
status: draft
title: Under --http a command's response root follows the published Result, or is refused by name
relations:
- serves: vision:O2
revision: 1
---
## Finding

Measured by adversary pass 1 of wave D343 (`review-result:adversary-d343-u1-pass-1`, F4, out of scope
there, not run): under `ess generate types --http <binding>` the `.response` root is selected
unchanged even when every outcome on the route answers `response.<field>`, `{member: error}` or
`none`, in which case the bound OpenAPI document publishes no `{command}.Result`
(`crates/generate/ess-gen/src/openapi.rs`, `crates/generate/ess-gen/src/model_types.rs`). The root then
describes a body nobody sends.

## Acceptance

- Under `--http`, a command's `.response` root equals the bound `{command}.Result` where one is
  published, and is refused by name under `--root` and omitted by `--all-commands` where none is;
  without `--http` every output keeps its bytes.
