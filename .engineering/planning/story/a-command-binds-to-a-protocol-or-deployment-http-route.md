---
format: aep.planning-md/3
id: story:a-command-binds-to-a-protocol-or-deployment-http-route
kind: story
status: draft
title: A command or view binds to the HTTP method and path a protocol fixes or a deployment chooses
refs:
- provider: github
  reference: beyond10x/ess#493
relations:
- serves: vision:O2
revision: 2
---
## Request

https://github.com/beyond10x/ess/issues/493 (2026-10-07): a specification of a published HTTP
protocol has operations whose method and path are not ESS's to choose.

- Fixed by a standard: a discovery document served at one well-known path.
- Chosen by each deployment: a protocol that leaves an endpoint's location to the server, so two
  implementations of one specification serve the same command at different paths.

ESS 0.55.0 derives every route (`POST /{domain wire}/commands/{command wire}`,
`GET /{domain wire}/views/{view wire}`, `crates/generate/ess-gen/src/http.rs`) and refuses any
routing key on a command (`unknown field`). An `ess-realization/2` entrypoint carries one base URL,
not a route per command. An external implementation's conformance adapter holds the
command-to-URL mapping in hand-written code that nothing checks against the model.

The requester's proposed syntax, labelled as theirs: `http: {method: POST, path: /token}` beside
`naming:` on the command.

## Fit review

Not written yet. Written with `.agents/skills/assessing-external-requests/SKILL.md` before this
story is dispatched. Questions it has to answer, beyond the seven:

- where a protocol-fixed route lives (the specification) and where a deployment-chosen route lives
  (the realization or a deployment document), and whether one construct covers both;
- whether the sibling `ess-cli/1` presentation binding is the shape to follow for an HTTP binding;
- views as well as commands (a fixed `GET` discovery document is a view);
- what the OpenAPI projection, the conformance runner and every generated target do with a declared
  route, or which of them refuse it by name.

## Acceptance (draft, settled by the fit review)

- A specification can declare the HTTP method and path of a command or view that a protocol fixes,
  and a deployment can declare the path of one the protocol leaves open; `ess specify validate`
  accepts both and refuses a route two operations share.
- `ess generate --kind openapi` emits the declared method and path instead of the derived route.
- The conformance runner reaches an implementation at the declared route, so the mapping is checked
  against the model rather than held in adapter code.
- Every target that cannot honour a declared route refuses it by name.

## Fit review

Reviewed 2026-10-08 with `ess 0.56.0` at checkout `5797beda06`, following
`.agents/skills/assessing-external-requests/SKILL.md`. Every reproduction is a brand-free file written
for this review, under `target/wave-scratch/fit/493/`.

### 1. The need, apart from the syntax

A specification of a published HTTP protocol has to state, for each operation, the method and path
a caller uses. For some operations the protocol fixes them: a discovery read lives at one well-known
path. For others it leaves the path to each server: one server serves the token operation at `/token`,
another at `/oauth2/token`. Two conforming servers of one specification can differ only in their
routes, so a projection or a test needs to know which server it is aimed at.

`base.yaml` reproduces this with one command whose path the server chooses (`IssueToken`), one view
whose path the protocol fixes (`ServerMetadata`) and one view whose path would carry a parameter
(`ClientById`):

```console
$ ess specify validate --path base.yaml
demo v1 — 1 file(s), valid
$ ess generate --path base.yaml --kind openapi --out out-base
$ grep -n -A1 '^  /' out-base/openapi/grant-server.yaml
24:  /grant/commands/issue-token:
25-    post:
67:  /grant/views/client-by-id:
68-    get:
88:  /grant/views/server-metadata:
89-    get:
```

`client_id` is projected `in: query` (`out-base/openapi/grant-server.yaml:75-76`).

