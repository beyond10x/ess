# Changelog

## [Unreleased]

### Changed

- **Breaking for a realization of a view with parameters, and for hand-written server code**:
  synthesized Go and Rust servers decode a view's declared parameters from the query string by
  wire name and pass them, typed, to the view port, whose method now takes them. A missing
  required or undecodable value is a `400` refusal; undeclared keys are ignored; queries for views
  with parameters stay obligations. The generated Rust `http::Request` has a new public field
  `query: String` and derives `Default`, so a hand-written `Request { … }` literal sets `query` or
  ends with `..Default::default()`. Two parameters of one view that spell one identifier in a
  target are refused, and a served view with a non-scalar parameter is refused (beyond10x/ess#311).
- Served surfaces agree on what they refuse and in which words: Decimal, Uuid and Bytes values
  and map keys admit exactly the published pattern in Go and Rust, and both answer `431` past
  the request-head size or 100 headers. The Rust server no longer exits when a caller hangs up
  early, and drops a connection silent for 1 s.

- **Breaking for a Go realization**: `ess generate synthesize --target go` generates every command
  behaviour and view query the plan marks generated, at parity with the Rust target, in a new
  package `types/behaviour`: `<Entity>Storage` (`Get`, `Put`, `Delete`, `List` in a stable
  order), `Context`, `Ports` (one field per storage port) and `New(ports) *Generated`, evaluated in
  Rust's order, existence selection included. Entities check their invariants
  (`BrokenInvariant()`). `<ctx>.Unimplemented` now covers owed seams only, so code that passed it
  as the whole behaviour bundle no longer compiles; pass `behaviour.New(ports)` and implement the
  owed seams. Package names `behaviour` and `invariant` are reserved, and a domain named like a
  standard-library package a generated file imports gets a renamed package. A store and a server
  entry point are not generated yet (beyond10x/ess#314).
- The gatepass example's `AdmitVisitor` stores the printed badge (`sets: {badge: input.badge}`).

## [0.51.0] — 2026-10-01

### Added

- `ess_ui::binding`: the route table between an `ess-ui/1` document and the HTTP surface ESS
  synthesizes, built by `ess_ui_check::binding(document, sources)` from `ess_gen::http::routes`
  for what the document names. It refuses by node path a name no network component serves, a
  non-scalar view parameter, a paged view, and state held in `server` or `server_session`.
  `ess_ui::binding::classify(status, body)` reads a served command's answer (Accepted, Refused,
  NotGranted, Malformed, Unfinished, Transport); its cases are recorded from the gatepass Rust and
  Go servers in `crates/ui/ess-ui/tests/vectors/answers.json`. `ess ui check --model` reports
  `read_params`: a read binding a parameter the view does not declare, and a required parameter
  left unbound (beyond10x/ess#311).
- `ess generate ui --target react --model <spec>`: the generated React app reads and commands the
  synthesized server through the binding. It emits `src/binding.ts` and a binding-driven
  `httpAdapter`; the base URL per component comes from
  `<meta name="ess-base-url:<component>">` (same origin when absent) and `setAuthorization`
  sets the caller. Command answers go through `runtime/answer.ts`, a port of `classify` held to
  the same vectors. A refusal shows on the form, confirm, action or account-menu entry that sent
  it and keeps the draft. A bound app plays no fixture channel: live sections poll at `refresh:`
  (5 s by default; outside 1 s to 24 h is refused). Without `--model` the project is unchanged.
  `ess ui check --model` also binds a shell region's `does:` (beyond10x/ess#311).
- `ess ui run --tui --path <doc> --model <spec> --base-url <url>`: the terminal renderer reads
  and commands a synthesized server over HTTP/1.1 (`http://` only; `https://` is refused by name).
  With more than one served component, `--base-url <component>=<url>` is repeated; the caller
  comes only from `ESS_UI_AUTHORIZATION`. Answers go through `ess_ui::binding::classify`; a
  refusal shows on the open form, the confirm or beside the action row and keeps the draft, and a
  command answered committed is never sent again. A bound run plays no fixture channel and polls
  live sections at `refresh:`. `DataAdapter::run` returns `ess_ui::binding::Answer`
  (beyond10x/ess#311).
- `ess generate ui --target tui --path <doc> --model <spec> --out <dir>` emits a Rust terminal
  app crate: `Cargo.toml`, `src/main.rs` (clap), `src/binding.rs` and `src/ui.yaml`, depending on
  `ess-ui-tui` at the generating release's tag, so generate with a released `ess`. The app takes
  `--base-url`, `ESS_UI_AUTHORIZATION`, and `--screen-once <WxH>`, which prints one frame without
  a terminal and exits 3 when a read on the page failed. Generation refuses to overwrite files it
  did not write, and writes nothing when it refuses (beyond10x/ess#311).
- `ess-ui/1`, additive: style tokens and themes (`docs/design/ui-style-tokens.md`). A document
  may write `tokens:` (`color`, `space`, `radius`, `type`, `tone`), merged over a built-in table
  equal to today's React stylesheet; `themes:`, each the tokens it overrides; `theme:`
  (`default`, `chosen_by: shell.<name>`); and `tone_maps:`, named by `tone_by.tones` beside
  `tone_by.map` on a badge and by the new `tone_by` on an icon. The loader resolves `tones` to
  its map, and a widget parameter carrying a map name is typed `{ref: tone_map}`.
  `ess_ui::Document::base_tokens` and `theme_tokens` return a theme's full table. `ess ui check`
  adds `token_values`, `token_names`, `token_refs`, `theme_tokens`, `theme_choice`,
  `tone_map_refs` and `tone_map_unused`. A reader older than this release refuses a document
  using any of these keys rather than ignoring them. No renderer reads tokens yet.

### Changed

- **Breaking for a generated React project**: `ess generate ui --target react` emits a plain React
  project. Its only runtime dependencies are `react` and `react-dom`; `react-router`, Vite and
  `@vitejs/plugin-react` are gone. Routing is a generated `runtime/router.tsx` over the History API
  (`Link`, `Redirect`, `Outlet`, `useLocation`, `useNavigate`, `useSearchParams` with
  react-router's signatures), and `routes.ts` gains `matchPage`. `dev`, `build` and `preview` run
  esbuild 0.28 (`tsc --noEmit` stays the checker), serve on `127.0.0.1` and keep serving without
  stdin; `index.html` moves to `www/`. Aliases written as paths and aliases with encoded segments
  route as before, and the image primitive type-checks against `@types/react` 19.3
  (beyond10x/ess#315).
- **Breaking for a Rust realization that implemented such a command's `…Behavior` obligation**:
  generated Rust servers select an `existing_instance:` refusal beside a creation, and a creating
  `unknown_instance:` branch beside an update, by looking the input identity up in the storage
  port before dispatch. `Generated<P>` now runs that command's behaviour and no longer forwards it
  to `P`. A command whose lookup would not match its creations stays an obligation. Go, Web and
  Clap still refuse both forms by name (beyond10x/ess#310).

### Fixed

- Generated output committed with its `.ess-output` regenerates in another checkout (a clone, a
  second worktree, CI, any umask) when its owned files still have their recorded bytes. A
  regeneration that changes nothing leaves `state.json` byte-identical, and the first one that
  writes records the new root. A state carried without matching files refuses, lists them, and
  prints the steps to re-enroll with `ess generate output adopt`. A root replaced by a copy
  mid-transaction refuses recovery there (beyond10x/ess#306).
- Synthesis is no longer slowed by commands that read the caller: each such command cost two whole
  syntheses, and now costs one focused run; witness searches are answered once per model. One
  downstream specification goes from 151 s to 6 s with byte-identical suites (beyond10x/ess#301).
- An entity whose identity type has one value (a one-variant enum, or a newtype whose invariants
  admit one value) synthesizes as a singleton: existence scenarios use the one row, a second
  create is witnessed as `existing_instance`, decoys are left out, rows that reference it share it,
  and a command acting on it is sent by a second caller (beyond10x/ess#287).
- Synthesis no longer sends an optional input a boundary row leaves out, so a scenario can no
  longer require an accepting branch for an input that an enum-and-presence refusal claims; a step
  whose input an earlier branch takes is withdrawn and refused as ESS-SYNTH-019
  (beyond10x/ess#280).
- A suite is admitted with every number it was written with: a value just below 2^53 (an
  invariant bound such as 9007199254740991) was read back one lower, so the scenario at that bound
  failed a correct target (beyond10x/ess#251).
- Inside nested quantifiers, a binder on the right of a comparison is the binder, not a text
  literal; a quoted or shorthand word naming a binder is refused (beyond10x/ess#289).
- `ess-cli/1` callables may declare `invalid_input: <code>`, naming one of their declared errors:
  a typed shape failure, or an unparsable, empty or rejected dynamic payload, then answers that
  code with `{}` data and exit 2. A dynamic payload repeating a key is refused before the
  validator runs (beyond10x/ess#274).

## [0.50.0] — 2026-10-01

### Fixed

- Synthesis: an aggregate view whose creating command has a `when_related` guard is witnessed
  instead of refused with ESS-SYNTH-017. Each row's related row is arranged, shared by rows given
  one value of the guard's input; every input a guard predicate compares with that row's owner
  link names an arranged owner; and a group key read from a related row's owner link holds one
  owner per value (beyond10x/ess#272).

## [0.49.0] — 2026-10-01

### Added

- `ess/20`: a `when_related` predicate may read the related row's held lifecycle state as
  `state`; older formats refuse it with `unsupported_format_version` naming `ess/20`. From
  `ess/20`, synthesis arranges a related row of an entity already being arranged one level deep,
  in its initial state; suites for earlier formats are unchanged (beyond10x/ess#229).

### Changed

- **Breaking for a caller of a generated server in a specification that declares actors**: served
  surfaces enforce actor grants. The generated server's `dispatch` and `handle` (Rust) and
  `dispatch` (Go) take the caller your realization authenticated the request as, and `serve` /
  `Serve<Component>` take an `authenticate` function that returns it. `admit` (Rust) and `Admit`
  (Go) expose the check. Before a command runs, a request with no caller, or with an actor the
  specification does not grant the command, gets `403 {"refused": "not granted", "actor": <name
  or null>}`. A command no actor is granted is refused to every caller; views are not
  grant-checked. The OpenAPI contract of a served component declares this refusal on every
  command. For served components the synthesized suite adds `<command>/grant/denied`, which
  borrows a scenario whose send the command accepts and requires that the target's event log
  grows by nothing, and `<command>/grant/admitted/<actor>` where a granted actor would otherwise
  never send its command (suite/26). A specification that serves nothing gets a note that
  enforcement is the caller's. Authored scenarios can expect the refusal with `refused:
  not_granted` (ess-scenario/4). New codes: ESS-AUTHOR-038 and 039, and ESS-AUTHOR-040 for an act
  sending a served command no actor is granted. How a request proves its actor remains the
  realization's job (beyond10x/ess#265).

### Fixed

- Synthesis: a scenario sending a command that reads the caller now also runs with the callers
  swapped and a fresh caller-supplied identity. That identity is replaced in every copy the model
  makes of it, including converted ones, and is always drawn inside the guards that read it. A
  run with no fresh identity left is named in a note, and a guard that leaves no fresh identity
  refuses the scenario (beyond10x/ess#275).
- Synthesis: a command that copies a value from a related row (`{related: …}` in `sets:` or
  `payload:`) through the same input its `when_related` guard reads now gets its success
  scenario. One row the guard also accepts is created before it and another after it, under the
  same owner where the row has one, so a target copying from the first, last, least or greatest
  accepted row fails. Where no such row can be arranged, `Note::UnaccompaniedRelatedCopy` says so.
  Previously the branch was refused with ESS-SYNTH-008 (beyond10x/ess#270).
- Synthesis: a branch with a `when:` beside a `when_subject:`, declared before a sibling with the
  same `when:`, gets its outcome, transition and wrong-state scenarios; the first declared of
  several selected branches answers. A refused wrong-state witness no longer states `c and none
  of: c, …` for a sibling admitting the same inputs, however it is spelled: it names the one
  sibling taking those inputs first, or the row the witness needed (beyond10x/ess#278).
- Synthesis: an aggregate view grouped by a key the creating command copies from a related row
  (`{related: …}` in `sets:`) gets its `<view>/aggregate` scenario: each row first creates the
  related row holding the key it wants, so a target aggregating under the wrong key or ignoring
  the owner link fails. ESS-SYNTH-017 says "does not set" only of a key nothing sets
  (beyond10x/ess#257).
- Synthesis: a `when_related` guard over the `via` field of an `owns` relation is witnessed on
  both sides, with a row of another owner for the refusal and one of the input's owner for
  success, instead of being refused with ESS-SYNTH-003 (beyond10x/ess#271).
- `ess verify diff`: a declaration added or removed on one side, in any of the nine families,
  leaves no residual and yields only its `<family>/<name>/added|removed` change, not
  `unclassified-changed`. An `attributes` edit on an actor present on both sides is still
  `unclassified-changed` (beyond10x/ess#276).

## [0.48.0] — 2026-09-30

### Added

- `ess ui test --path <document> <tests…> [--format text|json] [--playwright <out>]` runs
  `ess-ui-test/1` files: tests that open pages, select nodes by canonical node path (row key for
  collection items), type, choose, act, page, expect text, rows and section state, play live
  events, advance time and expect commands, with per-test fixtures. They run headless against
  the terminal renderer and write `ess-ui-test-report/1` (exit 1 on a failure); `--playwright`
  writes a Playwright spec for the generated React project from the same file, selecting
  `[data-ui-path]`, where every step the terminal refuses is `test.fixme` with the same reason.
  `ess-ui-tui` gains a read-only `App::regions()`; generated React column headers carry their
  column's `data-ui-path`. Example tests: `examples/partner-portal/tests/`.
- `ess verify conform report --suite <suite.json> --results <results.json> --implementation <name>
  --report-out <path> [--runner <name>@<version>]` writes an `ess-conformance-report/2` from the
  per-scenario results of a runner outside ESS, in any language. ESS admits the suite (an original
  suite of any admitted format, or an `ess-conformance-input/1` carrier) and takes coverage, the suite
  reference and policy from that admission, so the report qualifies exactly as one from ESS's own
  run would, including a suite that nests as deep as `conform run --suite` admits. Exit 0 when
  written, 2 when refused.
- `ess-conformance-results/1` (unreleased) is the results document: closed `format`, exact u64
  `completed_at`, optional `suite_digest` and `results: [{scenario_id, status, message?}]`, where
  `status` is `passed`, `failed`, `error` or `unsupported`. The command refuses the whole document,
  naming every offending entry, for a result for a scenario outside the suite, a scenario with no
  result, two results for one scenario, any other status, and a `suite_digest` that is not the
  admitted suite's. An entry refused for its own structure (an unknown field, say) still counts as
  the result for the scenario it names, so that scenario is not also reported as having none.
  Nothing is written on a refusal. `message` is read and not carried into the report.
- report/2 `producer_profile` gains `external-scenario-status/1`, or
  `external-scenario-status/1;runner=<name>@<version>`, for supplied results. It follows the Rust
  category rules (`skipped` unavailable) and says that ESS executed nothing. ESS's own report/2
  reader admits it; a reader that admits only `rust-scenario-status/1` and `go-scenario-status/1`
  refuses it. `aep plan artifact evidence --from` admits it from aep 0.66.0; aep 0.65.0 and earlier refuse it.
- `ess generate release check-conformance`, `publish-conformance` and `publish` still qualify a
  passed report with this profile, and say so: `conformance: passed (results supplied by
  <name>@<version>; ESS executed nothing) for the supplied exact declared selection`, or `by an
  external runner` when the report names none.
- Library: `ess_conformance::results` (`report`, `admit_suite`, `ExternalResults`, `Runner`,
  `RESULTS_FORMAT`) and `CountReport::from_external`. `ProducerProfile` gains `External`.
  `CountReport::producer_profile()` and `CountReport::runner()` tell supplied results from an ESS
  run.

### Changed

- A served `501` says whether the command's effect was committed, in a boolean `committed` member
  every `501` body now carries (beyond10x/ess#260). An unmet obligation, where nothing was written,
  answers `{"refused": …, "committed": false}`; a committed command whose delivery to a binding
  failed answers `{"refused": "delivering what the command published: …", "committed": true}`,
  with the same words as before. The contract's `501` schema declares `committed` (boolean,
  required) for every command. The generated Rust and Go servers answer the same members, and a
  served view's `501` carries `committed: false`.
- **Breaking for a Rust shell that matches every variant of the server's `entry::Refused`
  without a wildcard arm**: the committed case is a new variant, `Refused::Undelivered(String)`,
  where 0.47.0 answered `Refused::Unmet`. Add an `Undelivered` arm. A shell that matched
  `Refused::Unmet(detail)` and tested `detail` for the prefix `delivering what the command
  published` no longer sees that case under `Unmet`; match `Refused::Undelivered`, or call
  `Refused::committed()`. A shell that only calls `status()` or `to_string()` is unaffected: both
  answer as before. `http::Response::from(&Refused)` renders a refusal as served, `committed`
  included.
- The web bridge's `unmet-obligation` refusal carries `committed` the same way, and a command
  whose delivery failed after it took effect is the new `BridgeError::Undelivered` (the same
  `kind`); a `match` over every `BridgeError` variant without a wildcard arm needs one more arm.
- A newer `ess` delegating to the release an `ess-inputs.yaml` pins (`requires: ess X.Y.Z`) now
  prints its one-line `note: ess … is the dispatcher; delegating to ess …` on stderr for every
  command, not only `--version`, so output written by the pinned release is not taken for the
  newer one's (beyond10x/ess#261). `ESS_TOOLCHAIN_QUIET=1` silences it; stdout is the delegated
  release's alone.

### Fixed

- `ess ui check --model <dir>` reads the specification through the directory's
  `ess-inputs.yaml`, as `ess specify validate --path <dir>` does, instead of every YAML file
  below it; `--model <dir>/ess-inputs.yaml` means its directory (beyond10x/ess#262).

## [0.47.0] — 2026-09-30

### Added

- `ess-ui/1`: a renderer-neutral UI document format (`schemas/ui/ess-ui.schema.yaml`) with
  shells, pages, sections, composites, primitives, widgets, reads, commands, live channels and
  state placement per node. `ess ui load` loads a document, `ess ui check` reports findings by
  canonical node path (`ess-ui-check/1`, with `--model` against an ESS specification),
  `ess ui docs` renders the format reference from its schema, `ess ui run --tui` runs a document
  in the terminal against its fixtures, and `ess generate ui --target react` writes a Vite +
  React + TypeScript project. Example: `examples/partner-portal/`.
- The system crate's `SystemEvent` has `name() -> &'static str`: the qualified name the
  specification declares each variant's event under. Generated for every variant, including
  events a generated binding delivery reacts to or escalates into.
- Where a component is `reached_by: network`, the server's `wire` module has
  `encode_system_event(&SystemEvent) -> String`, covering every `SystemEvent` variant and writing
  `{"event": "<qualified name>", "payload": {…}}` — the same envelope a command's answer lists a
  published event in. In `--layout crate` it is `server::wire::encode_system_event`, behind the
  `server` feature. A runner names and encodes what the system's log carries without keeping an
  event table of its own.
- `System::take_published() -> Vec<SystemEvent>` (Rust) and `System.TakePublished()` (Go) take
  every event the pump has already delivered off the log. A pump returns with every logged event
  delivered — a binding whose attempt stopped holds the event in its own held-back list, not on
  the log — and events published since the last pump stay for the next one. Where a binding's
  delivery is generated, `take_invocations()` / `TakeInvocations()`
  take the record of what the bindings invoked. A shell that pumps itself calls them to keep a
  long-running process bounded.
- `server::http::Request` has `headers: Vec<(String, String)>`: every request header in arrival
  order, the name lower-cased and the value trimmed, a repeated name kept twice. Routing does not
  read them; a shell reads `authorization` there to authenticate the caller before `dispatch`.
  `http::read` keeps at most `http::MAX_HEADERS` (100) headers and answers `431` beyond that.
- The Go target's server package has an `encodeEvent…` encoder for every generated event.

### Changed

- Every surface that answers a command now answers with `published`: every event the outcome
  published, in publication order, each `{"event": "<qualified name>", "payload": {…}}`. That is
  the Rust HTTP surface, each surface's transport-free `handle`, and the Go HTTP surface; the web
  bridge's `run` already listed them in that shape. The created identity of a creating command is
  in its event's payload. An outcome that publishes nothing answers `"published": []`. The other
  members (`outcome`, `error`, `payload`) are unchanged. Before this the Rust and Go servers'
  answers never carried the events; only the component-level `wire::encode_outcome_*` listed them.
- The OpenAPI projection declares it. Every command response schema requires `published`: one
  `prefixItems` entry per event the branch emits, in emit order, each `{event: <const qualified
  name>, payload: {$ref: <Event>.Event}}`, with `items: false`, so the same events in another
  order or one of them twice is not the branch's answer; a branch that emits nothing has at most
  zero items. Each emitted event's payload schema is added to `components.schemas` as
  `<qualified name>.Event`. The wording "published to consumers rather than returned here" is
  replaced: the events are published to consumers and listed under `published`.
- **`501` now has two meanings, and the contract declares it.** Every command operation in the
  OpenAPI projection, and in the contract a generated server serves, gains a `501` response
  (`{"refused": <text>}`): the realization is unfinished. Either a port the command runs reported
  an unmet obligation — as before — or, new, the command's effect was committed and delivering what
  *this* command published to a binding failed; then `refused` begins `delivering what the command
  published`, and each binding whose delivery failed keeps its event and is attempted again on a
  later pump, while no other binding receives it twice. An event an earlier command left
  undelivered never makes a later command's answer `501`. A client must not retry a `501`:
  after a failed delivery a retry performs the command a second time. The Rust and Go servers,
  and `handle`'s `entry::Refused::Unmet`, answer it the same way. The response-status set of every
  command grows by `501`, so a client generated from an earlier document sees a new status.
- **This is a wire change for existing clients.** Every command answer gains a required member,
  also for models whose commands publish nothing. A client that reads members by name is
  unaffected. A client generated from or validating against an earlier OpenAPI document, whose
  response schemas are `additionalProperties: false`, rejects the new answers until it is
  regenerated from the new document.
- The served dispatch — the HTTP route and `handle`, Rust and Go — pumps after every command, so
  every binding reacting to what it published is delivered before the answer, and then takes the
  delivered events and the binding record off the system. A long-running generated server no
  longer grows its log, its binding record or a component's outbox with every request. An
  in-process caller that read `System::published()` after `dispatch` or `handle` now finds it
  empty: the command's events are in the answer, and events a binding's command published in turn
  were delivered and taken. The log is taken whatever the pump answered. When delivering this
  command's events fails the command's effect stands and the answer is the unmet obligation
  (`501`, `entry::Refused::Unmet`); the binding that failed holds the event for a later pump.
- The served functions (`serve`, `dispatch`, `handle`) are bounded by what `System::pump` needs —
  each component's behaviours by its own port, and the system's obligations where it has any —
  instead of every component's behaviours by the served component's port.
- **Breaking for a caller that builds a `Request`: `server::http::Request` gains the public
  field `headers: Vec<(String, String)>`.** A shell that builds a `Request` with a struct literal
  (to call `dispatch` without a socket) no longer compiles until it supplies `headers` (for
  example `headers: Vec::new()`). A shell that only receives `Request`s from `http::read` is
  unaffected.
- A command whose contract requires no body — one with no input, or whose every input field is
  optional — is answered for a `POST` with no body or an empty (whitespace-only) body, as its
  input `{}`, by the Rust and the Go server alike. Before, such a request was refused `400` "the
  body is not JSON" although the contract declares no required `requestBody`. The document and
  the servers read that fact from one function, `ess_gen::http::body_required`.
- The generated Go server answers one request at a time against the system: the served dispatch
  holds one package-level lock from reading the input through the port call, `Pump` and
  `TakePublished`. `net/http` serves each connection on its own goroutine, and concurrent commands
  on one shared log could livelock `Pump` or race the realization's state. Requests still read
  their bodies concurrently.
- The pump — Rust `System::pump`, Go `System.Pump` — tracks delivery per binding, with a
  held-back list per binding. Every binding reacting to an event gets one attempt at it, and the
  pump passes the event once each has had its attempt, so an event that keeps failing no longer
  blocks every event after it. A binding whose attempt an unmet obligation stopped holds the event
  in its own held-back list when it declares `at_least_once`; so does a declared refusal its
  `on_failure: retry` answers, which before held the event for every binding. Each pump attempts
  the held events again for their own binding alone, after the events published since; one that
  fails again stays held and does not fail the pump. `pump`'s `Err` / `Pump`'s result is the first
  failure among the events it delivered for the first time — what was published since the last
  pump and what delivering that published — so a served answer is `501` only when delivering its
  own command's events failed. Before, an unmet obligation stopped the whole pump at the event:
  every later pump stopped there again, every later served command was answered `501`, and the
  bindings beside the failing one either lost the event or were delivered it again.
- The pump never delivers an occurrence again to a binding that already ran. An `at_most_once`
  binding runs once per occurrence: its stopped attempt is reported and not repeated, unless its
  own `on_failure: retry` puts the attempt back, and `redeliver` / `Redeliver` — which runs every
  `at_least_once` binding again — leaves it out. A failed selection ends that binding's attempt,
  not the event's: the other reacting bindings still run, and it is not held back, because its
  policy was applied.
- The web bridge runs the same Rust pump after every command, so it too keeps answering after an
  event that keeps failing: the command whose event stuck is refused `unmet-obligation`, and
  every command after it comes back with its own outcome.
- Regenerated: `generated/openapi/invoice-service.yaml`, `generated/openapi/email-service.yaml`;
  `generated/rust/gatepass` (`gatepass-server`: `entry.rs`, `http.rs`, `pass_service.rs`,
  `wire.rs`, `pass-service.openapi.json`; `gatepass-system/src/lib.rs`); `generated/rust/billing`
  (`billing-system/src/lib.rs`); `generated/go/gatepass` (`server/passservice.go`,
  `server/server.go`, `server/wire.go`, `server/pass-service.openapi.json`, `system/system.go`);
  `generated/go/billing` (`system/system.go`).

### Fixed

- `ess verify diff` no longer reports `system/<system>/unclassified-changed` beside an added or
  regrouped aggregate view: `aggregation` is compared as `grouping-changed` and
  `field-aggregate-changed` and is no longer left in the residual (beyond10x/ess#256).
- `ess import openapi` reads a closed tuple (`prefixItems` with `items: false`, or `maxItems: 0`),
  so the contracts `ess generate` now writes, whose `published` list is one, import without
  refusals.

## [0.46.1] — 2026-09-30

### Fixed

- `ess verify diff` classifies an outcome's error payload sources (ess/19 `payload:` keyed by the
  error a refusal reports) instead of reporting one `system/<name>/unclassified-changed`
  (beyond10x/ess#253). Declaring sources on an outcome is
  `command/<command>/outcome-error-payload-added/<outcome>`, dropping them is
  `outcome-error-payload-removed` and replacing them is `outcome-error-payload-changed`: one change
  per outcome, each side one `<Error>.<field> <- <source>` line per determined field, related
  `changed` as the event payload's `outcome-payload-changed` is.
- `ess verify diff` also classifies an outcome's `accepts: nothing` (ess/15), `returns:` (ess/17)
  and caller-decided refusal (ess/16) moving, as `outcome-accepts-nothing-changed`,
  `outcome-returns-changed` and `outcome-decided-by-caller-changed`, `{outcome, before, after}`
  booleans related `changed` as `outcome-refuses-changed` is. Each fell to
  `unclassified-changed` before.
- The new kinds need `ess-diff/12` (released in 0.46.1; `ess-diff/11` shipped in 0.42.0); writers
  and readers of `/3` to `/11` refuse them with `unsupported_format_version`. A delta without one of
  them keeps its format and bytes. A test now fails when a compiled outcome writes a key the diff
  neither compares nor deliberately leaves to the residual.

## [0.46.0] — 2026-09-30

### Added

- `ess generate synthesize --target rust` generates the behaviour of every command whose outcomes
  the specification fully determines, instead of owing it as a `…Behavior` obligation. The types
  crate gains a `behaviour` module: one storage trait per entity the generated behaviours read or
  write (get, put and delete a snapshot by identity; ess generates the trait, never a store), a
  `Context` trait (the caller's attributes, the identities and values the model says the
  implementation assigns, and the answer to each `external:` branch, forced or decided), and
  `Generated<P>`, which implements every generated `…Behavior` trait over those ports and forwards
  every behaviour and query still owed to `P`, so it is a complete bundle for every component port.
  Evaluation follows the conformance interpreter and the precedence order of
  `docs/design/cross-record-and-stored-field-guards.md`; moves go through the generated typestate,
  and a write to an entity declaring `invariants:` is refused when `broken_invariant` names one.
- Generated: `when:`, `when_subject:` (both shapes), `when_subject_state:`, `unknown_instance:`,
  `wrong_state:`, defaults, `external:` with and without an input guard; `creates:`, `moves:`,
  `updates:`, `deletes:`; `sets:` from input, literals, `{increment:}`, `{cleared}`, the caller,
  `{subject:}` and `{generated: true}`; payloads from input, subject fields, the caller and
  generated values; a declared error's fields where the held row determines them (its state, or a
  stored field of the same name and type).
- The Rust target also generates the query of every view whose rows the specification fully
  determines: projections of the source entity's own fields and `state`, `filter:` views,
  `order_by:` views, and `aggregation:` views with `group_by`, `count`, `count_distinct`, `sum`,
  `min`, `max` and `avg`, including `skip_absent`. The plan marks each such `view_query` capability
  generated. The query sits on `behaviour::Generated` and reads through `list`, a new method on the
  entity's storage port, emitted only where a generated query reads that entity. A filter keeps a
  row only where it holds; a false or unknown filter drops it. Aggregations follow the conformance
  suite's semantics: an absent group key forms one group, a number is keyed by its value, `avg` is
  rounded half-even to six digits, and an ungrouped aggregation is one row even when empty.
- The plan names, for each command behaviour or view query still owed, the construct that keeps it
  an obligation (`ObligationReason::Undetermined`, "kept an obligation by …"). For a command:
  `when_related:`, `{related:}`, `when_subject_state:` beside `external:`, a subject predicate
  choosing between a move and an update, an unknown identity no branch answers, `instances:`,
  `affects:`, a typed response, a retained result, error fields no source determines, a creation
  leaving a required field undetermined. For a view: a parameter (`params:`), paging (`paging:`),
  ordering by an optional field or by a `Timestamp`, or a filter with a guard the generator does
  not decide.
- The Go target keeps a seam, owed and stubbed, for each command behaviour and view query the plan
  marks generated, and names that in `TARGET.md` as a weakening. The unlinked web bundle refuses a
  generated query as `query ports`.
- The Rust synthesis target generates each entity's invariant check. An entity that declares
  `invariants:` gets `impl <Entity>Data { pub fn broken_invariant(&self) -> Option<&'static str> }`,
  returning the first declared invariant the value breaks (its declared text) or `None`. Every
  predicate form an entity invariant admits is evaluated: comparisons (numbers by exact decimal
  value, `Timestamp` by instant, other text by bytes), `any_of`/`none_of`, the string and
  case-insensitive operators, text and collection `.count`, list positions, `defined`, truthiness,
  `all`/`any`/`not`, and `forall`/`exists` over lists and maps. Reads of something absent — an
  empty `Optional`, a list position past the end, or `state`, which the data type does not hold —
  are unknown and break nothing, as the conformance interpreter reads them. An invariant the target
  cannot evaluate is refused at synthesis naming it. The shared evaluator is appended to the types
  crate's `primitives` module only in models that declare an entity invariant; other models keep
  their bytes.
- `ess generate synthesize --target rust` puts the declared actor grants into the types crate as
  data. The new `actor` module has an `Actor` enum with one variant per declared actor,
  `Actor::ALL`, `Actor::name()` (the qualified name) and `may(actor) -> &'static [&'static str]`,
  which lists the qualified names of the commands the actor may invoke. Generated code still
  enforces no grant. Each plan's `actor grants` row now says the grant is generated as data and
  that the caller enforces it. The Go target's `TARGET.md` names the missing grant table as a
  weakening. A model without actors produces identical bytes.
- **The generated Rust server has a transport-free entry point.** Each served component's module
  now has `pub fn handle(system, name, input: json::Value) -> Result<json::Value, entry::Refused>`,
  which runs any command or view of that surface by its qualified name with no socket. It calls the
  same decode, port and render function as the HTTP route, so the outcome is the one the route's
  body renders, and the refusals are the route's own: input the declared schema refuses
  (`Refused::Input`, the route's `400`) and an unmet obligation (`Refused::Unmet`, the route's
  `501`). A name the surface does not declare is `Refused::Unknown`, which names it. The dispatcher
  behind `serve` is now public too, so a caller can pass it a request value it built itself.
- `ess generate synthesize --target rust --layout crate` writes one crate at `--out` instead of a
  workspace: `Cargo.toml` and `src/lib.rs` at the root, one module per bounded context, the
  component ports under `src/ports/`, the bindings in `src/system.rs`, and the HTTP surface (`http`,
  `wire`, `json`, `entry`, route modules) under `src/server/` behind a `server` Cargo feature that
  is off by default. Without the feature the crate has no `std::net`; with it, the crate passes the
  same synthesized conformance suite as the workspace layout. `plan.json` records the layout in its
  `scope`, and `PLAN.md` and every file header name `ess synthesize --layout crate`. A bounded
  context named `ports`, `system` or `server` becomes `<name>_domain` in this layout only. The
  default `--layout workspace` output is byte-identical to before; `--target go`, `web` and `clap`
  refuse `--layout crate` with exit status 2 and write nothing. The library exposes it as
  `ess_synth::synthesize_laid_out` with `OutputLayout`, `LayoutRefusal` and `SynthesisFailure`.
- Specification format `ess/19`: an outcome that reports an error may say where the error's fields
  come from, with a `payload:` block keyed by the error —
  `payload: {orders.TooMany: {requested: input.quantity, limit: 10}}`. Each field takes the sources
  an event payload takes (an input, a literal, `{subject: …}`, `{caller: …}`, `{generated: true}`)
  and is checked the same way: a field the error does not declare is refused as
  `undeclared_reference` and a source of another type as `type_mismatch`. `{subject: …}` reads the
  row the refusal is answered for (`wrong_state:`, a held-state or stored-field guard) and is
  refused on an input-guarded refusal and on `unknown_instance:`. An earlier header refuses the
  block by name with `unsupported_format_version`; a model without it keeps its bytes and compiled
  digest. The compiled model carries the resolved sources as `error_payload` on the outcome.
- The conformance interpreter carries the declared fields on the error it reports, and a
  synthesized suite's `expect_error` compares every declared field whose value the scenario
  determines. An error field with no declared source is carried as none, as before.
- `ess generate synthesize --target rust` generates the behaviour of a command whose every error
  field has a declared source (or is read from the held row, as before), filling each field from its
  source.
- The generated `PLAN.md` has a **Ports — yours to provide** section wherever the plan generates a
  command behaviour or a view query: one storage port per entity a generated behaviour or query
  reads or writes, and a context port for the caller's attributes, the values the implementation
  assigns and the `external:` branches. Synthesis generates each port's contract and never an
  implementation of one. A plan that generates neither keeps its bytes.
- The synthesis guide states the rule — what the specification fully determines is generated; what
  it cannot determine is an obligation — and lists the ports, the constructs that keep a command or
  a view query an obligation, and actor grants as generated data. The commands-and-outcomes guide
  documents `ess/19` `payload:` sources for an error's fields.

### Changed

- The committed billing plan is 48 capabilities: 40 generated, 4 obligations, 4 refused
  (`IssueInvoice`, `CancelInvoice` and `SendEmail` are generated); gatepass's is 29: 26 generated,
  1 obligation, 2 refused (`AdmitVisitor` and `SignOutVisitor` are generated). The committed trees
  under `generated/rust/billing` and `generated/rust/gatepass` are regenerated, including the new
  `entry.rs` in `generated/rust/gatepass/crates/gatepass-server/src/`.
- The generated Rust `Decimal` and the web target's notes no longer state that behaviour is never
  synthesised.

### Fixed

- A served command outcome whose declared error has no fields no longer gives the generated server
  crate an unused `error` binding, which failed a consumer's `-D warnings` build.

## [0.45.0] — 2026-09-29

### Added

- `ess generate synthesize --target go` represents `Json` as the generated `primitives.Json`, a
  wrapper over the document text, at every position the Go target types: newtypes, struct members
  (bare, `List`, `Map`, `Optional`), union variants, entity fields, command inputs and responses,
  events, errors, view rows and binding-copied fields. The served surface reads a `Json` value with
  its object members in the order they arrived and its numbers as spelled, and writes it back
  through `json.RawMessage`; a model without `Json` synthesizes the same Go bytes as before
  (beyond10x/ess#224).
- `ess generate synthesize --target web` represents `Json` at every position the web target types:
  newtypes, struct members (bare, `List`, `Map`, `Optional`), union variants, entity fields, command
  inputs, events, errors, view rows and binding-copied fields. The bridge crate re-exports the Rust
  types crate's `json::Value`, so the module carries a `Json` value unchanged (member order and
  number spelling kept). The page edits one as JSON text and holds it as `JSON.parse` answers it,
  typed `JsonValue` for `tsc --checkJs`; `TARGET.md` states the page's limit (numbers are doubles,
  integer-like member names first). A model without `Json` synthesizes the same web bytes as before
  (beyond10x/ess#224).
- `ess generate synthesize --target clap` accepts a model that uses `Json` (beyond10x/ess#224). The
  generated CLI carries the Rust types crate's `json` module byte for byte, and a `Json` flag (bare,
  optional, repeated or through a newtype) takes one argument holding a JSON document, read at parse
  time into `json::Value` with member order and number spelling kept; a malformed document is a
  usage error naming the byte it stopped at. A handler prints a `Json` response unchanged with
  `json::push_value`. A struct, union or map input holding `Json` stays one free-text flag, as for
  any member type. A model without `Json` synthesizes the same bytes.
- With `rust` representing `Json` since 0.44.0, no code target refuses a model for using `Json` any
  more.

## [0.44.0] — 2026-09-29

### Added

- `ess generate synthesize --target rust` represents `Json` as the generated types crate's
  dependency-free `json::Value` (beyond10x/ess#224). It covers newtypes, struct members (also in a
  list, a map or an `Optional`), union variants, entity fields, command inputs and responses, event
  and error payloads, view rows and binding mappings. The types crate carries the `json` module only
  when the model uses `Json`, and the server crate re-exports it. The wire codecs carry an object,
  array, number, string, boolean or null unchanged: members keep their order and numbers keep their
  spelling. `--target go`, `web` and `clap` still refuse `Json` and name the target. A model without
  `Json` synthesizes the same bytes as before.

### Changed

- The public documentation is reorganized for somebody adopting ESS. Start here pages cover
  install (macOS and Linux), a first specification, a first conformance run, one page per runner
  (TypeScript, Go, Rust) and use with an agent, and every command they show is run by
  `crates/edge/ess-cli/tests/tutorial_page.rs`. The two long guides are split into task pages, with
  the old pages kept as indexes so old links and anchors still land. The CLI reference's command
  sections are generated from the command definition (`cargo xtask cli-reference`), a new
  diagnostics page lists every code and refusal name with its repair (`cargo xtask diagnostics`), a
  what-changed page is generated from `changes/`, and `cargo xtask docs` now fails for any format
  family a page names that is not tracked, and for a README that installs an older release.

## [0.43.0] — 2026-09-29

### Fixed

- **An input-guarded refusal beside held-state branches** (beyond10x/ess#227, first filed as
  #213). A `when:` + `error:` branch naming no subject may now sit beside `when_subject_state:` and
  `when_state_changes:` branches, so "the new value is refused whatever the record's state" has a
  form on a command acting on an existing record. It was refused as ESS-COMMAND-003
  (`unobservable_fact`), and naming a subject on it is still `refusal_mutated_state`. No key is
  added, so no format gates it: it validates at every format that has `when_subject_state:`. It is
  answered before existence and before the held state (the #209 precedence): the joint state × input
  partition counts an input it claims as its alone in every state, and where the prover cannot
  decide its guard (`secret.count < 12`) the other branches are proved over every input on their
  own. Synthesis sends the refused input for an identity nothing stores, then on a record of its own
  arranged and observed in each lifecycle state, and at the overlap with each sibling that runs in
  that state and reads the input; each send requires the error, no event and the row unchanged. A
  target reading the held state, the record, or a state-guarded sibling before the input fails. The
  model interpreter answers an input-guarded refusal before anything else is read, so it decides a
  request such a refusal claims on a command whose other branches it does not interpret yet; Entity
  Runtime already decides it before the row is loaded. Of two input-guarded refusals one request selects, the first declared answers, as Entity Runtime orders them: validation admits the overlap (it was `conflicting_declaration` on a held-state command), the interpreter returns that one refusal, and synthesis has a refusal's witness refute only the refusals declared before it, so a refusal nested inside an earlier one keeps its scenario rather than being withdrawn. Where the prover cannot decide one refusal's guard, only that refusal leaves the joint proof; a decidable one beside it still counts. One precedence order now answers every command, written once in the cross-record guards design note and linked from the others: on a `when_related:` command `existing_instance:` then `exists: false`; then input-guarded refusals, first declared; then existence of the addressed row; then the held state; then accepting and external branches in declaration order. The interpreter answers a missing related row by its `exists: false` branch before an input refusal, and synthesis sends that branch an input a refusal claims, so a target checking the input first fails it. Committed suites are byte-identical.

- **The generated explorer decides a step in Entity Runtime's order (beyond10x/ess#235).** The
  seeded explorer in the Go and TypeScript conformance packages answered a command's `wrong_state`
  branch before evaluating any guard, so a target that refuses a bad input whatever state the
  subject is in — as Entity Runtime and synthesis do — was reported as disagreeing. The explorer now
  takes an input-guarded refusal (a `when:` with an `error:`) first, the first declared whose guard
  holds, before it reads the record, its state or an external branch; then the accepting guarded
  branch or the default; and answers `wrong_state` only where that branch moves from a state no
  move of the command starts from. A branch that moves nothing answers in every state, an eligible
  external branch included: on a subject resting where no move starts, the explorer offers it
  beside `wrong_state` and, once arranged, expects it. Two accepting guards that both hold still
  leave the draw ambiguous, unless every one of them moves from such a state.
  `docs/design/mutation-audit-and-model-runner.md` states the order.

- **Synthesis witnesses a stored-row predicate over an `Optional` field the creating command leaves
  absent (#239).** A `when_related: {via: …, predicate: {site: {exists: false}}}` branch, where the
  related row's creating command does not write the `Optional` `site` and a later command sets it,
  was refused `ESS-SYNTH-003` ("no candidate of the 2 tried … over the rows 2 bounded arrangements
  left"): the search offered the row as the creator leaves it, but read its unwritten `site` as
  undetermined, so neither `exists: false` nor `not defined(site)` held on any row. An arranged
  row now records the `Optional` fields no step since its creation wrote, and a predicate reads
  each as absent until a later `sets:` writes it. The branch is witnessed on the row the creator
  left, its sibling on a row the later command wrote, and a target reading the field as present
  fails. The same holds for `when_subject:` and for a `defined()` guard in either direction. A
  field some writer outside the row's own steps can reach — an `affects:` or `instances:` outcome
  on the entity, or an `updates:`/`moves:` of a command a binding invokes — is still read as
  undetermined, since a decoy's act or the target's own reaction may have written it. The
  observation before the command names the row's identity and state but does not require the
  absent field, since a view row may leave an absent field out. An `Optional` member of a struct
  field, left out through an omitted `Optional` input, is still refused `ESS-SYNTH-003` by name: a
  struct with an undetermined member is not determined as a whole. Committed suites regenerate
  byte-identical.

- **A guard over a stored `Map`'s entries is witnessed on both sides (#240).** A stored
  `redirect_uris: Map<String, String>` set from a map input held one entry on every arranged row,
  but conformance published no fact for a map's entries, so `exists`/`forall` over a map — on a
  command's input, on its subject (`when_subject`) or on a related row (`when_related`) — was
  undecided everywhere, and `{not: {exists: {in: redirect_uris, as: r, that: r ==
  input.application}}}` and its accepting default were both refused as `ESS-SYNTH-003`. A map now
  publishes its values at their positions in key order, which is what the quantifier binds: the
  compiler types `r` as the value type and Entity Runtime folds the values in canonical key order,
  and `predicates.md` now says so. A comparison between the bound element and an `input.` field
  inside a quantifier over a stored list or map is grounded on each element the arranged row holds,
  so the input is sent once equal to a value the row holds and once equal to none; a target that
  ignores the entries, reads the map's keys or reads only whether it is empty fails. The same
  guards over a stored `List` compared with an `input.` field were refused the same way and are now
  witnessed. The arranged row holds several entries where the collection is written from an input:
  where one element decides the quantifier — an `exists` that holds, a `forall` that fails — that
  element is neither the first nor the last in key order, and where the whole collection decides it
  the row holds two or more, so a target reading only the first value, only the last, or `forall` for
  `exists` (and the reverse) fails a scenario too. A `Timestamp` map value orders by its instant
  inside a quantifier, as a list element does. A model that quantifies over no map and compares no
  quantified element with its input keeps its suite bytes.

- An authored act that names an `external:` branch under `outcome:` now compiles into a
  `configure_external_outcome` for that branch immediately before its `execute_command`, as a
  synthesized scenario does, so a target is told which answer to give for that one call
  (beyond10x/ess#243). Before, the act compiled into a plain `execute_command` + `expect_outcome`
  that no target could satisfy deterministically. The branch name is the stated answer, so no new
  key and no new `ess-scenario` version are needed; acts naming a branch the input decides compile
  byte for byte as before.
- `ess verify conform author` (and `ess specify validate` with a `scenarios:` list) refuses an act
  with `ESS-AUTHOR-037` when one of its claims holds only on an `external:` answer it does not state,
  whether or not `outcome:` is written. Checked: its error, its direct response, each event it claims
  published and each it claims absent. Answers reached: those of the act's own command and of every
  command a binding invokes from what it publishes, transitively; a binding's escalation needs its
  invoked command to fail. The refusal names every such branch as `command/branch`. The one answer an
  act states is its own command's external branch under `outcome:`; there is no key for a binding's
  call, so a claim of an escalation such as `billing.email.DeliveryEscalated` on a `CreateInvoice` act
  is refused naming `billing.email.SendEmail/failed` and left to synthesis. An event another command
  publishes on an input-decided branch exempts a claim only when the act reaches that command.
  Coverage inventories (Rust, Go, TypeScript and the browser admission) accept `ESS-AUTHOR-037` as an
  authored refusal.

- **Synthesized inputs satisfy the invariants over their nested members, and guards over nested
  input paths are witnessed (beyond10x/ess#234).**
  - An entity invariant over the members of a struct input a branch copies into a stored field
    (`sets: {fingerprint: input.fingerprint}` beside `fingerprint.version == "canonical-v1"` or
    `fingerprint.origin == fingerprint.route`) is met by every input synthesis sends, so the
    scenarios creating or updating the entity no longer leave a row its own invariant refuses.
    Before, the members carried their placeholder witnesses and every such scenario failed
    against a correct target.
  - A struct type with invariants used as a command input is synthesizable. Before, every
    candidate was refused by the type (`ESS-SYNTH-003`, "no candidate of the 0 tried").
  - The witness solves its base for those invariants once, per instance: the leaves an invariant
    reads are tried at its literals, one either side, and at each other's value for a comparison
    of two members, and a list whose `.count` an invariant reads at the lengths it names. A further
    instance tries each numeric literal moved by its ordinal first, so it still differs from the
    first wherever an invariant bounds a member rather than pinning it (`start >= 5`). A base that
    already satisfies them keeps its bytes.
  - Every input is held to the entity invariants of the branches it can reach, not to all of them
    at once. Two branches copying one input into entities that disagree (`version == "v2"` and
    `version == "v1"`) each send an input their own entity accepts. A guard moving a member an
    invariant ties to another (`origin == "eu"` beside `origin == route`, `low > 10` beside
    `low < high`) moves the other with it. An input that still breaks one is never sent.
  - A branch whose entity invariants no bounded input meets (a strict chain over five members) is
    refused `ESS-SYNTH-003`, naming each invariant. Before, it was sent an input the entity
    refuses, and its scenarios failed against a correct target.
  - A `when:` equality between an input and a member of another input (`owner == ticket.owner`)
    is witnessed on both sides: each side is also tried at the other's value. Before, the
    accepting branch was refused `ESS-SYNTH-003`.
  - A wrong-state scenario whose sibling `when_subject` compares a stored field with a nested
    input path (`fence != input.publication.expected_fence`) sends an input equal to the value the
    arranged row holds. Before, it was refused `ESS-SYNTH-003` ("a row in this state refuting
    every one of: …").
  - An input-guarded refusal declared beside a `when_subject` over a member of a stored optional
    struct is witnessed on a row whose struct no arranging step determined: the refusal answers
    before the row is read.
  - A `when_subject` or `when_related` guard over a member of an `Optional` struct the row holds
    absent (`meta.tier == Gold` where the creator left `meta` unwritten, as in beyond10x/ess#239)
    is read as a target reads it: unknown, so not taken. The default and every other branch are
    witnessed on that row, a wrong-state scenario counts the guard as refuted there, and only the
    guard no row takes is refused with its `ESS-SYNTH-003`. Negated, the guard stays unknown and
    is not taken either. An `Optional` field whose only later writer generates it is arrangeable
    from the row its creator leaves it absent on. Before, every branch of the command was refused.

- **`{$instance: …}` inside a list, a map value or a struct member of an authored step**
  (beyond10x/ess#242). `ring_sequence: [{$instance: a}, {$instance: b}]` for a
  `List<ReleaseRingId>` input was refused as ESS-AUTHOR-015 "expected Uuid, found a mapping", so a
  command taking several identities could not be authored; inside a map value the reference was
  not checked at all and reached the target as the mapping `{"$instance": "a"}`. A reference is now
  admitted wherever the declared type at its position is the instance's identity type, at any
  depth, a union payload typed by the variant its tag names included, and resolves to the identity the run bound, as a whole-field one does. At a position of
  any other type it is refused as ESS-AUTHOR-022 naming the position (`labels[1]`, `pair.note`,
  `tags[owner]`, `target.value`); one that nothing can place, at an undeclared member or inside a
  value of the wrong shape, is refused naming it in every container, inside map values and unions
  too, where the shape check reads no member. Inside an event payload or an error it is refused
  as ESS-AUTHOR-021, as a whole-field one is. Such a value is written as the new scenario value kinds `list` (`items`) and `members`
  (`members`), whose elements are literals, instances or those kinds again, in suite
  `ess-conformance/32` (`/33` with coverage); each implies every major below it, older readers
  refuse the envelope, and a structured value holding no reference stays a `literal`, so every
  other suite keeps its format and bytes. The Rust runner and the model interpreter resolve it;
  Go and TypeScript generation refuse such a suite, naming the Rust runner; the browser player
  describes it element by element.

## [0.42.0] — 2026-09-29

### Changed

- `ess verify conform mutate` no longer scores as `survived` a mutant no scenario could kill
  (beyond10x/ess#218), on `--target` and `--emit`/`--collect` alike:
  - A guard mutant that leaves its outcome's guard satisfied by no input — `any: [x == A, x == B]`
    flipped to `all:` — is `equivalent` (`ESS-MUTATE-005`), and its entry names the guard as
    `unsatisfiable_guard`. The guard is decided only for equality, membership and truth tests of
    non-optional scalar input fields (not a `Timestamp`) against literals, with the baseline's
    guard at the same outcome satisfied, by trying every combination of each field's values that
    could matter — a boolean's two, an enum's variants, and for any other field its literals and
    one value none of them equals — where there are at most 64. The combinations are counted
    before any invariant applies, so a guard is never called dead that an input satisfies; an
    ordering, a text match, a quantifier, a comparison of two fields, or a guard with more
    combinations is not decided, and such a mutant is scored as before. A failing scenario still
    kills it, and a changed scenario nothing scored keeps it `inconclusive`; otherwise
    `equivalent` outranks `unwitnessed`, so the #203 shape (a flipped connective over two
    disjoint equalities) is now `equivalent` with its added refusal still named. `counts` gains
    `equivalent`; an equivalent mutant does not change the exit status, but a run whose every
    mutant is equivalent or stillborn exits 3.
  - A mutant on an outcome whose scenario the baseline suite already refused at synthesis (for
    example `ESS-SYNTH-003` on a branch no arrangement reaches), and that no baseline scenario
    takes, is `unwitnessed` (`ESS-MUTATE-004`, exit 3) instead of `survived`; its entry names that
    refusal as `baseline_refusals` (`{code, scenario, subject}`). So is a `from-drop` or
    `transition-to` mutant on a transition that only such outcomes perform, naming each of their
    refusals; `--collect` reads the emitted baseline `ir.json` for which outcomes perform a
    transition. A mutant on a witnessed outcome or transition that no scenario kills is still
    `survived`.
  - The audit writes `ess-mutation-report/3`, and `--emit` writes `ess-mutation-manifest/3`,
    which records `unsatisfiable_guard` on each mutant whose guard the emitter found dead;
    `--emit` reports how many mutants have one. `--collect` still reads the
    `ess-mutation-manifest/2` 0.41.0 wrote and the `/1` before it, and scores no mutant of either
    `equivalent`; a `/1` or `/2` manifest carrying `unsatisfiable_guard` is refused.

### Fixed

- **Synthesis witnesses a `when_subject` guard comparing a link field with an input (#193).**
  `account_id != input.account_id`, where `account_id` is the link to the row's owner and the
  input is of the owner's identity type, was refused as `ESS-SYNTH-003`. The branch the input
  selects when it names the row's own owner is sent that owner; the branch it selects when it
  names another is sent a second owner arranged beside the row, holding a row of its own, so a
  target that ignores the guard, compares with the wrong owner, or asks only whether the named owner
  holds any row, fails. Where the comparison sits in `all:` or `any:` beside another stored
  field, each conjunct and disjunct is witnessed alone with the input bound. Such a row is refused
  as `ESS-SYNTH-003`, naming it, where it lies past the bound on further rows or where the bounded
  search missed it with an input naming an arranged owner leaving it undecided; a row every
  candidate decided and no bounded arrangement meets adds nothing, as for every other guard.
  Under a `cardinality: one` owner relation, a branch that files the row under the owner the input
  names is sent a second owner holding no row, so the owner never holds two. Further rows take
  instance names no earlier step of the scenario binds.
  Both sides are decided on the opaque instance tokens the
  view-filter fix binds: only `==` and `!=` between the link and that input are decided, and an
  ordering, a literal or a text test over either stays refused as before. Where the second owner
  cannot be arranged, the branch is refused as `ESS-SYNTH-004`, naming the owner's entity.

- A suite synthesized from a model whose actors declare `attributes:` records that model's
  `spec_digest` and `contract_digest` (beyond10x/ess#216). Synthesis reads such a model once per
  caller assignment, each time over a copy with the caller's values written in, and the suite took
  its digests from that copy. `ess verify conform run --target interpreted` then refused the very
  specification it had synthesized the suite from ("its spec_digest … is not the suite's
  spec_digest …"), and an adapter binding the model's digest would have refused the suite too. A
  model without actor attributes keeps its suite bytes; a suite synthesized from a different model
  is still refused.

- **Two overlapping accepting guards have a declared answer** (beyond10x/ess#217). `validate`
  accepted `small: amount < 100` beside `flagged: amount > 50`, and nothing said which branch
  `amount: 75` takes. Among the accepting guarded branches of one command, the first declared whose
  guard holds answers; input-guarded refusals are still taken first
  (`docs/design/input-guard-overlap-precedence.md`). An `external:` branch takes its place in the
  same declaration order: an accepting branch declared before it answers an input its guard claims,
  whatever the provider says, which is the order Entity Runtime already applied. No source or suite
  format change. Synthesis holds a target to it for each pair of accepting `when:` branches: a later
  branch's witness refutes every accepting branch declared before it, an external branch's witness
  refutes every accepting branch declared before it, the first-declared branch's scenario also sends
  an input in each overlap and requires that branch, and a branch whose every tried candidate an
  earlier one claims is refused with `ESS-SYNTH-003` naming it. In a command whose branches also read
  the held state, the overlap is sent in the state the scenario arranged. A binding that would force
  an external branch declared after an accepting one is refused like a guarded one.
- **An overlap no scenario sends is listed, not dropped.** A `Decimal` compared with two literals less
  than two apart (`11 < amount < 12`) held no value the witness ladder tried, so the overlap was
  skipped and a branch there was refused as unreachable. Every witness and overlap search now tries
  a second pass at the exact midpoint of each two adjacent literals (`11.5`, `0.15` for
  `amount > 0.1 and amount < 0.2`); a witness the ladder already found is unchanged. An overlap the
  scenario of the branch taken first still does not send — one no candidate reaches, or one arranged
  over a stored row, a replay, a preserved subject or a held state another branch also claims — is
  reported as a note naming both branches, unless the candidates cover every region and none lies
  in both.
- **The interpreter answers by the declared precedence.** It selected every branch whose `when:`
  held and refused the overlap as open, and an Unknown guard anywhere made the call undecidable. It
  now reads input-guarded refusals first (beyond10x/ess#178), then accepting and external branches
  in declaration order, and stops at the first that answers: a later guard it cannot decide no
  longer matters once an earlier branch holds. A forced external branch declared after a holding
  accepting branch is not taken. Entity Runtime already selected this way; tests now pin the
  accepting order and an accepting branch declared before an external one. Suites for models
  without such an overlap keep their bytes.

- **`ess verify diff` names a newtype's `prefix:` change** (beyond10x/ess#219). A `prefix:`
  (ess/15) declared on a newtype is `type/<T>/prefix-added`, which narrows; one dropped is
  `prefix-removed`, which widens; one replaced is `prefix-changed`, which narrows when the new
  prefix extends the old one, widens when the old one extends the new one, and is `changed`
  otherwise. Each is `ess-diff/11` vocabulary (unreleased; `ess-diff/10` shipped in 0.41.0
  without them): an `ess-diff/3`–`/10` writer refuses it, and such a reader refuses it with
  `unsupported_format_version`. The prefix no longer falls to the residual, so such a change is
  no longer reported as `system/<name>/unclassified-changed`. A delta without a prefix change
  keeps its format. The relation compares the declared prefixes: an outer newtype restating its
  inner layer's prefix is reported as narrowed or expanded although its admitted values do not
  move.
- **A type's reading-contract change is reported once.** `reading-contract-changed` was reported
  with a `system/<name>/unclassified-changed` beside it, because the type's `reading` was also
  left to the residual. The delta for such a pair loses that second entry; its format stays
  `ess-diff/3`.

- **Synthesis witnesses a branch selected by a stored counter at its limit (#226).** A branch
  whose stored guard compares a counter with a number literal (`retries >= 3`, `retries == 3`,
  `credits <= 0`), where the counter is moved by `{increment: n}`, was refused as `ESS-SYNTH-003`:
  every row short of the limit decided the guard alike and was one search node, and a row the
  raising command left no longer held a determined value. The stored-row search now follows such a
  counter's value within 16 of each literal its guards compare it with, and carries the value each
  raise leaves, so it repeats the raising command — the guarded command itself, or another one — up
  to the limit. Each side of the limit is witnessed: the branch at the limit is the first row the
  search reaches that holds it, and the row beside it is witnessed once more, and past it for `==`,
  so a target whose limit is off by one either way fails. Each side row sits at the nearest value a
  run of the counter holds there — from the values it is created or set to, moved by its
  increments — so `{increment: 2}` from 0 against `retries >= 3` is witnessed at 2 and 4. A side row
  asserts whichever branch the command answers on it, so a sibling band (`blocked: 3..4` below
  `over: >= 5`) is asserted there rather than refusing the default. The comparison may be a
  conjunct of the guard, a disjunct of an `any:` among its conjuncts, or under `not:`
  (`{not: retries < 3}` is witnessed as `retries >= 3`); at the limit the other disjuncts are
  refuted. A side the model never holds a value on — past a refusal that stops the raises — adds
  none. A side whose nearest value lies farther than 16 from the limit, or that the search does not
  reach within that bound, is refused as `ESS-SYNTH-003` naming the step and the limit, and only
  that row: the branch's own scenario stands. A limit farther than 16 from where the counter can
  be followed, or counters whose followed values exceed the search budget, is refused as
  `ESS-SYNTH-003` naming that bound; a search that left every row a raise moving away from the
  limit reaches no longer claims it. A counter compared with an input (`retries >= input.max`) is
  reached through the input as before. Models without such a guard keep their suites.

- **A relation carried by the entity's own identity** (beyond10x/ess#230). `via:` may name the
  identity of the entity holding the relation: an entity keyed by `user_id` declares
  `{name: user, kind: references, target: demo.provisioning.User, cardinality: one, via: user_id}`,
  a one-to-one link keyed by the same id, and the same relation may be declared from the other
  side. It was refused as `missing_declaration` ("carried by `user_id`, which … does not declare").
  The identity is type-checked against the target's identity as a declared field is.
  `cardinality: many` through an identity, a `references` from an entity to itself through its own
  identity, and an `owns` through the owned entity's identity are refused as
  `conflicting_declaration` naming the cause. The `missing_declaration` hint for an unknown `via:`
  now lists the identity beside the fields.
- The identity-carried relation reaches every consumer: `x-ess-relation` on the identity property
  of the entity schema and `x-ess-entities`, the relations sentence of the generated docs, and the
  identity line of the synthesised Rust `…Data` struct. A `references` is never arranged as an
  owner.
- A `{related: …}` read through an identity follows the relation it carries: through the existing
  subject's identity, through the identity a `creates:` branch fills from its input, and through
  the input a branch names its instance by (`via: input.<field>`), where the identity type alone
  names several entities. Synthesis arranges the referenced row between two decoys and keys the
  subject by it, as for a field-carried reference. A `when_related:` guard on an update or move
  whose input names both the subject and the related row is refused under `arrange_related_row`.
  The identity counts as a carrier only where it carries such a relation to another entity: a
  `creates:` branch cannot read the row it is creating through its own identity, and an earlier
  branch keyed by the same input does not hide the relation that decides a `when_related:` guard.
- Models without the construct keep their bytes and validate as before.

## [0.41.0] — 2026-09-29

### Added

- **`ess/18`**, the source format of this release's new authored constructs. Every construct below
  is refused under an earlier header with `unsupported_format_version`; a model without them keeps
  its bytes and compiled digest.
- **A refusal scoped to some held states** (beyond10x/ess#201). `when_subject_state:` may list
  several states (`[Delivered, Cancelled]`), and a refusal may carry it without naming a subject:
  it reads the subject its siblings name. A command can now accept a re-send in one state no move
  starts from and refuse in the others, where `wrong_state:` gave them all one answer. Each state
  is named once; every state left to the default must be one its move starts from.
  `when_state_changes:` still needs the branch's own move. Synthesis writes one
  `<entity>/state/<S>/refuses/<command>` scenario per wrong state `S`: it witnesses every guarded
  branch that answers `S`, each on its own row arranged and observed in `S` with an input that
  selects it, then the plain wrong-state row where an input still reaches it. An accepting branch
  with a list is witnessed in every state it lists. Entity Runtime lowers a list to membership of
  `$from_state`.
- **`state` in a `when_subject` predicate** (beyond10x/ess#204): the held lifecycle state beside
  the stored fields, `{all: [state == Ready, hold_note != ""]}`. A branch reading it that moves must
  be able to move from every state it may be selected in. Guarded branches select before
  `wrong_state:` applies, so the predicate may name a state no move starts from. Synthesis arranges
  the row in the state and witnesses each conjunct's boundary; Entity Runtime lowers `state` to
  `$from_state`.
- **A guard over a row of another entity** (beyond10x/ess#211). An outcome may carry
  `when_related: {via: input.<field>, exists: false}`, taken when no row of the entity whose
  identity `input.<field>` carries exists, or `when_related: {via: input.<field>, predicate: …}`,
  taken when that row exists and the predicate over its stored fields (and `input.`) holds. It sits
  on any branch, a `creates:` and a refusal that names no subject included, and composes with
  `when:`. One hop, keyed by the other entity's identity only; a lookup by any other field stays out
  of scope. A missing row makes the predicate unknown and selects only the `exists: false` branch,
  which a command with a predicate branch must declare. A missing row is answered by that branch
  before any other, so it carries no `when:`. `existing_instance:` may sit in the same command and
  answers first: the command's own identity is checked before the related row is read. On one
  branch beside a `when_subject*`, `wrong_state`, `unknown_instance`, `existing_instance`,
  `input_absent`, `external` or `replays` key it is `conflicting_declaration`.
- Conformance synthesis witnesses every branch of such a command: `exists: false` with an identity
  no row carries beside two rows that carry others, and each other branch on a related row created
  between two decoys, searched toward the predicates so each side is witnessed. A related predicate
  with two or more connective children is witnessed once more per child, on a row isolating it (a
  conjunct refuted alone, a disjunct held alone), with the branch the command answers there
  asserted; a boundary no arrangement reaches is refused under the branch's scenario id. A creator
  needing a related row of an entity it is already arranging (a folder inside a folder) gives way
  to the next creator. Where the related row is the row the creation is owned by, through the same
  input field, it is arranged once. A driver running such a command for another scenario arranges
  the related row first, and each further row it creates carries its own identity and its own
  related row. Families that would send the command without a related row (boundaries, unknown
  identities, illegal moves) refuse with the new strategy `arrange_related_row` named.
- The interpreted target reports a scenario of such a command `unsupported`, naming the guard.
  Entity Runtime refuses the command with `RelatedGuardUnsupported`.
- **An event binding reads the delivery context its event arrived with (`ess/18`, beyond10x/ess#195).**
  A binding declares `when.context_fields` (a typed record separate from the payload) and
  `when.context_authority` (the external channel whose authority binds it), and reads a field as
  `context.<field>` in `mapping:`. The context is admitted only for an event that no command
  outcome emits and no binding escalates into. Beside `periodic:` it is refused. Below `ess/18`,
  both the keys and `context.<field>` are refused, in YAML and JSON sources. A missing or
  undeclared context field is a refusal, never a payload lookup, and `event.<channel>` is still
  not a field. The compiler resolves the context into `ResolvedBinding.context`, beside the
  unchanged event cause, and the mapping value into `ResolvedMappingValue::DeliveryContext`. A
  binding without a context keeps its bytes. The domain cause is
  `BindingCause::External { event, authority, context_fields }`. `ess verify diff` reports a
  context change as a cause change, rendered with the event, the channel and the context fields
  (each with its type and any `wire` name) on each side. A context field's `display` or
  `summary` is documentation and gets its own kind: `context-field-display-changed` or
  `context-field-summary-changed`.
- **Conformance delivers such an event itself (suite/30 and /31).** The new target method is
  `deliver_event`. It is optional, and its default answer is unsupported, which records the
  scenario `unsupported` with the target's reason. The new steps are `deliver_event` and
  `expect_every_invocation`. The `mapping` scenario delivers one event under two contexts and
  requires each invocation to carry its own. The `delivery` scenario redelivers the second
  occurrence and requires every invocation for it to carry the second context, so a redelivery
  carries its original context. Each mapped context value differs between the two deliveries.
  Within each delivery it also shares no value, at any depth, with any payload value or other
  context value, whatever their names and types. That includes a flag inside a payload struct
  and a list element. So a target that reads the context from the payload, or swaps two context
  fields, fails. Values are chosen within what each type's invariants admit. Where no admitted
  values keep the fields apart, only `mapping` and `delivery` are refused, and the reason names
  the two values that collide. Two mapped `Boolean` context fields beside a `Boolean` payload
  field are one such case. `flow` and `on-failure` are still synthesized. Go and TypeScript
  generation refuse these suites.
- AsyncAPI carries the declared delivery context on the reaction and a `delivery_context` mapped
  source. Generated docs name the channel and its fields. The synthesis plan owes the delivery
  to the host, as an external obligation.

### Changed

- Before `ess/18`, a mapping value written `context.<x>` was literal text. It is now refused as a
  delivery-context reference that nothing declares.
- **`ess-diff/10` (unreleased).** A binding cause change whose before or after is an `external`
  cause (an `ess/18` delivery context) needs `ess-diff/10`, and so do the two context-field
  documentation kinds. Writing it as `ess-diff/3` to `/9` is
  refused, and readers of those formats refuse it with `unsupported_format_version`. Every other
  change keeps its earlier format.
- Below `ess/18`, `state` in a `when_subject` predicate is refused as `unsupported_format_version`
  (it was `unobservable_fact`).
- `ess verify conform mutate` writes `ess-mutation-report/2`, and `--emit` writes
  `ess-mutation-manifest/2`. `--collect` still reads an `ess-mutation-manifest/1` emission.
- A mutant whose suite gained synthesis refusals the baseline's does not have, and that no scored
  scenario killed, is now `unwitnessed` (`ESS-MUTATE-004`, exit 3) instead of `survived`. Every
  mutant entry lists the refusals it added as `added_refusals` (`{code, scenario}` or
  `{code, subject}`), and the text output names them beside the verdict. The manifest records each
  suite's refusals as `refused`. (#203)
- A baseline is red (`ESS-MUTATE-001`) only when a scenario failed or ended `error`, on both
  `--target` and `--collect`. Baseline scenarios reported `unsupported` or `skipped` are listed as
  `baseline.not_scored` with their status, and each mutant is scored on the scenarios the baseline
  executed. A mutant scenario the baseline reported but did not execute is listed on the mutant as
  `excluded`. A baseline that executed nothing is refused with `nothing scored` (exit 3). `--target
  interpreted` on a specification with views now scores instead of being refused. (#210)

### Fixed

- Conformance synthesis arranges a stored-row search from every creating command, not only the
  first one. Creations are tried in command-name order (the order the IR keeps commands in), each
  within its own node budget. A `when_subject:` branch that only a later creation's row selects is
  now witnessed. A creation that cannot leave a row gives way to the next one, and synthesis
  refuses only when every creation fails, keeping the first creation's cause. The lifecycle
  arrangement also tries the next creation when the one on the shortest route cannot be arranged.
  (beyond10x/ess#198)
- An input-guarded refusal on a command that addresses an existing record keeps its plain send,
  because the input refusal is answered before existence. It now also has an arranged half: the
  record is created through a declared creation and driven to a state the command runs from, then
  the refused input is sent for it, and the scenario requires the error and no event. Where an
  identity view shows the row, it also requires the row unchanged. If the record cannot be
  arranged, the scenario is withdrawn and refused at synthesis with the arrangement's cause, rather
  than filed only to be skipped at run time. A refusal whose guard reads the identity field stays a
  plain send. (beyond10x/ess#209)
- An input-guarded refusal's witness now also refutes every overlapping sibling input-guarded
  refusal, so each send selects exactly one declared outcome. Where no input does that, synthesis
  refuses the scenario and names both guards. (beyond10x/ess#209)
- A further source of a multi-source transition can now be arranged through a branch that a stored
  fact selects, the same way that branch's own scenarios are arranged. The stored-row fallback
  search runs under the further instance's own name. Owner arrangement still does not fall back.
  (beyond10x/ess#199)
- A `Map` command input is no longer witnessed as `{}` (beyond10x/ess#196). Synthesis sends one
  entry — the key is the key primitive's own witness at the map's path, spelled as a setup key is
  (`"tags"`, `1`, `true`), and the value is built at `<map>.0` like a list element — so a `sets:` or
  payload copy of a map is asserted to hold that entry, and a target that drops or empties it fails.
  A further instance moves the key and the value, so an update that writes a map is sent a value
  different from the prior one (as #161 does for scalars). `<map>.count` is now a fact on command
  input, so a `.count` guard over a map is decided and tried at the empty map and at the lengths
  either side of its literal. A map keyed by `Decimal` (no setup spelling), or whose value has no
  finite witness, is still `{}`. No suite format change; the committed gatepass suite is
  regenerated (`notes` now carries one entry).
- A system precondition can now open a session whose command takes a list, map or struct
  (beyond10x/ess#205). A precondition's literal input is checked against the input's declared type
  all the way down: a list against its element type, a map against its key spelling and value
  type, a struct against its declared fields, its required fields and its invariants, and `null`
  wherever the type is optional. Each scalar leaf is checked exactly as an `example:` is. A `Json`
  or `Binary64` leaf and a union literal are still refused. Before this fix, `accounts: []` was
  refused as "not a scalar", leaving the input out was refused as missing, and a fixture reference
  was refused by both generated explorers.
- The generated Go and TypeScript explorers now send a precondition whose command they leave out of
  sequences only because they cannot draw one of its inputs (a list, for example). The precondition
  supplies that input as a literal. The model then starts from the row it leaves, and a target
  that stores a different value is reported as a setup failure. No sequence draws that command,
  and it is still listed as excluded.
- A precondition literal is now checked the way the conformance setup reader checks a value. Every
  newtype invariant around a list, map or struct applies to the whole literal. A struct invariant
  reads nested members and list `.count`, and it must be true (unknown is refused). A member whose
  type allows absence through a newtype over `Optional` may be left out. The row a precondition
  creates is checked against its entity's invariants. `{fixture: name}` is read as a fixture
  reference only on an input that has a fixture input; anywhere else it is a literal
  (beyond10x/ess#205).
- The branch a precondition selects is now decided over its literal input the way the interpreter
  decides it: a guard over a list count or a struct member, and `defined()` over a structured value.
  The created row is also checked against literal `sets:` values, `{input: x, else: …}` fallbacks and
  struct sources. The conformance setup reader now treats a newtype over `Optional` as optional, as
  validation does (beyond10x/ess#205).
- **Synthesis binds a view filter over the identity or a link field (#193).** The identity of a row
  a scenario made, every field an arrangement filled with an instance (the link to an owner), and
  every view parameter sent as one are bound as opaque tokens, one per instance. `id == param.id`
  is decided and read with the created row's identity as `param.id`; `account_id == param.account`
  is read with the arranged owner. Before, both were refused as `ESS-SYNTH-005`. A parameter compared
  with the identity or a link field is bound from that comparison, whatever it is named, when it is
  declared at the field's type. Only `==` and `!=` between two tokens are decided: the same token is
  equal, two instances of one type (a captured instance, or the identity a `creates:` branch
  publishes) are different; an ordering, a literal, a text test or a length over an identity stays
  refused as before. Where such a filter holds the scenario's row, a further instance the filter
  refuses is arranged and asserted `Excludes`, so a target that ignores the filter fails; for a
  filter over the identity it is created under the subject's own owner, so a read answering the
  owner's rows fails too. An ordered list read by the owner (`account_id == param.account` with
  `order_by:`) arranges its second row under the same owner; it was refused as `ESS-SYNTH-014`.
  A filter over the link gets a row under another owner asserted `Excludes`, also where the view
  projects the link and not the identity. An owner is given several rows only where the owning
  relation is `cardinality: many`; under `cardinality: one` the by-id row goes under a second
  owner, aggregate rows each get their own, and an ordered list read by the owner stays refused as
  `ESS-SYNTH-014`, now naming the cardinality.
- **An aggregate view grouped by a link field is synthesized (#193).** Rows of one group are created
  under one arranged owner, each group under its own, and each group is asserted with the owner's
  identity as its key. Before, the view was refused as `ESS-SYNTH-017`. Rows given one value of a
  link the view aggregates (`count_distinct: account_id`) share an owner too. An aggregate whose
  group keys are all scoped (`group_by: [account_id, memo]`) gains a group repeating the later
  keys under another first key, so a target dropping the first key fails.
- A filter reading `defined(<link field>)` on a row whose link the arrangement filled with an
  instance is now `true`; it was evaluated `false`.
- Synthesis separates the inputs a `sets-retarget` mutant joins, even where more same-typed inputs
  feed `sets:` than the type has values (beyond10x/ess#202). Within each branch, every `sets:`
  source is paired with each input of its type that no `sets:` entry reads — the input a retarget
  leaves behind, whether it retargets onto an input feeding another field or onto one only a
  payload or guard reads. Every such pair is sent apart where it can be and pinned, so no later
  move gives it one value again; pairs the model shows joined (one input into two fields, or an
  unread input named like the field) go first. This holds whether field and input names match or
  not, and also on branches whose row and input the stored-row search chooses. Where a joined pair
  cannot be sent apart, the synthesis carries a new note, `UnseparatedSources`, naming the
  scenario and the two inputs; a note never names a scenario the finished suite does not hold.
  Committed suites of unmutated specifications are unchanged.

## [0.40.0] — 2026-09-28

### Added

- **`ess-composition/3`: reader-side conformance (#191).** A `conformances:` entry may carry
  `reader: true`, asserting that the consumer's type reads every value the imported type allows.
  Beyond the `/2` rule it may read an imported newtype, through any chain, as what it wraps; an
  enum as `String` (`Optional<String>` where the imported enum is optional); enum variants by wire
  name, with every imported wire name present; a value that is always a JSON object (a struct, or a
  map with `String` keys) as `Map<String, Json>`; and a subset of a struct's fields, where an extra
  local field is compared with the imported field sharing its wire name, omitted or not, and one
  that meets none is `Optional` and not `null_when_absent`. Anything that could reject an imported
  value stays `type_conformance_drift`: a different primitive, a missing wire name, a local field
  required where the imported one may be absent, `Json` read as any map (it may be an array or a
  scalar), an extra field reading an imported key as another type. `reader: true` also asserts
  that the consumer's reader ignores keys it does not declare; ESS-generated closed types
  (`additionalProperties: false`, `deny_unknown_fields`) do not, so a consumer reading through
  them must not use `reader` for a field subset. An entry without `reader` is compared exactly as
  before. A `/1` or `/2` document carrying `reader`, whatever its value (`null` included), is
  refused as `unsupported_format`; `/2` documents keep their meaning and their authored, compiled
  and client-plan bytes.

### Changed

- The generated Go and TypeScript packages grow the target surface the new suite versions need:
  Go `CommandRequest.Caller`, `ViewResult.Total` and the optional `AbsentInputTarget` and
  `RepeatedOutcomeTarget`; TypeScript optional `executeCommandWithoutInput` and
  `configureExternalOutcomeRepeatedly`, `CommandRequest.caller` and `ViewResult.total`. A target
  without an optional method has the scenarios that need it reported skipped (unsupported), never
  passed. Request values past 2^53 reach a TypeScript target as a `JsonNumber` (`RequestValue`),
  and a TypeScript target's answers are read as `JSON.stringify` reads them.

### Fixed

- The generated Go and TypeScript runtimes run every suite version the same release synthesizes,
  `ess-conformance/22` to `/27` included (TypeScript also `/12` to `/17`), instead of refusing it
  with `suite admission: unsupported suite version` and zero verdicts (#188). An unchanged
  specification whose nested `sets:` struct has one generated leaf synthesizes `/26`, and ran on
  0.36.0 but not on 0.38.0 or 0.39.0. The runtimes now execute dotted-leaf payload and row values,
  subject absence and whole-view preservation, presence policies on payload leaves and response
  fields, a command sent with no input, `caller`, `now_offset`, `changed_by`, `page` with a view
  total, bounded retries (`times`, `count`, `final-failure`) and `defined()` over `Optional`
  aggregates, and give the Rust runner's verdict per scenario on the repository's fixtures. The Go
  runtime now also holds an eventually observed event to its payload and shape, a named event
  value or dotted leaf to being carried, and list, map and union leaves to their types. A test
  fails if the synthesizer can write a suite version either runtime does not admit.
- The generated Go and TypeScript conformance packages describe what their runner does
  (#186). `CommandResult.Outcome` (`outcome` in TypeScript) now says a refusal returns the
  refusing outcome's name, which the runner has always compared. For suites at
  `ess-conformance/5` and later, the package README's run instructions state that the runner
  executes only with `ESS_REPORT_FORMAT=2` and stops before the first scenario without it. The
  TypeScript runner's refusal names the actual rule. `ess verify conform synthesize --target
  go|typescript` refuses a suite version its generated runner would refuse at admission instead of
  writing a package that cannot run, and `--help` no longer says `--target ir` writes
  `ess-conformance/1`.
- A name written as two different kinds (a command and an event, say), or a type name written
  twice, is now refused as a duplicate even when one of the declarations fails its own conversion
  or sits in a file with no `domain:`. Before, the refusal appeared only once both copies were
  sound and in a domain, so fixing one error revealed a second fault that had been there all along.
- Every extra writing of a name is refused exactly once. Two sound copies of one kind were refused
  twice, once per reporter; the second refusal is gone. The remaining refusal's hint names the file
  that wrote the name first when that is another file, and both domains when the copies are filed
  under different ones.
- A repeated name filed under a domain that cannot hold it is now refused for that as well, rather
  than only after the duplicate is removed.
- The document schema publishes the charset ESS already enforces on ten name positions that were a
  bare `type: string`: component, command-line binary, command-group and binding names, topology
  workload keys, transition names, selection input and selector names, and a selection mapping's
  selector and path. A schema-aware editor now refuses the spellings `ess validate` already
  refused (for example `Invoice_Service` or `serve--hosted`); no document that validated before is
  refused now, so no format version changes.
- A refusal is no longer cited at a line belonging to another construct when the trailing-key
  guess for its path (`<last>:`) happens to occur exactly once elsewhere. No guess is built for an
  element of a list written `- name: <x>` (`outcomes`, `input`, `fields`, `params`, `response`,
  actor `attributes`, entity `relations`; a system precondition's `input` map keeps its guess),
  and once the refused declaration is located, any other guess is reported only inside that
  declaration's list item. Relation, attribute and response refusals are now cited at the entity,
  actor or command they belong to. Paths with no declaration needle, such as
  `topology.workloads.<component>`, are still cited at their key.
- The Go and TypeScript concurrent explorers record view reads: they draw reads from the seed beside
  commands (a specification with no views draws exactly as before), send each client's last command
  token on `read_your_writes` reads, and write the answer's row identities as `rows` on a returned
  read, so `check-history` judges stale reads and phantom rows. A read keeps every row it answered,
  less at most one unnamed row per lost creation of the view's entity invoked before it returned.
- `ess-history-adapter/1` is specified in `models/recorded-log-adapter/` and published as
  `schemas/ess-history-adapter.schema.json`, which admits exactly what `import-history` reads; the
  reader refuses every spelling the model does not declare (YAML local tags, arrays and nulls in
  JSON, a field or completion word written twice).
- `verify conform synthesize` writes the `state/<S>/refuses/<command>` scenario for a command whose
  sibling branch combines `when:` over the input with `when_subject:` over the subject (#192): a
  guarded branch is selected in any state before `wrong_state:` applies, so the witness misses an
  input-only branch through its input, a branch guarded by the stored row alone through the row,
  and a branch needing both through either. Complementary input guards on two siblings no longer
  refuse the scenario with ESS-SYNTH-003; where no row refutes every stored guard, it is refused
  naming the input and stored guards.

## [0.39.0] — 2026-09-28

### Added

- Concurrent history conformance (#189, `docs/design/concurrent-history-conformance.md`): every
  earlier check drives a target with one client, one call at a time; these record several clients
  at once and search for an order the specification's own model accepts.
- The Rust interpreter executes commands from the model: `ess verify conform run --target
  interpreted --path SPEC` runs outcomes, transitions, `sets:` writes, emitted events and declared
  refusals, and refuses a specification whose `spec_digest` is not the suite's. Views and
  bindings are not interpreted yet; a scenario that reads one is an unsatisfied obligation.
- `ess-history/1`: one run of several clients, each call with its client, command, subject,
  invoke and return instants, `Returned` or `Indeterminate` completion and outcome, plus the rows
  of a view read and the `retry_of` of a retried request. Specified in `models/concurrent-history/`
  and published as `schemas/ess-history.schema.json`.
- `ess verify conform check-history --history FILE`: Wing–Gong–Lowe linearizability search over
  the interpreter, partitioned by subject, within `--budget` model executions (default 1,000,000).
  A call that never answered is placed after every other one. Exit 0 linearizable, 1 violation
  (the longest partial order and a shrunk history that still violates), 3 unknown when the budget
  ran out, which is never a pass, 2 refused. The same history and budget print the same report;
  `--format json` prints it as JSON.
- The same check holds each view to its declared consistency: a `read_your_writes` read is judged
  per client session, and an `eventual` view must converge once the writes stop, after at most
  `--settle` behind reads per session (default 4; a count of reads, not of instants).
- `ExploreConcurrent` (Go) and `exploreConcurrent` (TypeScript) in the emitted conformance
  packages drive 2–4 clients against a target on a seeded logical clock, write
  `history-<seed>.json` and call `ess` to check it; one seed writes the same bytes in both ports.
  With `Inject`/`inject`, they inject every fault the specification declares and no other: a second
  delivery for `delivery: at_least_once`, a client retry for `replays:`, and a delayed or
  unanswered answer for another `external:` branch. No restart is injected.
- `ess verify conform web --history FILE [--out DIR]` draws a checked history as one
  self-contained `index.html`: a lane per client, each call's invoke–return bar, the linearization
  points found, and for a violation the failing call, the call it conflicts with and the shrunk
  history. It exits 0 whatever the verdict.
- `ess verify conform import-history --log FILE --adapter FILE [--output FILE]` converts a JSON
  Lines call log into `ess-history/1` through an `ess-history-adapter/1` document mapping each
  field to a JSON pointer or `absent`. Nothing is guessed: a field a call cannot be judged without
  is refused by line and field (exit 2); every other missing field is a coverage gap on stderr and
  in `FILE.gaps.json`.
- Four planted faults that no earlier check catches: `LostUpdate` and
  `StaleReadUnderReadYourWrites` are caught by a recorded concurrent history,
  `DoubleApplyOnRedelivery` and `RetryCreatesSecondEntity` only by declared fault injection.
- Direct library returns: `returns: true` in source `ess/17`, literal `response:` assertions in
  authored `ess-scenario/4`, and typed direct-response observations in suites `ess-conformance/28`
  and `/29`. The Rust runner checks actual return values without invented events or persistence;
  unsupported Go/TypeScript generation refuses explicitly. Released source `ess/16` and suites
  `/26` and `/27` retain their existing meaning and bytes.

## [0.38.0] — 2026-09-28

### Added

- Source format `ess/16` collects the round-3 retrofit constructs (#162–#179). Each is refused
  under an earlier header as `unsupported_format_version`; suites carrying a new step or
  expectation take `ess-conformance/26` (ordinary) and `/27` (coverage), which the Go and
  TypeScript runtimes refuse by version. A suite without them keeps its earlier format and bytes.
- `{input: <field>, else: <literal>}` in `payload:` and `sets:` (#163): the literal is checked
  and refused exactly as a bare literal in that place. An outcome scenario with a literal fallback
  sends the input and asserts it, then invokes the branch again without it and asserts the literal.
- `defined(x)` and `missing(x)` accept an `Optional` struct, list or map (#176). An entity
  invariant can require an optional field to be gone outside a state
  (`any: [state == Paused, {not: "defined(metrics)"}]`), checked after every branch that changes
  the entity. Rust, Go and TypeScript read a present struct, list or map as defined even when
  empty, and `null` as absent.
- A nested `sets:`/payload struct with an undetermined leaf is asserted leaf by leaf under dotted
  paths (`lead.number`) in the event payload and the view row; the generated leaf by the payload
  shape (#179).
- `input_absent: true` with an `error:` answers a request with no input at all, as opposed to `{}`
  (#170). Conformance gets an `execute_command_without_input` step, OpenAPI marks the body not
  required, and code targets refuse the branch by name. A guard that cannot hold because it needs
  a required path absent is refused with a hint naming `input_absent:`.
- `{related: {via: <field>, field: <field>}}` (#166): a `payload:` or `sets:` value read from a
  field of the row the subject, its input, or a `creates:` field references. Synthesis arranges
  the referenced row between two decoys, so a read of another, the first or the last row, or an
  earlier copy, fails.
- An ungrouped aggregate view with no parameter (#148) is witnessed instead of refused with
  ESS-SYNTH-016: the scenario snapshots the view and asserts the change in every `count` and
  `sum` (new `changed_by` view expectation). Decimal aggregate inputs are arranged as whole
  amounts; a concurrent writer to the source can fail a correct shared target.
- A command `when:` can order a `Timestamp` input against the current time: `now`, `now - 60s`,
  `now + 5m` (#171). Synthesis witnesses it one second either side of the boundary, suites carry
  the value as `now_offset`, and `ess verify conform run` resolves it from the machine clock; the
  library default stays deterministic. Entity Runtime refuses the guard (`CurrentTimeUnsupported`).
- An outcome can be selected by whether the addressed record exists (#164): `unknown_instance:
  true` on a `creates:` beside the `updates:`/`moves:` of the same record (create or update), or
  `existing_instance: true` with an `error:` beside a creation (create or refuse). Conformance
  witnesses both with calls sharing one identity; the served surface answers `existing_instance:`
  with 409; code targets and Entity Runtime refuse both by name.
- `attributes:` on an actor, read as `{caller: <attribute>}` in `payload:`/`sets:` and as
  `caller.<attribute>` in `when:`/`when_subject:` equality guards (#168). Conformance command steps
  carry `caller`, synthesis runs a refusal the caller decides as two callers, OpenAPI answers 403
  for it, and Entity Runtime refuses caller reads with `CallerUnsupported`.
- A binding can bound its retry: `on_failure: {retry: {attempts: 3, final: [<refusal>]}}` (#165).
  `final` names refusals of the invoked command that end the retry at once; after the last attempt
  the event is lost. Conformance requires exactly `attempts` invocations with a retried refusal
  forced, and one with a final refusal, watched for the whole eventual window. Generated Rust, Go
  and Web targets refuse the bound by name.
- A view with `order_by:` can declare `paging: {page, size, first_page, total}` (#174).
  Conformance reads three pages with the new `page` expectation: two one-row pages and one partial
  page. OpenAPI documents page and size and an optional `total`; code targets refuse paged views
  by name; `ess-diff/9` adds `paging-changed`. The caller-supplied free-form filter of #174 is
  declined.
- An outcome can change every record a filter selects: `instances: {where: …}` on `moves:`/`updates:`, with `{count: changed}` for the number changed (`ess/16`, #167); an outcome with one subject can change other records with `affects: [{entity, where, sets}]`, where `where` may read `subject.<field>` (#175). Conformance arranges two matching rows, one non-matching row and, for a move, one row outside its `from` states, and requires exactly the matching rows changed and the count right. Entity Runtime refuses both (`SetEffectUnsupported`); Rust, Go, Web and Clap synthesis refuse them by name; `ess-diff/9` adds `outcome-set-effect-changed`.
- `ess-composition/2` (#162): a composition may reference any type declared in a domain the
  selected component owns, and `conformances:` asserts that a consumer's local type matches an
  imported component type; any difference other than a consumer treating a required value as
  optional is refused as `type_conformance_drift`. `ess-composition/1` keeps its rule and bytes.

### Changed

- An `Optional<T>` input reads as `T` in a default branch once a sibling refuses exactly its
  absence, and in a branch whose own guard requires `defined(x)` (#169); copying it into a
  required field needs no `conversions:` entry.
- An input-guarded refusal (`when:` + `error:`) is taken before any accepting branch whose input
  guard it overlaps (#178); synthesis sends it at every overlap point, a shadowed branch is refused
  naming the refusal, and Entity Runtime lowering orders these refusals first.
- A `when_subject:` command whose entity has only `eventual` views gets scenarios for every branch
  (#172), and `state/<S>/refuses/<command>` is written for a `when_subject:` command with a
  guard-less fallback; the internal ESS-SYNTH-008 "drifted apart" message is gone (#173).
- Synthesis witnesses guards over many inputs (seven comparisons in one `all`) instead of
  refusing with ESS-SYNTH-003 after 64 candidates (#155 follow-up).
- Synthesis asserts a view whose filter reads a row field over a row that filter matches;
  case-insensitive filters are also asserted over a row with the literal in its other ASCII case.

## [0.37.0] — 2026-09-27

### Added

- Outcome shapes, source format `ess/15` (`docs/design/outcome-shapes.md`): `unknown_instance:`
  answers an identity no record carries, before an external not-found refusal and `wrong_state`
  (#145); `deletes:` removes the subject, and synthesis asserts it is gone from every
  read-your-writes view and that a re-send gets the unknown-instance answer (#151); `into:` creates
  into a declared state, and generated Rust gets one constructor per creation state (#150);
  `accepts: nothing` declares an accepted no-op without a subject (#144); system-level
  `preconditions:` run before every scenario and explorer sequence and must select exactly one
  success branch (#152). Suites using the new steps take `ess-conformance/22` and `/23`; Entity
  Runtime refuses the shapes (`OutcomeShapeUnsupported`).
- Aggregates over and group keys of `Optional` fields, source format `ess/15` (#148):
  `aggregate: {sum: duration, skip_absent: true}` skips absent values with SQL semantics, and an
  optional group key makes the absent value its own group. Synthesis arranges one absent row per
  skipping input.
- `prefix:` on a `String` newtype (#146), a `Json` primitive (#138), a field-level
  `presence: null_when_absent | omitted_when_absent` on `Optional` fields (#139), and the nested
  `naming: {wire: …}` spelling on fields (#142), source format `ess/15` for the first three.
  Presence is enforced in payload shapes and responses; suites carrying it take
  `ess-conformance/24` and `/25`. Generated Rust, Go, web and CLI targets refuse `Json` by name,
  as they refuse `Binary64`. (`docs/design/wire-presence-json-prefix.md`)
- A field name may start with underscores before its first letter (`_url`), everywhere a field is
  named, including fact paths and the Go, TypeScript and browser runtimes (#141).
- A map key may be a newtype of an admitted key primitive; it resolves to that primitive while the
  specification, authored scenarios and CLI contracts are read (#143).
- `ess verify conform mutate --emit DIR` writes each mutant's suite for a project's own runner and
  `--collect DIR` scores the reports it wrote (#153).
- The Go and TypeScript model explorers take `external:` branches through the target's external
  control and report reach per branch (#156).
- `ess specify toolchain install|list|which`: an exact `requires: ess X.Y.Z` pin makes any `ess`
  run that release from a verified per-version cache (#147).
- A `when_subject` predicate may compare a stored field with the command's input,
  `recording_id != input.recording_id`, source format `ess/15` (#157). The finite partition treats
  such a guard as open, so a default is required. Synthesis arranges the row and sends the input
  equal to the stored value for one side and different for the other. Entity Runtime lowers the
  operand to the command's arguments. A stored field named `input` keeps being read as itself.
- `equals_ignore_case` (one text literal) and `in_ignore_case` (a list of them) over `String` and
  its newtypes, ASCII case folding only, source format `ess/15` (#140). Rust, Go and TypeScript
  evaluate them identically; synthesis witnesses the guarded branch with the literal in the other
  case and the default with one character changed. Suites carrying one in a view expectation take
  `ess-conformance/20` and `/21`. Entity Runtime lowering and `infra-spec/1` refuse them
  (`CaseFoldUnsupported`). (`docs/design/value-expressions.md` E6, E7)

### Changed

- Synthesis kills more mutants: a state dropped from an all-states transition gets its refusal
  scenario (#154); `any:` guards are witnessed once per disjunct and `all:` once per conjunct,
  over inputs and stored fields (#155); ordering boundaries are witnessed on both sides, including
  `.count` and near zero (#160); same-typed sources are sent apart and a written value is arranged
  over a different prior value (#161). Committed generated suites are unchanged.
- A wrong-state refusal whose subject no immediate view fully covers is witnessed over the fields
  the declared views publish, with `Note::PartialObservation` naming what is left out; the refusal's
  help line points at declaring a view (#132).

## [0.36.0] — 2026-09-27

### Added

- Value expressions in `payload:` and `sets:`, source format `ess/14`: `{subject: <field>}` reads
  the addressed entity before the outcome (#133), `{increment: <number>}` adds to a stored
  `Integer` or `Decimal` and `{generated: true}` is admitted in `sets:` (#134), `{input: <field>,
  else: {generated: true}}` takes an optional input or a minted value (#137), and a nested mapping
  gives each field of a struct-typed target its own source (#136). Synthesis asserts each value
  where the arrangement determined what it reads. Entity Runtime lowering refuses them
  (`ValueExpressionUnsupported`). Models without them keep their IR bytes.
  (`docs/design/value-expressions.md`)
- A literal over a `Decimal` target, quoted or unquoted, in every format (#135).

### Fixed

- The text `subject.<field>` in `payload:` or `sets:` is refused as `misspelled_reference`,
  naming `{subject: <field>}`; it compiled as the literal text (#133).
- A synthesized suite compares a payload literal over an `Integer`, `Boolean` or `Decimal` field
  with the value it spells, where it compared the text: `retries: 0` required the string `"0"`.

## [0.35.1] — 2026-09-27

### Fixed

- An unknown instance is answered by the command's declared not-found outcome, not its
  `wrong_state` outcome. Since 0.34 synthesis sent a fresh identity to a command declaring both
  (`not-found` and `wrong-state`) and required `wrong-state`, so an implementation
  answering the outcome its own specification declares for a missing record failed. A not-found
  outcome is an `external:` refusal without input guard whose `error:` carries a field of the
  identity's type; its scenario now sends the fresh identity instead of injecting the cause, and
  the command gets no `…/outcome/wrong-state` scenario. A command declaring no such outcome keeps
  the 0.34 rule; one declaring two gets a `note:` and neither is assumed. No format change.

- `ess verify bindings --live --observation-out` and `cargo xtask infra-acceptance --scratch` no
  longer take a `.git` directory Git cannot open (one holding only `info/exclude`, as a harness
  leaves in a directory that is no repository) for a checkout. A `.git` directory holding `HEAD`,
  `objects`, `refs` or `commondir`, a `.git` file, a `.git` symlink and an unreadable marker are
  still refused. Both guards share one rule; the xtask check previously also refused an empty
  `.git` directory and admitted an unreadable one.
- `cargo xtask docs` reads every phrasing the published documents have used for an unreleased
  format — "not yet released", "not released", "next release", "upcoming" and "not yet shipped"
  as well as "unreleased" — where it read only the last, and missed `ess/13` called "not yet
  released" after 0.35.0 shipped it. A bare `/N` counts for the family named before it on the line,
  as the revised-envelope table writes `ess-scenario/` and `/3`. It tracks the release of every
  family the version history gives one (`ess-scenario`, `ess-normalization`, the revised
  envelopes and `infra-*` besides `ess`, `ess-diff` and `ess-conformance`), and refuses a family
  the history gives a release that it does not track.
- The two ess-xtask regressions that execute the schema-metadata guard no longer fail when the
  caller sets `CARGO_TARGET_DIR`, a job count, a debug profile or a compiler wrapper: they decide
  over controlled build and process facts, and one now asserts that the same authority refuses each
  of the four. Production qualification still reads this build and this process and refuses
  exactly what it refused.

## [0.35.0] — 2026-09-26

### Added

- Typed pre-execution fixture values: a command's `fixture_inputs:` (source format `ess/13`) and
  authored `fixtures:` with `{$fixture: name}` references (`ess-scenario/3`) compile to suites
  `ess-conformance/18`/`19`. Rust, Go and TypeScript resolve and type-check the values from an
  independent provider before `BeginScenario`, copy them per scenario, and compare the first direct
  event occurrence with them. Invalid values stop before target activity; a missing provider is an
  explicit skip; browser replay refuses fixtures. Fixture-free models and suites keep their bytes.
  (#58)

### Fixed

- `schemas/generated/ess.schema.json` describes `ExternalRef` as the `provider:key` string and a
  periodic cause's `every` (`Period`) as the `PT<seconds>S` string ESS reads and writes, instead of
  an object with `provider`/`reference` and a bare integer. Each pattern admits exactly what the
  parser reads, held by a parity test over the committed schema. No source format changes.

## [0.34.0] — 2026-09-26

### Added

- String predicate operators `starts_with`, `ends_with` and `contains` (map form, `String` and its
  newtypes, byte-wise and case-sensitive) in every predicate position, source format `ess/8`.
  Synthesis witnesses both branches; suites carrying one take `ess-conformance/14`/`15`; Rust and Go
  evaluate them, TypeScript and the browser refuse them by version. entity-core is pinned by rev to
  `718a702` (beyond10x/entity-runtime#40). (#95)
- `when_subject: {predicate: …}` guards an outcome by a predicate over the addressed entity's stored
  fields, source format `ess/9`; refusals read the sibling subject; conformance arranges the row
  through `sets:` mappings, adds rows refuting one conjunct at a time and an absent-subject witness.
  `{field, equals}` keeps `ess/6` and its bytes. Timestamp orderings lower to entity-core
  `before`/`after`; text orderings are refused at lowering. (#75)
- Aggregate views: `group_by:` and field `aggregate:` (`count`, `count_distinct`, `sum`, `min`, `max`,
  `avg`), source format `ess/10`, `ess-conformance/16`/`17`, `ess-diff/7`, `ESS-SYNTH-016`/`017`.
  Empty groups are absent; `avg` is `Optional<Decimal>`, 6 places half-even. Models without aggregate
  views keep their IR bytes. (#96)
- Synthesis gives both sides of a presence guard a scenario by omitting the optional; command-input
  lists publish `.count` and their elements, and `.count`, `exists`, `forall` and positional guards
  are synthesized. (#93, #94)
- `website/docs/reference/predicates.md` lists every predicate form, where it is accepted and what
  synthesis can witness; a test runs every example on it. (#92)
- A `String` newtype may declare `alphabet:`, a command input `example:`, and `.count` on a `String`
  is its length in Unicode scalar values in every predicate position (source format `ess/11`).
  Synthesis draws text from the alphabet, starts from the example and tries lengths either side of a
  `.count` literal up to 1024, for text and lists; beyond that `ESS-SYNTH-018`. `ess-diff/8` reports
  `alphabet-changed` and `input-example-changed`. (#103, #104)
- `outcome_groups:` declares one external refusal once for a command list or an `actor:`/`domain:`
  selector (with `except:`), source format `ess/12`; it expands before validation and compiles to the
  same IR as copying the outcome by hand. A same-named outcome is refused as `ESS-COMMAND-006`. (#105)
- `ess-inputs/2` adds optional `requires: ess X.Y[.Z]`: an older `ess` refuses naming the release and
  `b10x upgrade`, a newer one warns once, `--strict-requires` refuses. `ess-output-state/2` records the
  producer, and a regeneration by another release prints a note. (#106)
- A literal in `sets:` or `payload:` may be an unquoted YAML boolean or integer, compiling to the
  quoted form's bytes; over text or an enum it is refused with `quote it`. (#113)
- An unknown instance has a declared answer: a `moves:`/`updates:` command whose `instance:` names no
  record answers its `wrong_state` outcome, witnessed once per such command with a fresh identity.
  The generated Rust and Go seams gain `<WrongState>UnknownInstance` for it, served as `409` without
  a `payload`; models where the case cannot arise keep their generated bytes. (#113)
- `ess verify conform mutate`: a mutation audit over nine specification mutant classes against the
  built-in targets, writing `ess-mutation-report/1` (`ESS-MUTATE-001`/`003`). The TypeScript and Go
  conformance packages carry a seeded random-walk explorer over the IR model, bound to `spec_digest`
  through `ir.json`, with shrinking and a hard failure on unreached outcomes. (#114)
- A release note for 0.31–0.33 (`website/blog/2026-09-26-1200-a-secret-is-only-present.md`): the
  Entity Runtime lowering, `OBS-BIND-008` and acknowledged containers, and Secret presence. Without
  it the newest note trails 0.34.0 by four minors and `cargo xtask docs` refuses the tree.

### Changed

- `ess specify validate` compiles the authored scenarios `ess-inputs.yaml` lists and fails on their
  `ESS-AUTHOR-*` refusals, counting them in its summary; `ESS-COMMAND-018` refuses an invariant that
  reads a required field a `creates:` outcome leaves unset. The bundled examples set such fields. (#112)
- Synthesized suites assert the target state of every transition whose view projects `state`, run a
  multi-source transition from every source, give updates a value different from what the row holds,
  and probe numeric and timestamp guard boundaries on both sides. Committed suites gain scenarios;
  no format change. (#111)
- `ess generate --kind openapi|asyncapi` names every domain no component owns; `--strict` refuses. (#102)
- Go and TypeScript runners that write `ess-conformance-report/1` with a skipped scenario point once
  at report/2, which carries passed, failed and skipped counts; the guide says where they live. (#110)
- An unquoted `x == null` / `!= null` (and `~`, `Null`, `NULL`) is refused with `ESS-SPEC-017`,
  naming `defined(x)`; a quoted `"null"` is text. Ordering a `Duration` and an unquoted compact
  operand containing `&&` or `||` are refused. Artifacts written by 0.32.x that carry such literals
  must be regenerated. (#93)
- Text orders byte-wise in every evaluator; Timestamps order as instants wherever declared types are
  known. (#94)
- A create's omitted `Optional` input leaves its mapped field null in generated suites.

### Fixed

- The Go conformance runtime compares numbers by value whatever Go type carries them (`int`…`uint64`,
  `float32`, `float64`, `json.Number`), with Rust's exact semantics. (#101)
- The `ess-mutation-report/1` row of the formats reference sat above the page's front matter; it
  is in the change and conformance records table.

## [0.33.0] — 2026-09-26

### Added

- `ess verify bindings` reads `ess-observed-bindings/2`, whose optional `foreign_containers`
  acknowledges, per bound workload, a container or native sidecar the service does not build (a
  mesh proxy, a vendor agent) by `name` and nonempty `reason`. `OBS-BIND-008` counts it as
  accounted for instead of requiring a binding that would misstate it, and the report lists it
  under the binding's `acknowledged`, not as a binding. An acknowledged container running a
  binding's declared image or an implementation's artifact locator, or the same `@sha256:` digest
  under another repository or tag, or a container artifact's `identity` digest, violates
  `OBS-BIND-008`; tags alone are not compared. An
  acknowledgement naming no observed container or native sidecar leaves `OBS-BIND-008`
  `unknown`, naming it: the infrastructure model does not record plain init containers, so it
  may name one. One naming a bound container, a workload no binding binds, a duplicate or a
  blank reason is refused before collection. `ess-observed-bindings/1` is read unchanged,
  acknowledges nothing and keeps its binding digest; a `/1` document carrying the key, even as
  `[]`, is refused, and older readers refuse `/2`. The report is now `ess-observed-bindings-report/3`: it adds `acknowledged`, and a
  satisfied `OBS-BIND-008` no longer means every entry is bound.

### Security

- A full Kubernetes scan no longer writes a digest or length of any Secret value. It used to write
  each value's unsalted SHA-256 and exact byte length, so anyone holding the observation file could
  confirm a guess of a low-entropy secret such as `hunter2`. The scanner now writes
  `infra-observation/3`, where each Secret `data`/`stringData` value is `{"present": true}`: the
  key name and that a value exists. The observation reader refuses anything else beside the marker
  (`INFRA-SECRET-003`).
- Compiling any full observation with a Secret key, `infra-observation/1` included, writes
  `infra-ir/3`, whose Secret keys hold the same marker and nothing derived from the value. A
  legacy `/1` observation is still read; its digests are checked for shape and discarded. An IR
  without a Secret key keeps `infra-ir/1` and its bytes.
- A legacy `infra-ir/1` is read with its own digest checked and then returned with each Secret
  key reduced to presence, as `infra-ir/3`. `ess infra import kubernetes --path <legacy> --out`
  writes that IR/3, and every derived document — graph `source_digest`, drift `from`/`to`,
  the simulation snapshot, the projection `snapshot_digest` and `SUMMARY.md`, and the
  observed-bindings observation digest — names its model digest. None of them chains to the
  legacy file's own digest any more: that digest, computed over the unsalted Secret digests, would
  confirm a guessed value beside the IR/3 written from the same file.
- A Secret item in an observation, or a Secret block in an IR document, that does not read as its
  shape is refused with fixed text (`INFRA-OBJECT-001`, `INFRA-IR-003`). The refusal used to be
  the parser's message, which quotes the value it could not read, such as
  `invalid type: string "…"`.

### Changed

- `ess infra diff` writes `infra-drift/3` for full scans and no longer detects a rotated Secret
  value. For a Secret, `config_content_changed` names added and removed keys, and `changed_keys`
  is always empty, including between two legacy `infra-ir/1` documents, whose digests are
  dropped when they are read. An empty list therefore means "not known"
  where `infra-drift/1` meant "not rotated", hence the new version; `/2` stays the namespace
  topology profile. Detecting a rotation needs a record derived from the value, and an unsalted
  digest of a low-entropy secret is a guess oracle. Configmap content changes are detected as
  before.
- A lowering targets Entity Runtime `0.24.1`: `entity-core` moves from `0.23.0` and
  `ENTITY_RUNTIME_REVISION` names `4746bd7c`, the locked commit. The new `starts_with` and
  `ends_with` conditions are not emitted, so lowered definitions are unchanged apart from the
  target revision they name.

## [0.32.1] — 2026-09-25

### Changed

- `task check` runs `consumer-check` only when `CONSUMER_CHECKS=true`; it used to run it unless
  `SKIP_CONSUMER_CHECKS=true`. Feature-preservation accounting is parked: its consumers are this
  repository's own profiles, and at 0.31.0 it needed about 7,680 per-cell judgement decisions. It
  comes back when an external adopter pins a released `ess/N` and a change reaches them that
  conformance, `ess verify diff` or a format refusal did not catch. `task consumer-check` itself
  is unchanged.

### Fixed

- The wall-clock ratio test in `literal_representation_adversary_pass3` and the Firefox browser
  binaries run with their CI shard to themselves; a neighbouring test on the same runner made
  them fail on changes that did not touch them.

- The `README.md` that `ess verify conform synthesize --target go` writes wires the package into
  `package yourservice_test` importing `example.com/yourservice/essconform`, and says both stand
  for the adopter's own module. It named `acd_test` and `example.com/acd`, a project unrelated to
  the synthesized system.
- Generated files spell the grouped commands the CLI help and the agent skills use: the Go and
  TypeScript conformance packages (`README.md` and source headers) say `ess verify conform
  synthesize` and `aep plan artifact evidence`, the TypeScript headers name `--target typescript`
  rather than the nonexistent `--target ts`, a synthesized clap binary says `ess specify validate`,
  and the Kubernetes projection's `SUMMARY.md` says `ess generate project kubernetes`. Three
  refusal hints follow: `ess specify realization generate`, `ess generate release bundle` and
  `ess generate output recover`. The flat spellings still run.
- `ess verify conform synthesize --target ir`, `ess verify conform author` and
  `ess verify conform select` create the missing parent directories of their `--out` file, as the
  directory targets already did; `synthesize --out missing/suite.json` failed with
  `No such file or directory`. An existing ancestor that is a symlink or not a directory is still
  refused, and the destination file now passes the same checks as other named outputs.
- `ess specify validate` reports a declaration refused by its own checks once. A reference to it
  is no longer refused again as a reference to something undeclared: one bad payload source in a
  command gave four refusals, three of them saying the command did not exist (an actor's `may`, a
  component's `accepts`, and `missing_causation` for the transition only that command `moves:`).
  This holds for refused entities, commands, events, errors and components, wherever an actor,
  component, command outcome, binding or topology names them. A name nobody declared is still
  refused, and diagnostic codes and wording are unchanged.

## [0.32.0] — 2026-09-25

### Added

- `ess infra import openapi` imports OpenAPI 3.0 documents instead of refusing them at
  `/openapi`. Each schema is rewritten to its 3.1 form: boolean `exclusiveMinimum`/`exclusiveMaximum`
  fold into the numeric keyword, and `nullable: true` becomes the `null` union, whose `null` member
  is a `feature-unpreserved` gap at `…/nullable` because the interface has no null. A 3.0 construct
  with no faithful 3.1 form (`nullable` beside `$ref`, a boolean bound without its `minimum` or
  `maximum`) is refused at its own pointer. Swagger 2.0 is refused at `/swagger`.
- OpenAPI 3.1 `type: [<type>, "null"]` imports as `<type>` with the same `null` gap at `…/type`,
  so the 3.0 and 3.1 spellings of one meaning agree; it was refused as a type array. Other 3.1
  imports are byte-identical.
- `ess verify bindings` reports `OBS-BIND-008`: a container or native sidecar (`initContainers`
  with `restartPolicy: Always`) in a bound workload's template that no binding names is a
  violation naming it, and a binding may name a native sidecar. Plain init containers are not
  checked, and the detail says so. The report is now `ess-observed-bindings-report/2`. The input
  stays `ess-observed-bindings/1`, and its meaning widens: binding a workload claims it runs
  nothing else, so a document whose bound workload carries an unbound sidecar, which was
  satisfied, is now violated.
- The namespace-topology collector keeps `initContainers`, sanitized like containers, and writes
  the key even when a template has none. The infrastructure model records native sidecars in an
  optional `native_sidecars` field: absent when the observation did not record init containers,
  empty when it recorded none. Before 0.32.0 the field is written only for a native sidecar or an
  explicit `initContainers: []`; a missing key or a list of plain init containers only, and any IR
  without the field, leaves `OBS-BIND-008` `unknown` rather than satisfied, naming the producer
  version. Every such document keeps the bytes and digest ESS 0.31.0 computed for it.
- `task infra-acceptance` accepts one generated service placement against a disposable k3d
  cluster it creates and deletes: live observation, a projection produced twice and compared,
  three sensitivity cases, a Secret redaction check on every written byte, and verified teardown.
  It needs Docker and stays out of `task check`.

### Fixed

- `--path` and `--spec` help no longer calls the `system.yaml` directory layout "legacy". It is a
  supported layout and the one a first specification starts from; the reference page on directory
  input says so too.
- `ess verify conform synthesize` orders two `Timestamp` values by the RFC 3339 instant they name
  (`+01:00` and `Z` spellings included), in a guard over two fields or against an instant
  literal. It refused every such guard with "no declared scale contains both values".
- `ess specify validate` refuses what synthesize cannot decide, with the same reading in both:
  a bare word on the right-hand side that names a declared field (`ends_at > starts_at` compares
  against the text `"starts_at"`) is refused and the message names the working spelling
  (`window.ends_at > window.starts_at`); ordering a `Timestamp` against text that is not an
  instant is refused. A bare word that is a variant of the compared enum is unchanged.

## [0.31.0] — 2026-09-25

### Added

- `ess-entity-runtime` projects an admitted service contract into validated Entity Runtime
  definitions and typed host binding obligations. It preserves conditional outcome selection,
  exact values and event order, and makes omitted operation-field actions explicit without
  executing a service or choosing the host's policy.
  Subject-field, state-change and external conditions lower to ER predicates; an operation
  that clears a field and a preserve that returns no response are refused by name
  (`ClearedValueUnsupported`, `SilentPreserveUnsupported`).
  A lowering targets Entity Runtime `0.23.0`: `ENTITY_RUNTIME_REVISION` is the locked
  `entity-core` commit, and a test refuses a lock file that names another.
- `ess-service-contract` extracts a borrowed service contract for an exact component and
  synthesis plan, preserving selected compiler values, contextual capabilities, obligations,
  refusals and their original order.
- A release note for 0.28–0.30 (`website/blog/2026-09-25-1200-what-a-retry-returns.md`): subject
  history in `ess/6`, retried results in `ess/7`, and the plugin's one release here. Without it the
  newest note trails 0.31.0 by four minors and `cargo xtask docs` refuses the tree.

### Removed

- **Breaking:** `ess skill` and the agent plugin in `plugins/ess/`, with both marketplace files
  (`ess@ess`), the `build.rs` that embedded the skills and `cargo xtask plugin check`. The plugin
  lives in `beyond10x/agentplugins` with every other Beyond10x plugin (`ess@b10x`, skills
  `ess:init`, `ess:specifying`, `ess:retrofitting`, `ess:testing-conformance`, `ess:upgrade`);
  its `b10x` CLI installs this binary prebuilt or with `cargo` and migrates an `ess@ess` install.
  `ess --help` lists the four areas and nothing else.

## [0.30.0] — 2026-09-23

### Added

- The ESS agent plugin lives in this repository. `plugins/ess/` carries the skills `ess` (front
  door), `specify`, `coverage` and `retrofit`, and the Claude agents `author`, `retrofitter` and
  `conformance`. `.claude-plugin/marketplace.json` and `.agents/plugins/marketplace.json` serve it
  as `ess@ess` for Claude Code and Codex. `specify` and `coverage` move from the `ess-specify`
  plugin in `beyond10x/agentplugins`.
- `ess skill [<path>]` prints the skills and agents embedded in the binary: the front door and an
  index with no path, one file with a path, the index as JSON with `--json`. An unknown path exits
  `2` and lists the valid ones. `ess --help` lists `skill` after the four areas.
- `cargo xtask plugin check`, run by `cargo xtask release verify`, refuses a plugin manifest whose
  version differs from the workspace, a marketplace under another identity, a skill named unlike
  its folder, and an agent wrapper that names no existing skill.

## [0.29.0] — 2026-09-22

### Added

- Source `ess/7` declares command-local `replays`: a retry returns the originating
  success's typed result and preserves its original subject. An effect-free named
  error can cover the remaining finite subject states. Ordinary `wrong_state`
  witnesses in this source version observe the complete subject and reject every
  direct event. Earlier source contracts and generated bytes remain unchanged.
- Conformance suites `12` and `13` capture actual original results, input, actor
  and identity before an unforced retry. Rust and generated Go compare exact
  typed responses and fresh complete subject observations. Native adapters retain
  exact integers; JSON adapters must preserve declared Integer values before
  observation or refuse. Decimal/Binary64 response positions remain unsupported.
  TypeScript/browser readers refuse these suite envelopes before callbacks.
- Complete snapshots carry a finite typed row descriptor and validate both
  observations before comparison. Missing required fields cannot pass as an
  unchanged partial row. Replay synthesis checks the retry's own eligibility
  against the original input and post-command subject; unavailable immediate
  witnesses receive a named refusal.
- Native Rust/Go outcomes retain the typed response even without an event mapping.
  Documentation, OpenAPI, AsyncAPI and the compiled graph expose the replay
  relation; `ess-diff/6` records replay origins and complete-refusal observation
  changes. The generated source schema includes the new declaration.

The generated witness covers an immediate retry. Persistence, restart, later-head
retries and absence of physical writes remain adopter-owned acceptance.

## [0.28.0] — 2026-09-21

### Added

- `ess/6` models input eligibility beside an independently arranged external cause,
  and bounded enum subject history with explicit silent preservation. Conformance synthesis
  observes history separately from lifecycle and keeps same-state histories distinct.
  Suite formats 10/11 add no-error assertions and actual subject snapshots; the Rust, Go and
  TypeScript runners reject missing/duplicate subjects and changed values. Legacy documents
  retain their formats and reject the new vocabulary. The published source schema is regenerated.

- **`website/docs/reference/spec-versions.md` — what each format version number changed.**
  Eleven format families have been revised past their first version, and `ess/5` is the fifth
  number in one of them: nothing public said what any of them meant. The page carries one row per version, the release that introduced
  it, and what an older reader does with a newer document. Two facts a reader would otherwise
  hunt for: `ess/3` and `ess/4` both arrived in 0.23.0, and `ess-conformance/6` through `/9` did
  too, so a build advertising `SUPPORTED_SUITE_FORMATS = [1..9]` is not the record of nine
  releases.

- **`WHATS-CHANGED.md`, rendered from `changes/*.yaml` by `cargo xtask whats-changed`.**
  `CHANGELOG.md` records every change at the level the change was made, which is 1466 lines and
  the wrong record for somebody deciding whether a release is worth adopting. The fragments
  already carry that judgement — a title, a summary, a kind and an impact — and until now they
  were write-only from this repository's side: nothing here read them and nothing here checked
  them.

- **`beyond10x.github.io/ess/` serves this repository's own documentation site again.**
  `/ess/` has answered with a redirect to the unified site since the migration. The unified site
  renders `docs/**/*.md`, `blog/*.md` and `static/**`, so the browser lab, the landing page and
  the per-release index had no public address, and `docs/examples/specification-to-contracts.md`
  is excluded from the bundle outright because its MDX imports a React component.

  `pages.yml` already built the site on every push to `main`; it now also uploads it as an
  ordinary artifact. The generated `.github/workflows/b10x-docs-site.yml` fires when that build
  finishes and hands `beyond10x/website` the artifact of that exact run, which deploys it. A
  documentation change reaches `/ess/` when it reaches `main`, with no dispatch and no wait on the
  organization's publication cycle. This repository still deploys nothing itself and holds no App
  credentials, and `.github/workflows/b10x-docs-pages.yml` is gone, because one Pages deployment
  answers `/ess/` and two callers would race for it.

  The nineteen documents that both sites render carry `<link rel="canonical">` at their
  `/docs/ess/` address. `/ess/lab`, `/ess/releases` and the worked example, which the unified site
  does not carry, stay canonical to themselves.

- **Six release notes, for the nineteen minors that had none.** `/ess/releases` stopped at 0.7.1
  on 21 August; 0.8.0 through 0.27.0 went unwritten. One note per release would have been nineteen
  pieces, several about a release whose only content is a fix, so the catch-up is thematic: a
  deployment described (0.8–0.9), the claims a generated scenario could not make (0.10–0.11), four
  areas and a command line (0.12–0.14), the scenarios an author writes (0.16–0.18), a pass over
  nothing (0.19–0.21), and the names nobody may invent (0.22–0.27). `/changes/` keeps the
  per-release record; these say what the releases were for.

- **`cargo xtask docs` refuses release notes that have stopped.** The newest note's `release_tag`
  may trail the newest release by at most `BLOG_LAG` minors, three today. Not one note per release,
  because a gate that fires on a release nobody wanted to write about is a gate somebody silences —
  but nineteen is not a gap, it is a stop, and nothing noticed.

- **`cargo xtask docs`, a checker for the release claims the public documents make.**
  `where-this-stands.md` carries one generated block and the block was current at every release;
  everything a person wrote beside it had drifted five releases. The lane reads the three
  `SUPPORTED_*` constants and `CHANGELOG.md` and refuses three things: a supported format version
  with no recorded release, a supported version named in neither reference page, and a document
  that calls a released format unreleased. It also refuses an install walkthrough that names a
  version other than the newest release. It joins `projection-check`.

- **Seven `changes/*.yaml` entries**, for 0.20.0, 0.21.0, 0.22.0, 0.23.0, 0.25.0, 0.26.0 and
  0.27.0. Atlas publishes that directory as the organization's change feed, so those seven
  releases were ones nobody outside this repository heard about.

### Changed

- **`cargo xtask release status` checks two more things**, in the same shape and the same report.
  A pushed tag must have a dated `CHANGELOG.md` section — `release notes` renders a release body
  from that section, so a tag without one can be neither published nor backfilled, and until now
  the section was verified for the workspace version alone. And every minor release must have a
  `changes/*.yaml` entry, with the one exemption named in source and the reason stated. The
  command now reports every class of defect it found rather than stopping at the first, because
  stopping at the first is how four versions reached this state unnoticed.

- `cargo xtask whats-changed --check` joins `projection-check`, so a fragment edited without
  re-rendering fails the gate. It also enforces the 360-character `summary` bound that the public
  Docs System validator applies — a bound this repository could not see before, and which refused
  `changes/source-driven-realization-0.19.0.yaml` after its documentation bundle had already been
  built.

### Fixed

- **The published documents said `ess/2`, `ess/3`, `ess/4`, `ess-diff/3`, `ess-diff/4`,
  `ess-conformance/6`–`/9`, `ess-scenario/2` and `ess-target-failure/3` were unreleased.** They
  shipped in 0.20.0 and 0.23.0. Twenty-nine sentences across six pages carried the word; every one
  now names the release that introduced the thing it describes. The same held for the
  `ess-inputs.yaml` manifest, generated-output ownership, the output-management commands and
  `ess verify bindings`, all 0.21.0, and for suite/5, the original-byte carrier and paired replay,
  also 0.21.0.

- **`ess/5` and `ess-diff/5` were supported and documented nowhere.** `reference/formats.md`
  stopped at `ess/4` and `ess-diff/4`; `guides/write-a-specification.md` had no section on a
  variant carrying its own wire spelling. Both formats now have rows, and the authoring guide has
  the section.

- **The install walkthrough downloaded 0.13.2**, fourteen minors behind. `getting-started.md`,
  `examples/specification-to-contracts.md` and `guides/write-a-specification.md` all named it as
  the current tag.

- **`status/where-this-stands.md` reported 0.22.1 as the latest release.** It reports 0.27.0 and
  its assets, dated 20 September 2026.

- Every published document declares a `sidebar_position`. The unified site generates its sidebar
  from the directory tree and cannot see `website/sidebars.ts`, so the order a reader met there
  was alphabetical rather than the authored one.

- Three version tags existed only in one clone and were never pushed: `0.3.0`, `0.5.0` and
  `0.15.0`. `task release-status` reads `git ls-remote`, so it never saw them, and the earlier
  note calling them "pushed tags with no release" was wrong about why. They are deleted locally;
  their commits are on `origin/main` and their changelog sections stand as the record. The code
  each names shipped in 0.4.0, 0.5.1 and 0.16.0.

- **Four releases that promised a download and carried none now carry one.** `0.1.1`, `0.2.0`,
  `0.2.1` and `0.4.0` predate the packaging job, which arrived in 0.6.0, so each had a release
  body and no archives. A `workflow_dispatch` backfill gated and packaged each tree at its own
  commit; every one now carries the four archives and `SHA256SUMS`.

### Known

- **Three trees cannot pass today's gate, so three backfills failed.** The release workflow's
  dispatch path runs the current `ci.yml` against the tagged source, and current is not what
  September's source was written against.

  | Tag | Run | What refused |
  |---|---|---|
  | 0.1.0 | 35512364896 | `task site-lab`, exit 200 — the browser lab at 2026-09-01 source |
  | 0.21.0 | 35511832399 | `consumer_coverage::metadata::tests::current_compiled_provider_executes_one_guard_and_binds_its_opaque_proof_to_this_run` |
  | 0.22.0 | 35512351961 | five cases in `consumer_coverage::enforce::metadata_tests` |

  `0.1.0` and `0.22.0` keep a published release with no archives; `0.21.0` keeps a tag with no
  release. None of the three tags is touched — deleting a tag that has been public for weeks
  breaks whatever pins it, to tidy a report. `0.21.0` is named in `WITHOUT_RELEASE` in
  `crates/edge/ess-xtask/src/main.rs` with that reason, so `release status` reports it as
  exempted rather than as an open defect.

- **`0.17.0`'s section says "The release workflow for it produced no assets" and that is no
  longer true** — the release carries all five. The rest of that section stands: the tag holds
  one of the two features written under it, and both first ship in 0.18.0.

- `0.17.0` is the one minor with no `changes/*.yaml` entry, named in `WITHOUT_FRAGMENT` with the
  reason. Its two features are recorded under 0.18.0, which is where they shipped.

## [0.27.0] — 2026-09-20

### Added

- **An enum variant carries its own wire spelling.** `TypeBody::Enum` carried `Vec<String>`, so
  `Naming` was reachable from a domain, a command, an event, an error, a view and a field, and not
  from a variant. A variant whose wire form is not derivable from its name could not be declared at
  all. A variant is now authored either as a bare name or as a mapping that also carries `wire`,
  `display`, `summary` and `code`:

  ```yaml
  variants:
    - name: Stop
      wire: ""
    - Flag
  ```

  The case that asked for it is one command over several route shapes, where one variant's path
  suffix is empty and the others are not; without a declared spelling the alternatives were
  splitting the command to satisfy a path, or writing the table again in every target. An authored
  empty string is a wire spelling rather than an absent one, because that variant is the route with
  no suffix. `docs/design/enum-variant-wire-names.md` states the shape.

- **Specification format `ess/5`.** A variant that declares naming under `ess/1` … `ess/4` is
  refused with `unsupported_format_version` at `types.<type>.variants.<variant>`. A bare variant
  list is admitted by every format, and a variant that declares nothing serializes back as a bare
  name, so every specification and IR document written before this keeps its bytes.

- **Delta format `ess-diff/5`, and three cases on `TypeChange`** — `VariantWireNameChanged`,
  `VariantDisplayNameChanged` and `VariantSummaryChanged`. A variant's own name does not move when
  its wire spelling does, so the variant set and the variant order both said nothing and the
  comparison returned an empty delta for a change a deployed consumer breaks on. A delta carrying
  one of the three is refused by a reader of `ess-diff/1` … `ess-diff/4`.

### Changed

- A generated JSON Schema enumerates an enum's wire spellings rather than its variant names. For a
  variant that declares no naming the two are the same string, so no existing model moves. The CLI
  contract keeps the authored names, which are what an operator types.

- The workspace's internal dependency requirements move from `0.26.0` to `0.27.0`. A minor bump
  invalidates a caret requirement, so all 20 `workspace.dependencies` entries move with the
  workspace version. `fuzz/Cargo.lock` is its own workspace and moves with them.

### Known

- A variant name is still not checked against a spelling pattern on the way in, as it never has
  been. The published schema therefore declares none for either authored form, so an author's
  editor cannot refuse a document this repository assembles.

- `task consumer-check` is red, for two reasons, and `ci.yml:89` runs the gate with
  `SKIP_CONSUMER_CHECKS=true`, so it gates neither CI nor this release. The lane reports both in one
  refusal:

  ```
  extraction/classification refused: unclassified concrete consumer entry
  ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript; finite review required;
  accounting diagnostics: {"SchemaDocumentMetadata":0,"details":"unknown claimed model
  wire:RawSpecFile#/definitions/NamedType/oneOf/2/properties/variants/items/type",
  "format":"ess-consumer-accounting/1","no_cells_admitted":true,
  "status":"PROVISIONAL_ACCOUNTING_REFUSAL"}
  ```

  The unclassified `SuiteTarget/variant/Typescript` arrived with the `origin/main` merge `7bf315a9`
  (PR 47) and is untouched here — `SuiteTarget` at `crates/edge/ess-cli/src/main.rs:601` held only
  `Ir` and `Go` at 0.26.0. The unknown claimed model is this release's: `variants` became an `anyOf`
  of a bare name and a mapping, so `.../variants/items/type` no longer exists and
  `crates/edge/ess-xtask/src/consumer_coverage/reviewed-candidates.json` still claims it. That file
  is a reviewed artifact — `cargo xtask consumer-extract` writes a fresh `UNACCEPTED` checkpoint
  elsewhere and never rewrites it (`consumer_coverage/mod.rs:47-133`), and `preservation.rs:15`
  only reads it — so retiring the claim is a finite review, not a regeneration, and is not done
  here.

## [0.26.1] — 2026-09-17

### Fixed

- **An optional leaf may be absent or null, and both runners now say so.** The synthesizer wrote
  `"optional": true` on every leaf it walked an `Optional` through, the scenario document carried it,
  the Go format validator admitted the key and the canonicalizer defaulted it — and the struct
  `encoding/json` unmarshals into never NAMED it. The flag was discarded in silence at the last step,
  so a declared-optional path could not be satisfied by absence or by an explicit null.

  The same defect sat one level up in the Rust runner, which is what made this more than a one-line
  fix: a null on the PATH read as blocked, a hard failure regardless of `optional`, so
  `Optional<Struct>` published as null was unsatisfiable in both lanes. Of the 84 optional leaves in
  the shape that exposed this, 82 are nested under an optional struct at depth 3-5 — repairing only
  the Go struct would have left the two lanes disagreeing on all 82.

  Optional means "may be absent", NOT "unchecked", and that is the case a careless repair breaks. Ten
  cases are pinned side by side against the pre-change and post-change runtime, and the load-bearing
  ones are the failures: present-and-wrong with optional still fails, a wrong leaf inside an optional
  struct that WAS sent still fails, and a scalar where a struct is declared still fails — now named
  as blocked rather than mis-reported as absent. Required behaviour is unchanged.

  Measured on an adopter's suite: 163 shape-bearing steps, 326 optional leaves concentrated in 15 of
  them. Before, 15 of 163 steps were unsatisfiable as written. After, none.

  No committed suite drifts, because the synthesizer was not touched — one of them already carried
  two `"optional": true` leaves the Go runtime was discarding.

### Known

- The Go runtime does not check an `enum` leaf at all: `holds()` skips everything where the leaf is
  not a primitive, so a declared enum admits a string outside the enum, a number or a boolean. The
  Rust runner, the Rust admitter and the browser admitter all check membership; only the runner an
  adopter executes does not. Measured at 72 of 591 leaves in one adopter's suite. Same shape of bug
  one type along, and it has its own change.

### Added

- **A conformance suite can be emitted as a TypeScript test package.** `ess verify conform
  synthesize --target typescript --out <dir>` writes the runner, the predicate evaluator, the
  response decoder, the reading accessor and the coordinate reader as an ESM package `node --test`
  runs, beside `suite.json`, a `package.json`, a `tsconfig.json` and a README. An adopter whose
  server is Go and whose client SDK is TypeScript could hold the server to a specification through
  `--target go` and could hold the client to nothing; the client is the half an integrator writes
  against.

  **Spelled in full, not `ts`.** This binary already calls the language `typescript` in three other
  `--target` values — `ess generate types`, `ess normalize generate` and `ess schema types-bundle`.

  Every place TypeScript and Go could not be the same is a difference of mechanism and none is a
  difference of verdict; the runtime lists all five with its reasons. `ErrUnsupported` is a sentinel
  `Error` whose wrapping survives because `isUnsupported` walks the `cause` chain. The four optional
  capabilities are discovered by method presence, an interface being erased at run time. Every
  `Target` method may return a promise. A JSON number is carried as `JsonNumber` holding its own
  digits, because `JSON.parse` collapses 9007199254740993 onto 9007199254740992 and the two runtimes
  would then admit different lineages. Object keys sort by UTF-8 bytes, which is where Go and
  JavaScript disagree above the basic plane.

  The package's own 201 cases run in the repository gate, their formatting is checked against the
  committed configuration, and both the emitted package and the repository's sources are
  typechecked against Node's declarations — a `--noCheck` standing in for a typecheck is how the
  emitted `tsconfig.json` first shipped without `types: ["node"]`.

- **A generated scenario that creates an owned entity now creates its owner first.** `owns` says the
  far side does not stand on its own, and synthesis was not reading that: a `creates:` subject was
  taken to need nothing arranged, so every generated scenario reaching an owned row invented an
  owner id and handed it to the implementation. On an adopter's model — `Agent owns AgentDraft` —
  that was fifteen scenarios reporting `unsupported`, which is not a failure and not a pass but **no
  information about the implementation**, produced by a suite asking a question with no answer.

  The arrangement recurses (a three-deep chain is three commands) and stops at each owner's
  **initial** state: the relation says the owner must exist and says nothing about the state it must
  be in. The link from the relation to the input that names the owner is the creating branch's
  `sets:`, and nothing else — matching on a shared field name, or on the one input typed as the
  owner's identity, is a guess that does not fail loudly but points a scenario at somebody else's
  row. A model closes the gap with one line: `sets: {account_id: input.account_id}`.

  **Four ways there is no owner to arrange, and every one falls back to the suite as it was rather
  than refusing**: nothing owns the entity (a root is not an error); `sets:` does not determine the
  carrying field; nothing creates the owner — which is `examples/billing/`, where `Account` owns
  `Invoice` and no command brings an account into existence; and an ownership cycle, guarded by the
  chain of entities being arranged rather than by a depth count. Refusing instead would have deleted
  every billing invoice scenario, and those scenarios pass.

  `EssIr::owner_of` is the lookup, beside `relations_carried_by` and deliberately narrower than it:
  a `references` is a link the far side outlives, so it is not something a row's existence depends
  on. Design: `docs/design/owned-subject-arrangement.md`.

- **A `sets:` field filled from an input that names an arranged row is now asserted as that
  reference.** `settled` kept a source only where the invocation supplied a literal, so exactly the
  field carrying an owner was dropped and nothing was said about it. A suite now asserts that the
  new row holds the id of the owner *this scenario created* — a value neither the suite nor the
  specification can spell, and the assertion an implementation that files the row under a different
  owner fails.

- **A branch may say it is the FIRST report of the state it moves to: `when_state_changes: <bool>`.**
  A `when:` guard is a predicate over the input, so a push that re-reports a state the entity already
  holds was indistinguishable from the push that first put it there. The new key is conjunctive with
  `when:`, admitted only on an outcome that declares `moves:`, and it names no state — the held states
  it admits are that transition's own `from` set partitioned by whether each state is the arrival
  state, computed rather than authored. `true` admits the states the move moves away from; `false`
  admits the arrival state itself, a restatement of what is held. An empty side is refused as an
  unreachable branch. Compiled to `ResolvedCondition::StateChange`, carrying the partition so a
  consumer does not re-implement the rule beside every generator that asks. Requires `ess/4`.
  Design: `docs/design/state-change-outcome-guards.md`, including the five spellings rejected.

### Fixed

- **A browser that becomes ready late no longer carries the leftover startup budget as its call
  timeout.** `reach_bidi` clamps the upgrade socket to what is LEFT of the startup deadline, so that
  one connect iteration cannot overrun that deadline by its own length — it did, by 20s, which is
  why the clamp exists. But that socket becomes `Browser::stream` and is the browser's transport for
  its whole life, so the clamp decided how long every later `session.new`, `open`, `evaluate` and
  `receive` had.

  A stand-in ready at 2.700s of a 3.000s deadline therefore served its first call with 300ms and
  died in `receive` with a bare `WouldBlock` — no stage, no measured startup, no `firefox.stderr`.
  The same browser ready at 0.050s passed the same call in 0.85s. The harm scales with slowness,
  which is the one condition a loaded CI runner guarantees, and it reached CI as a `Gate` failure in
  `coverage_browser` whose test took 37.64s where the same target takes under 5s locally.

  The socket's read and write timeouts are restored to `SESSION_TIMEOUT` on the successful return,
  so a call's budget is a property of the SESSION rather than a leftover of the start. 20s is what
  this transport had before the clamp was introduced.
  `tests/browser_startup_slow_serve_boundary.rs`'s
  `a_browser_that_became_ready_late_does_not_inherit_the_leftover_deadline_as_its_call_timeout` was
  written by the wave-24 adversary and left `#[ignore]`d against this story; it is no longer ignored
  and is the case that fails if the restore is removed.

- **A `sets:` literal is now type-checked, and one whose text spells a value of the target's
  primitive is asserted rather than silently dropped.** Three defects in one path, found by reading
  it end to end:
  - `ess-domain` never checked a `sets:` literal at all. The comment claimed it "is checked where a
    payload literal is"; `check_payload_literal` was only ever called from the payload path. So
    `paused: "false"` over a `Boolean` compiled with no diagnostic.
  - synthesis then dropped every literal — `settled()` handled `Cleared` and `InputField` and fell
    through the rest — so the field was neither checked nor asserted. A branch that declared
    `campaign_id: ""` produced a suite that said nothing about it.
  - the first repair overshot: type-checking alone refused 52 literals in an adopter's model with no
    valid spelling available, since an unquoted `false` is not a literal source. Admitting text that
    spells a value of the primitive — `"true"`/`"false"` for `Boolean`, canonical decimal for
    `Integer` — turns those 52 refusals into 52 asserted fields. `"perhaps"` over a `Boolean` still
    refuses, and the diagnostic now says what IS accepted.

  The admitted spellings are read from one place: `ess-conformance`'s own setup reader, extracted so
  both crates share it. A non-canonical integer (`007`, `+7`) is refused at authoring time rather
  than reshaped. `Decimal` and `Binary64` still abstain deliberately — admitting a spelling without
  the reader that sends it is how two crates come apart.

  Measured on an adopter's suite: 130 view-assertion fields now carry a Boolean literal, 7 a numeric
  one, and 13 the empty string, where all 150 previously carried nothing.

- **An arrangement step no longer leaves an earlier step's value asserted.** `arrange()` extended
  `settled` without the invalidation `run()` applies, at both its accumulation sites. Where a later
  step wrote a literal synthesis could not spell, the suite kept demanding the earlier step's value —
  and because a Boolean witness base is `true` while the branch wrote `false`, the stale claim was
  the NEGATION of the specification's final write, not merely unproven. A test carrying it could only
  pass against an implementation that ignored the branch. The rule now lives in one `absorb`
  function called at all three sites, because it had to hold at all three and was written at one.

## [0.26.0] — 2026-09-16

### Added

- **A branch may say the field it owns holds nothing: `sets: {field: {cleared: true}}`.** `sets:`
  admitted `input.<field>` or a literal, and neither can say "absent". The three ways to write it
  before this, measured on an adopter's model:

  | written | what happened |
  |---|---|
  | nothing | the field kept what an earlier act determined, so the suite required the old value |
  | `metrics: none` | compiled, then dropped by synthesis — a literal is not read as the field's type |
  | `lane_id: ""` | an empty string, which is a different reading from absent, and impossible for a struct |

  The first cost that adopter a red scenario in one run of three: its `dispose` branch clears a lead
  and an asynchronous backend read sometimes put it back before the assertion.

  Two rules, and each is about a place rather than a value. The field's type must be `Optional<…>` —
  a required field cannot hold nothing, and a specification saying it can is wrong rather than
  surprising. And it is an **entity** source: refused on an event payload, because an event field the
  emitter does not determine is one the outcome does not list, which is what `PayloadSource` already
  calls undetermined.

  Synthesis **asserts** it rather than abstaining: the scenario reads the view and requires the field
  empty, so a target that leaves the old value fails. That is the difference from every other source
  this could have been modelled as.

### Fixed

- **A suite required a field at the value an earlier act supplied, after a later act overwrote it.**
  Synthesis accumulates what each invoked branch leaves in the entity's fields, and it abstains on
  two sources it cannot read as a value — a literal, and a field crossing a declared conversion.
  Abstaining is a statement about *that* act; the older determination was left standing, which turned
  it into a claim about the row, and the opposite claim.

  Measured on an adopter's model: a `leave` branch writing `campaign_id: ""` produced a suite whose
  view assertion went on requiring the campaign id the creating act had supplied. Two scenarios
  failed against an implementation that cleared the field exactly as the specification said to, and
  nothing in either document said why. A branch that writes a field now drops any earlier
  determination of it, whether or not it determines a value in its place.

  What is still unsaid, and is a construct rather than a defect: a branch that *clears* an
  `Optional` field has no spelling. `sets:` admits `input.<field>` or a literal, so "this branch
  leaves nothing here" can only be written as a literal the reader must not believe. Filed as
  `story:a-branch-may-clear-the-field-it-owns`.

- **A skipped scenario says why.** The emitted Go runtime discarded the target's own error on
  `ErrUnsupported` and printed only the construct's name, so three different causes rendered
  identically. Measured on an adopter's run: **41 skips, three distinct reasons, one message** — and
  the only way to learn anything was to patch the generated file, which an adopter did four times in
  one day before this landed. All four call sites now include the error: `execute_command`, both
  view reads and the binding invocation.

  The Rust runner never had this defect — it carries a `Diagnostic` (`report.rs:487-493`). This was
  the Go emitter alone, and the asymmetry is why it went unnoticed.

  The **report** still carries `<status> <id>` and no reason. It cannot carry one without a new
  field: `failed_scenarios` is a list of strings and `evidence.rs:99-104` reads everything after the
  first space as the scenario identity, so appending a reason would corrupt that field for every
  reader. Filed as `story:a-report-says-why-a-scenario-was-skipped` rather than forced into a format

- **A filtered run published a report claiming the scenarios it never ran had passed.** `-run` skips
  a subtest's body, so its status kept the optimistic default set before `t.Run` and report/1
  recorded it anyway. Measured on an adopter's suite/4: **one** selected scenario, `go test` exit
  **0**, and a document asserting `"status": "passed"`, `"scenarios_total": 112`,
  `"scenarios_failed": 0` — with the 61 that normally skip counted as passes. That is the document an
  adopter's own instructions say to trust *instead of* the exit code.

  Both formats now refuse a run that did not reach every scenario the suite holds, and say what is
  missing and what to do:

      report/1 refused: incomplete execution: 1 of 99 scenarios reached a conclusion, so this
      run cannot say what the other 98 came to. A `-run` filter is the usual cause — drop it to
      publish a document, or unset ESS_REPORT_OUT to filter without publishing one

  report/2 already refused, via `countDocument`, but said only "selected subtests omitted or did not
  terminate" — neither the counts nor the remedy. Both now share one check.

  Filtering without asking for a document is unaffected: no `ESS_REPORT_OUT`, no document, nothing to
  be wrong. An unfiltered run is unchanged — verified on the same consumer: `inconclusive`, 99 total,
  59 not passed.
  with no room for it.

## [0.25.0] — 2026-09-16

### Added

- `naming: { code: … }` names the identifier a code emitter spells a declaration as. A
  target with no dotted type names flattens a qualified name to one identifier, so an
  entity's derived `X.State` and an authored view `XState` both want to be `XState`.
  One of them has to move, and this is the author saying which — because the author
  knows which name carries meaning and an emitter does not. Optional and serialising
  out when unset, so every document that does not use it compiles to the bytes it
  always did; the source format stays `ess/4`, and a build older than this one refuses
  a document that uses the key, because `Naming` is `deny_unknown_fields`. Only code
  emitters read it: wire names, document schemas and the qualified name are untouched,
  so setting one cannot break a deployed consumer.

### Fixed

- A legal specification could not be emitted for `rust` or `web`. Since 0.19.0, which
  added the workspace feasibility check, an authored declaration whose flattened
  identifier equals a derived one refused the entire workspace — including a view
  named for the entity whose lifecycle it projects, which is an ordinary thing to
  write. `--target go` emitted the same workspace, so the model was legal, compiled,
  and unemittable by two of three targets. Measured on a released model: five
  collisions, all of the form `X.State` against an authored `XState`.

  The refusal stays, because a collision an emitter silently repaired would be worse,
  and it now names the remedy rather than only the symptom.

- `go` repaired that collision backwards. `allocate_names`' own doc comment states the
  rule — "a declared type's name comes from the specification and a derived one is this
  emitter's to move" — and `allocate_declared` swept `ir.types()` first, where an
  entity's derived state enum is filed alongside authored types. `X.State` sorts before
  `XState`, so the derived enum took the identifier and the **author's** declaration was
  renamed `XState_`, silently, with no test covering it. Authored names are now reserved
  before derived ones.

- `rust`'s entity-snapshot allocation did not reserve the lifecycle enum's name. It
  reserved `XData`, `AnyX` and `x_state` but not `XState`, so a snapshot fallback was
  free to choose a name the state enum already held. Found while fixing the above; no
  committed specification reaches it.

  No generated bytes change for any specification that does not collide.

## [0.24.0] — 2026-09-13

### Added

- `settings:` on a component states each configuration input once, with a type drawn
  from the specification's own `types:`. `ess specify runtime compile` derives the
  `ess-runtime/1` config and secret slots from that declaration — the environment
  variable upper-snake-cased from the setting name, the kind from the setting's type
  and any literal — and refuses a runtime document that hand-authors a slot for a
  component that declares settings, or two declarations that would bind one
  environment variable twice in one container, naming the settings and the components.
  A setting states absence with `Optional<…>` and with nothing else: a secret typed
  `Optional<…>`, and a setting typed by a name whose representation is `Optional<…>`,
  are refused where they are written. The key is optional and serialises out when
  unset, so every document that does not use it compiles to the bytes it always did;
  the source format stays `ess/4`, and a build older than 0.24.0 refuses a document
  that uses the key, because a component's fields are `deny_unknown_fields`.
- `ess conform run --target interpreted` is selectable beside `billing` and
  `oracle-fixture`. The target decides nothing yet: every method refuses with
  `TargetError::unsupported`, so every scenario returns one unsatisfied obligation and
  the run fails rather than errors. It is the seam the model-driven interpretation
  epic fills.
- The ESS toolchain is specified in ESS, under `models/toolchain`, and its enums are
  gated against the source they describe.

### Fixed

- A command name declared twice is refused even when the first of the two declarations
  has an error of its own. Before, a first declaration that failed its own `try_from`
  never reached the name check, so the second silently took the name and the author
  was told about one error instead of two.
- Every literal a document writes is checked, however deep. A literal inside an
  `Optional<…>` of a type whose ring is broken is owned by one pass rather than
  falling between two.
- `uniqueItems` is decided the same way wherever `schema-contract` validates.
- A refusal about a closed enum names the enum it is about and cites the file that
  holds it.

## [0.23.0] — 2026-09-11

### Added

- Source `ess/4` declares error wire names without merging semantic error identities.
  Native HTTP responses honor explicit codes and preserve the qualified-name fallback.
- Typed command response fields can fill emitted event payloads through explicit
  response mappings. Source/4 requires complete payload ownership, including explicit
  generated fields; ordinary suite/8 and coverage suite/9 compare actual returned values.
  New error and response deltas use `ess-diff/4`.

- Authored `ess-scenario/2` setup establishes typed, isolated backend entity rows
  for subsequent real view assertions. The optional Rust/Go adapter capability
  uses suite/6 or coverage suite/7 with explicit report/2.
- `when_subject_state` in `ess/3` combines a declared held lifecycle state with
  input guards. Shared bounded coverage and actual identity/state view assertions
  distinguish equal input applied to different existing states.
- Fresh conformance synthesis accepts `--compact`; ordered compact JSON plus one
  newline preserves decoded meaning and retains its own exact suite-byte identity.
- Opt-in `ess/3` binding accessors read two or three declared field segments from
  event envelopes. Struct/newtype traversal, Optional and union availability, and
  whole terminal values have bounded typed plans shared by native Rust and Go.
  Computed mappings and implicit session context remain unsupported.
- Accessor observations use ordinary conformance suite/6 or declared-coverage suite/7
  with explicit report/2 execution. Ambiguous nested-Optional observations and
  conversions without a mechanical observation rule retain capability refusals.
  Existing source/1-/2 and conformance/4-/5 behavior stays unchanged.
- Complete target failures for accessor models use `ess-target-failure/3`, including
  the new `accessor-resource` cause. Legacy models retain their existing failure
  formats and bytes.
- Binding-local ordered selection adds first-match, occurrence-based exclusion and
  fallback over declared record lists, with whole-input admission and Rust/Go
  helpers for explicitly prepared host conversion results.
- Periodic bindings declare host ownership, typed context/read inputs, fixed-rate
  timing, serial work and acknowledged stop. Controlled Rust/Go observations verify
  actual scoped occurrences; native generation retains the explicit host obligation.
- Clock-reading newtypes retain encoding and origin requirements. Native and
  conformance Rust/Go normalize observed readings only with matching source/epoch
  authority. New cause, selection-plan and reading-contract deltas use `ess-diff/3`.

### Fixed

- Compact predicates refuse trailing Boolean syntax after a quoted literal and point
  to structured `any`, `all` or `not`. Rust, Go and browser admission agree.
- Structured text comparisons preserve literal values across canonical round trips.
  Comparisons requiring the corrected text reader select suite/8 or coverage suite/9;
  earlier suite envelopes refuse those forms before execution.
- Go list selection validates canonical primitive types and rejects incorrectly typed
  list members, including values after an otherwise valid first match.
- CLI regression coverage verifies complete compiled view declarations reach both
  stdout and disk. Report adoption guidance distinguishes report/2 failure and skip counts.

- Closed-enum outcome partitions can validate and synthesize witnesses without a
  fictional default. Missing and overlapping values are reported from the declared
  finite domain; unsupported or unknown domains retain conservative requirements.
- Binding documentation describes enum membership only when the compiler reached the
  representation within its existing traversal bound. String-backed literals explicitly
  state that type invariants and external resources are not checked.

## [0.22.2] — 2026-09-10

### Documentation

- Refresh the dated published-release observation to 0.22.1 while preserving the scope of
  earlier conformance evidence and the generated current-source support matrix.

### Fixed

- Shared Gates 0.1.1 requires exact automation authors on every post-baseline commit before
  scanning or receipt reuse. Local hooks also require exact bot author and committer.
- Classify the two native xattr helpers explicitly in consumer discovery, without granting
  model support or changing the frozen initial accounting baseline.

- Generated output accepts SELinux and SMACK access labels imposed by the platform. Foreign
  attributes, file capabilities, ACLs and overlay control attributes still refuse publication;
  metadata outside the ownership ledger is never silently discarded.

## [0.22.1] — 2026-09-10

### Fixed

- **The release gate is green at the tag again.** 0.22.0 was tagged from a commit whose own
  `task check` failed — eleven `ess-xtask` tests — so its `Release` run published no archives, and
  0.21.0 had failed the same way. Two causes:
  - The three reviewed `wire:RawSpecFile#/definitions` rows in
    `crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json` carried the
    definitions-container digest from before `delivery: at_most_once`. The container changed with
    that word and the rows were not re-reviewed. Re-reviewed here: the reason and the decision page
    still hold, and only the shape digest moves.
  - `website/docs/status/where-this-stands.md` states the workspace version inside the rendered
    support block, so a version bump that does not re-run `cargo xtask support` leaves
    `support-check` red at exactly the commit a tag points at. Re-rendered for this version; a
    release commit carries the re-render beside the bump.
  - `fuzz/Cargo.lock` still pinned the workspace crates at 0.21.0: the 0.22.0 bump did not touch
    it, so `fuzz-check`'s `--locked` builds refuse at the tag. Updated beside the root lock.

## [0.22.0] — 2026-09-10

### Added

- Bindings accept `delivery: at_most_once` for one attempt without redelivery. Loss remains
  possible, and the guarantee does not impose an idempotency obligation on the handler.
- OpenAPI requires `Idempotency-Key` only for commands invoked by an `at_least_once` binding.
  AsyncAPI, documentation, graph and browser projections describe both delivery guarantees.
- Conformance records `BindingGap::DeliverySingleAttempt` instead of synthesizing a redelivery
  for an `at_most_once` binding. Existing `at_least_once` scenarios remain unchanged.
- `BindingChange::DeliveryChanged` reports changes between delivery guarantees. The published
  schema admits the new value; the format remains `ess/1`, with existing IR and digests unchanged.

### Changed

- Common source security and privacy checks use pinned public Gates tooling and
  signed local evidence. Bot delivery no longer requires an Atlas checkout.
- The repository test runner declares the same compiler and wrapper profile as
  consumer qualification, so the compiled-provider regression runs with admissible evidence.
- Consumer checks refuse drift in the ESS evolution preservation mapping before running
  expensive extraction and behavioral cases. Existing semantic eligibility remains unchanged.
- CI disables the consumer-check lane by explicit operator request. Local `task check`
  and `task consumer-check` retain the complete consumer coverage exercise.

## [0.21.0] — 2026-09-10

### Added

- Typed CLI presentation bindings through `ess-cli/1`, `ess specify cli` and
  `ess generate cli`. Local actions, service calls and schema-selected dynamic
  calls produce deterministic Rust/Clap packages, help and Bash completion.
  Generated adapters validate typed inputs, results and errors, keep process
  context separate from payloads, and support protected credential entry.
  Application behavior is supplied through explicit handlers; the default handler
  reports unavailable.
- Finite deployment recovery for the `SingleHostGeneratedHelm1` profile:
  explicit authority admission, a durable invocation journal, authenticated
  observations and bounded Helm execution. Each operation attempts at most one
  mutation; uncertainty remains explicit instead of implying rollback or retry.
- Generated-output ownership records, complete owner inventories and recovery of
  interrupted publication. Native path admission protects authored and foreign
  output, preserves supported filename spelling and checks filesystem conflicts
  before publication.

- Native `ess verify bindings` connects exact implementation selections to scoped Kubernetes
  workload templates, with deterministic JSON/Markdown and satisfied, violated or unknown exits.
  Mutable tags and source-to-image provenance remain unknown; live collection never falls back
  to a cached observation. Explicit `ess-realization/2` admits implementation-only selections
  without inventing executable entrypoints; v1 admission and canonical bytes remain unchanged.

- Native namespace topology collection with `--namespace`: exact namespace and referenced-node
  reads, payload omission before writing, and typed coverage in observation/IR version 2.
  Graph and drift retain the scope; intent checks withhold conclusions and projection refuses
  omitted content. Failed cluster reads no longer retry in the current namespace, and unsupported
  selector terms are refused instead of becoming match-all selectors.

- Opt-in `ess-conformance/5` records selected generated and authored coverage,
  omitted scenarios, source identities and every refusal occurrence. Checked
  `ess-conformance-input/1` carriers retain original suite and parent bytes;
  Rust, generated Go, the CLI and the generic browser admit complete lineage
  before execution. Report 2 distinguishes complete passing conformance from
  empty, unknown or incomplete coverage. Browser replay and impact analysis
  retain the same selection boundary. Legacy suite/report meanings and defaults
  remain unchanged.
- Executable offline registry examples for generated source-syntax and contract
  schemas, using adopter-owned resource IDs and separate strict selector envelopes.
  Generated schema bytes remain unchanged; syntax admission, system semantics and
  restricted TypeScript projection have distinct documented boundaries.

### Fixed

- Exact numeric predicates preserve integer distinctions that binary64 cannot
  represent across Rust, generated Go and browser execution. Shared primitive
  admission aligns integer bounds, decimal and base64 grammar; strings are never
  silently treated as numeric facts.
- Migrated command diagnostics carry typed rule identity and source sites through
  validation and compilation, so changing message wording does not change their
  codes or locations. Existing rendered location fields remain available.
- Concurrent cache-writer tests provision independent authority registries while
  retaining the shared-cache race. Firefox tests use browser-assigned ports to
  avoid port reservation races, and generated CLI packages are checked as
  independent adopter workspaces.
- Offline specification fuzzing exercises real source readers, generated ownership
  checks and bounded retained conformance inputs, with deterministic regression
  replay and explicit unsupported coverage.

- Specification and scenario commands accept an optional `ess-inputs.yaml` manifest
  with exact file lists, allowing nested authored inputs alongside generated files.
  Invalid selections refuse before outputs or execution; selected source identities
  and original bytes remain intact. Existing layouts work without a manifest, and
  recursive legacy model discovery visits directory aliases deterministically.
- Release commands distinguish consistency checks, OCI content verification and
  conformance qualification. New check-conformance and publish-conformance routes
  require a complete nonempty passing report for the exact expected selection and
  compiled context. The release action snapshots those inputs before checks, rejects
  inconsistent context before effects, and uploads the checked report bytes. Action
  adopters must update its required inputs and ESS revision together; producer origin,
  signatures and artifact execution remain unverified.
- The browser test harness waits for Firefox's WebSocket route, retrying its startup 404 within
  the existing deadline while preserving strict upgrade validation and response evidence.

- Both generic replay players preserve typed inputs and ordered declarations, mark
  unavailable assignment, subject and view results as unknown, and cancel stale
  playback callbacks after reset, selection or pause. Expectations remain
  unexecuted; existing suite and replay formats retain their bytes.
- Public capability documentation now distinguishes current source from the separately
  observed release, describes the actual site output and four synthesis targets, and
  checks the maintained support table against real CLI and projection outputs offline.
- The model reference and specification/realization guides distinguish logical
  components, interface contracts and delivery metadata, including the current
  single-reach limit and the separate model and realization digest boundaries.
- Release-bundle and Helm cache hits now verify the requested original OCI manifest
  and every referenced blob's size and digest. Legacy cache entries trigger cold
  acquisition, corrupt proof entries refuse, and Helm consumes a private verified
  chart snapshot. Cold fetches have bounded reads and one shared deadline.

## [0.20.0] — 2026-09-06

### Added

- Standalone TypeScript normalization from `Plan::typescript(package)` and
  `normalize-generate --target typescript --package NAME`. Generated ES2022 ESM
  packages execute formats 1–6 from exact JSON text, use bigint for exact integer
  representation, retain source pins and expose strict retained-base64 helpers.
  The fixed schema profile refuses unsupported constraints before generation;
  `uniqueItems: true` is explicitly outside that profile. Complete Rust/Go maps
  retain their bytes at the same generator version.
- Authored `ess/2` adds finite `Binary64` fields, distinct from integer and decimal
  values. Model projection retains compiler-owned numeric locations; old authored
  formats and Binary64 map keys refuse at every declared type position.
  `ess-normalization/5` requires explicit input paths and adds token-preserving
  `binary64_literal` constants and finite `binary64` conversion/steps. Reference
  and generated Rust/Go preserve signed zero, subnormals and nearest-even rounding.
  Two typed Binary64 operands use IEEE equality; authored model predicates retain
  their existing Number comparisons. Complete format 1–4 generated maps remain
  frozen. Whole-system synthesis and conformance refuse unsupported Binary64
  before publication; TypeScript structural output reports the finite codec obligation.
- Standalone model Rust/Go libraries now emit checked finite Binary64 wrappers.
  Original-token codecs preserve signed zero, subnormals and nearest-even rounding
  through supported aliases, recursive containers and unions. Rust requires
  source-backed Serde decoding; mixed declared-field/Binary64-extra records refuse
  before output. Existing non-Binary64 generated maps remain unchanged.
- `ess-normalization/6` adds explicit fixed string array preparation and checked
  `position` reads in the reference, Rust and Go targets. Declared boundaries
  preserve absence, turn null values/elements into empty strings, pad short arrays
  and discard excess tokens after strict lexical checks. Active policies require
  original JSON text. Format 6 retains target report 3; complete format 1–5 maps
  remain byte-identical at the same generator version.
- Opt-in `ess-conformance-report/2` separates passed, failed, error, unsupported
  and skipped counts, binding outcomes to the exact executed suite bytes.
  Rust/CLI also expose checked `ess-conformance-run/2` detailed output; generated
  Go supports standalone report 2 and explicit strict execution. Suite versions
  1–4 retain unknown coverage, so these counts cannot establish complete
  conformance. Report 1 and diagnostic defaults retain their existing behavior.
- Checked `ess-openapi-import/1` envelopes retain original source identity and
  semantic accounting. Reloading verifies the source digest, normalization and
  derived interface before projection; constraints that cannot be preserved
  produce located gaps or refusals. Legacy `ess-service-interface/1` readers
  retain their existing representation.
- `ess-normalization/4` explicitly captures selected JSON field, array-item or root
  tokens as canonical standard base64 before first-stage schema validation. Exact
  token spelling, duplicate members and huge numeric lexemes are retained inside
  captures; JSON grammar, Unicode and the 64-level depth bound remain enforced.
  Reference, generated Rust and Go expose strict base64-to-JSON entrypoints for
  separately checked retained-document composition. Decoded-value APIs refuse
  capture branches without original token bytes. Targets use report version 3;
  frozen legacy templates preserve formats 1–3 emitted bytes and file maps.
- `ess-normalization/3` model-owned stage roots, pinned to complete compiler
  provenance and explicit type selections. `Plan::check_with_models` and CLI
  `--model` inputs reuse checked model wire projections without duplicate schemas
  or trusted imported annotations. Rust and Go retain model sources and version 2
  target reports. Unevaluated model invariants refuse planning; existing Go pattern
  limits still apply. Versions 1 and 2 retain their bundle recipe representation.
- Explicit `ess-normalization/2` ordered string concatenation/joining, exact integer
  rendering, list concatenation, original collection indices, filtered mapping and
  first-match selection. The reference engine and generated Rust libraries preserve
  order, duplicate values and lazy selected-value evaluation. Version 1 refuses the
  new operations and retains its existing semantics and canonical representation.
- Version 2 branch-specific `binary64_inputs` declarations and ordered
  `binary64_to_integer` conversion with finite multiply/minimum/maximum steps,
  nearest-even decoding and explicit out-of-range refusal. Undeclared numbers
  keep the exact JSON policy; signed integer scaling remains separately governed
  by its reject/wrap policy. Reference, CLI and generated Rust share the semantics.
- Standalone Go normalization libraries from `Plan::go(package, module)`, with
  typed operation bindings, offline pinned schema validation, exact integer and
  declared binary64 input policies, and complete source/file provenance. Native
  Go fixtures exercise old, ordered and numeric recipes against reference results
  and located refusals. The exact frozen base64 pattern used by model `Bytes` is
  qualified against the pinned reference through a shared ASCII, Unicode, padding
  and long-input corpus in generated Go and Rust. Every other selected pattern
  retains its source-located `go_schema_pattern` generation refusal.
- `ess generate schema normalize-generate` exposes Rust, Go and TypeScript normalization
  libraries with explicit package/module identity and complete source provenance.
  It checks every branch and target before destination preflight, protects source
  inputs, and offers read-only planned-file drift checking through `--check`.

### Changed

- Composition documentation states the generated client's selected-operation and
  model-identity guarantees alongside its byte-buffer transport boundary. An
  executable downstream example shows compatible and incompatible payloads
  forwarded unchanged, with separate authority and endpoint/error controls.
- Document the frozen normalization equality limitation: formats 5/6 compare two
  floating representations numerically even in an admitted Integer expression.
  TypeScript preserves reference behavior; mixed integer/floating operands retain
  integer eligibility refusal. Static Binary64 admission and output provenance
  remain distinct. This release does not tighten the core equality contract.
- In this pre-1.0 minor release, `ess_synth::{go,clap}::workspace` return
  `Result<Emission, TargetFailure>`. Their new finite-codec failures use
  `ess-target-failure/2`; Rust/Web failure envelopes keep version 1 and old bytes.
  `ess_conformance::{go,web}::emit`, `ConformanceSuite::to_canonical_json`, and
  `Runner::run` now return `Result` with a located `AdmissionError`. Readers,
  model producers, direct typed suites and CLI routes reject Binary64 before
  artifact creation or target interaction, including sparse models with no
  generated scenarios. Existing admitted suite bytes remain unchanged.

### Fixed

- Specification admission checks complete expression paths, both operands,
  membership values and lexical collection bindings across invariant, command
  and view owners. Conformance reuses the same type rules while retaining its
  separate projection and witness limits; authored operand errors are distinct
  from unreadable paths.
- Authored conformance commands refuse an explicitly selected directory with no
  immediate lowercase YAML inputs before writing artifacts or selecting a runner.
  Diagnostics explain shallow discovery. Omitted selections, direct files and
  committed suites retain their existing behavior.

### Changed

- Define execution recovery boundaries for interrupted apply and removal, manual
  drift and fresh observation. Current absence does not establish who removed a
  release. Applied-state modeling and executable recovery remain follow-on work.

## [0.19.0] — 2026-09-05

### Changed

- Rust/Web workspace emitters and synthesis facades now return checked `Result`
  values. Complete `ess-target-failure/1` refusals carry the neutral plan and
  source-located causes before any artifacts are written. Callers must handle the
  fallible API; successful plans and generated bytes retain their contract.
- Sliced provenance uses the explicit `slice-sha256/2:` digest profile. Regenerate
  older sliced artifacts when adopting this release. Semantic diff/impact now
  accounts for previously omitted constructs, served views and reusable row shapes.

### Added

- Source-pinned normalization library with strict `ess-normalization/1` recipes,
  explicit external dispatch, ordered input/output schema boundaries, separate
  missing/null behavior, lazy choices, case-sensitive string-prefix selection,
  signed-integer overflow policies, collection
  mapping and distinct-category counts. Every branch checks before reference execution.
  `schema normalize-check` checks and canonicalizes recipes; `schema normalize-run`
  executes an explicit branch without overwriting any input. The JSON text boundary
  refuses duplicate keys and numbers that cannot round-trip without precision loss.
  The Rust library API also emits standalone normalization crates with an
  `ess-normalization-target/1` provenance report. Go/TypeScript normalization targets
  and a target-generation CLI remain pending; structural libraries are not application decoders.

- Complete JSON Schema document-root import (`schema import-document`), preserving
  the root record, local definition closure and original source locations through
  validation and all three data targets. Explicit `ess-schema-bundle/2` root identity
  is replay-checked; component-bundle `/1` bytes remain unchanged. Conflicting root
  names, incompatible dialects and unsupported resource references refuse.

- Direct ESS-model data realization (`generate types`) with explicit qualified roots
  or all-type selection. Reuses the existing schema wire mapping and shared native/TS
  targets, refuses wire-key collisions and records model versus bundle input provenance
  in `ess-types-report/3`. Model invariants, map-key validation and TypeScript nominal
  identity limitations remain explicit obligations, not silently claimed behavior.

- Root-selected structural type planning over checked schema bundles and accounted
  Go, Rust and TypeScript data-only targets (`schema types-bundle`). Requiredness, nullable values,
  reference siblings, open objects and prefix tuples retain explicit shape, source
  provenance and runtime obligations. Native packages include presence-aware JSON
  codecs, exact number retention and explicit package/module identity in the versioned
  target report. Application decoder semantics remain separate from these structural targets.

- Qualified schema-only component import with explicit JSON Schema 2020-12 interpretation,
  source-byte retention, complete local reference closure and persisted replay checks.
  `schema import-bundle`, `project-bundle` and `validate-bundle` do not fabricate a service,
  apply decoder defaults or silently accept unsupported schema semantics.

- Authored site pages resolve links against explicitly included source documents and
  `--asset` UTF-8 downloads. `--front-page` preserves an override's source location;
  `--strict-links` refuses unpublished local targets with source-line diagnostics
  before output is written. Downloads share the existing output containment checks.

- A source-backed format/digest catalogue and historical maturity outlook, together
  with preserved configuration and invariant planning from older worktrees.

### Fixed

- CI explicitly provisions the WASM compiler target required by generated-workspace
  feasibility checks, including the shared release gate.
- Kubernetes imports reject malformed Secret shapes and redact failed subprocess
  diagnostics before credential-bearing values reach output.
- CLI generation preflights complete destination sets, including composed outputs,
  to reject containment and alias collisions before writes.
- Persisted delivery documents and standalone conformance reports validate claims
  through generic deserialization as well as explicit readers. Existing conformance
  formats remain current; the successor coverage formats are designs, not shipped APIs.
- Checked infrastructure model transformations preserve owning handles and resolved
  reference membership. TypeScript projection checks one final binding namespace.
- Rust/Web feasibility checks reject generated name, path, recursive layout and
  incompatible binding representations. Web dependency-module and outcome-arm
  buffer/encoder collisions are checked in their actual scopes; legal names elsewhere
  remain valid. Delivery-arm checks also account for the actual ordering of local
  bindings without rejecting legal first-input transformations. Web workspaces with
  no published events emit a valid empty event log. Supported Integer, Boolean and
  Bytes map-key decoders borrow nested diagnostic paths correctly in HTTP/Web output.

- Specification validation refuses colliding effective wire field names across structs,
  entity identity/state/fields, command inputs, event/error payloads, view rows and
  separate view parameters before any projection can silently overwrite a property.

- Long unbroken identifiers in authored prose wrap within narrow site viewports,
  including links and emphasized text; code listings retain horizontal scrolling.

- Authored Markdown `mermaid` fences in generated HTML sites now use the bundled
  diagram renderer, just like generated diagrams. Other fenced code remains a
  listing, and diagram source remains escaped and readable without JavaScript.

## [0.18.0] — 2026-09-04

### Added

- **A specification's scenarios can be rendered as a page somebody presses play on.**
  `ess verify conform web` emits a player: the scenarios on the left, a swimlane in the middle with
  one lane per actor and one row per act, and the state, the views and an optional device surface on
  the right. It is emitted for any specification and knows nothing about any of them — the page and
  the engine are static assets, and the only generated file is `model.json`, the projection the page
  reads.

  The reason to have it is that a specification nobody has run is a specification nobody has checked,
  and until now the only way to run one was to hand-write an implementation of it. Two exist —
  `reference::Billing` and `reference::Oracle` — and they exist so a generated suite has a
  known-good target, not so that every adopter writes one before seeing anything move.

  It **replays rather than executes**, and says so on the page. A scenario declares which outcome
  each command took; the player applies the effect the model attaches to that outcome. Every
  transition it shows is the model's — an outcome names its transition and the states it runs from —
  so a walk that stays legal says the specification is coherent, never that an implementation works.

  Three things the page keeps apart, because the model does: **state** is the truth now, a **view**
  is a projection with a filter, parameters and a consistency, and the **UI** is whatever a hand-
  written `skin.js` beside the emitted files renders. A view that selects nothing shows no rows; one
  whose parameter nothing has bound says which parameter it wants; an `eventual` one is allowed to be
  behind the state panel next to it.

  A binding's consequence is drawn in a lane of its own, dashed, because the model declares it and no
  scenario asserts it. That lane is where a command runs on nobody's grant, which is visible rather
  than argued about.

  `assets/vue.esm-browser.prod.js` is vendored unmodified with its licence beside it, the way
  `assets/mermaid.min.js` already is. No `package.json` enters the repository.

## [0.17.0] — 2026-09-04

> **No release was published for this version, and the tag does not carry what is written below.**
> `0.17.0` was cut from a branch off 0.16.0 holding the scenario player, while the halt assertion
> described here was on `main`; neither commit is an ancestor of the other, and `halts_after`
> appears zero times at the tag. The release workflow for it produced no assets. **Everything in
> this section first ships in 0.18.0**, which is the first release to hold both. A binary that
> calls itself 0.17.0 has one of the two features and which one depends on where it was built —
> check it for `halts_after` rather than trusting the number.

### Added

- **An authored scenario can claim that a consumer halted an ordered scan.** The six assertion forms
  are all predicates over the rows a read returned — `contains`, `excludes`, `counts`, `ranked`,
  `at`, `satisfies` — and two implementations that return the same rows are indistinguishable under
  every one of them. "The producer stopped" is exactly the fact that separates them, and until now
  the format could not say it: a migration of forty-five ACD slotmatcher cases hit the wall twice,
  refused to write a prefix assertion in its place, and quoted this crate's own documentation back —
  *a prefix read in silence is still a prefix*. The claim it could not make is `ScanOrdered`
  returning "stop" after two of three elements, and the implementation it would have hidden is real:
  one of the two engines has no early-stop iterator at all.

- **`halts_after: <n>`**, the seventh claim key, exclusive with the other six. A reader of the view
  takes `n` rows, says stop, and the producer stops too. It compiles to a step of its own rather
  than to a seventh `ViewExpectation`, and that is the whole design: an expectation is decided
  against the rows a read returned, so a claim filed there would be decided by evidence that cannot
  see it. It changes the **read** instead.

- **Two steps and one target method.** `expect_halt` and `eventually_halt` join the closed
  vocabulary, which now has nineteen words; which of the two a scenario gets is read off the view's
  declared consistency, exactly as it is for every other claim about a view. `ConformanceTarget`
  gains `scan_view`, with a default body answering `Unsupported`, so a target written against the
  earlier interface compiles unchanged; the emitted Go runner asks for an optional `OrderedReader`
  interface for the same reason. A target that cannot read a view a row at a time reports the
  scenario **unsupported**, which §28 makes a failing run. There is no path on which a scan nobody
  stopped passes.

- **Two observations, and the claim needs both.** `scan_view` reports how many rows the ordered
  *source* produced and whether the read ended because the reader said stop — facts about the
  target's own control flow, never a verdict. The count alone is satisfied by a listing that
  happened to hold exactly `n` rows and ran out; the flag alone is satisfied by a target that
  materialised everything and then broke out of a loop over the copy. An adopter can report both
  without instrumentation, because the callback shape is the ordinary one every ordered collection
  already has.

- **One refusal**, `ESS-AUTHOR-034`: a halt claimed after no rows at all. A reader that takes none
  never sees a row and so never says stop, and what an honest source produced before being refused
  its first row differs between two implementations that are both right. A halt claimed of a view
  that declares no order is refused by `ESS-AUTHOR-024`, which already says so — one mistake, one
  repair, one code. The format now numbers thirty-four, and `tests/authored.rs` still holds a case
  per code and a case asserting the numbering and the documents agree.

### Changed

- **`ess-conformance/4`.** The step vocabulary grew, which is a change to the shape of the persisted
  document, and that shape is exactly what a suite format versions — the same argument `3` made, on
  the same reader. An old Rust reader parses a closed tagged enum, so a `4`-shaped suite labelled
  `3` fails with `unknown variant`, blaming the document for the age of the tool; an old Go runner
  reports a step it does not know as skipped, which is the right answer reached by accident.
  `SUPPORTED_SUITE_FORMATS` keeps `1`, `2` and `3`, because a `3` suite means in `4` exactly what it
  meant in `3`. The committed suites under `suites/generated/` are regenerated at the new number and
  are otherwise byte-identical.

Written as 0.17.0; released in 0.18.0. See the note at the top of this section.

## [0.16.0] — 2026-09-04

### Added

- **An authored scenario can claim a length of time.** `ess-scenario/1` could say what happened and
  in what order, and nothing about how long anything took: `at:` ordered the file and reached no
  runner. A migration of forty-five ACD scenarios into the format reported that as its one class of
  loss, and enumerated it — a twenty-second hold that *is* an experiment's independent variable, two
  wrap-up windows, a one-second queue-exit threshold, a one-minute callback TTL and a five-second
  teardown margin, all of them gaps between two instants and enforced by nothing. A system that
  fires every timer the moment it is armed passed every check ESS could write.

- **Three bounds, each on an anchor somebody wrote.** An act names its instant with `mark:`, and a
  later act states what must be true of the gap: `not_before:` is the hold, `within:` is the
  deadline, and `quiet: {for: …, events: […]}` is the bounded negative. Every window names the
  instant it opens at, and there is no implicit anchor anywhere — the suite this was built for had
  negatives whose window opened wherever the preceding block happened to end, so inserting one
  arrangement step moved five assertions and no diff showed it.

- **`at:` is load-bearing at last.** It still reaches no runner, and it is now what a duration claim
  is held against: `not_before: PT20S` in a file whose two instants are five seconds apart is a
  document saying two things, and it is refused rather than compiled into whichever one the compiler
  read first.

- **Six refusals**, `ESS-AUTHOR-028` to `ESS-AUTHOR-033`: a window measured from an instant nothing
  marked, one name for two instants, a window of no seconds, a bounded negative that forbids no
  event, a claim the timeline contradicts, and a window stating other than exactly one bound. The
  format now numbers thirty-three, and `tests/authored.rs` still holds a case per code and a case
  asserting the numbering and the documents agree.

- **Four steps and two target methods.** `mark_instant`, `expect_not_before`, `expect_within` and
  `expect_quiet` join the closed vocabulary, which now has seventeen words. `ConformanceTarget`
  gains `mark_instant` and `observe_elapsed`, both with default bodies answering `Unsupported`, so a
  target written against the earlier interface compiles unchanged; the emitted Go runner asks for an
  optional `Clock` interface for the same reason. A target that implements neither reports the
  scenario **unsupported**, which §28 makes a failing run. There is no path on which an unheld
  window passes.

- **The clock stays the target's.** A duration claim could have been built on wall-clock waiting,
  which makes every suite slow and flaky; on a logical clock, which most systems have none of; or on
  an observation reported after the fact, which on its own cannot make twenty seconds happen. This
  asks for the third and permits the first two to produce it: the suite states a *length* and the
  *instant* it is measured from, `observe_elapsed` says how much of the window to let close before
  answering, and the target reports a reading in milliseconds it stands behind. An end-to-end target
  waits; an in-memory target advances a clock it owns and answers instantly; the runner compares the
  reading with the claim and will not round towards it.

### Changed

- **`ess-conformance/3`.** The step vocabulary grew, which is a change to the shape of the persisted
  document, and that shape is exactly what a suite format versions. The reader this number is for is
  the one that would otherwise be worst off: an old Rust reader parses a closed tagged enum, so a
  `3`-shaped suite labelled `2` fails with `unknown variant` — a message blaming the document for
  the age of the tool. An old *Go* runner does not return a wrong verdict here; its step switch
  abandons a scenario whose first word it does not know and reports it skipped, which is the right
  answer reached by accident. `SUPPORTED_SUITE_FORMATS` keeps `1` and `2`, because a `2` suite means
  in `3` exactly what it meant in `2`. The committed suites under `suites/generated/` are
  regenerated at the new number and are otherwise byte-identical.

Releases 0.16.0.

## [0.15.0] — 2026-09-04

### Added

- **A specification can carry the scenarios an author wrote.** `ess verify conform` had two verbs
  and neither read a scenario anybody wrote: `synthesize` generates what the specification obliges,
  and `run` executes a generated or committed suite. What was missing was an authoring surface, and
  the reason it is needed is a refusal synthesis already prints — *the contract is declared; the
  algorithm is not*. A router's matching order, a scorer's tie-break and the rule that decides which
  of two waiting calls is dispatched first are not derivable from any model, and until now the only
  places to write them down were a bespoke runner in each consuming repository or nowhere.

- **`ess-scenario/1`**, the document. An arrangement of modelled entity instances, an ordered
  timeline of modelled commands with explicit instants, and expectations in the vocabulary a suite
  already has — `contains`, `excludes`, `counts`, `ranked`, `at`, `satisfies`, over literals,
  captured instances and observed event fields. Three decisions in it are worth stating. The
  timeline's instants must strictly ascend, because a list is ordered by where its entries sit on
  the page and moving two blocks would otherwise change what the scenario means, silently. Whether a
  view assertion retries is read off the view's declared consistency and never written by the
  author, and the ranking a positional claim is relative to is the view's own `order_by:` — a second
  copy is the copy that goes stale. And a reference is spelled `{$instance: …}`, because a declared
  struct may perfectly well have a field called `instance` and `$` cannot begin an ESS field name.

- **Twenty-seven refusals**, one per way a scenario can fail to typecheck against the model, each
  with a stable `ESS-AUTHOR-nnn` code, the file it was read from and what would have to change. A
  scenario naming a command, actor, outcome, event, error, view, entity, field, enum variant or
  lifecycle state the specification does not declare is refused when it is compiled rather than at
  the first run that reaches it — which is the whole value over a bespoke runner, where a scenario
  naming a field that was renamed last week keeps passing in each consumer independently until
  something executes it.

- **`ess verify conform author`**, and `--scenarios` on `synthesize` and `run`. An authored scenario
  compiles into the same `ess-conformance/2` document synthesis emits, over the same closed step
  vocabulary, and runs on the runners that already exist — the Rust one and the emitted Go one —
  with no change to `ConformanceTarget`. The committed billing suite now holds thirty scenarios,
  twenty-nine obligations and one assertion, and the Go conformance test runs all thirty against a
  hand-written implementation with the fixture untouched.

- **The two populations are told apart by their ids.** `ScenarioId::Authored` renders
  `<domain>/authored/<name>`, so a suite, a report, a fault matrix and a `go test -run` filter can
  each tell an obligation the specification derived from an assertion a person made, without being
  told. A coverage number that counted the second as the first would describe a model that does not
  exist. The format stays `ess-conformance/2`: the id is a new word in a vocabulary that already
  grew once, an old Go runner executes an authored scenario correctly because every step in it is
  one it already knows, and the number moves for what an old reader does *wrong*, not for what it
  has not seen.

### Changed

- A value naming something a declared enum does not have as a variant is now its own refusal rather
  than a wrong shape. The repair can be stated exactly — the model declares a closed set — where
  `expected one of Email, Post, found a string` reads as a type error about a value whose type is
  right. It is what lets an authored scenario distinguish a misspelt lifecycle state from a misspelt
  enum variant, and name the entity in the first case.

Releases 0.15.0.

## [0.14.0] — 2026-09-04

### Added

- A component can declare that its callers are people at a terminal. `reached_by: command_line`
  is a third answer to the question the other two already answer — *where are the callers* — and
  like them it names no wire, no port, no path and no verb. What it states is that the surface
  leaves the process as a **grammar** rather than as a call, which is the one fact neither
  `in_process` nor `network` can state: a command-line caller is deployed with the binary and is
  not a program.

- A `cli:` block says where each accepted command sits in that grammar. Paths within a group are
  derived from `naming.wire`, as `OpenAPI` paths already are; grouping is declared, because which
  *activity* a command belongs to cannot be derived from anything the model holds. Eight refusals
  come with it, and they are the reason the tree is declared here rather than written beside a
  parser: a block on a component reached another way, a command-line surface with no block, a
  placed command the component does not accept, a placed view no domain it owns projects, an
  accepted command or a projected view the tree places nowhere, and either placed twice.

  The view half exists because of a defect found in a consuming repository rather than imagined
  here. `connectors` serves `kubernetes.workloads` from its personal-local daemon and has no
  command-line verb that reads a datasource at all, so an operator on that machine cannot reach a
  projection the process beside them is already publishing. Nobody wrote that down until somebody
  noticed. It is now a refusal at `ess validate`, before a tree is generated from a declaration
  that forgot it.

- `ess generate synthesize --target clap` emits the grammar and the completion of it: a `clap`
  command tree, one flag per declared input field, a `Handler` trait carrying one obligation per
  word the tree places, and a `completions` verb that writes a script for every shell `clap`
  supports. An enum-typed field completes its whole closed set, so a shell completes the *values*
  a flag accepts and not merely the word in front of them; a field the model cannot enumerate
  completes as free text, because offering a guess would complete values the system refuses.

  It emits no type layer. The Rust target already emits every input, outcome, event and error as
  a type, and a fourth rendering of that layer would be a fourth thing to keep in step — so a
  handler receives `clap::ArgMatches`, and `TARGET.md` states that as a weakening rather than
  leaving it to be discovered.

  The format is still `ess/1`. Both additions serialize out when unset, so every existing
  document digests exactly as it did; the published schema grows by 93 lines and loses nothing.

## [0.13.5] — 2026-09-04

### Added

- The reusable component-release action accepts an exact Rust toolchain and optional exact Node
  and pnpm toolchains for repository-owned checks. Components with generated browser surfaces can
  now use the same ESS release path without preinstalled runner tools or a bespoke release job.

## [0.13.4] — 2026-09-04

### Fixed

- The component-release composite action invokes its packaged script through Bash, so clean GitHub
  Actions checkouts do not depend on a repository executable bit.

## [0.13.3] — 2026-09-04

### Fixed

- Helm projections now materialize every typed runtime secret slot as a credential-free
  `{name, key}` default and describe that closed shape in the values schema. A newly projected
  chart therefore renders and lints before an environment supplies its Secret object name.

### Added

- The ESS-owned `release-component` action lets any component repository run its own gate, build or
  adopt an exact runtime image, package and sign its chart, publish SBOM/provenance/signature/
  conformance evidence, verify release manifests, and publish the canonical OCI component bundle.
  It has no Service SDK manifest or registry-coordinate assumptions.

## [0.13.2] — 2026-09-04

### Fixed

- Canonical build IR restores an omitted empty secret set while reading, so compiler output
  round-trips through release verification and OCI bundle construction.

## [0.13.1] — 2026-09-04

### Fixed

- Configuration-neutral Helm projections now provide a schema-valid default service account,
  allowing a freshly generated chart to pass `helm lint` before private environment bindings
  replace it.

## [0.13.0] — 2026-09-04

### Added

- `ess-component/1` and canonical `ess-component-ir/1` make the implementation repository the
  owner of its semantic, realization, build, runtime, and independent runtime/chart release units.
- `ess-release-bundle/1` carries the complete verified release chain as one OCI payload. `ess
  generate release publish` uses ORAS at the credential edge; `ess generate release fetch` requires
  a digest-pinned source, revalidates canonical bytes, and caches them by OCI manifest digest.
- `ess generate build execute` retains the deterministic BuildKit projection and invokes Docker
  Buildx Bake. `ess generate deployment reconcile` computes the affected release set, follows the
  rollout DAG, fetches charts by digest, and invokes Helm only for changed releases. It refuses
  implicit removal.
- Runtime models can declare component-owned HTTP endpoints, named persistent volumes, and explicit
  container mounts. Helm projection now emits Services, workload-specific selectors, stateful
  governing Services, claims, and mounts. Named required endpoints can be derived from locked
  component or external-system providers without duplicating URLs in environment bindings.

### Changed

- Side effects are an explicit CLI executor boundary. The deployment compiler and every projection
  remain deterministic and offline while BuildKit, OCI, and Helm execution reuse their exact IR.

## [0.12.0] — 2026-09-04

### Changed

- **`ess --help` lists four commands, one per area, and every verb keeps the spelling it had.**
  The crates moved under `crates/{specify,generate,verify,infra}/` in 0.11.1; the command surface
  now says the same thing, so where a thing is implemented and how it is spelled are one fact
  instead of two. The first level is `specify`, `generate`, `verify`, `infra` and nothing else.
- **Every flat spelling is a hidden alias of its area path.** `ess validate --path .` is
  `ess specify validate --path .`: the same arguments, and when the command runs, the same stdout,
  the same stderr and the same exit status, with no deprecation line anywhere. For a refusal clap
  writes — a missing required argument — and for `--help`, only the `Usage:` line differs, because
  it names the path that was typed, which is what the flat spelling has always printed. The alias
  is left out of `--help` only so the listing stays the four areas. Both spellings are mounted from
  one definition in the derive, so they cannot drift apart, and a test enumerates every leaf of the
  clap tree and asserts the pairing rather than a list somebody keeps up to date. Nothing is
  deprecated and no pinned caller — agentide's gate invokes `ess compile` and `ess generate` by
  name — needs changing.

  | Flat spelling | Area path |
  |---|---|
  | `ess validate` | `ess specify validate` |
  | `ess compile` | `ess specify compile` |
  | `ess compose` | `ess specify compose` |
  | `ess inspect` | `ess specify inspect` |
  | `ess graph` | `ess specify graph` |
  | `ess realization` | `ess specify realization` |
  | `ess runtime` | `ess specify runtime` |
  | `ess generate` | `ess generate generate` |
  | `ess synthesize` | `ess generate synthesize` |
  | `ess project` | `ess generate project` |
  | `ess schema` | `ess generate schema` |
  | `ess build` | `ess generate build` |
  | `ess release` | `ess generate release` |
  | `ess stack` | `ess generate stack` |
  | `ess deployment` | `ess generate deployment` |
  | `ess conform` | `ess verify conform` |
  | `ess diff` | `ess verify diff` |
  | `ess impact` | `ess verify impact` |
  | `ess infra <operation>` | `ess infra infra <operation>` |
  | `ess import` | `ess infra import` |

  Two verbs share the name of their area. `ess generate --path …` is the flat spelling of
  `ess generate generate --path …`, and `ess infra diagnose …` of `ess infra infra diagnose …`.
  The `generate` area therefore offers the verb's five options beside its eight subcommands, and
  refuses the two written together — `ess generate --path X synthesize` says two things at once and
  exits 2, as it did before the area existed.

## [0.11.1] — 2026-09-04

### Changed

- **Every crate moved under the area it serves — `crates/<area>/<crate>/` — and no crate or binary
  was renamed.** `specify/` carries an authored system to a validated IR, `generate/` turns that IR
  into artifacts, `verify/` holds an implementation or a later revision to it, `infra/` is the
  separate bounded context over an observed cluster, and `edge/` is the `ess` binary and this
  repository's own tooling. Every `[package] name`, every `[workspace.dependencies]` key and the
  `ess` binary name are the bytes they were: nothing an adopter builds, links or runs changed, and
  `cargo metadata` names the same twenty-two packages. `ess-deployment` is under `generate/`
  because it depends on `ess-compiler` and `ess-realization` and on no `infra-*` crate.
- A workspace test now asserts both halves of that layout: every member sits under one of the five
  areas, and every literal `crates/<crate>/…` path this repository writes down — a fixture argument
  in `Taskfile.yml`, a source path a test reads, a path named in prose — resolves to something that
  exists. The paths that break on a move are the ones no compiler reads.
## [0.11.0] — 2026-09-03

### Added

- **A wrong-state branch that accepts rather than refuses: `refuses: false`.** `wrong_state:` could
  say one thing — the command refuses here, and reports this error — so a command that is
  deliberately idempotent had no way to be written down. ACD's `EndCall` on a call that has already
  ended answers and does nothing, on purpose and documented; its specification claimed a refusal the
  code never makes, and that claim was filed as a defect in ACD before anybody read the code. A key
  rather than "leave `error:` out": a branch missing its error is a mistake the validator has caught
  since `wrong_state:` shipped, and making the omission mean *accepts* would turn every one of those
  mistakes into a silent claim. The generated scenario's id says which claim it carries —
  `…/accepts/EndCall` beside `…/refuses/CancelInvoice`.
- **`params:` on a view, and a `query_view` that carries them.** Most views are one answer the whole
  system shares. ACD's position in a queue is not: it is per queue, and one ranked list of every
  queued call is a different thing from what the implementation can be asked. A parameter is read in
  the filter as `param.<name>`, so the predicate grammar needs nothing new; `params:` and the
  parameters the filter reads must be the same set. Synthesis binds each from what the arrangement
  settled — the scenario asks for the lane it just put the order in — and the runner refuses to send
  a request with a parameter it could not resolve, because a query with a made-up parameter reads a
  different set of rows and every assertion after it is about the wrong thing. `SemanticViewRequest`
  and the Go runner's `ViewRequest` gain `Params`.
- **`task release-status` checks that every pushed version tag is on `origin/main`.** See *Fixed*.

### Changed

- **A view's filter is now decided against what the arrangement settled, not only against the
  state.** The state was the only fact a synthesised scenario knew about the instance it had just
  created; `sets:` made the rest knowable in 0.10.0. Without this, `lane_id == param.lane_id` stays
  undecidable however well the parameter is bound, because the left side is the one nothing had
  answered. A filter over a field the scenario supplied is now decidable, so more views are asserted
  and fewer are refused as undecidable.

### Fixed

- **A release could be tagged on a branch that never reached `main`.** `task release-status` asked
  the remote for the tag list and GitHub for the release list, and neither says which line a commit
  is on — so `0.10.0`, tagged on `feat/outcome-sets-entity-fields` and never merged, reported
  "tagged and published" while `main` did not have a line of it. It is checked now, and `AGENTS.md`
  says to merge before tagging.

## [0.10.0] — 2026-09-03

### Added

- **`sets:` on a command outcome: which fields of the entity the branch determines, and from
  what.** The twin of `payload:`, pointed at the subject instead of an emitted event. Before it,
  nothing in the model related a command's input to the entity state it produced, so a generated
  scenario could find the row it had just created and say nothing about what was in it — an
  implementation that stored somebody else's amount passed. Validated like `payload:`: the field
  must exist on the entity, the source must be an input the command takes, and the types must be
  assignable or have a declared conversion.
- **A generated `contains` names the values the row holds, not only which row it is.** Read from
  `sets:` where the source is an input the scenario chose, the types do not cross a conversion, and
  the view projects the field at the entity's own type; left out rather than guessed at otherwise.
  Measured on the normative billing example, which now declares `sets:` on `CreateInvoice`: the
  deliberate negative-total fault was caught by 6 of 29 scenarios before and is caught by 13 after.
  It is a claim about a row this scenario made, so it holds on a target §8 permits to be shared.
- The documentation projection writes a sentence for `sets:` on every outcome that declares one.
- **The billing example declares an invariant over a top-level field of its own** —
  `reminder_count >= 0`, projected by `InvoiceById` so it can be decided. Nothing shipped here had
  that shape: the entity's other invariant is `total.amount >= 0`, which has a dot and parses the
  same either way, so the Go runner's left-operand defect below was invisible to every fixture.
- `cargo xtask schema [--check]` regenerates `schemas/generated/ess.schema.json` from
  `RawSpecFile`, and `task projection-check` runs it. The file was documented as drift-checked and
  was checked by nothing; the schema an adopter validates against had not moved since the model
  gained its last two constructs.
- **`ess conform synthesize --component <name>`: the suite one component can be held to.** A
  specification with two components obliges two implementations, and an implementation of one
  answers `ErrUnsupported` to every scenario about the other — which the runner reports as a skip,
  and a run with skips in it cannot say it passed. The scoped suite holds exactly the scenarios
  whose every command, event and view the component accepts, publishes or owns; the rest are
  printed as `outside:` with what they need, so they are visible somewhere other than by their
  absence. `SuiteProvenance.component` records the scope and is left out of an unscoped suite, so
  every existing suite is the bytes it was. The refusals are the whole system's, untouched.
  Measured on the ACD ↔ backend model: 44 scenarios for the system, of which the `acd` component
  can be held to the ones its own commands, events and views make answerable.
- **The emitted Go runner writes `ess-conformance-report/1`.** `ESS_REPORT_OUT=<file> go test`
  leaves the same closed document the Rust runner writes — specification, digest, implementation,
  status, counts, the non-passing scenarios by status — so `aep artifact evidence --from <file>`
  records a Go implementation's run without anybody typing a count. Before this the Go suite
  produced a `go test` exit status and nothing a workflow system could read. A skipped scenario
  makes the run `inconclusive`: a target that could not answer has not shown the answer. Recording
  a skip means the runner routes every `Skipf` through one helper that sets the status first, which
  is exercised here for the first time — no shipped fixture skips, so nothing before this release
  ran that path at all.

### Fixed

- **The emitted Go runner read the left-hand side of a comparison as a literal.** `Operand::parse`
  is documented for the *right* side — a bare word is a literal there, which is what makes
  `state == Bridged` name an enum variant — and the Go mirror put both sides through it. So an
  entity invariant over a top-level field of its own compared the *word*: `reschedule_count >= 0`
  asked whether the string `"reschedule_count"` is at least zero, which is Unknown and reported as
  a defect in the view, and `any: [{not: state == Bridged}, defined(agent_id)]` evaluated
  `"state" == "Bridged"`, which is false, so the `not` made the implication vacuously true and the
  invariant checked nothing. The Rust runner never had this: `parse_expression` puts the left side
  through `FactPath::new`. Found by the first run of a generated Go suite against an adopter.
- **`ErrUnsupported` from `ExecuteCommand` failed the scenario instead of skipping it.** Every
  other method's sentinel was honoured; this one was compared and then reported as an execution
  error. A command whose actor is the implementation itself has no caller a target can be, and a
  target saying so was being told its implementation is wrong. Every sentinel comparison now goes
  through `errors.Is`, so a wrapped one carrying the reason is recognised too.

### Changed

- `at` is still not synthesised, and the reason is now a different one. The values are known —
  `sets:` says where each row's ranking value came from — but nothing says a view holds only what
  the scenario running it put there, so naming the *first* row would be a claim about another user
  of the target. The gap table records which half is answered.

## [0.9.2] — 2026-09-03

### Added

- `ess build graph` renders the exact validated `ess-build/1` DAG and its independent release-unit
  outputs as deterministic Mermaid source, so adopter documentation can show the graph CI actually
  executes and refuse a hand-maintained diagram that drifts from it.

## [0.9.1] — 2026-09-03

### Fixed

- `ess project buildkit` emits source, cross-stage, and artifact Dockerfile `COPY` instructions in
  valid JSON form, so generated multi-output Dockerfiles are accepted by BuildKit.

## [0.9.0] — 2026-09-03

### Added

- Typed `ess-build/1`, `ess-runtime/1`, `ess-release/1`, `ess-stack/1`,
  `ess-stack-lock/1`, `ess-environment/1`, and `ess-deployment/1` formats establish the deterministic
  lowering seam from semantic systems to independently released Helm deployments. ESS projects
  `BuildKit` and chart inputs, verifies immutable executor evidence, and refuses unresolved
  stage-owned obligations; it still never builds, publishes, or applies anything.
- The `ess` CLI compiles build and runtime IR, verifies releases, resolves stacks from an
  explicit offline catalogue, projects `BuildKit` and Helm files, compiles environment deployments,
  and reports the exact component releases changed between two deployment documents.

## [0.8.0] — 2026-09-03

### Added

- **Physical realizations are typed data without changing `EssIr`.** An adopter-authored
  `ess-realization/1` binds one exact ESS system, version, and semantic digest to resolved
  components and actors, immutable implementation artifacts, typed runtime requirements, and local,
  loopback, or network entrypoints. `ess realization compile` emits deterministic
  `ess-realization-ir/1` and rejects stale locks, unresolved or out-of-subset references,
  incomplete implementation coverage, malformed placeholders, and inline secret arguments.
- `ess realization generate` projects the same resolved IR into a deterministic run-mode guide and
  supports `--check` for committed-document drift.

## [0.7.0] — 2026-09-03

### Fixed

- **A declared `order_by` was asserted against nothing.** Synthesis created at most one instance
  per scenario and a `ranked` expectation passes on fewer than two rows by design, so every
  ordering assertion a suite emitted was a no-op: an author declared an order, saw scenarios
  generated, saw them pass, and was covered by nothing. Scenarios that assert an order now arrange
  a second row through declared command outcomes, differing on the ranking keys. Measured on an
  adopter's model: 11 ordering assertions, 0 of them running against two rows before, 11 after.
  Where a second row cannot be arranged the order is refused (`ESS-SYNTH-014`) and not asserted.

### Changed

- **`ess-conformance/2`.** The expectation vocabulary grew, and the emitted Go runner reports an
  expectation it does not know as a *failed scenario* — a wrong verdict about an implementation,
  caused by the age of the tool reading the suite. A reader that checks the format first refuses
  the document instead. Both `1` and `2` are read: a `1` suite means in `2` exactly what it meant.

### Added

- A `counts` view expectation, asserting how many rows a view holds. Synthesis writes only a floor,
  and only from two up: a target may be shared, so "exactly N" is a claim about every other user of
  it, and "at least one" is what `contains` already says. Both bounds are in the vocabulary.
- An `at` view expectation, asserting what sits at a position. Both runners read it and both refuse
  it on a view that declares no order. **Synthesis does not yet emit it**: nothing in the model
  relates a command's input to the field a view ranks by, so choosing a position would be matching
  on a shared field name. The construct that would license one is named in the gap table.

## [0.6.0] — 2026-09-03

### Changed

- **`--kind site` writes a website, not inputs for one.** It used to emit markdown with frontmatter
  and a sidebar list, leaving an adopter to run a static site generator over it — measured on the
  one that did: `npm ci` plus a webpack build peaking at 3931 MiB, killed twice by a CI runner, to
  render seven pages of prose that were already written. It now writes the pages as HTML with
  navigation, a stylesheet and a diagram renderer beside them, so publishing a specification is one
  command and needs no Node. **An adopter consuming the old markdown output must change.**
- The documentation projection is built as an `ess-docs/1` document and rendered, rather than
  written as markdown directly. The markdown a repository reads is unchanged, byte for byte, and
  the documentation of every example is now pinned and compared to keep it so. The point of the
  layer is that the site renderer reads the same document instead of parsing the markdown back —
  which is what it had to do before, scanning for a heading to recover a title it had just written.
- The site's stylesheet and its vendored Mermaid bundle stay out of the committed projection
  sample. Neither derives from a specification, and the bundle alone is 3.5 MB.

### Added

- `--kind docs-ir` writes the `ess-docs/1` document as JSON, so a presentation layer of your own
  needs nothing from this crate.
- `--kind site` opens on the `README.md` beside the specification, and takes
  `--include <page-id>=<path>` for any number of pages beside the generated ones — a plan board
  another tool rendered, a runbook. Both are markdown somebody wrote, read into the document and
  styled like every other page. Raw HTML in them is dropped rather than passed through: a generated
  site that embedded it would inherit that markdown's scripts and its styling.

## [0.5.1] — 2026-09-03

### Fixed

- **The browser lab runs again.** `CreateInvoice` took an `account_id` from 0.5.0 and the lab's
  script did not send one, so the module refused the first command of the run —
  `{"kind":"undecodable","at":"input.account_id","expected":"a value","found":"nothing"}` — and
  `task site-build` failed after the tag was cut. The script now names the account its invoices
  belong to, which is the caller's to name because `billing.invoice.Account` declares it `owns`
  them `via account_id`.
- The lab read the identity for the next command out of the *first* entity in the catalogue, which
  was the invoice only for as long as the invoice was the only entity. `billing.invoice.Account` is
  declared above it, so `InvoiceCreated`'s `account_id` was taken for the new invoice's id and the
  next command was issued against an account. It now reads the identity of the entity the taken
  outcome `creates:`.
- `website/src/pages/lab/_source.ts`, the lab's committed copy of `examples/billing/domains/invoice.yaml`,
  was not refreshed for 0.5.0, so every line the left panel highlighted was off by the relation's
  own lines. Refreshed — and a stale copy is now a red gate rather than a wrong panel:
  `task site-lab` compares the two.
- `examples/billing-web/smoke.mjs` asserted the pre-relation input list and sent the pre-relation
  input; it is run by no task, which is why nothing caught it. Same fix, and it passes.
- The documentation pages that quote the command were carrying its old shape:
  `specification-to-contracts.md` said three values are chosen where there are now four and quoted
  a JSON Schema "in full" that was missing `account_id`, and `write-a-specification.md` showed the
  payload block without its first line. Both are the generated files again.

## [0.5.0] — 2026-09-03

- **An entity declares what it owns and what it references.** `relations:` on an entity names a
  relation, its kind (`owns` or `references`), the entity at the other end, how many of it there
  are (`one` or `many`), and the field that carries it. It is declared on the source and nowhere
  else: the reverse direction is a lookup, and a second declaration is a second thing to keep in
  step. Previously an ownership was a typed id field on the child plus an invariant somebody
  remembered to write, which is prose to every projection and refusable by nothing.
- `ess validate` refuses five things a relation can get wrong: a target that is not a declared
  entity, a `via` field the carrying entity does not have, a `via` field typed as anything but the
  identity the design requires — `Optional<…>` or `List<…>` where the cardinality says so — a second
  entity claiming to own one that is already owned, and two relations claiming one field. Each is an
  existing `ValidationCode` with a hint naming what to write instead, and each has a test that
  breaks it on purpose.
- **One extension key, `x-ess-relation`, carries a relation into every projection**, on the property
  that carries the field. The JSON Schema projection gains a document per entity,
  `schema/entities/<name>.schema.json`, because an entity was previously rendered by the
  documentation and by nothing a tool reads; the `OpenAPI` projection publishes the same shapes under a root
  extension, `x-ess-entities` — no path, no method, no query parameter — and adds a `$ref` to the
  schema of what the property identifies, which that document has and a self-contained schema
  document does not. An extension rather than `components.schemas` because that table is what
  `ess import openapi` reads back, and an entity's shape reaches a `Map` and a tagged union, which
  the adapter's subset does not carry: publishing them there made this repository's own adapter
  refuse the document it had generated. The synthesised Rust names the relation in the carrying field's doc comment rather than as a
  typed field: the specification describes, and nothing generated here has a store to navigate with.
- `examples/billing` carries one of them — `billing.invoice.Account` owns many
  `billing.invoice.Invoice`, by the invoice's `account_id` — so the pattern an adopter is pointed at
  is validated, compiled and projected rather than described. `CreateInvoice` takes the account and
  `InvoiceCreated` announces it, because an owner nobody named is an owner the implementation
  invented.
- The committed `generated/` tree is regenerated, which also lands 0.4.0's documentation filenames:
  `docs/index.md` and `docs/domains/billing-invoice.md` replace the `README.md` and dotted names the
  tree still carried. `schemas/generated/ess.schema.json` is regenerated from the Rust types, which
  additionally publishes the `order_by:` and quantifier constructs 0.4.0 added and the file had not
  caught up with.

## [0.4.0] — 2026-09-02

- **The documentation projection writes different filenames.** `--kind docs` now writes `index.md`
  rather than `README.md`, and `domains/acd-routing.md` rather than `domains/acd.routing.md`: a dot
  in a path segment makes a static file server read the name as having an extension, so every one
  of those pages was unservable behind GitLab Pages and an adopter carried a rename pass of their
  own. Committed output moves once; the links inside it move with it.
- Add `forall:` and `exists:` to the predicate language, quantifying an invariant over a `List` or
  a `Map`. A collection publishes its size as `<path>.count`, so no existing fact source changes to
  gain them, and one nobody observed evaluates to `unknown` rather than to the vacuous truth an
  empty one gives. Quantifying over anything that is not a collection is refused as
  `type_mismatch`.
- Add `order_by:` to a view, so a view named for a position says something about position.
  Generated conformance scenarios assert it on adjacent rows; a key the view does not project is
  refused, because a rank over a field nobody can read is a promise nothing can check.
- Add `ess conform synthesize --target go`, which writes a Go test package — the runner, a
  three-valued predicate evaluator and the suite — so an implementation in Go is held to the
  specification by `go test`. Previously `conform run` reached only the reference targets in this
  workspace, and an adopter's suite was regenerated on every model change and never executed.
- Add `refs:` to commands, outcomes, bindings and components, written `provider:key`, so a
  construct can name the ticket or incident that explains it. `Conversion.because` was the only
  prose field in the model, and everything else went into YAML comments no projection reads.
- Add `ess generate --kind site`: the documentation with frontmatter and a sidebar, for publishing
  as a static site.
- Print the refusals `conform synthesize` finds instead of counting them. A count says something is
  unchecked without saying what.
- Introduce `ess-docs/1`, an internal document representation between the model and the pages, with
  the markdown projection as one renderer of it. No output changes; the documentation of every
  example is pinned byte for byte and compared.

## [0.3.0] — 2026-09-01

- Add reusable named struct `shape` declarations for views, preserving expanded checked fields in
  compiler IR while OpenAPI reuses one component schema through `$ref`.
- Expose the compiler-owned semantic source digest on `EssIr`, so downstream builders and
  provenance records bind to one canonical implementation.
- Expose the exact versioned browser catalog through `ess_synth::web::browser_catalog`, so service
  generators and documentation hosts consume the same document as the ESS browser target.

## [0.2.1] — 2026-09-01

- Restore the extracted `schema-contract` command surface under `ess schema`: offline validation
  against an explicit schema registry and deterministic TypeScript projection with byte-check mode.
- Document and gate both commands so consumers no longer depend on the retired repository's CLI.

## [0.2.0] — 2026-09-01

- Make `ess-conformance-report/1` the only ESS conformance handoff and remove the legacy
  workflow-shaped evidence, producer, provenance, and verifier API. AEP adaptation now lives only
  in AEP's optional adapter.
- Use the canonical `ess` command in newly generated provenance guidance and current engineering
  documentation. Persisted ESS and infrastructure IR envelopes remain version 1.

## [0.1.1] — 2026-09-01

- Publish the extracted ESS adopter documentation and browser lab at the standalone ESS Pages
  site, with commands, links, and regeneration guidance written for the canonical `ess` CLI.

## [0.1.0] — 2026-09-01

- Extract ESS, infrastructure modeling, schema contracts, generators, conformance, examples, and
  suites into a standalone repository with no AEP dependency.
- Add the canonical `ess` command and explicit import/project adapter contract.
- Add typed OpenAPI service-interface import, deterministic projection, coverage reporting, and
  semantic round-trip checks for the adapter's declared subset.
- Import the Kubernetes credential-edge scanner with pre-write Secret sanitization.
- Publish `ess-conformance-report/1` for optional workflow-side evidence adapters.
