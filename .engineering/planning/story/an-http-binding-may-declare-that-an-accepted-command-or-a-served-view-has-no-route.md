---
format: aep.planning-md/3
id: story:an-http-binding-may-declare-that-an-accepted-command-or-a-served-view-has-no-route
kind: story
status: draft
title: An HTTP binding may declare that an accepted command or a served view has no route
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/src/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding_rust.rs
- confidence: inferred
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/compile.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: inferred
  path: website/docs/guides/http-routes.md
revision: 15
---
## Outcome

An `ess-http/1` binding can name commands and views that the served surface does not route. Such an operation stays accepted or projected by its component, so its in-process handler, conformance, docs and diff stay the same. It has no path in the OpenAPI document, and no generated Rust or Go server dispatches it. The document lists it as deliberately not served. A `reached_by: network` component that owns a domain can now accept the commands its own worker issues without publishing a route that nothing serves.

## Need

A downstream specification reported this on ess 0.97.1 (one collected report, three points). It was reproduced with a minimal model of our own: domain `runs.job`, entity `Job`, commands `SubmitJob` (called over HTTP) and `StartJob`, `RecordProgress`, `FinishJob` (issued by a worker, never over HTTP), view `JobById`. Commands and outputs: `http-surface/run.txt` (scratch of the planning session, 2026-11-22).

- The network owner accepts all four commands. `ess generate --path model.yaml --kind openapi --out out` then writes 5 operations, `POST /job/commands/{FinishJob,RecordProgress,StartJob,SubmitJob}` and `GET /job/views/JobById`. Three of these routes are served by nothing.
- The worker commands are accepted by no component. The model is valid and the document has 2 operations. But the docs graph puts the three commands under "owned by no component", and no generated component handles them. Routing one of them in the binding is refused: `error[ESS-HTTP-013]: routes[1].command: no component accepts command `runs.job.StartJob`, so it has no HTTP surface to route`.
- The worker commands are granted only to a `system: true` actor (ess/27). All 5 routes stay, each with `x-ess-may-invoke: [runs.job.Platform]`. This is by decision: `docs/design/system-issued-commands.md:402` says "the flag says who issues the command, not whether it is served".
- A second component accepts the worker commands: `error[ESS-COMPONENT-004]: `job-worker` accepts `runs.job.StartJob`, and `job-api` owns `runs.job``.
- Views have the same gap. Every view of a domain a network component owns derives `GET /{domain}/views/{view}` (`crates/generate/ess-gen/src/http.rs:694-710`), and the binding cannot withdraw it ("an unnamed operation keeps its derived route", `docs/design/http-presentation-binding.md:182,843-844`).

What can be written today, tested: the worker's component owns the record domain and accepts the writes (`in_process`). The API component owns a read domain whose view sources the worker's entity. `ess specify validate` reports `runs v1 — 4 file(s), valid`. The network document has the single operation `GET /status/views/JobById`. The worker's document has 4 command paths and says "no request reaches them". The synthesized suite checks the view (18 references). This shape fails as soon as the network component must also accept one write to the record domain over HTTP, and that is the gap this story closes.

## Design

The binding gets a top-level `unserved:` list. Each entry names exactly one `command:` or `view:`, in the same shape a route uses to name its operation. Optional; when absent, the IR is omitted and its bytes do not change.

- Compile (`ess-http`):
  - An entry naming no operation, naming both, or naming a missing one is `ESS-HTTP-004`.
  - An operation that is both routed and unserved, or unserved twice, is `ESS-HTTP-005`.
  - A command no component accepts, or a view no network component serves, is `ESS-HTTP-013`. Its row is extended so that it also refuses a network component the binding leaves with no served operation, the binding counterpart of `EmptyDeclaration` in `crates/specify/ess-domain/src/component.rs:18`.
  - A refining binding keeps every entry of the binding it refines. It may add an entry only for an operation that the refined binding does not route; anything else is `ESS-HTTP-012`.
  - `ess-http-ir/1` records the list and leaves it out of the derived routes.
