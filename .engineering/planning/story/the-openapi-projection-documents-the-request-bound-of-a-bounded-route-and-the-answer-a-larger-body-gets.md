---
format: aep.planning-md/3
id: story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets
kind: story
status: draft
title: The OpenAPI projection documents the request bound of a bounded route and the answer a larger body gets, and the design page, guide and changelog of the binding state the rule
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- depends_on: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi_http.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
revision: 6
---
## Outcome

`ess generate --kind openapi --http FILE` (and the `openapi.json` the generated servers serve) states, for every operation a binding bounds, the largest request body it reads and the answer a larger one gets; and the design page, the guide and the changelog say what the two new binding keys mean. Three things stay true:

- A binding that declares neither `limits` nor `refusals.too_large`, or declares `limits` only where no route reads a body, generates every OpenAPI byte it generates today (parent D7: no `413` is documented for a binding declaring nothing).
- A view route, a route that reads no body and a derived route get no `x-ess-request-max-bytes` and no new response.
- The documents say what the servers do: a bounded operation names its bound under `x-ess-request-max-bytes`, and its refusal is the declared `too_large` response at the declared status, or `413` with `{"refused": ...}`.

## Why

Parent: `story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets` (decisions D1, D2, D5, D6, D7). Needs h1 (`HttpIr::request_bytes`, `bounds_request`, `FIXED_REQUEST_BYTES`, `RefusalKey::TooLarge`, and the one-arm edit of `refusal_sentence` h1 makes). Independent of h2 and h3 (the servers): the document and the servers read one answer, `bounds_request`, so they agree without waiting for each other. h4 lands with them in one release.

## Where

Tree read: the plan tree, `git rev-parse --short HEAD` = `cfe9dabcb85` (code as of `e3f502951bd`; the difference is planning files). Lines are read in that tree.

- `crates/generate/ess-gen/src/openapi.rs` (h1 has added one arm to `refusal_sentence`, 1909, before this unit starts: rebase on it)
  - Module doc table of the binding's keys (217-229): after the `refusals.<key>` rows add `` | `limits.request_bytes`, `routes[i].limits.request_bytes` | `x-ess-request-max-bytes` on each operation that reads a body, and a response at the `too_large` status | `` and extend the `refusals.<key>` row to name `too_large`.
  - `paths` (727-765): per route compute `bound_request: Option<u64>` once, `binding.zip(route.declared).filter(|(binding, declared)| binding.ir().bounds_request(declared)).map(|(binding, declared)| binding.ir().request_bytes(declared).unwrap_or(ess_http::FIXED_REQUEST_BYTES))`, and pass it to `operation` (commands only; `query` gets none: a view reads no body, and `bounds_request` is false for it).
  - `answered_by_binding` (676-685): gain a parameter `bound_request: Option<u64>` and a clause `|| bound_request.is_some()`, so a bounded route is answered in the bound form (`x-ess-outcomes` on each response), as one with a `refusals` override is. Its three call sites (647, 882, 994) pass it: `description` computes it per route with the same expression as `paths` (factor one helper `fn bound_request(binding: Option<&http::Binding>, declared: Option<&ess_http::Route>) -> Option<u64>` and use it at all four places); `query` passes `None`.
  - `operation` (808-1008): new parameter `bound_request`; the `Operation` literal (833) gains `max_request_bytes: bound_request`; `bound_responses` gets the parameter.
  - `Operation` (3270-3345): new field after `route_source` (3280): `#[serde(rename = "x-ess-request-max-bytes", skip_serializing_if = "Option::is_none")] max_request_bytes: Option<u64>,` with a doc comment: the largest request body, in bytes, the operation reads; absent where nothing bounds it, so a document generated without a limit keeps its bytes. The second literal of `Operation` (`query`, 1019) gets `max_request_bytes: None`.
  - `bound_responses` (1376-1445): in the `standard` array (1397-1413) add, after `Malformed`, the entry `(ess_http::RefusalKey::TooLarge, bound_request.is_some(), Some((http::TOO_LARGE, Answer::TooLarge(limit))))` where `limit = bound_request.unwrap_or(ess_http::FIXED_REQUEST_BYTES)`. The existing loop does the rest: a declared `too_large` (the merged route-over-binding `refusals`, as `paths` already passes) is answered at its own status as `Answer::Refusal`, else the default at `413`. `http::TOO_LARGE` is a new `pub const TOO_LARGE: &str = "413";` in `crates/generate/ess-gen/src/http.rs` beside `UNFINISHED` (find it with `grep -n "pub const UNFINISHED" src/http.rs`); if the file has a different convention for status constants, follow it.
  - `Answer` (1337-1357): new variant `TooLarge(u64)`, "the standard refusal for a body larger than the route reads, as the servers answer it today". `rendered` (1650-1700): new arm `Answer::TooLarge(limit) => Rendered::standard(Some(too_large_schema()), too_large_meaning(*limit))`. The only other exhaustive match on `Answer` found is that one (the two other sites at 1401 and 1411 build values); the compiler finds any more.
  - New `fn too_large_schema() -> Value` beside `unfinished_schema` (1314): `{"type": "object", "additionalProperties": false, "required": ["refused"], "properties": {"refused": {"type": "string", "description": "Why the body was refused, in words."}}}`. New `fn too_large_meaning(limit: u64) -> String`: `` format!("No declared outcome: the request body is larger than the {limit} bytes this operation reads (`x-ess-request-max-bytes`). It is refused from the request head, before the body is read, so nothing was run, written or published.") ``. Both servers answer `{"refused": <words>}` with `413` by default (`rust/http.rs` 2037-2042, `go/http.rs` 2778-2781).
  - `refusal_sentence` (1909-1927): h1 added the `TooLarge` arm; this unit does not touch it.
  - `description` (556-665): after the paragraph that starts `The binding also declares how some of those operations are encoded` (647-662) add one more, written only when some bound route has `bound_request(..).is_some()`: `` "\n\nAn operation that names `x-ess-request-max-bytes` reads at most that many bytes of its request body: a larger body is refused from the request head, before it is read, with the response the operation lists for it." `` The two existing paragraphs are not edited.
