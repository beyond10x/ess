---
format: aep.planning-md/3
id: story:http-rust-server-serves-the-binding
kind: story
status: draft
title: The synthesized Rust server serves an ess-http/1 binding
relations:
- depends_on: story:http-binding-response-headers-challenges-and-redirects
revision: 1
---
## Outcome

`ess generate synthesize --target rust --http <binding> [--refines …]` emits a Rust server that
serves the bound surface: template dispatch with several methods on one path, path parameters,
form and query decoders, and the declared status, body, headers, challenge and redirect per
outcome and per standard refusal. `/openapi.json` and `/docs` stay reserved, and a route claiming
either is refused at synthesis. Without `--http` every synthesized byte is unchanged. `--http` with
any target other than `rust` and `go` is refused by name (`go` arrives in k6; until then `go` is
refused too). Unit k5 of wave k.

Part of stage 2 of https://github.com/beyond10x/ess/issues/493
(`docs/design/http-presentation-binding.md`, lines 199-214 and 391-402).

## Acceptance

Failing tests first, in a new file `crates/generate/ess-synth/tests/http_binding_rust.rs` (its
harness follows `adversary_served_pass2.rs`, which serves a synthesized model):

- `without_a_binding_the_rust_server_keeps_its_bytes`.
- `a_bound_command_is_dispatched_by_template_and_method`: two methods on one path, a path variable
  read into the input.
- `a_form_request_decodes_and_an_unknown_parameter_follows_the_binding`: `ignore` admits, `refuse`
  answers `malformed`; a scalar key given twice is malformed.
- `a_query_command_reads_its_inputs_from_the_query_string`: a `List` repeats its key.
- `a_declared_status_body_and_headers_are_written_per_outcome`.
- `a_bearer_challenge_quotes_the_error_wire_name`: a wire name containing `"` cannot leave the
  quoted string.
- `a_query_redirect_keeps_an_existing_query_and_omits_an_absent_optional`; the same for
  `mode: fragment`.
- `a_route_claiming_openapi_json_or_docs_is_refused_at_synthesis`.
- `a_declared_refusal_override_answers_not_granted_unfinished_and_malformed`.
- `crates/edge/ess-cli/tests/http.rs`: `synthesize_http_with_another_target_is_refused_by_name`.

## Spec first

`ess generate synthesize` gains `--http` and `--refines`: model the input first in
`models/toolchain/domains/generate.yaml`, as `toolchain.generate.PlanProjection` models `http`
(lines 194-222, a `binding-without-openapi` guard and an external `binding-refused` outcome), then
regenerate and implement against it.

## Scope

| surface | confidence |
|---|---|
| `crates/generate/ess-synth/src/rust/http.rs` (dispatch at 824-834, status at 1244) | cited |
| `crates/generate/ess-gen/src/http.rs` (`bound_routes` 531, `outcome_status` 243) | cited |
| `crates/edge/ess-cli/src/main.rs` (`Synthesize`, lines 312 and 1639) | cited |
| `models/toolchain/domains/generate.yaml` and its regenerated outputs | cited |
| `crates/generate/ess-synth/tests/http_binding_rust.rs` | inferred (new) |

## Delivery

- depends on: `story:http-binding-response-headers-challenges-and-redirects`
- du: about 10G (builds and serves a synthesized Rust crate)
- adversary: yes (decoders and header/challenge writers face the caller)
