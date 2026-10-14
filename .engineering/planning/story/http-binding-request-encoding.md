---
format: aep.planning-md/3
id: story:http-binding-request-encoding
kind: story
status: draft
title: An ess-http/1 route declares its request encoding
relations:
- depends_on: story:http-stage-2-design-answers
revision: 1
---
## Outcome

An `ess-http/1` route declares how its request is encoded: `request: {encoding: json | form | query,
unknown: refuse | ignore}`. `ess specify http validate` and `compile` read the key, refuse the cases
of `ESS-HTTP-020` and `ESS-HTTP-021`, and write it into `ess-http-ir/1`. A binding that uses no
stage-2 key compiles to the same IR bytes as under 0.60.0. Unit k1 of wave k.

Part of stage 2 of https://github.com/beyond10x/ess/issues/493
(`docs/design/http-presentation-binding.md`, "Request encoding", lines 289-302).

## Acceptance

Failing tests first, in a new file `crates/specify/ess-http/tests/request_encoding.rs`:

- `a_route_without_request_keeps_json_and_the_stage_1_ir_bytes`: the stage-1 fixture
  `crates/specify/ess-http/tests/fixtures/catalog.yaml` compiles to IR bytes equal to a fixture
  pinned from 0.60.0 output (old-reader compatibility test; AGENTS.md "Determinism and formats").
- `form_and_query_encodings_compile_into_the_route`: `encoding: form` on `post` and
  `encoding: query` on `get` appear in the IR route; `unknown: ignore` is recorded, `refuse` (the
  default) is omitted.
- `ess_http_020_an_encoding_outside_the_set_or_on_the_wrong_method_is_refused`: `xml`; `form` on
  `get`; `json` on `get`; `query` on `post`; each names `routes[i].request.encoding`.
- `ess_http_021_a_form_or_query_field_without_a_flat_spelling_is_refused`: a struct, a nested list,
  a map and a `Json` field each refused, naming the field and its type; a `List<String>` and an
  `Optional<Integer>` admitted.
- `crates/edge/ess-cli/tests/http.rs`: `specify_http_validate_reports_ess_http_020_with_its_path`
  runs the binary on one refused document.

## Spec first

The toolchain model's specify domain (`models/toolchain/domains/specify.yaml`) carries no
`specify http` command today (grep for `http` finds none, cited). If the unit adds one, it is
modelled there first; otherwise the refusals stay external to the toolchain model, as the stage-1
`binding-refused` outcome of `toolchain.generate.PlanProjection` does
(`models/toolchain/domains/generate.yaml:218-222`).

## Scope

| surface | confidence |
|---|---|
| `crates/specify/ess-http/src/lib.rs`: `RouteSpec` (line 134), `Route` (510), `CODES` (29), compile table doc (886-897) | cited |
| `crates/specify/ess-http/tests/request_encoding.rs`, `tests/fixtures/` (new IR fixture) | inferred (new) |
| `crates/edge/ess-cli/tests/http.rs` | cited |
| `website/docs/reference/formats.md` (stage-1 unit touched it, story Scope table) | inferred |
| new diagnostic codes move pins outside the crate: run `task test-xtask` (AGENTS.md) | cited |

## Delivery

- depends on: `story:http-stage-2-design-answers`
- du: about 6G (wave-k proposal estimate)
- adversary: no
