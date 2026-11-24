---
format: aep.planning-md/3
id: story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
kind: story
status: active
title: The generated Go server reads at most the request body its binding declares for the matched route, and answers a larger one with the refusal the binding declares
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- depends_on: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- depends_on: story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: .github/workflows/release.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-cli/tests/generate_ui_http.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/no_test_builds.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/fixtures/body-cap-rust/http.yaml.in
- confidence: cited
  path: crates/generate/ess-synth/tests/fixtures/body-cap-rust/model.yaml.in
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/body_cap_go.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/body_cap_rust.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/shared.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding_rust.rs
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:38Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"review_outcome":7}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:40Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"review_outcome":7}}}
---
## Outcome

A Go server generated with `ess generate synthesize --target go --http FILE` for a binding that declares `limits` or `refusals.too_large` reads at most the declared number of bytes of the matched route's request body, and answers a larger one with the declared `too_large` refusal (`413` with `{"refused": "..."}` when the binding declares limits and no refusal). A body announced by `Content-Length` is refused before one byte of it is read; a body of unannounced length (chunked) is read up to the cap plus one byte and refused if it goes past. Three things stay true:

- A binding that declares neither key, and every synthesis without a binding, writes byte for byte the files it writes today.
- A route that reads no body, a derived route, an unrouted path and a path that matches no row keep today's fixed 1 MiB bound and today's `413` words.
- The refusal never blocks on bytes the caller has not sent: a request that announces `Content-Length: 65` against a cap of 64 and sends nothing after its head is answered at once.

## Why

Parent: `story:an-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets` (decisions D5, D6). The Go server reads the body in `readBody` before it has looked at the path (`go/http.rs:2768`, called first in both dispatch texts), against the constant `maxBody`; a binding can move neither the bound nor its answer. Needs h1 (the keys and `HttpIr::request_bytes`, `bounds_request`, `bounds_requests`, `FIXED_REQUEST_BYTES`). Depends on h2 (the Rust server) and lands after it: the two share `Taskfile.yml`, `AGENTS.md` and a module line (below), and the capped-binding parity case (Tests first, 5b) runs the Rust server h2 builds through `task body-cap-rust-server`. h1 first, then h2, then h3.

## Where

Tree read: the plan tree, `git rev-parse --short HEAD` = `cfe9dabcb85` (code as of `e3f502951bd`; the difference is planning files). Lines are read in that tree. Files under `crates/generate/ess-synth/` unless a path says otherwise.

- `src/go/http.rs`
  - `helpers_file` (162-236): `SURFACE_HELPERS` (2662) takes, in order, the `json` substitutions (178-182), `refusing_lone_surrogates`, grant helpers, `QUERY_HELPERS`, and for a binding (199-232) `bound::surface_helpers`, `bound::BOUND`, `BARE_BODY`, `bound::refusals` and one `bound::route_refusals` per served component. In capped mode (below) add: `bound::capped_read_body(&helpers)` right after `bound::surface_helpers`, `helpers.push_str(bound::CAPPED)` after `BOUND`, and the `too_large` flag into the two refusal calls.
  - `readBody` (2767-2784) and `const maxBody = 1048576` (2677): NOT edited. Uncapped output keeps its bytes; `maxBody` stays and is the fixed bound a capped package falls back to.
  - `surface_file` (1265): `bound` at 1281 is true when some route of the component is declared; `bound::dispatch` is called at 1332. Capped mode is decided by one function (`bound::caps_a_request(binding, routes)`: true when some `routes` entry has `declared` and `binding.ir().bounds_request(declared)`), called with the bound routes of the served components in `helpers_file` (as `checks_grant` is computed at 213-224) and with this component's routes in `surface_file`. The emitter allows one served component per package, so the two answers are one; if a later change allows more, the helper text and every surface must agree, and a synthesis that does not is a bug to fail loudly (`assert_eq!`).
  - `Serve{exported}` and `Serve{exported}WithStatic` (1294-1325, 1385-1415) call `dispatch{exported}`; unchanged. `announce` (1626-1634) writes the startup record to stdout with `"runtime":{"address":...,"language":"go","port":...}`, which the driver reads, as the Rust server's does.
