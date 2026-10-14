---
format: aep.planning-md/3
id: story:http-go-server-serves-the-binding
kind: story
status: draft
title: The synthesized Go server serves an ess-http/1 binding
relations:
- depends_on: story:http-rust-server-serves-the-binding
revision: 1
---
## Outcome

`ess generate synthesize --target go --http <binding>` emits a Go server with the same behaviour as
k5's Rust server: template dispatch, form and query decoders, and the declared status, body,
headers, challenge and redirect per outcome and per standard refusal, with `/openapi.json` and
`/docs` reserved. Without `--http` every synthesized byte is unchanged. With k5 this closes stage 2
of `ess-http/1`, and https://github.com/beyond10x/ess/issues/493 with it. Unit k6 of wave k.

## Acceptance

Failing tests first, in a new file `crates/generate/ess-synth/tests/http_binding_go.rs`, an
in-package `go test` of the generated `server` package driven by `httptest`, as
`crates/generate/ess-synth/tests/adversary_served_pass2_go.rs` does (lines 11-12), with
`go test -timeout`:

- `without_a_binding_the_go_server_keeps_its_bytes`.
- `a_bound_command_is_dispatched_by_template_and_method`.
- `a_form_request_decodes_and_an_unknown_parameter_follows_the_binding`.
- `a_query_command_reads_its_inputs_from_the_query_string`.
- `a_declared_status_body_and_headers_are_written_per_outcome`.
- `a_bearer_challenge_quotes_the_error_wire_name`.
- `a_query_redirect_keeps_an_existing_query_and_omits_an_absent_optional`; and `mode: fragment`.
- `a_route_claiming_openapi_json_or_docs_is_refused_at_synthesis`.
- `the_go_and_rust_servers_answer_one_binding_alike`: one binding, the same requests, equal status,
  headers and body bytes from both servers (reuses k5's cases).
- The design page's stage-2 table marks the servers built; the issue's closing comment names the
  release.

All Go build and test steps run with `GOCACHE=<tree>/target/go-build`.

## Scope

| surface | confidence |
|---|---|
| `crates/generate/ess-synth/src/go/http.rs` (path switch at 1132 and 1300, status at 1546-1557) | cited |
| `crates/generate/ess-synth/tests/http_binding_go.rs` | inferred (new) |
| `crates/edge/ess-cli/src/main.rs` (admit `go` for `--http`) | inferred |
| `docs/design/http-presentation-binding.md` (stage table, line 52) | cited |

## Delivery

- depends on: `story:http-rust-server-serves-the-binding` (CLI flag, shared `ess-gen::http` reader)
- du: about 10G
- adversary: yes
- closes: https://github.com/beyond10x/ess/issues/493 (with k5)
