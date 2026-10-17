---
format: aep.planning-md/3
id: story:http-binding-response-headers-challenges-and-redirects
kind: story
status: draft
title: An ess-http/1 outcome declares response headers, a challenge and a redirect
relations:
- depends_on: story:http-binding-response-body-and-status-per-outcome
revision: 1
---
## Outcome

An `ess-http/1` outcome declares response headers (`{literal}`, `{source}`, `{challenge: {scheme,
params}}`, which carries `WWW-Authenticate` with its `error` attribute per RFC 6750 §3) and a
redirect (`redirect: {location, mode: query | fragment, parameters}`) on a `3xx` status. Compile
refuses `ESS-HTTP-026` to `-028` and writes both into `ess-http-ir/1`, header and parameter maps in
name order. Unit k3 of wave k.

Part of stage 2 of https://github.com/beyond10x/ess/issues/493
(`docs/design/http-presentation-binding.md`, "Response headers", "Redirects", lines 332-361).

## Acceptance

Failing tests first, in a new file `crates/specify/ess-http/tests/headers.rs`:

- `headers_compile_in_name_order_whatever_the_authored_order`.
- `a_bearer_challenge_with_an_error_source_compiles`: the page's `ReadPrivateItem` example.
- `ess_http_026_a_bad_or_reserved_or_duplicate_header_is_refused`: a non-token name;
  `Content-Type`, `Content-Length`, `Transfer-Encoding`, `Location`; `cache-control` beside
  `Cache-Control`; a source naming no field; a source naming a struct.
- `ess_http_027_a_challenge_with_a_bad_scheme_or_param_is_refused`: non-token scheme, non-token
  param name, a param twice.
- `ess_http_028_a_redirect_without_3xx_or_3xx_without_redirect_is_refused`: also a non-`String`
  `location`, `mode: hash`, a parameter twice.
- `a_query_redirect_with_an_optional_state_parameter_compiles`: the page's `ApproveOrder` example.

## Scope

| surface | confidence |
|---|---|
| `crates/specify/ess-http/src/lib.rs`: `Source` (199), route outcome types k2 adds, `CODES` | cited |
| `crates/specify/ess-http/tests/headers.rs` | inferred (new) |
| `task test-xtask` for the new codes | cited |

## Delivery

- depends on: `story:http-binding-response-body-and-status-per-outcome` (same file)
- du: about 6G
- adversary: yes (challenge quoting and the open-redirect note are security-relevant)