- Projections:
  - `ess_gen::http::bound_routes` drops unserved operations, so the OpenAPI document, the Rust and Go servers and `ess-ui-check` agree from one table.
  - OpenAPI loses the path and every schema that only it pointed at. `info.x-ess-unserved` lists the qualified names, so a reader can tell a deliberate absence from a broken generator (the `openapi.rs:38` principle).
- Unchanged:
  - `ess/N` and component ownership.
  - Conformance: the targets are semantic. The HTTP target (stage 3, not built) already refuses by name an operation the binding cannot reach (`http-presentation-binding.md:823-824`).
  - `ess verify diff`, which does not compare bindings (`http-presentation-binding.md:850,924-925`).

Alternatives, one line each:

- Change nothing and answer with the two idioms. This leaves the mixed-writer case on a network owner unexpressible (Need).
- `http: none` on a command or view in `ess/N` (the requester's proposal). Refused: route facts live only in the binding (`decision-blocker:http-surface-scope`, `http-presentation-binding.md:831`).
- A per-component `internal_commands` list in `ess/N` (the requester's proposal). It would be a second reach statement per operation and cost a source format. Whether a deployment serves an operation is a presentation fact that can differ between deployments of one model, which is what the binding is for.
- Derive the route's absence from `system: true`. This reverses `system-issued-commands.md:402`. The system's own scheduler can be a separate deployment that calls over HTTP. It does not cover views, and it needs actors, which turn on grant checks on every served command (`http.rs:107-119`).
- Binding-level "routes only what it names" (the deferred stage-1 answer 2, `http-presentation-binding.md:843`). It would make every author restate every derived route they keep. It stays possible later as an addition.
- Derive routes only for `reached_by: network` components. This is the D137 operator-held question, and it does nothing for a network owner.
- A second accepting component. It is refused for good reason: one handler per command (`component.rs:72-81`).

## Acceptance

- `cargo test -p ess-http --locked --test compile` gains passing cases:
  - an `unserved:` command and view compile, and the IR lists them;
  - `ESS-HTTP-004` for an entry naming both a command and a view;
  - `ESS-HTTP-005` for an operation both routed and unserved;
  - `ESS-HTTP-013` for an unserved command no component accepts, and for a network component left with no served operation;
  - `ESS-HTTP-012` for a refinement that unserves a route of the refined binding.
- A binding with no `unserved:` compiles to the same `ess-http-ir/1` bytes as before: `cargo test -p ess-http --locked --test compile` (existing stage-1 IR pin `catalog-stage-1.ir.json`).
- `cargo test -p ess-gen --locked --test openapi_http_presentation`, new case: for the minimal model above with `unserved: [{command: runs.job.StartJob}, {command: runs.job.RecordProgress}, {command: runs.job.FinishJob}]`, the network document has exactly 2 operations. It has no schema that only those commands used, and `info.x-ess-unserved` names the three. `every_schema_the_document_declares_is_pointed_at_by_something` (`crates/generate/ess-gen/tests/openapi.rs:389`) holds for it.
- Every document generated without `--http`, and with a binding that has no `unserved:`, keeps its bytes: `cargo test -p ess-gen --locked --test openapi_http a_stage_1_binding_keeps_its_openapi_bytes`, `cargo xtask generate --check`.
- The generated Rust and Go servers given the same binding answer 404 on `POST /job/commands/StartJob` and still serve `SubmitJob` and the view. The component's in-process port still handles `StartJob`. The test is a new case beside `crates/generate/ess-synth/tests/http_binding_rust.rs` and its Go counterpart.
- `ess verify conform synthesize` writes the same suite bytes with and without the binding (conformance is semantic).
- `docs/design/http-presentation-binding.md` (operations section, the code table, answer 2 of stage 1) and `website/docs/guides/http-routes.md` state the key, and `CHANGELOG.md` names it; `task test-xtask` passes.

## Fit review

1. **Need.** A network component that owns a domain cannot accept a command that only callers deployed with it issue without publishing an HTTP route for it. A view it projects cannot be kept off the served surface either. The reproduction with a minimal model of our own is under Need (`run.txt`). The requester proposed, as theirs: `http: none` on the command or route, or a per-component `internal_commands`, and/or letting a second component accept writes to a domain it does not own.
2. **Class.** The three points take different classes:
   - Point 1 is a **gap** for the mixed case. The binding cannot withdraw a derived route (`http-presentation-binding.md:182,843-844`, "A binding cannot require that it routes every operation in stage 1"). The pure case, where the API writes nothing, is a **convenience**: the read-domain idiom expresses it.
   - Point 2 is a **convenience**, declined. COMPONENT-004 protects §9's one handler per command, which a binding's `invoke.command` needs in order to resolve to one destination (`crates/specify/ess-domain/src/component.rs:72-81`). The idiom covers "written by a worker, read over HTTP" (Need).
   - Point 3, "an `in_process` component gets HTTP operations", is **not a defect** but the documented meaning. `Reach::Network` is "the declaration that makes the published `OpenAPI` document load-bearing rather than documentary" (`component.rs:256-260`). Every component gets a document whose non-network paths are documentary (`crates/generate/ess-gen/src/openapi.rs:39`, `website/docs/concepts/ess.md:58`, `website/docs/guides/generate-artifacts.md:27`). Each non-network document says so in its first paragraph ("It is not declared `reached_by: network`: its callers are deployed with it, so these paths describe its commands and no request reaches them", reproduced), and only network documents carry `info.x-ess-reached-by: network`. This was decided in D137 (`story:a-component-not-reached-over-the-network-has-no-http-route`, implemented in 0.71.0). Removing those routes is held there as an operator question that has not been raised.
   - Reproduced counts for 4 accepted commands and 1 view: `network` 5 operations, unstated 4, `in_process` 4, `command_line` 4, `in_process` accepting none 0. `reached_by` takes exactly these three values (`component.rs:278`).
3. **Already expressible?**
   - These were tried on 0.97.1 (`run.txt`):
     - leave the command unaccepted: valid, but the handler edge is lost;
     - `system: true`: routes stay, by decision;
     - a second acceptor: `ESS-COMPONENT-004`;
     - the read-domain idiom: valid, and the view stays checked by the suite, so the idiom passes the policy-1 condition (`docs/design/generic-language-features.md` § Policy);
     - an exclusion in `http.yaml`: no such key; routing an unaccepted command is `ESS-HTTP-013`;
     - a `cli:` placement: command-line only.
   - Not expressible: a network owner that serves one write over HTTP and accepts others from its own worker.
   - Also found, and not to be used as an idiom: a command declared in another domain may `moves:`/`updates:` the record domain's entity. `ess specify validate` admits it (`m7-cross-domain-write`, valid), so two components write one domain. That contradicts `component.rs:74-76` ("the handler is the component that owns the domain whose state the command changes") and the sibling refusal of a cross-domain delete (`crates/specify/ess-domain/src/command/set_effects.rs:2636-2665`). It is reported separately (Out of scope).
4. **Fit.**
   - Not a cell of the language grids (`docs/design/language-grids.md` covers reads and operations, not presentation).
   - Placement and vocabulary:
     - It lives where every HTTP-surface fact lives, the binding, so `ess/N` keeps refusing `http:` (`http-presentation-binding.md:831`).
     - It names operations exactly as a route does (`command:`/`view:`).
     - It reuses codes 004, 005, 012 and 013 and adds no new code.
   - Siblings:
     - Commands and views are both covered.
     - The command-line sibling has the same question: an accepted command with no place in the `cli:` tree is `ESS-COMPONENT-005` (reproduced). That tree is authored in `ess/N`, so opening it costs a source format, and no adopter has the case. It is not opened here, for that reason.
   - Targets:
     - OpenAPI, the Rust and Go servers and `ess-ui-check` all read `bound_routes`.
     - The interpreter, synthesis, conformance, entity runtime and `ess verify diff` do not read bindings, and stay unchanged.
     - The TypeScript, web and clap targets refuse `--http` already (`http-presentation-binding.md:836`).
5. **Second adopter.** A ticketing service is reached over the network. Agents create tickets over HTTP, while its own escalation scheduler, deployed with it, issues `EscalateTicket` and `ExpireTicket`, and an internal backlog view feeds that scheduler. The public binding leaves both commands and the view unserved. An admin deployment's binding of the same model serves them.
6. **Cost.**
   - One additive `ess-http/1` key with no format bump: D137 added `operation_id`/`servers` the same way, and an older reader refuses the key by name.
   - `ess-http-ir/1` gains an optional member, which is omitted when empty.
   - No new code. The rows of `ESS-HTTP-005`, `012` and `013` are extended.
   - No `ess/N` version and no migration.
   - OpenAPI bytes move only for a binding that uses the key. That change is breaking for a consumer of that one served document, by the author's choice.
   - Crates: `ess-http`, `ess-gen`, and `ess-synth` (through `bound_routes`); `ess-ui-check` is inferred. Docs: the design page, the guide, `CHANGELOG.md`.
7. **Considered.** Change nothing; the requester's `http: none` (refused location); a per-component `internal_commands` (a source format for a presentation fact); deriving from `system: true` (reverses a decision and conflates who issues with how it is reached); a binding that routes only what it names (heavier); routes only for network components (D137's held question, and it does nothing for a network owner); a second acceptor (breaks one handler per command). The chosen design wins because it adds no source surface, covers commands and views, and keeps every non-HTTP target unchanged. The requester's need is taken; both proposed shapes are moved into the binding.

## Decisions

Decision kind: **accept, redesigned**, by the planning session on 2026-11-22.

- Point 1: accepted, redesigned. The requester's `http: none` on a command and their per-component `internal_commands` become an `unserved:` list in the `ess-http/1` binding, because route facts live only in the binding (`decision-blocker:http-surface-scope`). Format: none, neither `ess/N` nor `ess-http/2`.
- Point 2: declined, with the idiom. The worker's component owns the record domain and accepts the writes, and the API component owns a read domain whose view sources that entity (tested, `run.txt`). If the API must also accept a write to that domain, it owns the domain and the binding leaves the worker's commands unserved (point 1). `ESS-COMPONENT-004` stays.
- Point 3: declined, with the idiom. It is the documented meaning (D137). Select served documents by `info.x-ess-reached-by: network`. The operator-held question in D137 is not reopened here.
- Draft only: no wave and no release ordered.

## Out of scope

- Removing the documentary command routes of non-network components (D137, operator-held).
- The `cli:` sibling: an accepted command with no place in the command tree.
- A binding comparison in `ess verify diff` (the stage-2 item without a unit, `http-presentation-binding.md:924-925`).
- The HTTP conformance target (stage 3).
- Admitting a second accepting component.
- The cross-domain write finding (fit review 3): whether `moves:`/`updates:`/`creates:` on an entity another domain owns should be refused like a cross-domain delete is a separate assessment for the coordinator.

## Scope

- inferred: crates/specify/ess-http/src/lib.rs
- inferred: crates/specify/ess-http/tests/compile.rs
- cited: crates/generate/ess-gen/src/http.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- cited: crates/generate/ess-synth/src/rust/http.rs
- cited: crates/generate/ess-synth/src/go/http.rs
- inferred: crates/generate/ess-synth/tests/http_binding_rust.rs
- inferred: crates/ui/ess-ui-check/src/model.rs
- inferred: crates/edge/ess-cli/src/http.rs
- cited: docs/design/http-presentation-binding.md
- inferred: website/docs/guides/http-routes.md
- inferred: CHANGELOG.md

## Second request: an interface that exports a named subset of a component

### Need

A downstream specification of reusable building blocks reported this on 0.99.2 (reproduced on 0.99.9). An architecture exports over HTTP only some commands and views of a component and calls that subset an interface. A component can have several interfaces, one per audience. ESS has no way to say "this document serves these operations of the component and no others". The downstream keeps every path open in a protocol binding, projects the OpenAPI document of the whole component without a binding, and cuts it down to the interface's operations and the schemas they reference. It asked for an OpenAPI projection and a binding validation limited to a named list of one component's operations.

Reproduction, `.engineering/repro/an-http-binding-may-declare-that-an-accepted-command-or-a-served-view-has-no-route/`: one component `shop` reached by network, accepting `ListItem` and `RetireItem`, with views `Items` and `ItemById`. `binding.yaml` routes `RetireItem` (post) and `Items` (get), both with no path.

- `ess specify http validate --path . --binding binding.yaml` answers `binding.yaml — 2 route(s), 2 open, valid`. The binding is not refused at validation, so a binding that names a subset is already valid.
- `ess generate --path . --kind openapi --http binding.yaml --out <dir>` exits 1 with `binding.yaml cannot be projected to OpenAPI:` and, for `repro.shop.Items` and for `repro.shop.RetireItem`, `has an open route: the binding leaves its path to each server, and a published document needs one; fill it in with a binding that refines this one`. The text carries no `ESS-HTTP` code: the refusal is the projection's, as the design page says (`docs/design/http-presentation-binding.md`, the OpenAPI row of "What generated artifacts change").
- With both paths filled in, validation answers `2 route(s), 0 open, valid`, and the document holds four operations: the two named, plus `ListItem` at `/shop/commands/ListItem` and `ItemById` at `/shop/views/ItemById`. This is the design's rule: "An operation the binding does not name keeps its derived route". A binding cannot select a subset of a component's surface.
- The whole-component projection with no binding holds the four derived routes, which is what the downstream cuts down.

The requester's wording, "the binding refuses to leave the other commands' paths open", is not what runs: the other operations are not required to have a path, they keep a derived one. The need is the same as the story's first request: the binding cannot withdraw a derived route.

### Acceptance

- A binding that names the operations an interface exports, and declares that it names all of them, projects an OpenAPI document with exactly those operations and no other. `ListItem` and `ItemById` of the repro have no path in the document.
- The document declares no schema that only an omitted operation pointed at. `every_schema_the_document_declares_is_pointed_at_by_something` (`crates/generate/ess-gen/tests/openapi.rs`) holds for it.
- `ess specify http validate` on that binding requires nothing of the operations it does not name: it reports the named routes only, and a newly accepted command of the component changes neither the verdict nor the document.
- A refinement of that binding keeps the closed set: a route added for an operation the refined binding does not name, or a dropped closure, is `ESS-HTTP-012`.
- A binding that does not declare it keeps its bytes, and so does every document generated without `--http`.
- The generated Rust and Go servers given the same binding answer 404 for an omitted operation, as the first request's acceptance says.
- Named test: a new case in `crates/generate/ess-gen/tests/openapi_http_presentation.rs` on the repro above (2 operations, no `ListItem` or `ItemById` schema that only they used) and one in `crates/specify/ess-http/tests/compile.rs` for the refinement. Neither runs a synthesis.

### Fit review

1. **Need.** An architecture publishes less than a component accepts. Today the only document that ESS publishes for a network component is the whole accepted set, so the interface is cut by a tool outside ESS, which must also understand which schemas OpenAPI references. The requester's proposal is the wording of the need (an interface as a named list of one component's operations); its spelling is not taken.
2. **Class.** A **gap**, the same one as the first request: the binding cannot withdraw a derived route. The first request's cause (a worker's commands that nothing serves) and this one (a subset per audience) differ in who decides: here the audience, not the component, so one component can have several interfaces over one model. That points at the binding, one per interface.
3. **Already expressible?** Not with a released version. With the first request's `unserved:` list, an interface is the component minus the list: every operation outside the interface is listed. It would express the repro, but it fails open: a command added to the component is exported by every interface until someone lists it there. For an export, the closed form is what the requester states (a named list).
4. **Fit.**
   - `ess-composition` selects one component per service key and pins its system by digest. The component's `accepts` surface it selects is the whole set (`docs/design/ess-evolution/service-contract.md`, `operations()` is exactly `component.accepts`). A composition relates systems (types, roles, conformance) and is read by neither `ess-http` nor `ess-gen`, so putting the list there would make the OpenAPI projection read a second document and pin a presentation fact to a system relation.
   - A named interface on the component in `ess/N` would cost a source format and put a per-audience fact into the model that the binding exists to hold (`decision-blocker:http-surface-scope`; the first request refused `http:` and `internal_commands` for the same reason).
   - The binding already holds the list: the repro's `routes:` names exactly the interface's two operations. What it lacks is the statement that the routes are the whole served set.
