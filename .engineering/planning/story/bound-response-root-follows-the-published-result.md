---
format: aep.planning-md/3
id: story:bound-response-root-follows-the-published-result
kind: story
status: active
title: Under --http a command's response root follows the published Result, or is refused by name
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/model_type_events.rs
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/bound_command_input_roots.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:06Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:07Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Finding

Measured by adversary pass 1 of wave D343 (`review-result:adversary-d343-u1-pass-1`, F4), reproduced
on 0.92.0: under `ess generate types --http <binding>` the `.response` root is selected unchanged
even when every outcome on the route answers `response.<field>`, `{member: error}` or `none`, in
which case the bound OpenAPI document publishes no `{command}.Result`. The root then describes a
body nobody sends.

```yaml
# spec.yaml
format: ess/27
system: tokens
version: v1
domain: tokens.auth
actors:
  - {name: tokens.auth.Client, may: [tokens.auth.Issue]}
commands:
  - name: tokens.auth.Issue
    input: [{name: subject, type: String}]
    response:
      - {name: token, type: String}
      - {name: expires_in, type: Integer}
    outcomes:
      - {name: issued, returns: true}
components:
  - component: auth
    owns: {domains: [tokens.auth]}
    accepts: {commands: [tokens.auth.Issue]}
    reached_by: network
# http.yaml (ess-http/1): route tokens.auth.Issue, post /token,
#   outcomes: {issued: {status: 200, body: response.token}}
```

```
$ ess generate --path spec.yaml --kind openapi --http http.yaml --out oa
  # components.schemas: tokens.auth.Issue.Input, tokens.auth.Issue.issued.Response (type: string); no tokens.auth.Issue.Result
$ ess generate types --path spec.yaml --http http.yaml --target typescript --root tokens.auth.Issue.response --out ts
1 model type(s), written to ts; …   (exit 0)
export type TokensAuthIssueResponse = { "expires_in": number; "token": string; };
$ ess generate types --path spec.yaml --http http.yaml --target typescript --all-commands --out ts-all
  # exports TokensAuthIssueInput and TokensAuthIssueResponse
```

Cause: `bound_command_roots` lists `<command>.response` whenever the command declares a response
(`crates/generate/ess-gen/src/model_types.rs:192-193`), and `select_root` builds it from
`Message::of_response` without reading the binding (`model_types.rs:367-385`).

## Acceptance

- `cargo test -p ess-gen --locked --test bound_command_input_roots`:
  `responses_and_rows_are_unchanged_under_a_binding` passes, so under a binding whose route
  publishes `demo.notes.Create.Result` the `demo.notes.Create.response` root's schema equals that
  component byte for byte.
- In the same target, `a_bound_route_that_publishes_no_result_has_no_response_root`, on the
  reproduction above: `ModelTypes::select_bound` for the root `tokens.auth.Issue.response` returns
  an error with rule `unknown_root` whose detail names `tokens.auth.Issue` and says its route
  publishes no `Result`, and `ModelTypes::bound_command_roots` holds no
  `tokens.auth.Issue.response`.
- `cargo test -p ess-cli --locked --test model_type_events`: on the reproduction,
  `ess generate types --http http.yaml --target typescript --root tokens.auth.Issue.response` exits
  1 with an `unknown_root` refusal naming `tokens.auth.Issue`.
- In the same target, `ess generate types --http http.yaml --target typescript --all-commands` on
  the reproduction exits 0 and writes `TokensAuthIssueInput` and no `TokensAuthIssueResponse`.
- `without_a_binding_every_root_keeps_its_bytes` in `bound_command_input_roots` passes unchanged.

## Decisions

- The response-root cases go in `bound_command_input_roots.rs` rather than a new sibling file,
  because its binding helpers (`bound_by`, `published`) and its response case
  (`responses_and_rows_are_unchanged_under_a_binding`) are already there.

## Scope

- cited: crates/generate/ess-gen/src/model_types.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/bound_command_input_roots.rs
- inferred: crates/edge/ess-cli/tests/model_type_events.rs

confidence: high

Moves no shared surface.
