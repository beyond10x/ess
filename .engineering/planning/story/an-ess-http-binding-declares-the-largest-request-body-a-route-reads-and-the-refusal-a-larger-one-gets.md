---
format: aep.planning-md/3
id: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
kind: story
status: active
title: An ess-http/1 binding declares the largest request body a route reads, and the status and body of the refusal a larger one gets, and compiles them to ess-http-ir/1
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi_http.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: crates/specify/ess-http/tests/compile.rs
- confidence: cited
  path: crates/specify/ess-http/tests/fixtures/board.yaml
- confidence: cited
  path: crates/specify/ess-http/tests/fixtures/catalog.yaml
- confidence: cited
  path: crates/specify/ess-http/tests/fixtures/orders.yaml
- confidence: cited
  path: crates/specify/ess-http/tests/route_refusals_and_view_answers.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:31Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:33Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`ess-http/1` takes three new optional keys and `ess-http-ir/1` carries them, with no new format version:

- `limits: {request_bytes: N}` at the top of a binding, and the same under a route (`routes[i].limits`), where the route's own value stands in place of the binding's. `N` is a whole number of bytes from 1 to 16,777,216.
- `too_large` as a fourth key of `refusals:` (binding level and `routes[i].refusals`), with `status` (any `4xx`), `body` and `headers` as `malformed`, `not_granted` and `unfinished` have them.
- A new code `ESS-HTTP-034`, and `ESS-HTTP-029` and `ESS-HTTP-012` extended to the new keys.

`ess specify http compile` writes `limits` into the IR only where the binding declares one, and a binding that declares neither key compiles to the IR bytes it compiles to today. The crate exposes what the two servers and the OpenAPI projection need to read the effective bound of a route, so each of them reads one answer: `Route::reads_body`, `HttpIr::limits`, `HttpIr::request_bytes`, `HttpIr::bounds_request`, `HttpIr::bounds_requests`, `FIXED_REQUEST_BYTES`, `MAX_REQUEST_BYTES`.

## Why

Parent: `story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets`. Both generated servers refuse a body over a fixed 1 MiB with a fixed `413` and a protocol cannot state its own bound or its own refusal. This unit is the language half: the keys, their checks, the IR and the accessors; the servers (h2, h3) and the document (h4) read what it compiles.

## Where

Tree read: the plan tree, `git rev-parse --short HEAD` = `cfe9dabcb85`; the code is that of `e3f502951bd` (the commit between them adds planning files only). Every line number below is read in that tree. All edits in `crates/specify/ess-http/src/lib.rs` (7468 lines) unless named.

