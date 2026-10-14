---
format: aep.planning-md/3
id: story:http-openapi-stage-2
kind: story
status: draft
title: The OpenAPI projection carries the ess-http/1 stage-2 keys
relations:
- depends_on: story:http-binding-response-headers-challenges-and-redirects
revision: 1
---
## Outcome

`ess generate --kind openapi --http <binding>` projects the stage-2 keys: `requestBody` with the
declared media type (or query parameters for `query`), one response per declared status with the
bare schema, the error object or no content, response `headers` including a redirect's `Location`,
and `x-ess-outcomes` on each response naming the outcomes it answers. An OpenAPI document generated
from a binding with no stage-2 key keeps its 0.60.0 bytes. `securitySchemes` stays omitted (k0,
answer 8). Unit k4 of wave k.

Part of stage 2 of https://github.com/beyond10x/ess/issues/493
(`docs/design/http-presentation-binding.md`, "What generated artifacts change (stage 2)", lines
391-402).

## Acceptance

Failing tests first, added to `crates/generate/ess-gen/tests/openapi_http.rs`:

- `a_stage_1_binding_keeps_its_openapi_bytes` (compared with the output of the existing stage-1
  tests, e.g. `a_bound_operation_is_served_at_its_declared_method_and_path`, line 85).
- `a_form_route_takes_a_form_urlencoded_request_body`.
- `a_query_command_takes_its_unmapped_inputs_as_query_parameters_and_has_no_request_body`.
- `each_declared_status_is_its_own_response_with_x_ess_outcomes`.
- `a_bare_response_body_carries_the_declared_response_schema_without_the_envelope`.
- `an_error_body_names_the_member_and_the_errors_fields`.
- `declared_headers_and_a_challenge_are_response_headers`.
- `a_redirect_answers_its_status_with_a_location_header_and_no_content`.
- `a_single_view_answers_one_row_and_its_empty_status`.
- `no_security_schemes_are_emitted_for_a_challenge`.

## Scope

| surface | confidence |
|---|---|
| `crates/generate/ess-gen/src/openapi.rs` (request body 223-225, envelope 57-70 and 231-234, view rows 1080-1091) | cited |
| `crates/generate/ess-gen/src/http.rs` (`bound_routes`, line 531) | cited |
| `crates/generate/ess-gen/tests/openapi_http.rs` | cited |
| `website/docs/guides/http-routes.md` (stage-1 guide) | cited |

## Delivery

- depends on: `story:http-binding-response-headers-challenges-and-redirects` (reads every stage-2 IR key)
- du: about 7G
- adversary: no