5. **Second adopter.** The first request's ticketing service: its public binding and an admin deployment's binding serve different subsets of one component. A closed binding states each as the routes it names.
6. **Cost.** One additive `ess-http/1` key, no format bump (an older reader refuses it by name, as for `operation_id`); an optional member of `ess-http-ir/1`, omitted when absent; `bound_routes` drops what a closed binding does not name, so OpenAPI, both servers and `ess-ui-check` agree from one table; rows of `ESS-HTTP-012` and `ESS-HTTP-013` extended, no new code. It shares every file of the first request, so the two ship together or one after the other, never in parallel.
7. **Considered.** The requester's tool-side cut (kept as the workaround until a release carries the key); `unserved:` only (fails open); a list in the composition; a list on the component in `ess/N`; a derived path for an open route (D3).

### Decisions

Decision kind: **accept, redesigned**, recommended by the planning session on 2026-11-28; the picks are open.

- **IF-D1, where the list lives.**
  - A (recommended): the `ess-http/1` binding. The list is the binding's `routes:`, one binding per interface. No source format, no second document read by the projection.
  - B: the composition. It selects the component and could name the operations of that selection. `ess generate --kind openapi` does not read compositions, a composition is for relating systems, and the document would depend on two files.
  - C: the component, in `ess/N`. A new source format and a presentation fact in the model.