**The requester's proposed syntax:** `http: {method: POST, path: /token}` beside `naming:` on a
command or view in the specification, for routes a standard fixes. Per-deployment routes go in the
realization. Then `ess generate --kind openapi --realization <file>`, completeness, collision and
parameter checks in `ess specify realization validate`, and the synthesized servers plus a Go/TS
suite adapter reading the binding.

### 2. Class: gap (and a separate defect found along the way)

**Gap.** No construct can state a method or a path:

- `crates/specify/ess-domain/src/component.rs:257-261`: "No variant names a wire, a port, a path or
  a verb".
- `crates/generate/ess-gen/src/openapi.rs:25-31`: "The model has no `exposures` construct … Until
  `exposures:` exists, the convention is the whole answer".
- Every route is derived (`crates/generate/ess-gen/src/http.rs:449-473`), and the method set is
  `{Get, Post}` only (`http.rs:321-332`).

**Defect, separate from the request.** `naming.wire` is documented as "an HTTP path segment"
(`crates/specify/ess-domain/src/name.rs:202`), yet it accepts `/` and `..`, and the projection
splices the value in unchanged:

```console
$ ess specify validate --path attempt-wire-slash.yaml
demo v1 — 1 file(s), valid
$ grep -n -A1 '^  /' out-wire-slash/openapi/grant-server.yaml
24:  /grant/commands/oauth2/token:
67:  /grant/views/../.well-known/demo-configuration:
```

### 3. Can it already be expressed? No.

| attempt | what `ess` answers |
|---|---|
| `http:` on the command (`attempt-command-http.yaml`) | `` unknown field `http`, expected one of `name`, `input`, `fixture_inputs`, `response`, `outcomes`, `naming`, `refs` `` |
| `http:` on the view (`attempt-view-http.yaml`) | `` unknown field `http`, expected one of `name`, `source`, `shape`, `fields`, `params`, `filter`, `group_by`, `order_by`, `paging`, `consistency`, `naming` `` |
| the path smuggled into a wire name (`attempt-wire-slash.yaml`) | admitted, but it is the defect above. The `/grant/commands/` prefix stays. A client that removes dot-segments (RFC 3986 §5.2.4) asks for a path that the exact-match server table (`crates/generate/ess-synth/src/rust/http.rs:824`) does not hold. This last point is inferred, not run |
| one realization entrypoint per operation, with the full route as its `url` (`realization-per-route.yaml`) | `server-a — 2 entrypoint(s), valid`. It records a URL per surface but carries no method, and nothing that projects, synthesizes, runs conformance or renders UI reads a realization: only `ess-deployment`, `ess-cli` and `ess-xtask` depend on `ess-realization` (their `Cargo.toml`) |
| an HTTP broker in `ess-transport/1` (`transport-http.yaml`) | `` brokers[0].protocol: unknown variant `http`, expected `nats` `` |
| `ess-cli/1` `service_forward` to the command (`cli.yaml` over `cli-model.yaml`) | `cli.yaml — valid CLI presentation binding`. It names the operation, but "transport and command-outcome mapping remain an explicit handler obligation" (`docs/design/cli-presentation-binding.md:101-106`) |

No idiom exists. The closest, a URL per entrypoint, is a record that nothing checks.

### 4. Does a design fit what is already there?

**Where each route lives.** Neither kind of route belongs in `ess/N`. Every existing binding from the
model to a carrier is a separate document, pinned to the specification's source digest:

- events to a broker: `docs/design/event-transport-binding.md:19-23`, "never part of `ess/N`. The
  model does not change when the same event is carried by another broker";
- components to an implementation: `crates/specify/ess-realization/src/lib.rs:1-7`.

The transport format already makes the two-level split this request needs, inside one document.
`brokers[].host` is declared "when the contract fixes it; absent, AsyncAPI gets a `{host}` server
variable" (`event-transport-binding.md:61`). Routes can work the same way:

- A route the protocol fixes is declared in the protocol's binding.
- A route the protocol leaves open is declared there without a path. Each server's binding, in the
  same format, fills the path in. It pins the protocol's binding by digest and may not change any
  route the protocol fixes.

So one construct covers both levels. The server's binding belongs to the implementation, not the
environment. `ess-deployment` lowers ports, health paths and base endpoints
(`crates/generate/ess-deployment/src/runtime.rs:93-104, 133-146`), and a realization entrypoint
carries one base `url` (`ess-realization/src/lib.rs:353-364, 377-395`). The route a server's code
serves is the same in every environment. The base URL is what changes between environments.

**Is `ess-cli/1` the shape to follow?** Partly. The structure matches:

- a closed authored document, compiled against `EssIr` into a canonical plan;
- legacy derivation untouched when no binding is given (`cli-presentation-binding.md:3-6, 253-263`);
- an operation named by owner and qualified name (`crates/specify/ess-cli-contract/src/wire.rs:112-117`).

It differs in two ways:

- `ess-cli/1` pins no specification (`ess-cli-contract/src/lib.rs:15-22`). A protocol's published
  binding has to, as transport does (`crates/specify/ess-transport/src/lib.rs:34-44`).
- `ess-cli/1` owns the whole presentation, not only the location: argument sources, the `{"ok":…}`
  envelope and exit codes (`cli-presentation-binding.md:231-242`). An HTTP sibling built the same way
  would own bodies and statuses too. That is the open decision.

**Vocabulary.** The proposed `http:` key clashes with three spellings that already exist:

- `exposures:` is the name §6 reserves (`docs/design/ess-implementor-design-v0.1.md:448-460`, quoted
  in `openapi.rs:12-25`).
- The OpenAPI import already spells an HTTP operation `{path, method}`, with a closed lower-case
  method set and the refusal "two operations claim the same method and path"
  (`crates/generate/ess-openapi/src/lib.rs:76-80, 110-152`).
- `ess_gen::http::Method` spells methods in upper case (`http.rs:327-345`).

A binding should reuse the import's method set and refusal wording. Path expressions should use the
binding-mapping source spelling, `input.<field>` and `param.<name>`, as `ess-transport/2` did for
subject tokens (`docs/design/parameterized-event-channel-addresses.md:56-58`). `param.<name>` is what
the view filter already uses: the first draft of `base.yaml` was refused with "read it in `filter:` as
`param.client_id`".

**Views.** A `reached_by: network` component already serves every view with `GET` and its `params:`
as query parameters (`http.rs:413`, `openapi.rs:608-613`), so a path expression can name a view
parameter. But a view answers `{rows: [...]}` (`out-base/openapi/grant-server.yaml:204-213`). A
protocol's discovery document is one bare object, so as a view it would sit at the right path with the
wrong body.

**The route alone does not give the protocol's contract.** Project the token operation with the
issue's own route and it still declares:

- a JSON request body;
- `200 {outcome, published, response: {access_token}}`;
- `502` for the refusal.

(`out-base/openapi/grant-server.yaml:30-66, 181-198`; statuses from `http.rs:173-240`.) A server of
that protocol takes a form-encoded request and answers with a bare token object and
`400 {"error": …}` (RFC 6749 §4.1.3, §5.1, §5.2). The issue lists encoding and redirects as
follow-ups. The response body and the status per outcome are the same kind of gap, and the issue does
not list them. A route-only binding pointed at such a server publishes an OpenAPI document whose paths
are right and whose bodies are all wrong.

**Targets.** What each one does with a declared route:

