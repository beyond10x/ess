---
format: aep.planning-md/3
id: story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
kind: story
status: draft
title: The generated Rust server reads at most the request body its binding declares for the matched route, and answers a larger one with the refusal the binding declares
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- depends_on: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/body-cap-rust/http.yaml.in
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/body-cap-rust/model.yaml.in
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/body_cap_rust.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/shared.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding_rust.rs
revision: 10
---
## Outcome

A Rust server generated with `ess generate synthesize --target rust --http FILE` for a binding that declares `limits` or `refusals.too_large` reads at most the declared number of bytes of the matched route's request body, and answers a larger one with the declared `too_large` refusal (`413` with `{"refused": "..."}` when the binding declares limits and no refusal), before it reads the body. Three things stay true:

- A binding that declares neither key, and every synthesis without a binding, writes byte for byte the files it writes today.
- A route that reads no body, a derived route, an unrouted path and a path that matches no row keep today's fixed 1 MiB bound and today's `413`.
- A request that announces no `Content-Length` has no body to read (the server reads exactly `Content-Length` bytes and answers `411` to chunked transfer), so nothing is cut: the bound compares the announced length, and there is no unannounced length in this server.

## Why

Parent: `story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets` (decisions D5, D6). The Rust server's `http::read` compares the announced length with the constant `MAX_BODY` before it allocates (`rust/http.rs:2037`); a binding cannot move that bound or its answer. Needs h1 (the keys and `HttpIr::request_bytes`, `bounds_request`, `bounds_requests`, `FIXED_REQUEST_BYTES`).

## Where

Tree read: the plan tree, `git rev-parse --short HEAD` = `cfe9dabcb85` (code as of `e3f502951bd`; the difference is planning files). Lines are read in that tree. Files under `crates/generate/ess-synth/`.

- `src/rust/http.rs`
  - `server_crate` (124-230): the `http.rs` artifact is `bound::http_module(HTTP)` + `QUERY` + `bound::BOUND` when a binding is given (176-196). In capped mode (below) pass the module text through the new `bound::capped_read` first.
  - `HTTP` const (1769): `MAX_BODY` at 1788; `pub fn read` at 1943-2057 with the `413` block at 2037-2042 (`if length > MAX_BODY { return Err(Response::refusal(413, &format!("the body is {length} bytes and this surface reads at most {MAX_BODY}"))); }`) and `reason` at 2130-2135. This text is NOT edited: uncapped output must keep its bytes.
  - Surface module assembly (440-485): after `bound::dispatch` (465) call the new `bound::body_limit` and, from the existing `if bound { bound::refusals(...) }` (481-483), the refusals gain the `too_large` function (below). Both only in capped mode.
  - `SERVE_BODY` (849-880) calls `http::read(&mut reader)` at 864, once; `serve_with_static` builds its body from it with a chain of `.replace` at 766. In capped mode add one more replace before that chain: `http::read(&mut reader)` to `http::read(&mut reader, &body_limit)`.