- `CODES` (line 36, last entry `"ESS-HTTP-033"` at 64): append `"ESS-HTTP-034"`.
- Constants beside `COLLISION` (line 72): `pub const FIXED_REQUEST_BYTES: u64 = 1_048_576;` (the bound both servers apply today: Rust `MAX_BODY`, Go `maxBody`) and `pub const MAX_REQUEST_BYTES: u64 = 16_777_216;`.
- `RefusalKey` (829): add `TooLarge` between `NotGranted` and `Unfinished` (the enum derives `Ord`, and its order is the order the IR writes keys in: `malformed`, `not_granted`, `too_large`, `unfinished`). Doc: "A request whose body is larger than the route reads; today `413`. Answered with a `4xx`." Then `ALL` (`[Self; 3]` becomes `[Self; 4]`), `as_str` (`"too_large"`), `class` (`TooLarge` is class `4`). Nothing else matches on the enum in this crate; outside it the only exhaustive match is `refusal_sentence` in `crates/generate/ess-gen/src/openapi.rs` (line 1909), see below.
- New types next to `RefusalSpec` (875): `LimitsSpec { request_bytes: i64 }` (authored; `#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]`, `#[serde(deny_unknown_fields)]`, `request_bytes` required: a missing key is the parse error `missing field `request_bytes``, as a missing `status` is) and `Limits { request_bytes: u64 }` (compiled; `Serialize`, `Deserialize`, `deny_unknown_fields`). A non-integer value (`64.5`, `"64"`) is a parse error naming `request_bytes`, exactly as a non-integer `status` is one; it does not reach `ESS-HTTP-034`.
- `RouteSpec` (1545-1597): `#[serde(default)] pub limits: Option<LimitsSpec>` after `refusals`. `HttpSpec` (1828-1850): the same, after `refusals`.
- `Route` (2088-2157): `#[serde(default, skip_serializing_if = "Option::is_none")] pub limits: Option<Limits>` after `refusals`. `HttpIr` (2206-2232): the same field `limits: Option<Limits>` (private, like `refusals`) between `refines` and `refusals`. Accessors beside `refusals()` (2237): `pub fn limits(&self) -> Option<&Limits>`.
- `impl Route` (2161-2185): `pub fn reads_body(&self) -> bool`: the route serves a command, its method is not `Get`, and its request encoding is not `Query` (an absent `request` is `json`). A view, a `get` command (it carries its input in the path or query) and a `query`-encoded command read no body.
- `impl HttpIr` beside `routes()` (2256): `pub fn request_bytes(&self, route: &Route) -> Option<u64>`: `None` when `!route.reads_body()`, else the route's own `limits` over the binding's; `pub fn bounds_request(&self, route: &Route) -> bool`: `route.reads_body()` and (`request_bytes(route).is_some()` or the route's own or the binding's `refusals` hold `TooLarge`); `pub fn bounds_requests(&self) -> bool`: any route of `routes()` bounds its request. These three are the only place the rule "a limit applies to a route that reads a body, and a `too_large` answers there" is written; h2, h3 and h4 call them and never re-derive it.
- `HttpIr::invariants` (called from `from_json` at 2332; its body starts ~2351) and `route_invariants` (2487): refuse, naming the key, a `limits.request_bytes` outside `1..=MAX_REQUEST_BYTES` (`` `limits.request_bytes` `{n}` is not a whole number of bytes from 1 to 16777216 ``), a route `limits` on a route for which `!reads_body()` (`` `{at}.limits` is on {operation}, which reads no request body ``). `refusal_invariants` (2776) needs no change: `class()` already holds `too_large` to `4xx`. `from_json`'s write-back check (2340-2349) already refuses a `limits` the IR would not write.
- `fn standard` (3213-3290): the key-list message (3230-3239) reads `` `{name}` is not a standard refusal; `refusals:` takes `malformed`, `not_granted`, `too_large` and `unfinished` `` and the status message (3246-3258) reads `` ... and `{key}` answers a `{class}xx`: `malformed`, `not_granted` and `too_large` refuse the request (`4xx`), `unfinished` is the server's (`5xx`) ``. `RefusalKey::parse` finds the key through `ALL`.
- New `fn limits(written: Option<&LimitsSpec>, at: &str, refusals: &mut Refusals) -> Option<Limits>` next to `standard`: `at` is `limits` or `routes[i].limits`; refuses `ESS-HTTP-034` at `{at}.request_bytes` with `` `{n}` is not a request size: `limits.request_bytes` is a whole number of bytes from 1 to 16777216 `` (the route's text says `limits.request_bytes` too, the key is spelled the same at both levels), and returns `None` when refused.
- `compile_refining` (3146-3175): `let limits = limits(spec.limits.as_ref(), "limits", &mut refusals);` beside `envelope`, written into `HttpIr { limits, .. }`. `refinement` (3315-3400, where the refined document is checked with `envelope` and `body` and reported under `refined.`): call `limits` on `refined.spec` into `own` the same way.
- `fn routes` (4138-4235): after `carried` (4562) is computed and `request` is known, resolve the route's `limits` with the same `limits` fn at `{path}.limits`; then, when the route is `Some` and `!reads_body` (compute from `operation`, `method`, `request`), refuse `ESS-HTTP-034` at `{path}.limits` with `` {operation} reads no request body ({why}); `limits.request_bytes` belongs on a command route that carries its input in a `json` or `form` body `` where `{why}` is `a view takes no request`, `` `get` carries none `` or `` `request: {encoding: query}` carries the input in the query string ``. Set `Route.limits` where the route is built (4197-4213). A `too_large` under `routes[i].refusals` is NOT refused on a route that reads no body: `standard` takes no route, and no key of the set is refused for the kind of route it is on today.
- `fn changes` (3611-3703) and `refusal_changes` (3705-3770): `refusal_changes` compares by key string, so `too_large` is covered with no change. Add `limits_changes` called from `changes` for the top level (`limits`) and per refined route beside `refusal_changes` (3691): `ESS-HTTP-012` at `limits` or `routes[i].limits`, naming the compared values; a limit added, dropped or restated is a change. Message: `` `limits` is {new} here, and the refined binding fixes {old}; a refinement restates the refined binding's limits exactly `` and at a route `` {operation} bounds its request at {new} here, and the refined binding fixes {old} at `routes[{at}]`; a refinement restates the limits of every route it restates ``, where a value is `` `request_bytes: N` `` or `no limit`. The comparison is over the written integers (a `0` is `ESS-HTTP-034` in both documents, so it never reaches the comparison).
- Documentation comments in `lib.rs` that list the codes: the table in the `compile` doc (3090-3137: the `ESS-HTTP-029` row at 3102 becomes `at refusals.<key>, or a route's own routes[i].refusals.<key>: a key outside malformed, not_granted, too_large and unfinished ... 4xx for malformed, not_granted and too_large`; add an `ESS-HTTP-034` row after the 033 row at 3106; the `ESS-HTTP-012` row at 3137 gains `at limits or routes[i].limits: a request limit added, dropped or restated`); the doc at 369 and at 786 that name the refusal keys.
- `crates/specify/ess-http/tests/compile.rs` 1255-1305, `every_code_this_crate_can_emit_is_one_of_the_codes_built_so_far`: add `| "ESS-HTTP-034"` to the `matches!` list. Its second half (the source's own `"ESS-HTTP-` literals equal `CODES`) holds the list to the source both ways.
- `crates/generate/ess-gen/src/openapi.rs` `refusal_sentence` (1909-1927): add the arm `ess_http::RefusalKey::TooLarge => "a request whose body is larger than the route reads"`. This is the only edit h1 makes outside `ess-http`; it keeps the workspace compiling. h4 owns the rest of that file and lands after h1.
- `docs/design/http-presentation-binding.md`: the Keys table (593-639: add rows `limits.request_bytes` and `routes[].limits.request_bytes`, `refusals.<key>` row at 614 gains `too_large`); "Standard refusals" (530-543: `refusals:` takes four keys; one paragraph "Request size" after the route paragraph says what `limits` bounds, which routes it applies to (the routes the binding names and that read a body: a `json` or `form` command not on `get`; never a view, never a derived route), the ceiling, that `too_large` answers there with a `4xx` of its own and `413` when absent, and that a binding declaring neither keeps today's fixed 1 MiB and its answers); "What compiling checks (stage 2)" table (545-564: `ESS-HTTP-029` row at 558 gains the fourth key and the class text; add the `ESS-HTTP-034` row); "Codes per unit" (640-660: a row `| request limits | ESS-HTTP-034 | ESS-HTTP-029 (key set), ESS-HTTP-012 (limits) |`); "Refinement" (661-669: one clause for `limits`). The Precedence list, the servers section and the OpenAPI section are h4's.
- `website/docs/reference/formats.md` line 228 (one very long cell): after the sentence that ends `its own `refusals:` standing in place of the top-level entries on that route alone.` add the `limits` and `too_large` sentence (the same wording as the design page's "Request size", two sentences); in the list of codes at the end of the cell, `` `029` (refusals) `` stays and `` and `034` (request limits) `` is added after `` `033` (servers) ``. The version of the release is not known here: write the new sentences without a version and leave "added in 0.x.y" to the release commit.

## Tests first

All eight tests are appended to the existing `crates/specify/ess-http/tests/route_refusals_and_view_answers.rs`, a new section "Request limits" after its last test. No new test binary: that file already holds every helper the cases use (`model`, `derived`, `header`, `compiled`, `located`, `refused`, `ir_json`, `round_trips`, `two_commands`, `view_route`, lines 20-123) and the fixture `tests/fixtures/orders.yaml` (commands `catalog.orders.PlaceOrder`, `catalog.orders.NoteOrder`; views `catalog.orders.OrderById`, ...), the crate has 22 test binaries already, and a new file would copy the helpers (about 100 lines) and add a link. Where the fixture has no command that fits a case (a `get` command whose input the path carries, a `delete` command with `request: {encoding: query}`), add a second constant `BOARD: &str = include_str!("fixtures/board.yaml")` in the section and compile that case against it with the helpers' own `model` shape, as `request_body.rs` does; `catalog.yaml` if `board.yaml` lacks the shape. Each test is milliseconds; report the runner's own figure for the file (`cargo test -p ess-http --test route_refusals_and_view_answers`, test time only) at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`), on a warm build.

1. `a_binding_limit_and_a_route_limit_compile_and_round_trip`: `limits: {request_bytes: 64}` at the top and `limits: {request_bytes: 256}` on the `PlaceOrder` route compile; the IR JSON holds `limits` at the top (`{"request_bytes": 64}`) and on that route only; `round_trips`; `request_bytes(route)` is 256 for `PlaceOrder` and 64 for `NoteOrder`. Red today: panics `parses: unknown field `limits``.
2. `a_limit_is_a_whole_number_of_bytes_from_one_to_the_ceiling`: `0`, `-1`, `16777217` at the top level are `ESS-HTTP-034` at `limits.request_bytes` and at `routes[0].limits.request_bytes` on a route; `1` and `16777216` compile; `64.5` and `"64"` fail `HttpSpec::from_yaml` with a message naming `request_bytes`; `limits: {}` fails with `missing field `request_bytes``; `limits: {request_bytes: 64, extra: 1}` fails with `unknown field `extra``. Pin the message text of the `034` diagnostic for `0` with `assert_eq!`, and assert that `0` at the top level yields exactly one diagnostic, `(ESS-HTTP-034, limits.request_bytes)`, through `located` (this is the test that holds acceptance 3). Red today: unknown field `limits`.
3. `a_route_limit_on_a_route_that_reads_no_body_is_refused`: a `limits` on a view route; on a `get` command (all input in the path); on a command with `request: {encoding: query}` on `delete`: each `ESS-HTTP-034` at `routes[i].limits` and nothing else; the same three routes without a route `limits` but under a binding `limits` compile, and `request_bytes` is `None` for each (the binding's limit applies where a body is read, and is not an error elsewhere). A `post` command with `request: {encoding: form}` and a `delete` command with `json` accept a route `limits`.
4. `the_fourth_refusal_is_too_large_and_takes_any_4xx`: `refusals: {too_large: {status: 413, body: none}}` and `{status: 422, body: {error: {literal: too_big}}, headers: {X-Limit: {literal: '64'}}}` compile and round-trip; `status: 500` is `ESS-HTTP-029` at `refusals.too_large.status`; `too_large` written twice is `ESS-HTTP-029` at `refusals.too_large`; `too-large` is `ESS-HTTP-029` and its message lists the four keys; under `routes[i].refusals` the same; a `too_large` on a view route is accepted (it never answers). Red today: `refusals:` refuses the key.
5. `a_binding_that_declares_neither_key_keeps_its_ir_bytes`: compile the existing `two_commands("")` document and a binding with the bound routes of `route_refusals_and_view_answers.rs`; assert the canonical JSON holds no `limits` anywhere and that `ess_http::digest(ir.to_canonical_json().as_bytes())` equals a constant. Capture the constants by running the test on the UNMODIFIED tree before any change (step 1 below); the test is green today and must stay green.
6. `the_effective_request_bound_is_one_answer`: for a binding with `limits` and a `refusals.too_large` on one route: `reads_body`, `request_bytes`, `bounds_request` and `bounds_requests` as listed under Where (a `too_large` with no `limits` bounds the route: `request_bytes` is `None`, `bounds_request` is true; a binding of neither has `bounds_requests() == false`; a limit declared only for views and bodiless routes at binding level gives `bounds_requests() == false`). `FIXED_REQUEST_BYTES == 1_048_576` and `MAX_REQUEST_BYTES == 16_777_216`.
7. `a_refinement_restates_limits`: refined binding with `limits` and a route limit; a refining document that drops, changes or adds either is `ESS-HTTP-012` at `limits` or `routes[i].limits`; one that restates both compiles. Use `compile_refining` and `Refined` as `a_refinement_restates_route_refusals` (same file, line 228) does.
8. `compiled_ir_with_limits_out_of_rules_is_refused`: `HttpIr::from_json` refuses, naming the key: `limits.request_bytes` `0` and `16777217`; a route `limits` on a view route and on a `get` command; a `limits` written `{}`; a `too_large` refusal with a `5xx` status; an IR with `limits` and `"request_bytes": 64.0` (the write-back check). Model it on `compiled_ir_with_view_answers_out_of_rules_is_refused` (line 345 of `route_refusals_and_view_answers.rs`).

Also the existing `every_code_this_crate_can_emit_is_one_of_the_codes_built_so_far` (`compile.rs:1255`): add `ESS-HTTP-034` to its list FIRST; it passes (the list is a superset), and then fails with "a code in the source is not in CODES" until the code is in `CODES`, which holds the two together.

## Steps

1. On the unmodified tree, write test 5 with placeholder constants, run it, copy the printed digests into it, run it green. Commit nothing yet.
2. Write tests 1-4 and 6-8 and the edit to `compile.rs`; run `cargo test -p ess-http --test route_refusals_and_view_answers`; they fail as listed.
3. Types, `RefusalKey::TooLarge`, `limits` fn, route resolution, IR fields and invariants, accessors, `changes`.
4. The `refusal_sentence` arm in `ess-gen`.
5. Docs: the design page and `formats.md` as listed.
6. Run the gate. Pins that move: `CODES` and the `matches!` list in `compile.rs` (claimed: new code); `docs/design/review-typed-diagnostics.md` and `macro-guards.json` do NOT move because `ESS-HTTP-0xx` codes are strings, not `ValidationCode` (read `crates/specify/ess-http/src/lib.rs:36`: it is a `&[&str]`); `task test-xtask` still runs, as AGENTS.md says for a code and `website/` and `docs/` files. No `*_TABLE_WRITE` model table moves: a binding is not a model. If any committed IR or OpenAPI byte moves, it is a defect of this unit: stop and find why (every committed example has no binding).
7. Report the runner's own figure for the test file, at the load named under Tests first. No generator gains a dependency here; h2, h3 and h4 decide their output by counts.

## Owned files

- `crates/specify/ess-http/src/lib.rs`
- `crates/specify/ess-http/tests/route_refusals_and_view_answers.rs` (the appended section), `crates/specify/ess-http/tests/compile.rs` (the one list)
- Read as pins, not edited: `crates/specify/ess-http/tests/fixtures/orders.yaml`, `board.yaml`, `catalog.yaml`; `crates/generate/ess-gen/tests/openapi_http.rs` and `openapi_http_presentation.rs` (run in the Gate)
- `docs/design/http-presentation-binding.md`: Keys table, "Standard refusals", "What compiling checks (stage 2)", "Codes per unit", "Refinement"
- `website/docs/reference/formats.md`
- `crates/generate/ess-gen/src/openapi.rs`: the single arm in `refusal_sentence` (1909) and nothing else

## Not yours

- Every file under `crates/generate/ess-synth/` (h2, h3), the rest of `crates/generate/ess-gen/src/openapi.rs` and `crates/generate/ess-gen/tests/` (h4), `Taskfile.yml`, `AGENTS.md`, `CHANGELOG.md` (h4 writes the one bullet).
- The Precedence list, "What the servers write" and "What OpenAPI writes" of the design page, and `website/docs/guides/http-routes.md` (h4).
- `ess-conformance`: the conformance suite is not extended; it does not speak HTTP (design page "Stage 3" is another story). `ess-ui` reads routes only and is unchanged.
- No new format version, no `ess-http/2`, no `ess-http-ir/2`, no generic registry or property bag (AGENTS.md § Boundaries).
- The synthesis-sources stamp file and its fast digest test belong to t2, which lands after this unit and stamps the tree that holds it (this unit changes `src/` of `ess-http` and `ess-gen`, both hashed). Do not write the stamp and do not run that test in the Gate.
- Do not change the `ValidationCode` list, the schema `schemas/generated/ess.schema.json` (`RawSpecFile` is untouched: `HttpSpec` is a different type) or `crates/specify/ess-http/tests/fixtures/*`.

## Gate

Run the commands one after another, each through the shim or `slot`; none may run longer than 10 minutes unless it names its time. Every test stays under 60 s and each test step under 120 s of test time (the runner's own figure, at a 1-minute load of 60 or less, on a warm build).

```console
slot task fmt-check
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-http --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-http --locked
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-gen --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-gen --locked --test openapi_http_presentation --test openapi_http
SLOT_TIMEOUT=30m slot task test-xtask
```

Test files run by `cargo test -p ess-http`: every file under `crates/specify/ess-http/tests/` (small; the whole crate is the touched crate). No Python, no shell checker.

## Acceptance

1. `cargo test -p ess-http --test route_refusals_and_view_answers` passes its eight new tests and its existing ones, each of the new under 5 s at the stated load.
2. The pre-existing `ess-http` tests pass unchanged, except the one added list entry in `compile.rs`; `a_binding_that_declares_neither_key_keeps_its_ir_bytes` passes with constants captured before the change.
3. `a_limit_is_a_whole_number_of_bytes_from_one_to_the_ceiling` (target `route_refusals_and_view_answers`) passes: compiling a binding with `limits: {request_bytes: 0}` yields exactly one diagnostic, `ESS-HTTP-034` at `limits.request_bytes`, with the pinned message. `ess specify http compile` prints that diagnostic from the same `compile`; no gate builds the CLI, so no acceptance runs it.
4. `HttpIr::from_json` on the compiled IR of test 1 reads it and writes the same bytes; on IR with a route `limits` on a view it refuses naming `limits`.
5. `ess_http::CODES` ends with `"ESS-HTTP-034"` and `every_code_this_crate_can_emit_is_one_of_the_codes_built_so_far` passes.
6. `cargo test -p ess-gen --test openapi_http --test openapi_http_presentation` passes: nothing in a document generated without `limits` or `too_large` moved.
7. `slot task test-xtask` passes (the docs check reads the edited design page and `formats.md`).

## Decisions

- Keys are `limits: {request_bytes: N}` at binding and route level, an object so later keys fit (parent D1). Route stands in place of binding, as route `refusals` do.
- `request_bytes` is a typed `i64` in the authored form and a `u64` in the IR, like `status` (`i64`/`u16`): a non-integer is a parse error, not `ESS-HTTP-034`; `034` is for range and for a route that reads no body.
- A binding-level `limits` is never an error where no route reads a body, and applies to the routes the binding names that read a body; it does not bound a derived route (as `refusals:` does not, design page "Precedence", last paragraph). A route-level `limits` on a route that reads no body is `ESS-HTTP-034`: it states a bound that can never apply.
- "Reads a body" is: serves a command, method not `get`, encoding not `query`. A `delete` with `json` reads one.
- `too_large` takes any `4xx` (default when absent: `413`), body `none` or one literal member, headers as the other keys (parent D2).
- Ceiling 16 MiB, floor 1 byte (parent D4); the fixed bound when nothing is declared stays 1 MiB and is exported as `FIXED_REQUEST_BYTES` so both servers and the OpenAPI projection read one number (parent D5).
- `HttpIr::bounds_request` counts a declared `too_large` as bounding a body-reading route even with no `limits` (the 1 MiB bound then answers with the declared refusal).
- No new format version: `ess-http/1` and `ess-http-ir/1` already add optional keys this way (design page "Formats and determinism (stage 2)"; `deny_unknown_fields` makes an older reader refuse the key by name, which is the contract).
- `ess-ui` and `--target web` are unaffected: neither reads limits, and the web target takes no `--http` (`models/toolchain/domains/generate.yaml:199`).

## Open for the operator

None.

## Changelog

none (h4 writes the single `### Added` bullet for the whole feature in `CHANGELOG.md`).