| target | today | with a binding |
|---|---|---|
| OpenAPI | `routes()` becomes `paths` (`openapi.rs:504-505`) | reads the binding; refuses one that leaves a path open, naming the operation |
| Rust server | exact `match request.path.as_str()` (`ess-synth/src/rust/http.rs:368, 824`) | needs path-template dispatch and several methods on one path; `/openapi.json` and `/docs` stay reserved (`http.rs:42-45`) |
| Go server | `switch request.URL.Path` (`ess-synth/src/go/http.rs:1132, 1300`) | same as Rust |
| view queries, view-grant synthesis | read `routes()` (`ess-synth/src/view_query.rs:187`, `ess-conformance/src/synthesize/view_grant.rs:64`) | follow `routes()` |
| UI binding, React adapter, Playwright | `ess-ui-check/src/model.rs:1605`, `ess-ui/src/binding.rs:6-9`, `ess-ui-test/src/playwright.rs:151` | the binding becomes one more input to `ess ui` |
| conformance runner | semantic only: `execute_command(SemanticCommandRequest)` (`ess-conformance/src/target.rs:183-186`). `--target` takes `billing, oracle-fixture, interpreted`. The out-of-process adapter is "the named open gap" (`docs/design/running-a-suite.md:76-81`) | unchanged. No route is "checked by conformance" until an HTTP target exists, so the runner must refuse a binding by name rather than accept it and ignore it |
| web bridge, entity runtime, interpreter | no HTTP at all (`ess-synth/src/web/bridge.rs:1-10`; `ess-entity-runtime` has no route use) | untouched |
| `ess verify diff` | a route move is a wire rename, and it is classified (`base-renamed.yaml`): `command/demo.grant.IssueToken/wire-name-changed`, `unknown for callers, readers` | a route in a separate document is invisible to it, because it compares two ESS revisions only. It needs a binding comparison, or route moves go unclassified as transport subject moves do today |
| generated docs | print no route (`out-docs` contains none) | untouched |

### 5. Would a second, unrelated adopter need it?

- **A protobuf service with HTTP transcoding annotations.** The annotations fix
  `GET /v1/shelves/{shelf}/books/{book}`, and `POST /v1/shelves/{shelf}/books` with the book as the
  body. That needs a route, path parameters taken from input fields, and a body that is the response
  message itself, not an envelope. Keeping such annotations is already a recorded requirement
  (`task:deferred-protocol-ui-bindings`, "HTTP annotations").
- **A REST facade over an existing backend** (`story:a-resource-has-an-http-shape-of-its-own`,
  archived). It needs resource paths, a method and status per outcome, and list and item envelopes.
  Its "Precedent" section names `ess specify cli` as the shape.

Both need the body and the status as well as the route. A route-only need appears only for a server
that already speaks ESS's envelope, which means a server ESS generated, at a path ESS chose. Moving
that path is local policy, and a gateway prefix is the base URL the realization already holds.

### 6. What does each design cost?

| design | formats | new keys | diagnostics | diff | generated-API change | size |
|---|---|---|---|---|---|---|
| A, change nothing | none | none | none | none | none | 0 |
| B, as requested: `http:` in the spec plus realization routes | `ess/24`; `ess-realization/3` and `-ir/3` (`EntryPointSpec` is `deny_unknown_fields`, `lib.rs:377`) | `http` on command and view; routes on an entrypoint | route, parameter, collision and contradiction checks in the compiler and the realization | new command and view change kinds and their classification | `Route` comes from the IR; both servers gain templates and methods; JSON Schema projection regenerated (`cargo xtask schema`) | about 7-8 units |
| C, route-only sibling `ess-http/1` | new `ess-http/1` and `ess-http-ir/1` | `routes[]: {command or view, method, path?, parameters}`, `refines:` | `ESS-HTTP-001` onward | none, or +1 unit for a binding comparison | `routes()` takes an optional binding; servers change as in B; output byte-identical without a binding | about 6 units (+1) |
| D, staged `ess-http` presentation binding | as C, designed for every stage up front | C's keys, then request encoding, response body, status per outcome, redirect | C's, then more | as C | stage 2 changes OpenAPI bodies and both servers' codecs; stage 3 adds an HTTP conformance target and a `--target` value | C about 6, then bodies, status and encoding about 6-7, then the HTTP target about 3-4: about 15-17 in total |