- `src/rust/http/bound.rs`
  - `http_module` (35-165): the existing pattern of text rewrites of the `HTTP` const (a list of `(from, to)` pairs applied with `replace`). Add `pub(super) fn capped_read(module: &str) -> String` in the same style, applied after `http_module` in capped mode. It must (a) replace the signature `pub fn read(reader: &mut std::io::BufReader<std::net::TcpStream>) -> Result<Request, Response> {` by the same with a second parameter `limit: &dyn Fn(&str, &str) -> Option<(usize, fn(usize, usize) -> Response)>`, (b) replace the `413` block by `let (cap, too_large): (usize, fn(usize, usize) -> Response) = limit(&method, &path).unwrap_or((MAX_BODY, body_too_large)); if length > cap { return Err(too_large(length, cap)); }`, (c) add before the doc comment of `head_line` the function `pub fn body_too_large(length: usize, cap: usize) -> Response { Response::refusal(413, &format!("the body is {length} bytes and this surface reads at most {cap}")) }`. Assert each replace found its anchor (`assert!(replaced != module)` or a `panic!` naming the anchor), so a later edit of the `HTTP` const fails loudly rather than silently writing the uncapped read. The wording of the default `413` is today's, with the cap in place of `MAX_BODY`.
  - `dispatch` (637-703): rows come from `table(routes, ir)` in `http::dispatch_order(&keys)`; `routes` carries `declared: Option<&ess_http::Route>`. Reuse this walk.
  - New `pub(super) fn body_limit(out, server, routes)`: emits `fn body_limit(method: &str, path: &str) -> Option<(usize, fn(usize, usize) -> http::Response)>` with one block per row of `table` in `dispatch_order`, in that order, so a literal row answers before a variable row exactly as `dispatch` does: `if http::matched(<path>, path).is_some() { if method == <METHOD> { return <result>; } }`. `<result>` is `Some((<cap>, refusal_too_large<suffix>))` for a row whose route is declared and `ir.bounds_request(declared)` (cap is `ir.request_bytes(declared)` or `FIXED_REQUEST_BYTES`, written as an integer literal), and `None` for every other row (documents, derived, declared routes that read no body or bound nothing), so a later, more general row cannot capture a request an earlier row answered. The last line is `None`. The path argument is the raw request path as `dispatch` matches it. Emit `#[allow(clippy::...)]` nothing; the file is generated and not clippy-gated.
  - `refusals` (1322-1384) and `RefusalSet` (~1374) and `refusal_set` (1386-1474): add `too_large: bool` to `RefusalSet`. For the shared set it is true when any `shared` route `ir.bounds_request(declared)`; for a route's own set, when that route does. When true, `refusal_set` emits `fn refusal_too_large{suffix}(_length: usize, _cap: usize) -> http::Response` with `overridden(refusal)` (1500) when `lookup(RefusalKey::TooLarge)` is `Some`, else `fn refusal_too_large{suffix}(length: usize, cap: usize) -> http::Response { http::body_too_large(length, cap) }`. Emit it only when true: a private function nothing calls is a `dead_code` warning, and the generated crate builds with warnings denied. The doc comment names `{from}refusals.too_large`.
  - `answers_standard_not_granted` (1476-1498) and `refusal_suffix` (788-800) are read, not changed.