- `src/go/http/bound.rs`
  - `surface_helpers` (35-57): the existing pattern, a list of `(from, to)` pairs applied with `replacen` and each anchor asserted with `assert!(helpers.contains(from), ...)`. Add `pub(super) fn capped_read_body(base: &str) -> String` in the same style: one anchor, the whole text of today's `readBody` function (its doc comment, 17 lines, copied from `go/http.rs` 2767-2784 at the commit you start from), replaced by the capped function below. The assertion names the anchor, so a later edit of `SURFACE_HELPERS` fails loudly rather than writing the uncapped read into a capped package.
  - `BOUND` (starts at 58, ends just before `IMPORTS`, after `boundMethodNotAllowed` at 1013): add `pub(super) const CAPPED: &str` after it, holding `bodyCap` and `bodyTooLarge`, below. `spelledPath` (1001-1009) and `matched` (140) are in `BOUND` already, so a capped package has them.
  - `dispatch` (1135-1214): add a parameter `capped: bool`. The format string at 1147-1158 holds `body, refused := readBody(request)`; write it through a `{read}` placeholder: `readBody(request)` when uncapped (same bytes), `readBody(request, bodyLimit{exported}(request.Method, spelledPath(request)))` when capped. The digest pin (Tests first, 1) proves the uncapped text did not move. `bodyLimit` is read before `serving.Lock()`, as `readBody` is today.
  - New `pub(super) fn body_limit(out, binding, ir, layout, routes, rows, exported)`: emits `func bodyLimit{exported}(method string, spelled string) bodyCap` with one block per row of `http::dispatch_order(&keys)` (the same `keys` walk as `dispatch`, 1160-1167), in that order, so a literal row answers before a variable row exactly as `dispatch` does. Each block opens as `dispatch` opens it (`if _, ok := matched(<path>, spelled); ok {` for a declared row, `if spelled == <path> {` for a derived row or document, written with `go_string`) and holds `if method == <METHOD> { return <result> }`, where `<result>` is `bodyCap{limit: <cap>, tooLarge: refusalTooLarge<suffix>}` for a row whose route is declared and `binding.ir().bounds_request(declared)` (`<cap>` is `request_bytes(declared)` or `FIXED_REQUEST_BYTES`, an integer literal), and `bodyCap{limit: maxBody, tooLarge: bodyTooLarge}` for every other row, so a later, more general row cannot capture a request an earlier row answered. The last line is that same fixed `return`. Call it from `surface_file` right after `bound::dispatch`, capped mode only.
  - `refusals` (1826-1837), `route_refusals` (1843-1871), `RefusalSet` (1888-1894) and `refusal_set` (1898-2000): add `too_large: bool` to `RefusalSet`. For the shared set it is true when some declared route answering through the shared set `bounds_request` (a route answers through it when `declared.refusals.is_empty()`, `refusal_suffix` 1873-1885); for a route's own set, when that route does. When true, `refusal_set` emits, after the `refusalNotGranted` block, `func refusalTooLarge{suffix}(_ int) response` with `overridden(refusal)` (2003+) when `lookup(RefusalKey::TooLarge)` is `Some`, else `func refusalTooLarge{suffix}(limit int) response { return bodyTooLarge(limit) }`. The doc comment names `{from}refusals.too_large`. Emit it only when true: nothing in the package then carries a function nothing calls.