- `crates/generate/ess-gen/tests/openapi_http.rs`: append a section "Request limits" after the last test (1167); helpers `model_of`, `binding`, `bound_document`, `unbound_document`, `as_json`, `keys`, constants `ORDERS`, `ORDERS_ROUTES` (315-343) and `AUTH_ROUTES` (344-382) exist. The orders service has `PlaceOrder` (a `form` command), `NoteOrder` (a json command, path variable `order_id`) and the view `OrderById`.
- `docs/design/http-presentation-binding.md` (h1 owns the Keys table, "Standard refusals", "What compiling checks (stage 2)", "Codes per unit" and "Refinement": do not edit them; rebase on h1's text)
  - "What generated artifacts change (stage 2)" (566-577): the OpenAPI bullet gains the sentence `A bounded operation carries x-ess-request-max-bytes and a response at the status its too_large refusal declares, 413 where it declares none.`; the "Rust and Go servers" bullet gains `They read at most the bound of the route a request matches and refuse a larger body before reading it.`
  - "Formats and determinism (stage 2)" (579-584): one sentence: `A binding with neither limits nor too_large changes no byte of the OpenAPI document or the servers; no format version moves.`
  - "Precedence" (697-715, the numbered list): insert a new step 2 after step 1, `The body is larger than the bound that applies: the bound of the route the request's method and path match when the binding bounds it, else the fixed 1 MiB. The answer is refusals.too_large (the route's own first), else today's 413. The request head decides, before the body is read and before the 404, the 405 and the grant check, as the fixed bound does today; a body of unannounced length (chunked) is read up to the bound and one byte more and refused the same way.` Renumber the old steps 2 to 8 as 3 to 9, change the paragraph after the list from `Where steps 4, 6 and 8 read` to `Where steps 2, 5, 7 and 9 read`, and leave `(step 1 of "Precedence")` in "What the servers write" as it is. Nothing outside this page cites a step number (searched: `grep -rn Precedence` over the tree).
  - "What the servers write (k5, k6)" (717-745): a new bullet `**Request size.**` after "Redirects": the bound is `limits.request_bytes` of the matched route, else of the binding, else 1 MiB (`FIXED_REQUEST_BYTES`); it applies to a route the binding names that reads a body, never to a view, a `get` command, a `query`-encoded command or a derived route; a body announced above it is refused before it is read; a Go request of unannounced length is read to the bound plus one byte; the refusal is `refusals.too_large` or `413` with `{"refused": ...}`. Add the difference under "Go and Rust alike": the default `413` words differ (Rust names the announced length, Go does not), as an answer the binding does not write keeps each server's words.
  - "What OpenAPI writes (k4)" (the paragraph at ~728-742 of the current numbering, the last of the section): append `A route the binding bounds carries x-ess-request-max-bytes and the too_large answer as one more response (413 with {"refused": ...} where the binding declares no too_large); a binding that declares neither key writes no new byte.` Use the page's own word for the section if it has moved.
- `website/docs/guides/http-routes.md` (225 lines): a new subsection `### Limit the request body` after "Encoding, statuses, headers and redirects" (117-188) and before "Operation ids and servers" (190), about 20 lines: the two keys in one YAML example (the fixture's `limits: {request_bytes: 64}`, a route override and a `refusals.too_large`), then bullets: which routes it applies to; the ceiling (16 MiB) and floor (1); the default when nothing is declared (1 MiB, unchanged); what `too_large` takes (a `4xx`, `413` when absent, body and headers as the other refusals); that both servers refuse from the request head before the body is read; and `x-ess-request-max-bytes` in the OpenAPI document. In "In the OpenAPI projection" (96-116) add nothing: the new subsection carries it. Plain facts, no version number (the release commit adds "added in"). Front matter `description` unchanged.
- `CHANGELOG.md`: under `## [Unreleased]` / `### Added` (3-6) one bullet, written last, saying: an `ess-http/1` binding declares `limits: {request_bytes: N}` at the top and on a route, and `refusals.too_large`; both generated servers read at most that many bytes of the matched route's body and refuse a larger one, the OpenAPI document names the bound under `x-ess-request-max-bytes`, a binding declaring neither keeps every byte it writes, and `ESS-HTTP-034` refuses a limit outside 1 to 16777216 or on a route that reads no body. One bullet, no sub-list, no version change. (`release-notes/<version>.md` is the release preparer's, not this unit's.)

## Tests first

All in `crates/generate/ess-gen/tests/openapi_http.rs` (one test binary already; no new binary), in the section "Request limits". Each case builds its binding from `ORDERS_ROUTES` with the new keys in front, as `a_refusal_override_replaces_the_standard_answer_on_every_bound_operation` (807) does. Time the new tests; each under 5 s or named with its time.

1. `a_binding_that_declares_no_limit_keeps_every_stage_2_openapi_byte` (green today and after): the documents of the orders service with `ORDERS_ROUTES`, of the auth service with `AUTH_ROUTES` and of the orders service with the `refusals` of test 807 are each passed through `without_digests` (290) and digested with `ess_gen::ess_http::digest`; each digest equals a constant. Capture the constants on the UNMODIFIED tree first (step 1). It joins the existing `a_stage_1_binding_keeps_its_openapi_bytes` (300) and the committed example trees, which are not touched.
2. `a_bounded_route_documents_its_bound_and_the_answer_a_larger_body_gets`: `limits: {request_bytes: 64}` and `ORDERS_ROUTES`. `PlaceOrder` and `NoteOrder` carry `x-ess-request-max-bytes: 64`; each has a `413` response with `x-ess-outcomes: []`, content `application/json` whose schema is `too_large_schema()` as JSON, and a description that starts `No declared outcome:` and names `64`; each response of these two operations carries `x-ess-outcomes` (the bound form). The view `OrderById` has no extension and no `413`. A derived route of the same document (one the binding does not name) equals its operation in the unbound document, as test 807 compares. Red today: `parses: unknown field `limits``.
3. `a_route_limit_and_a_declared_too_large_replace_the_binding_ones`: top-level `limits: {request_bytes: 64}` and `refusals: {too_large: {status: 422, body: {error: {literal: too_big}}, headers: {X-Limit: {literal: '64'}}}}`; `NoteOrder` gets `limits: {request_bytes: 256}` and its own `refusals: {too_large: {status: 413, body: none}}`. `PlaceOrder`: extension 64; no `413`; a `422` response whose body schema is the literal object (`{"type": "object", "additionalProperties": false, "required": ["error"], "properties": {"error": {"type": "string", "const": "too_big"}}}`) and whose header `X-Limit` is `{"type": "string", "const": "64"}` (assert the header's schema; the header description is not pinned). `NoteOrder`: extension 256; a `413` with no `content`; no `422`.
4. `a_too_large_refusal_without_a_limit_documents_the_fixed_bound`: `refusals: {too_large: {status: 413, body: none}}` and no `limits`: `PlaceOrder` and `NoteOrder` carry `x-ess-request-max-bytes: 1048576` (the number is `ess_http::FIXED_REQUEST_BYTES`, asserted by name) and the declared `413`; the view has neither.
5. `a_limit_that_no_route_can_apply_changes_no_byte` (D7): a binding of only the view route of `ORDERS_ROUTES` (and the `get` command route, if the fixture has one) with a top-level `limits: {request_bytes: 64}` generates a document equal, byte for byte, to the same binding without `limits`; the document contains neither `x-ess-request-max-bytes` nor a `413`.
6. `the_description_names_the_bound_only_where_a_route_is_bounded`: the orders document's `info.description` of test 2 contains `x-ess-request-max-bytes`; that of test 5 does not, and still holds both paragraphs it holds today.
No test of this unit starts a server: the two servers embed `bound_json`'s output (`ess-synth` `go/http.rs` 1411 and the Rust equivalent), and h2 and h3 read that document through `GET /openapi.json`.

## Steps

1. On the unmodified tree write test 1 with placeholder constants, run it, copy the digests in, run green. Time `cargo test -p ess-gen --test openapi_http` (once, name the time).
2. Write tests 2 to 6; they fail (red, as above) or do not compile until h1 is on the tree: if h1 has not landed when you start, write them against its accessor names in h1's story and stop at the first red; do not stub the accessors.
3. Implement the helper `bound_request`, the `Operation` field, `answered_by_binding`'s clause, the `bound_responses` entry, `Answer::TooLarge`, `too_large_schema`, `too_large_meaning`, `http::TOO_LARGE` and the description paragraph. Tests 2 to 6 green.
4. Edit the design page, the guide and `CHANGELOG.md` as listed. The design page keeps its tables aligned and its line length; reflow nothing you did not change.
5. Re-time `cargo test -p ess-gen --test openapi_http`; report both times.
6. Pins that must not move, and the tests that show it: every document of `openapi_http`, `openapi_http_presentation`, `openapi`, `openapi_reach`, `openapi_publishes`, `openapi_returns`, `openapi_errors_of_one_wire_name` generated from a binding without a limit; `ess-openapi`'s `adversary_ess_issues_2_u6`; the committed `generated/` trees (their `openapi` documents). None is re-pinned; if one moves, the change is wrong.

## Owned files

- `crates/generate/ess-gen/src/openapi.rs` (everything but the `TooLarge` arm of `refusal_sentence`, which is h1's) and `crates/generate/ess-gen/src/http.rs` (the one constant)
- `crates/generate/ess-gen/tests/openapi_http.rs` (the appended section)
- `docs/design/http-presentation-binding.md` (the sections named above)
- `website/docs/guides/http-routes.md`, `CHANGELOG.md`
- `crates/generate/ess-gen/src/openapi.rs` is also edited by h1 (one arm): h1 lands first, h4 rebases on it. The design page is also edited by h1 (other sections): the two do not touch a line in common; whoever lands second rebases.

## Not yours

- `crates/specify/ess-http` and the Keys table, "Standard refusals", "What compiling checks (stage 2)", "Codes per unit" and "Refinement" sections of the design page, and `website/docs/reference/formats.md` (h1).
- Everything under `crates/generate/ess-synth/` (h2, h3), `Taskfile.yml`, `AGENTS.md`, `.github/workflows/ci.yml`, `ci_lanes.rs` (h2, h3).
- `docs/design/` other pages, `website/docs/reference/` other pages, `diagnostics.md` (it holds no ESS-HTTP rows), `release-notes/` (the release preparer), the version numbers of `Cargo.toml` and `fuzz/Cargo.lock`.
- `AsyncAPI`, the docs projection (`ess_gen::docs`) and `ess-ui` take no binding limit and are unchanged.
- No Python, no shell checker, no `/tmp`; no model call or network request in a test.

## Gate

Per `AGENTS.md` § Gate: a change under `crates/generate/ess-gen` gates `ess-gen`; `docs/`, `website/` and `CHANGELOG.md` add `task test-xtask`; a change to what `ess` prints for a generated document adds the byte-pin targets. Each command through the shim or `slot`; each job ends in 10 minutes unless it names its time; each test under 60 s; the test step within 120 s.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-gen --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-gen --locked --test openapi_http --test openapi_http_presentation --test openapi --test openapi_reach --test openapi_publishes --test openapi_returns --test openapi_errors_of_one_wire_name
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
slot task test-xtask
```

`ess-openapi`'s `adversary_ess_issues_2_u6` and `ess-cli`'s `tutorial_page` are the repository-wide byte pins for generated wording; neither should move. `task test-xtask` is the step `AGENTS.md` names for a `website/`, `docs/` or `CHANGELOG.md` change; time it and name the time (it may not fit 120 s: if it does not, say so rather than raising a timeout). `task site-build` fetches the docs system over the network and is not part of this offline gate: the guide's pull request runs it in CI; do not run it on the shared host. Do not run the whole workspace.

## Acceptance

1. Tests 1 to 6 pass; tests 2 to 6 failed before the change for the reason stated.
2. The documents of tests 1 and 5 are byte for byte what they were: the three digests unchanged, and the limited-but-unbounded binding equal to its limitless twin.
3. In the documents of tests 2 to 4: the bounded operations carry `x-ess-request-max-bytes` with 64, 256 and 1048576 as stated, the declared `422` and `413` responses are as stated, the default `413` is `{"refused": string}` with the described meaning, and the view and the derived route carry neither.
4. `openapi`, `openapi_http_presentation`, `openapi_reach`, `openapi_publishes`, `openapi_returns`, `openapi_errors_of_one_wire_name`, `adversary_ess_issues_2_u6`, `tutorial_page` and `task test-xtask` pass unchanged; no file under `generated/` changed (`git status --short generated` is empty).
5. The design page's Precedence list has nine steps, its cross-reference reads `steps 2, 5, 7 and 9`, and the guide has the subsection `Limit the request body`, whose YAML example uses the orders names the page already uses and only key shapes tests 2 and 3 compile.
6. `CHANGELOG.md` has one new bullet under `Added`; `git diff --stat` shows no other changed line there.
7. The generator timing (`openapi_http`) is reported before and after and is within 5 %.

## Decisions

- The extension name is `x-ess-request-max-bytes`, an annotation like `x-ess-may-invoke`: OpenAPI 3 has no standard field for a request-size limit, and an `x-ess-` field is where this projection states what the model or binding says and OpenAPI cannot. The value is bytes as a JSON integer.
- A bounded route is answered in the bound form (`answered_by_binding` is true for it), so each of its responses carries `x-ess-outcomes`, as a route with a `refusals` override does today. The alternative, appending a `413` to the stage-1 responses, needs a second code path for one response; the bound form already has the path, and it is the form a route with a declared `too_large` would take anyway.
- `bounds_request` is the one question both documents and servers ask (h1). A binding that declares only `refusals.too_large` therefore documents the fixed 1 MiB on every body-reading route it names, because the servers then refuse at 1 MiB with that answer.
- The default `413` of a bounded route is documented as a standard refusal with the schema `{"refused": string}`, which both servers write; the words differ between the servers (Rust names the length) and the document says only what both do. A derived route, and a route that reads no body, document nothing new: the servers' fixed bound still answers them, and the document did not describe it before either (parent D7).
- The renumbering of the Precedence list is done here, in the one unit that owns that section, rather than by an unnumbered inserted step: a reader follows one list. Nothing cites a step number outside the page.
- The guide states no version. The release commit adds "added in 0.x.y", as the design page's tables do for earlier keys.

## Open for the operator

None.

## Changelog

One bullet under `[Unreleased]` / `Added`, as described under Where. This unit writes it; h1, h2 and h3 write none.