- `Taskfile.yml`: new var after `UI_HTTP_ITEMS_SERVER` (line 16): `BODY_CAP_RUST_SERVER: '{{.ROOT_DIR}}/{{.NEXTEST_ARCHIVES}}/body-cap-rust-server'`; new task `body-cap-rust-server` after `ui-http-servers` (264-283), modelled on it; `- task: body-cap-rust-server` in `test` (89-94), `test-packages` (119-124) and `test-archive-workspace` (157-162); `ESS_BODY_CAP_RUST_SERVER: '{{.BODY_CAP_RUST_SERVER}}'` in the `env:` of `test` (81-87), `test-packages` (110-116) and `test-shard` (346-353).
- `AGENTS.md` lines 234-237 (the harness paragraph): add `` `task body-cap-rust-server` (`ess-synth`'s `body_cap_rust` cases, through `ESS_BODY_CAP_RUST_SERVER`) `` to the list.
- `tests/http_binding_rust.rs`: after the module declarations at 14-18 add `#[path = "http_binding/body_cap_rust.rs"] mod body_cap_rust;` (the same shape as `go`, line 17). One test binary, as `go.rs` is a module of it.

## Tests first

New module `tests/http_binding/body_cap_rust.rs` (a module of `http_binding_rust`, not a binary of its own; the file spawns no cargo, so it is not added to `ALLOWED` in `no_test_builds.rs`) and two fixtures `tests/fixtures/body-cap-rust/model.yaml.in` and `http.yaml.in` (the `.yaml.in` suffix keeps model walkers off them, as `json-body-closure/` does).

Fixture model (`model.yaml.in`; copy the shape of `crates/edge/ess-cli/tests/fixtures/ui-http/items/system.yaml`, which validates; no actors, so no `--callers`):

- `system: parcel`, `domain: parcel.intake`, `format: ess/23`; type `parcel.intake.ParcelId` newtype of `Integer`; entity `parcel.intake.Parcel` (identity `parcel_id`, field `note: String`, lifecycle `{initial: Held, states: [Held], terminal: [Held]}`); event `parcel.intake.ParcelHeld {parcel_id, note}`; three commands `parcel.intake.Receive`, `parcel.intake.Label`, `parcel.intake.Discard`, each `input: [parcel_id, note]` and one outcome `held` that `creates: parcel.intake.Parcel`, `instance: parcel_id`, `sets: {note: input.note}`, `emits: [parcel.intake.ParcelHeld]`, with the payload from the two inputs; view `parcel.intake.AllParcels` (source `Parcel`, fields `parcel_id`, `note`, `consistency: read_your_writes`); component `parcel` owning the domain, accepting the three commands, publishing the event, `reached_by: network`.

Fixture binding (`http.yaml.in`; `source_digest` is the model's, which the test holds it to as `json_body_closure.rs` `binding()` does, line 163):

```yaml
type: ess-http/1
specification:
  system: parcel
  version: v1
  source_digest: sha256:<the model's digest>
limits: {request_bytes: 64}
refusals:
  too_large: {status: 422, body: {error: {literal: too_big}}, headers: {X-Limit: {literal: '64'}}}
routes:
  - {command: parcel.intake.Receive, method: post, path: /parcels}
  - command: parcel.intake.Label
    method: post
    path: /labels
    limits: {request_bytes: 256}
    refusals:
      too_large: {status: 413, body: none}
  - {view: parcel.intake.AllParcels, method: get, path: /parcels}
```

`Discard` is not named, so it keeps its derived route. Tests (each starts nothing itself but a child process of the built server; time the module and report it; every test under 60 s, the module far under 120 s):

1. `a_binding_that_declares_no_limit_keeps_the_rust_server_bytes` (digest pin; green today and after): synthesize the shop fixture with `shared::binding(&ir, ROUTES)` for `Target::Rust` in `OutputLayout::Workspace` and `Crate`, join every artifact as `path`, `\n`, `contents` in path order, and assert `ess_gen::ess_http::digest(joined.as_bytes())` equals a constant. Capture the constants on the UNMODIFIED tree first (step 1).
2. `the_fixed_bound_is_the_one_the_binding_crate_exports` (green today; guards drift): the uncapped `http.rs` of the shop fixture contains `pub const MAX_BODY: usize = 1_048_576;` and `ess_http::FIXED_REQUEST_BYTES == 1_048_576`. Compiles only after h1; write it in step 3.
3. `a_capped_surface_reads_through_its_body_limit`: synthesize the fixture (`Target::Rust`, `OutputLayout::Crate`); the server crate's `http.rs` contains `limit: &dyn Fn(&str, &str)` and `pub fn body_too_large`; the surface file contains `fn body_limit(` and `fn refusal_too_large`; `Receive` and `Label` get `Some((64, ...))` and `Some((256, ...))`. The same synthesis of the shop fixture contains none of `body_limit`, `body_too_large` and `limit: &dyn`. Red today: the fixture binding does not compile (h1 not built) or, once it does, nothing is emitted.
4. Socket cases, one `#[test]` per case so a failure names the case, all against the server binary `ESS_BODY_CAP_RUST_SERVER` names (driver below). Bodies are built by padding `note` so the whole body is exactly N bytes; assert `body.len()` before sending. `baseline` is the status the server answers to a 10-byte-note `Receive` with a fresh `parcel_id`.
   - `a_body_of_the_cap_is_read_and_one_byte_more_is_refused`: `Receive` with a 64-byte body answers `baseline` and is not `422`; with a 65-byte body answers `422`, the body `{"error":"too_big"}` and the header `X-Limit: 64`.
   - `a_refusal_needs_no_body_read`: `Receive` with `Content-Length: 65` and no body bytes sent answers `422` (not `400 the body was shorter than Content-Length announced`), which shows the check runs before the read.
   - `a_route_limit_stands_in_place_of_the_binding_limit`: `Label` with a 256-byte body is accepted; with 257 answers `413` with no body (`Content-Length: 0`, no `Content-Type`), the route's own `too_large`; `Label` with a 65-byte body is accepted (the binding's 64 does not apply).
   - `a_route_that_reads_no_body_and_a_derived_route_keep_the_fixed_bound`: `GET /parcels` with 65 body bytes answers `200`; `POST` to `Discard`'s derived path (read from the server's `GET /openapi.json`: the path whose operation has `operationId` `parcel.intake.Discard`) with `Content-Length: 1048577` and no body answers `413` with the body `{"refused":"the body is 1048577 bytes and this surface reads at most 1048576"}`; the same on an unrouted path `/nowhere`.
   A surface that declares nothing is covered by test 1 and by the existing `without_a_binding_the_rust_server_keeps_its_bytes` (`http_binding_rust.rs:34`), not by a socket case.