A unit is one implementor story. These are estimates from the files listed in question 4, not
measurements.

### 7. What else was considered?

- **A, change nothing.** Costs nothing. The mapping stays in hand-written adapter code that nothing
  checks.
- **B, as proposed.** Fails question 4 in three ways:
  - it puts transport into `ess/N`, against `openapi.rs:3-31`, `component.rs:257-261` and the
    transport decision;
  - it spells one route in two formats;
  - it spends `ess/24` on a key that carries no semantics.

  Two parts of it carry into C and D: the split between fixed and server-chosen routes, and the
  completeness, collision and parameter checks.
- **C, route only, as a sibling document.** Fits question 4 on placement and vocabulary. But by
  question 4's finding on bodies, it publishes a false contract for the requester's own case, and it
  still makes no route checkable by conformance.
- **D, the HTTP presentation binding, staged.** The only design that meets the issue's third
  acceptance item and serves both second adopters. It reopens the HTTP half of
  `epic:specification-runs-as-a-fake-backend` (`story:a-resource-has-an-http-shape-of-its-own`,
  `story:a-conformance-suite-runs-against-any-realization`). The operator excluded that half on
  2026-09-15 (plan ess-evolution-20260915, revision 1) and left it to "a future approved plan" that
  selects adopters and acceptance targets (`task:deferred-protocol-ui-bindings`).
  `initiative:ess-evolution` is now `implemented`, so whether that plan exists is the open question.
  A sketch of stage 1 (not runnable, because the format does not exist):

```yaml
type: ess-http/1                       # the protocol's binding, published with its specification
specification: {system: demo, version: v1, source_digest: sha256:91d3c444…}
routes:
  - {view: demo.grant.ServerMetadata, method: get, path: /.well-known/demo-configuration}
  - {command: demo.grant.IssueToken, method: post}            # no path: each server chooses it
---
type: ess-http/1                       # one server's binding
specification: {system: demo, version: v1, source_digest: sha256:91d3c444…}
refines: sha256:<digest of the protocol's binding>
routes:
  - {view: demo.grant.ServerMetadata, method: get, path: /.well-known/demo-configuration}
  - {command: demo.grant.IssueToken, method: post, path: /oauth2/token}
  - {view: demo.grant.ClientById, method: get, path: '/clients/{client_id}', parameters: {client_id: param.client_id}}
```

## Decisions

- **defer.** The need is real (a gap, question 2), and precedent settles where routes live (question
  4). What is not settled is the scope of the format, and that call belongs to the operator because it
  decides whether excluded scope comes back. To record outside this review: a `decision-blocker` with
  `blocks: story:a-command-binds-to-a-protocol-or-deployment-http-route`, asking:

  **Should ESS describe an HTTP surface whose wire shape it does not choose? That would be one
  authored, digest-pinned `ess-http` presentation binding: the route now; request encoding, response
  body, status per outcome and redirects next; then an HTTP conformance target. Or should ESS keep its
  own wire shape and let only the route move?**

  - (a) Route only (design C), about 6 units. Serves ESS-shaped servers at chosen paths. The issue's
    protocol still projects wrong bodies and statuses, and gains no conformance.
  - (b) Presentation binding, staged (design D), about 15-17 units over three stages. One design page
    covers the whole format first, so stage 1 does not fix keys that stage 2 would have to bump.
    Reopens the archived HTTP stories under a new approved plan.
  - (c) Not now (design A), 0 units. Reply to the requester with the adapter as the idiom.

  The reviewer recommends (b). Both independent requests for an HTTP surface need bodies and statuses,
  and a route-only first version would publish contracts whose bodies are false.
- **Whatever the answer:** the requester's `http:` key in `ess/N` is refused (question 7). The
  wire-name defect from question 2 is fixed separately, about 1 unit: refuse `/`, `.` and `..` in a
  wire name that a path segment reads.