- **IF-D2, the form of the list.**
  - A (recommended): the binding says that its routes are the whole served set, so an operation it does not name is not served. It fails closed: a command added to the component is exported by no interface until a binding names it.
  - B: an `unserved:` list only, the first request's design. An interface is the component minus the list. It fails open.
  - C: both keys, refused together in one document. More vocabulary for no case the closed form does not cover.
- **IF-D3, an open route in a projection.**
  - A (recommended): stays refused as today. The interface's document is projected through a server binding that refines the interface's binding and fills the paths. No new surface; the repro's filled form shows it works once the subset is selected.
  - B: an open route takes its derived path when the binding is closed. A projection from a protocol binding would then work, but it reverses "a published document needs a path" and the meaning of "open".
  - C: a flag of `ess generate` that takes derived paths for open routes. A second place that decides it.
- **IF-D4, the name.** A key spelled for what it says about the routes (recommended; the spelling is chosen in the pull request and named in its line, as `BA-D1` asks). B: `interface` as a word in ESS. ESS would own a term of the architecture; the binding stays the one fact.

If IF-D1 is A and IF-D2 is A, the first request's `unserved:` list and this key share a design and a pull request. A pull request that carries either key names the pick in its line.

### Scope

Same files as the first request (the Scope list above). Added in this section: `crates/generate/ess-gen/tests/openapi.rs` as the schema-use check it names; no new crate. Out of scope here: a named interface in `ess/N`, a composition that selects operations, an HTTP conformance target, an `ess verify diff` of bindings.