Driver (in the module, ~60 lines; model it on `server`/`serve` of `crates/edge/ess-cli/tests/generate_ui_http.rs` 463-560): read `ESS_BODY_CAP_RUST_SERVER`, panic naming `task body-cap-rust-server` when unset or missing (the message shape of `generate_ui_http.rs` `server`), make the file executable, spawn it with `--listen 127.0.0.1:0` and piped stdout, read lines until a JSON record with `runtime.address`, kill on drop (`Served`). One server for all cases in a process under `nextest` is not possible (a process per test): each case starts its own server (about 10 ms); that is cheap, and it is not an expensive product built per test. Send raw HTTP/1.1 over `TcpStream` with a 10 s read timeout, write the head and the body bytes, parse the status line and headers, and read the body by its `Content-Length` (every answer this server writes carries one; the server also half-closes after an answer, `http::linger`, `rust/http.rs:2092`).

## Steps

1. On the unmodified tree write test 1 with a placeholder constant, run it, copy the digests in, run green. Time the Rust synthesis for the shop fixture (`target/debug/ess generate synthesize --target rust --http ... --out <scratch dir>` three runs, started directly without the `slot` prefix since it measures) and write the median down.
2. Write the fixtures and tests 3 and 4 (and the module declaration); `cargo test -p ess-synth --test http_binding_rust body_cap_rust` fails (red, as above).
3. Implement `capped_read`, `body_limit`, the `refusal_set` change and the `serve` replace; capped mode is `server.binding.is_some_and(|binding| binding.ir().bounds_requests())`, computed once in `server_crate`/`Server` and passed, never re-derived per site.
4. Add the Taskfile var, task, wiring and the `AGENTS.md` line. The task: `cargo build --workspace --exclude ess-xtask --locked --bin ess`; `rm -rf target/body-cap-rust-server`; `mkdir -p {{.WORK}} {{.NEXTEST_ARCHIVES}}`; `target/debug/ess generate synthesize --target rust --layout crate --path {{.FIXTURES}}/model.yaml.in --http {{.FIXTURES}}/http.yaml.in --out {{.WORK}}/parcel` (the `.yaml.in` model has to be copied to a `.yaml` name first, as `json-body-harnesses` copies `model.yaml.in` to `shelf.yaml`); then, as `ui-http-servers` does (`env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS -u RUSTC_WRAPPER CARGO_INCREMENTAL=0`), `RUSTFLAGS='-D warnings'` and `cargo build --offline --quiet --features server --bin parcel-server --manifest-path {{.WORK}}/parcel/Cargo.toml --target-dir {{.WORK}}/target`; copy the binary to `{{.BODY_CAP_RUST_SERVER}}`; remove `{{.WORK}}`. No `RUSTC_WRAPPER`, `SCCACHE_*` or `-j` is set (AGENTS.md § One compile cache): `env -u` unsets the variable and leaves the host's `rustc-wrapper`. If the server binary's name differs from `parcel-server`, read it in the generated `Cargo.toml`. If `-D warnings` turns an existing unrelated generated warning red, report the warning and drop the flag for this task; do not edit generated text of other features.
5. Run `ESS_BODY_CAP_RUST_SERVER=<path> cargo test ...` after `slot task body-cap-rust-server`; then the gate.
6. Re-time the synthesis of step 1; report both times. The generator gained a text rewrite and one table; a jump over 5 % is fixed in this change.
7. Pins that must not move, and the tests that show it: the committed `generated/rust/{billing,gatepass}` trees (`without_a_binding_the_rust_server_keeps_its_bytes`, `crates/generate/ess-synth/tests/http_binding_rust.rs:34`; `feasibility`; `adversary_related_guard_behaviour_pass1`); the bytes of a bound synthesis that declares nothing (test 1); the byte-pin targets of `AGENTS.md` for `ess-synth`: `served_codec_names` and `go`, and `http_binding_rust` with its `go` module, which compare committed example trees byte for byte. None of these is re-pinned; if one moves, the change is wrong.

## Owned files

- `crates/generate/ess-synth/src/rust/http.rs`, `crates/generate/ess-synth/src/rust/http/bound.rs`
- `crates/generate/ess-synth/tests/http_binding/body_cap_rust.rs` (new), `crates/generate/ess-synth/tests/fixtures/body-cap-rust/` (new), `crates/generate/ess-synth/tests/http_binding_rust.rs` (one module declaration)
- `Taskfile.yml` and `AGENTS.md`: the lines named above. h3 edits the same two files; h2 lands first, h3 rebases and keeps both sets of lines (adjacent additions in the same lists).

## Not yours

