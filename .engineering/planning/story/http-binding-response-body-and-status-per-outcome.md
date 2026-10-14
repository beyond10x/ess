---
format: aep.planning-md/3
id: story:http-binding-response-body-and-status-per-outcome
kind: story
status: draft
title: An ess-http/1 route declares the status and body of each outcome
relations:
- depends_on: story:http-binding-request-encoding
revision: 1
---
## Outcome

An `ess-http/1` route declares, per outcome, the status and the body it answers (`envelope`,
`response`, `{<member>: error}`, `none`), a view route declares `rows` or `single` with `empty`, and
top-level `refusals:` overrides the standard refusals (`not_granted`, `unfinished`, `malformed`).
Compile refuses `ESS-HTTP-022` to `-025`, `-029`, `-030` and the `ESS-HTTP-031` that k0 decides, and
writes the keys into `ess-http-ir/1`. Unit k2 of wave k.

Part of stage 2 of https://github.com/beyond10x/ess/issues/493
(`docs/design/http-presentation-binding.md`, "Response body per outcome", "Status per outcome",
"Standard refusals", lines 304-371).

## Acceptance

Failing tests first, in a new file `crates/specify/ess-http/tests/outcomes.rs`:

- `an_outcome_status_and_body_compile_into_the_route`: the design page's `PlaceOrder` example
  (`placed` 200 `response`, `out-of-stock` 400 `{error: error}`) is in the IR; an outcome not
  listed is absent from the IR (its status stays `outcome_status`, `crates/generate/ess-gen/src/http.rs:243`).
- `ess_http_022_an_unknown_outcome_or_outcomes_on_a_view_is_refused`.
- `ess_http_023_a_status_out_of_range_or_in_a_forbidden_class_is_refused`: `99`, `600`, `1xx`,
  `2xx` on an `error` outcome, `4xx` on a success; `3xx` admitted for both.
- `ess_http_024_a_body_form_the_outcome_does_not_admit_is_refused`: `response` without
  `returns: true`, `{error: error}` without `error`, a body on `204`, `single` on a command.
- `ess_http_025_two_outcomes_on_one_status_the_bodies_cannot_tell_apart_are_refused`, naming both
  outcomes; two `envelope` bodies, or two `{error: error}` with distinct wire names, admitted.
- `ess_http_029_a_refusal_key_outside_the_set_or_in_the_wrong_class_is_refused`.
- `ess_http_030_single_without_empty_or_empty_without_single_is_refused`.
- `ess_http_031_single_on_a_view_not_selected_by_identity_is_refused` (the rule k0 writes).

## Scope

| surface | confidence |
|---|---|
| `crates/specify/ess-http/src/lib.rs`: `RouteSpec`, `HttpSpec` (389), `Route`, `HttpIr` (562), `CODES` | cited |
| `crates/specify/ess-http/tests/outcomes.rs` | inferred (new) |
| error wire name lookup, `crates/specify/ess-domain/src/name.rs:240-242` (read, not changed) | cited |
| `task test-xtask` for the new codes | cited |

## Delivery

- depends on: `story:http-binding-request-encoding` (same file, `ess-http/src/lib.rs`)
- du: about 6G
- adversary: yes (the distinguishability rule `ESS-HTTP-025` is what stage 3's mapping relies on)