- `Taskfile.yml`: var after `UI_HTTP_ITEMS_SERVER` (line 16): `BODY_CAP_GO_SERVER: '{{.ROOT_DIR}}/{{.NEXTEST_ARCHIVES}}/body-cap-go-server'`; new task `body-cap-go-server` after `ui-http-servers` (264-283); `- task: body-cap-go-server` in `test` (89-94), `test-packages` (119-124) and `test-archive-workspace` (157-162); `ESS_BODY_CAP_GO_SERVER: '{{.BODY_CAP_GO_SERVER}}'` in the `env:` of `test` (81-87), `test-packages` (110-116) and `test-shard` (346-353). h2 has landed (h3 depends on it): its `body-cap-rust-server` lines are in the same lists, keep both and add yours after them. t7 (the test-budget unit) later adds timeout prefixes to the same three tasks, `test`, `test-packages` and `test-shard`, whose `env:` blocks you edit: whoever lands second rebases and keeps both.
- `AGENTS.md` lines 234-237 (the harness paragraph): add `` `task body-cap-go-server` (`ess-synth`'s `body_cap_go` cases, through `ESS_BODY_CAP_GO_SERVER`) `` to the list, after h2's entry if it is there.
- Two Go set-up steps, both owned here, because two jobs reach `task body-cap-go-server` once it is a dependency of `test-archive-workspace` and `test-packages`:
  - `.github/workflows/ci.yml`: `build-tests` (249-336) runs `task test-archive-workspace` with no Go installed, and its `Free runner disk` step (308-314) deletes `/opt/hostedtoolcache`, where `actions/setup-go` puts Go. The `test` job installs Go at 536-539 (`actions/setup-go@b7ad1dad31e06c5925ef5d2fc7ad053ef454303e # v7`, `go-version: "1.25.10"`, `cache: false`, with the comment "The generated modules intentionally have no go.sum"). Add the same step to `build-tests` after `Free runner disk` (308) and before `Fetch locked dependencies` (316), the same pin and version and `cache: false`, with a comment saying the Go server `body-cap-go-server` is built here once and archived beside the test archives.
  - `.github/workflows/release.yml`: the job `hotfix-tests` (387-457) runs `task test-packages PACKAGES=...` (457), which now runs `body-cap-go-server`, and installs no Go. It has no `Free runner disk` step. Add the same `actions/setup-go` step (same pin, `go-version: "1.25.10"`, `cache: false`) after the `actions/setup-node` step (436-438) and before `Fetch locked dependencies` (449). Without it a hotfix tag fails in its targeted gate on a missing `go`. (The Rust task needs no CI change; these do only because they build Go.)
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: one new test, `a_job_that_builds_the_go_server_installs_go_after_it_frees_the_disk`, covering both jobs (Tests first, 6). Helpers `yaml`, `text`, `step_using` (261), `lane_invocations`, `reached` exist. Other units append tests to the same file: t1b (`story:a-slow-ci-job-runs-task-test-slow-on-every-run-that-can-count-for-a-release-and-gate-carries-it`), t3, t6 and t7. Append yours at the end of the file; at merge keep both sets of tests, the conflict is two adjacent additions, never a choice between them.
- `tests/http_binding_rust.rs`: after the module declarations at 14-18 add `#[path = "http_binding/body_cap_go.rs"] mod body_cap_go;` (h2 adds `body_cap_rust` the same way: keep both lines). One test binary.

The Go behaviour this unit relies on, read in `/usr/lib/go/src/net/http` (go 1.27.1 on this host; CI builds with 1.25.10): `Request.ContentLength` is `-1` for a chunked body; `server.go` 1447-1472 makes the server drain up to 256 KiB of an unread body before it writes response headers, except when `closeAfterReply` is set, and every answer here sets `Connection: close` (`surface_helpers`, the `write` method), so that drain is skipped; `finishRequest` (1700-1722) flushes the answer to the client first and only then closes the body, so a close that waits for missing bytes delays nothing the client sees. What would block is a `Body.Close()` or read the handler does itself before it answers, which is why the early refusal below returns before `readBody`'s `defer Close`. `100 Continue` is sent on the first body read, so a refused request that carries `Expect: 100-continue` gets its refusal and no `100`.

The Go text of capped mode (write it with doc comments in the style of the file; the shape is load-bearing):

```go
type bodyCap struct {
	limit    int
	tooLarge func(limit int) response
}

func bodyTooLarge(limit int) response {
	return refusal(413, fmt.Sprintf("the body is longer than %d bytes, which is all this surface reads", limit))
}

func readBody(request *http.Request, reading bodyCap) ([]byte, *response) {
	if request.Body == nil {
		return nil, nil
	}
	if request.ContentLength > int64(reading.limit) {
		answer := reading.tooLarge(reading.limit)
		return nil, &answer
	}
	defer func() { _ = request.Body.Close() }()
	body, err := io.ReadAll(io.LimitReader(request.Body, int64(reading.limit)+1))
	if err != nil {
		answer := refusal(400, fmt.Sprintf("the body could not be read: %s", err))
		return nil, &answer
	}
	if len(body) > reading.limit {
		answer := reading.tooLarge(reading.limit)
		return nil, &answer
	}
	return body, nil
}
```

`bodyTooLarge` is today's `413` with the limit in place of `maxBody`; `readBody` is today's function with the bound taken from `reading` and the announced-length check added before the `defer`. `refusal` and `io` are in `SURFACE_HELPERS` already. A `go vet` of the generated package must stay clean (the task runs it).

## Tests first

New module `tests/http_binding/body_cap_go.rs` (a module of `http_binding_rust`, not a binary of its own; it spawns no cargo, so it is not added to `ALLOWED` in `no_test_builds.rs`) and NO fixture of its own: it reads the model and binding pair h2 wrote, `tests/fixtures/body-cap-rust/model.yaml.in` and `http.yaml.in` (see h2, "Tests first": four commands `Receive` 64, `Label` 256 with its own `too_large`, `Discard` on `/bulk` 2097152, derived `Archive`, and the view `/parcels`), through a path built from `env!("CARGO_MANIFEST_DIR")`. Nothing in the pair is Rust-specific, so the Go task and the Go cases read it as it is (the task copies the model to `parcel.yaml`, as h2's does); the directory keeps h2's name so h2's path does not move. h3 depends on h2 for this, and it must not edit the pair: a change to it is h2's, and h2 says so. The model is compiled in the test from the pair under the source name `parcel.yaml` (the Taskfile copies it to that name), and the test holds the binding's `source_digest` to the compile's as `json_body_closure.rs` `binding()` does (line 163), panicking with the digest and `run task body-cap-go-server again`.

Each test starts nothing itself but a child process of the built server (test 5b starts two). Report the runner's own figure for the module at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`), on a warm build, test time only; every test under 60 s, each new one under 5 s at that load or named with its time.

1. `a_binding_that_declares_no_limit_keeps_the_go_server_bytes` (digest pin; green today and after): synthesize the shop fixture with `shared::binding(&ir, ROUTES)` for `Target::Go` in `OutputLayout::Workspace`, join every artifact as `path`, `\n`, `contents` in path order, and assert `ess_gen::ess_http::digest(joined.as_bytes())` equals a constant. Capture the constant on the UNMODIFIED tree first (step 1). This is the proof that the `{read}` placeholder in `bound::dispatch` wrote today's text.
2. `a_capped_go_surface_reads_through_its_body_limit`: synthesize the fixture (`Target::Go`, `OutputLayout::Workspace`). The package's helpers file contains `type bodyCap struct`, `func bodyTooLarge(` and `func readBody(request *http.Request, reading bodyCap)`, and not `func readBody(request *http.Request) (`; the surface file contains `func bodyLimitParcel(` with `limit: 64` and `limit: 256` and `func refusalTooLarge`; `refusalTooLarge` for `Label` carries its own suffix. The same synthesis of the shop fixture contains none of `bodyCap`, `bodyLimit` and `refusalTooLarge`, and still contains `func readBody(request *http.Request) ([]byte, *response) {` and `const maxBody = 1048576`. Red today: the fixture binding does not compile (h1 not built) or, once it does, nothing is emitted.
3. `the_fixed_bound_is_the_one_the_binding_crate_exports` (guards drift): the uncapped helpers file contains `const maxBody = ` followed by `ess_http::FIXED_REQUEST_BYTES`. Compiles only after h1; write it in step 3.
3b. `the_cap_lookup_of_a_route_above_the_fixed_bound_is_that_route_cap` (the unit test of the cap lookup the parent asks for): in the generated `bodyLimitParcel` of the fixture, parse the blocks (one per row, in order) and assert that the block of `POST /bulk` returns `bodyCap{limit: 2097152, tooLarge: refusalTooLarge` and that `2097152 > ess_http::FIXED_REQUEST_BYTES` (read from the crate, not typed), that the blocks of `POST /parcels` and `POST /labels` return 64 and 256, and that the block of `GET /parcels`, the derived rows (found through `ess_gen::http::derived`) and both documents return `bodyCap{limit: maxBody, tooLarge: bodyTooLarge}`. The socket case `a_route_cap_above_the_fixed_bound_reads_a_body_the_default_refuses` runs it.
4. Socket cases, one `#[test]` per case so a failure names the case, all against the server binary `ESS_BODY_CAP_GO_SERVER` names (driver below). Bodies are built by padding `note` so the whole JSON body is exactly N bytes; assert `body.len()` before sending. `baseline` is the status the server answers to a 10-byte-note `Receive` with a fresh `parcel_id`; every request in a test uses a fresh `parcel_id`.
   - `a_body_of_the_cap_is_read_and_one_byte_more_is_refused`: `Receive` with a 64-byte body answers `baseline` and not `422`; with a 65-byte body answers `422`, the body `{"error":"too_big"}` and the header `X-Limit: 64`.
   - `a_refusal_needs_no_body_read`: `Receive` with `Content-Length: 65` and no body bytes sent answers `422`, which shows the check runs before the read and that nothing waits for the missing bytes: the test sends 0 body bytes and the answer must arrive before its 10 s socket read timeout expires (the expiry is the failure; no duration is asserted). This is the case a `Body.Close()` on the early path would hang: it must be red against that mistake.
   - `an_expect_continue_request_is_refused_without_a_continue`: the same head plus `Expect: 100-continue`; the first status line read is `422`, not `100`.
   - `a_route_limit_stands_in_place_of_the_binding_limit`: `Label` with a 256-byte body is accepted; with 257 answers `413` with no body (`Content-Length: 0`, no `Content-Type`), the route's own `too_large`; `Label` with a 65-byte body is accepted (the binding's 64 does not apply).
   - `a_body_of_unannounced_length_is_cut_at_the_cap`: `Receive` sent chunked (`Transfer-Encoding: chunked`, no `Content-Length`): a 64-byte body in two chunks answers `baseline`; a 65-byte body answers `422` with `X-Limit: 64`. Send the terminating chunk; read the answer by its `Content-Length`.
   - `a_route_cap_above_the_fixed_bound_reads_a_body_the_default_refuses`: `POST /bulk` (`Discard`, cap 2097152) with a 1,200,000-byte body (more than the fixed 1048576, built by padding `note`; assert `body.len()`) answers `baseline`; the same body to `Archive`'s derived path (below) answers `413` with the Go default wording (`the body is longer than 1048576 bytes, which is all this surface reads`); `POST /bulk` with `Content-Length: 2097153` and no body answers `422` (the binding's `too_large`).
   - `a_route_that_reads_no_body_and_a_derived_route_keep_the_fixed_bound`: `GET /parcels` with 65 body bytes answers `200`; `POST` to `Archive`'s derived path (read from the server's `GET /openapi.json`: the path whose operation has `operationId` `parcel.intake.Archive`) with `Content-Length: 1048577` and no body answers `413` with the body `{"refused":"the body is longer than 1048576 bytes, which is all this surface reads"}` (parse it with `serde_json`, which `ess-synth` depends on, and compare the member); the same on an unrouted path `/nowhere`. Today's Go server would wait for the missing megabyte here; the capped one answers at once.
   A surface that declares nothing is covered by test 1 and by the existing `without_a_binding_the_go_server_keeps_its_bytes` (`tests/http_binding/go.rs:330`), not by a socket case.
