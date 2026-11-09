---
format: aep.planning-md/3
id: story:bound-response-root-follows-the-published-result
kind: story
status: draft
title: Under --http a command's response root follows the published Result, or is refused by name
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
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

- Under `--http`, where the bound document publishes `{command}.Result`, the `.response` root's
  schema equals it byte for byte.
- Under `--http`, where it publishes none (every outcome on the route answers `response.<field>`,
  `{member: error}` or `none`), `--root <command>.response` exits 1, refused as `unknown_root` naming
  the command and saying its route publishes no `Result`; `--all-commands` leaves the root out. The
  reproduction above is that case.
- Without `--http` every output keeps its bytes.
- Tests in `crates/generate/ess-gen/tests/bound_command_input_roots.rs` (or a sibling
  `bound_command_response_roots.rs`) pin the three cases.

## Scope

- cited: crates/generate/ess-gen/src/model_types.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/bound_command_input_roots.rs
- inferred: crates/edge/ess-cli/tests/model_type_events.rs

confidence: high