- Everything under `crates/generate/ess-synth/src/go/` and `tests/http_binding/go.rs`, `tests/fixtures/body-cap-go/` (h3); `.github/workflows/ci.yml` and `ci_lanes.rs` (h3 only: the Rust task needs no CI change, the archive job already builds Rust and uploads `target/nextest-archives`).
- `crates/specify/ess-http` (h1), `crates/generate/ess-gen` (h4), `CHANGELOG.md` (h4), the design page and the guide (h1, h4).
- The conformance suite (`ess-conformance`) is not extended; it does not speak HTTP. The TypeScript web target takes no `--http` (`models/toolchain/domains/generate.yaml:199`) and is unchanged, as is `ess-ui`.
- The shop fixture, `shared.rs` and `go.rs`: read them, do not edit them. The existing cargo-spawning tests stay on the `ALLOWED` list; do not touch it.
- No Python, no shell checker, no `/tmp`; no model call or network request in a test.

## Gate

Per `AGENTS.md` § Gate, a change under `crates/generate/ess-synth` gates `ess-synth` and its byte-pin targets. Each command through the shim or `slot`; each job ends in 10 minutes unless it names its time; each test under 60 s; the test step within 120 s (the generator tests of `ess-synth` are the large ones: run only the named targets).

```console
slot task fmt-check
slot task body-cap-rust-server
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-synth --all-targets --locked -- -D warnings
ESS_BODY_CAP_RUST_SERVER=$PWD/target/nextest-archives/body-cap-rust-server SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test http_binding_rust --test feasibility --test adversary_related_guard_behaviour_pass1 --test served_codec_names
```

`http_binding_rust` runs its existing cargo and `go` cases too (go is on `PATH` on this host); run it as one target. The `go` target of `ess-synth`'s byte pins is `--test go`; add it to the last command. `ess-openapi`'s `adversary_ess_issues_2_u6` and `ess-cli`'s `tutorial_page` are not touched by this unit (no OpenAPI, no doc change). Do not run the whole workspace.

## Acceptance

1. `slot task body-cap-rust-server` builds `target/nextest-archives/body-cap-rust-server` with `RUSTFLAGS='-D warnings'`.
2. The four socket tests of test 4 pass: 64 bytes accepted and 65 answered `422` with the declared body and `X-Limit: 64`; `Content-Length: 65` without a body answered `422`; `Label` 256 accepted and 257 answered `413` without a body; a view with 65 body bytes answered `200`; a derived route and an unrouted path with `Content-Length: 1048577` answered `413` with today's wording at the cap 1048576.
3. Tests 1 and 2 pass before and after (digest of the declaring-nothing synthesis unchanged; `MAX_BODY` equals `FIXED_REQUEST_BYTES`).
4. `without_a_binding_the_rust_server_keeps_its_bytes`, `feasibility`, `adversary_related_guard_behaviour_pass1`, `served_codec_names`, and `go` pass unchanged; no file under `generated/` changed (`git status --short generated` is empty).
5. The fixture synthesis contains `body_limit` and the shop synthesis does not.
6. The two generator timings are reported and within 5 %; every new test is under 5 s, or is named with its time.
7. `cargo test -p ess-xtask --test no_test_builds` passes (run it by name; it reads the tree): the new module spawns no cargo.

## Decisions

- Capped mode is on exactly when `HttpIr::bounds_requests()` is true; otherwise no byte changes. Reason: parent's "a binding declaring nothing keeps today's bytes"; a text rewrite of the `HTTP` const in capped mode only is the smallest way to keep them.
- The cap check runs in `http::read` after the head is parsed (method and path known) and before the body is allocated or read; the lookup is a generated function in the surface module passed as `&dyn Fn`, so the `http` module stays free of surface knowledge (parent D6).
- Rows are looked up in `dispatch_order`, every row emitted (`None` for the unbounded ones), because a more general template must not capture a request an earlier literal row answers.
- A bound cap above 1 MiB allocates `Content-Length` zero bytes up to 16 MiB per request, one connection at a time; accepted (the ceiling is 16 MiB, parent D4).
- The default refusal keeps today's wording (`this surface reads at most {cap}`) so a surface that declares only `limits` answers in the words a caller already sees.
- A request without `Content-Length` has no body in this server and a chunked one is `411`: no separate unannounced-length rule is needed here; h3 has one for `net/http`.
- The new test file is a module of `http_binding_rust` and each case starts its own server process: no new test binary, no per-test build, and no `OnceLock` (which is per process under nextest).
- The `body-cap-rust` fixture is its own copy; h3 has a copy with the same bytes. Reason: the two units are built at the same time and each unit's story is its whole work order.

## Open for the operator

None.

## Changelog

none (h4 writes the single bullet).