5. `the_go_and_rust_servers_answer_one_binding_alike` (`go.rs:754`) is read, not changed; it runs the shop binding, which declares no limit, so it stays green and shows the uncapped parity did not move.
5b. `the_go_and_rust_servers_answer_a_capped_binding_alike` (the capped-binding Rust-equals-Go parity case, in `body_cap_go.rs`): start the Go server (`ESS_BODY_CAP_GO_SERVER`) and the Rust server of the same pair (`ESS_BODY_CAP_RUST_SERVER`, built by h2's task from the one pair both servers are built from; panic naming `task body-cap-rust-server` when unset) and send both the same raw requests: `Receive` with 64 and with 65 body bytes, `Label` with 256 and 257, `Receive` with `Content-Length: 65` and no body, `/bulk` with 1,200,000 bytes and with `Content-Length: 2097153`, `GET /parcels` with 65 bytes, and `Archive`'s derived path with `Content-Length: 1048577`. For each request assert the two servers' status codes are equal, and where the binding declares the refusal also the body and the `X-Limit` header. The `Archive` case compares status only: the two default `413` bodies differ in words by design (decisions). The chunked case is not in it: the Rust server answers `411` to chunked transfer, so it has no counterpart (h2 decisions). The two drivers are the module's own copies.
6. In `ci_lanes.rs`, `a_job_that_builds_the_go_server_installs_go_after_it_frees_the_disk` (one test, both set-up steps): for every job of `ci.yml` and of `release.yml` that reaches the task `body-cap-go-server` (through `lane_invocations` and `reached`; today `build-tests` through `test-archive-workspace` and `hotfix-tests` through `test-packages`), assert it has an `actions/setup-go` step through `step_using`, pinned to a full commit like the one the `test` job uses and with the same `go-version` and `cache: false`; that this step comes after any step named `Free runner disk` and before the step that runs the job's task; and that `Taskfile.yml`'s `body-cap-go-server` runs `go build`. Assert too that at least those two jobs are found, so a rename of one cannot make the loop empty. Mutate the workflows in memory as `the_workspace_split_fails_on_a_dropped_group_a_...` (748) does (in `ci.yml` drop the step, and move it before `Free runner disk`; in `release.yml` drop the step) and assert the test fails on each of the three. Red today: no job installs Go, and the task does not exist (the test is written first with the task name, and fails on the missing task).

Driver (in the module, ~70 lines, its own copy of the Rust module's driver; model it on `server`/`serve` of `crates/edge/ess-cli/tests/generate_ui_http.rs` 463-560): read `ESS_BODY_CAP_GO_SERVER`, panic naming `task body-cap-go-server` when unset or missing, make the file executable (an uploaded artifact loses its mode bits), spawn it with `--listen 127.0.0.1:0` and piped stdout, read lines until a JSON record with `runtime.address`, kill on drop. One server for all cases in a process under `nextest` is not possible (a process per test): each case starts its own server (about 10 ms); that is cheap, and it is not an expensive product built per test. Send raw HTTP/1.1 over `TcpStream` with a 10 s read timeout, write the head and the body bytes, parse the status line and headers, and read the body by its `Content-Length` (every answer this server writes carries one except `204` and `304`, and it closes the connection after each).

## Steps

1. On the unmodified tree write test 1 with a placeholder constant, run it, copy the digest in, run green. Write the digest down: it is the count that must not move.
2. Write tests 2, 3b, 4 and 5b and the module declaration (the pair is h2's, already on the tree); `cargo test -p ess-synth --test http_binding_rust body_cap_go` fails (red, as above). Write test 6; it fails.
3. Implement `capped_read_body`, `CAPPED`, `body_limit`, the `dispatch` placeholder, the `refusal_set` change and `caps_a_request`; write test 3. Capped mode is computed once per site by `caps_a_request` and passed, never re-derived per call.
4. Add the Taskfile var, task, wiring, the `AGENTS.md` line and the two Go steps (`ci.yml` `build-tests`, `release.yml` `hotfix-tests`). The task, modelled on `ui-http-servers` (264-283): `cargo build --workspace --exclude ess-xtask --locked --bin ess`; `rm -rf {{.WORK}}`; `mkdir -p {{.WORK}} {{.NEXTEST_ARCHIVES}}`; `cp {{.FIXTURES}}/model.yaml.in {{.WORK}}/parcel.yaml` (here `{{.FIXTURES}}` is `crates/generate/ess-synth/tests/fixtures/body-cap-rust`, h2's pair); `target/debug/ess generate synthesize --target go --path {{.WORK}}/parcel.yaml --http {{.FIXTURES}}/http.yaml.in --out {{.WORK}}/parcel`; then in one command `cd {{.WORK}}/parcel && env GOFLAGS=-mod=mod GOPROXY=off GOTOOLCHAIN=local go vet ./...` and, in the next, the same environment with `CGO_ENABLED=0 go build -trimpath -o {{.BODY_CAP_GO_SERVER}} ./cmd/parcel-server`; `rm -rf {{.WORK}}`. The environment is the one `tests/http_binding/go.rs` `go()` (151) sets. Read the real command directory in the generated tree (`cmd/<component>-server`, `go/entry.rs` 82) and fix the name if it differs. `go` goes through the slot shim, so the task takes its slots by itself; the vet runs once here, so no test spawns `go`. `GOCACHE` is the caller's or Go's default; do not set it to a path under `/tmp`.
5. Run `ESS_BODY_CAP_GO_SERVER=<path> cargo test ...` after `slot task body-cap-go-server`; then the gate.
6. Re-run test 1: the digest equals the one written down in step 1 (the generator gained a text rewrite and one table that run in capped mode only, so the shop synthesis is byte for byte the same). Report the byte count the capped fixture's helpers and surface files gain over the same fixture without `limits` (one number, no bound).
7. Pins that must not move, and the tests that show it: the committed `generated/go/{billing,gatepass}` trees (`without_a_binding_the_go_server_keeps_its_bytes`, `go.rs:330`; `feasibility`; `adversary_related_guard_behaviour_pass1`); the bytes of a bound synthesis that declares nothing (test 1); the byte-pin targets of `AGENTS.md` for `ess-synth`: `served_codec_names` and `go`, and `http_binding_rust` with its `go` module. None of these is re-pinned; if one moves, the change is wrong.

## Owned files

- `crates/generate/ess-synth/src/go/http.rs`, `crates/generate/ess-synth/src/go/http/bound.rs`
- `crates/generate/ess-synth/tests/http_binding/body_cap_go.rs` (new), `crates/generate/ess-synth/tests/http_binding_rust.rs` (one module declaration); no fixture directory of its own
- `.github/workflows/ci.yml` (the one step in `build-tests`), `.github/workflows/release.yml` (the one step in `hotfix-tests`) and `crates/edge/ess-xtask/tests/ci_lanes.rs` (the one new test, appended; t1b, t3, t6 and t7 append theirs too: keep both at merge)
- `Taskfile.yml` and `AGENTS.md`: the lines named above. h2 edits the same two files and lands first (h3 depends on h2); h3 rebases and keeps both sets of lines (adjacent additions in the same lists). t7 adds timeout prefixes to `test`, `test-packages` and `test-shard` later or earlier: whoever lands second rebases and keeps both.
- Pinned reads (no edit): `crates/generate/ess-synth/tests/http_binding/shared.rs`, `crates/edge/ess-cli/tests/generate_ui_http.rs`, `crates/edge/ess-xtask/tests/no_test_builds.rs`, `crates/generate/ess-synth/tests/http_binding/body_cap_rust.rs` and `tests/fixtures/body-cap-rust/` (h2's: the driver shape, and the model and binding pair that the Go task and the Go cases read as it is).
- No stamp involvement: `ess-synth` is not one of the `SYNTHESIS_CRATES` the stamp (t2) hashes, and `ess-conformance` does not depend on it. h3 touches nothing t2 digests and does not run t2's digest test.

## Not yours

- Everything under `crates/generate/ess-synth/src/rust/` and `tests/http_binding/body_cap_rust.rs`, `tests/fixtures/body-cap-rust/` (h2): read, not edited.
- `crates/specify/ess-http` (h1), `crates/generate/ess-gen` (h4), `CHANGELOG.md` (h4), the design page and the guide (h1, h4).
- No other job or step of `ci.yml` or `release.yml`: not the `test` job's Go step, not the cache or archive steps, not the ruleset. No other test of `ci_lanes.rs`; if one of the existing tests fails on a new step, fix the new step, not the test.
- The existing in-package Go harness (`go.rs` `HARNESS`, the cases that run `go test`) and `shared.rs`: read them, do not edit them. The existing cargo-spawning tests stay on the `ALLOWED` list; do not touch it.
- The TypeScript web target takes no `--http` and is unchanged, as is `ess-ui`. The conformance suite is not extended.
- No Python, no shell checker, no `/tmp`; no model call or network request in a test.

## Gate

Per `AGENTS.md` § Gate, a change under `crates/generate/ess-synth` gates `ess-synth` and its byte-pin targets; a workflow or Taskfile change adds `task ci-lint`. Each command through the shim or `slot`; each job ends in 10 minutes unless it names its time (at most 30 minutes); each test under 60 s; the test step within 120 s of test time (the runner's own figure) at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`), on a warm build. Run only the named targets; the generator tests of `ess-synth` are the large ones.

```console
slot task fmt-check
slot task body-cap-rust-server
slot task body-cap-go-server
slot task ci-lint
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-synth --all-targets --locked -- -D warnings
ESS_BODY_CAP_RUST_SERVER=$PWD/target/nextest-archives/body-cap-rust-server ESS_BODY_CAP_GO_SERVER=$PWD/target/nextest-archives/body-cap-go-server SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test http_binding_rust --test go --test feasibility --test adversary_related_guard_behaviour_pass1 --test served_codec_names
SLOT_TIMEOUT=30m CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked --test ci_lanes --test no_test_builds
```

`http_binding_rust` runs its existing cargo and `go` cases too (go is on `PATH` on this host); run it as one target. Both server variables are set (h2's cases and test 5b read the Rust one), or the cases panic naming their task. `ess-openapi`'s `adversary_ess_issues_2_u6` and `ess-cli`'s `tutorial_page` are not touched by this unit (no OpenAPI, no doc change). `ci_lanes` is large: name its runner time in the report, and if it passes 120 s say so rather than raising a timeout. Do not run the whole workspace.

## Acceptance

1. `slot task body-cap-go-server` vets and builds `target/nextest-archives/body-cap-go-server`.
2. The seven socket tests of test 4 pass: 64 bytes accepted and 65 answered `422` with the declared body and `X-Limit: 64`; `Content-Length: 65` without a body answered `422` after 0 body bytes were sent (the answer arrives before the 10 s read timeout), with and without `Expect: 100-continue` and without a `100`; `Label` 256 accepted and 257 answered `413` without a body; chunked 64 accepted and chunked 65 answered `422`; `/bulk` (cap 2097152) accepted a 1,200,000-byte body that `Archive`'s derived route refused with `413` at the cap 1048576, and answered `422` to `Content-Length: 2097153`; a view with 65 body bytes answered `200`; a derived route and an unrouted path with `Content-Length: 1048577` answered `413` with today's Go wording at the cap 1048576, with 0 body bytes sent.
3. Tests 1, 3 and 3b pass: the digest of the declaring-nothing synthesis equals the constant captured in step 1 (before and after), `maxBody` equals `FIXED_REQUEST_BYTES`, and the lookup table returns 2097152 for `POST /bulk`, above `FIXED_REQUEST_BYTES`; test 2 shows the fixture synthesis carries `bodyCap` and `bodyLimit` and the shop synthesis carries neither.
3c. Test 5b passes: for every request it sends, the Go and the Rust server answer the same status, and the same declared body and `X-Limit` header where the binding declares them.
4. `without_a_binding_the_go_server_keeps_its_bytes`, `the_go_and_rust_servers_answer_one_binding_alike`, `feasibility`, `adversary_related_guard_behaviour_pass1`, `served_codec_names` and `go` pass unchanged; no file under `generated/` changed (`git status --short generated` is empty).
5. `ci_lanes` passes with the new test, which finds `build-tests` and `hotfix-tests` and fails on each of its three mutations (two of `ci.yml`, one of `release.yml`).
6. The shop synthesis digest equals the constant of step 1 (bytes, not time); every new test is under 5 s at a 1-minute load of 60 or less, or is named with its runner time.
7. `cargo test -p ess-xtask --test no_test_builds` passes (run it by name; it reads the tree): the new module spawns no cargo.

## Decisions

- Capped mode is on exactly when a declared route of a served component bounds its request (`caps_a_request`, the route form of `HttpIr::bounds_requests`); otherwise no byte changes. Reason: parent's "a binding declaring nothing keeps today's bytes"; a text rewrite of one fixed function, in capped mode only, is the smallest way to keep them.
- The route lookup happens before the body read, in a generated `bodyLimit{exported}` that matches rows exactly as `dispatch` matches them (parent D6). `readBody` takes the answer as a `bodyCap` value rather than two arguments because Go passes the result of a call on only as the whole argument list.
- An announced length above the cap is refused before the body is touched and the handler never closes the body on that path. A length of `-1` (chunked) is read through `io.LimitReader(body, limit+1)`, so memory stays at the cap plus one byte, and refused when it goes past; the handler's own close then drains at most the 256 KiB `net/http` allows, after the answer is on the wire.
- In a capped package the fixed bound also answers an announced oversize at once (today's Go server waits for the bytes). The words stay today's: `the body is longer than {limit} bytes, which is all this surface reads`, with the route's limit in place of `maxBody`.
- Go's default `413` does not name the announced length, as today's does not; the Rust server's does. The two answers' words differ today and keep differing.
- The unit installs Go in `build-tests` and in the hotfix job `hotfix-tests` (both reach the task) rather than building the Go server in each test shard: the shards compile nothing and carry no Go build step of their own per test, and a per-test `go build` is the per-test expensive product the organization AGENTS.md forbids. The server is a static binary (`CGO_ENABLED=0`), so it runs in every shard from the archive directory.
- One model and binding pair for both servers, h2's, read by h3 from h2's path (h3 depends on h2). Reason: the parity case (5b) compares the two servers on one binding, so the two must be built from one pair, and a second copy could drift from it silently.
- A bound cap above 1 MiB lets one request hold up to 16 MiB in memory at a time per connection; accepted (the ceiling is 16 MiB, parent D4).

## Open for the operator

None.

## Changelog

none (h4 writes the single bullet).
