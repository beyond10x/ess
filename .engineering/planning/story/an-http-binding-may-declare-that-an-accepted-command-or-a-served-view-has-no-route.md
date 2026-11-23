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
revision: 14
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
