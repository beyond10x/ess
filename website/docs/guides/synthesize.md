---
title: Synthesize code from a specification
sidebar_position: 5
description: What the specification fully determines is generated and what it cannot is an obligation — the synthesis plan, the four targets, the ports you provide, and how the generated code is proven against the generated suite.
---

# Synthesize code from a specification

Synthesis follows one rule: **what the specification fully determines is generated; what it cannot
determine is an obligation.** It generates the part of an implementation that was never yours to
write — types, typestate lifecycles, component ports, one transport, and the behaviour of every
command and the query of every view whose outcome the specification spells out — and hands back
everything it cannot determine as a **named obligation**. Generated behaviour reads and writes
through storage ports. Network-served components also get an ephemeral in-memory implementation
and an executable; durable storage remains yours to provide.

```shell-session
$ ess generate synthesize --path examples/billing --target rust --out out/
```

`--target` is `rust`, `go`, `web` or `clap`; `--out` writes the tree, and without it the artifacts are
listed instead of written, for the same reason `ess generate` behaves that way — a verb
that scatters files over a working tree the first time someone tries it is a verb nobody tries
twice.

## The plan: every capability gets exactly one disposition

Synthesis starts with a language-neutral **synthesis plan**. Every capability of the specification
receives exactly one of three dispositions, with the reason recorded:

| Disposition | Meaning |
|---|---|
| **generated** | the specification determines it fully; the emitter writes it |
| **obligation** | the specification cannot determine it — a decision or an algorithm — so it is named and left to a person, with a declared seam to implement against |
| **refused** | the specification cannot even state what would be needed; the reason is printed |

The command summarizes the plan and the artifact inventory. `PLAN.md` and `plan.json` in the output
carry each disposition and its reason:

```shell-session
$ ess generate synthesize --path examples/billing --target rust | head -2
48 capabilities: 40 generated, 4 obligation(s), 4 refused
16 artifact(s), nothing written
```

A refusal reads the same way and says what it cannot state: *"actor grants
`billing.invoice.Auditor` — … generated as data, not enforced: … a grant is checked against a
caller identity, which types do not carry"*.

The plan is rendered as `PLAN.md` and `plan.json` in every emitted tree, and it is
**language-neutral**. The existing Rust, Go and Web billing example produces the same 48/40/4/4
summary and `plan.json` digest. Clap also carries the plan and reports its grammar-specific
weakenings separately; command behavior remains a handler obligation.

What a target holds more weakly or cannot represent at all is declared in a `TARGET.md` beside the
plan — a named weakening, never a silent downgrade. The Rust target is the one the others are
measured against and emits no such file; Go's names five weakenings and the browser's six, each
with the capabilities it touches. The browser tree's first row is the worked example: it cannot
carry `#![forbid(unsafe_code)]`, because a WebAssembly export is a `#[no_mangle]` item and rustc's
own `unsafe_code` lint flags one. The file says so, states that the crate contains no `unsafe`
block, no `unsafe fn` and no raw-pointer dereference, and a test asserts the property the lint
would have closed. What is lost is the compiler closing the question, not the property.

## The four targets

| Target | Emits | Dependencies |
|---|---|---|
| `rust` | a cargo workspace: semantic types, typestate lifecycles, component ports, generated behaviours and view queries over storage and context ports, one HTTP transport; network components also get an ephemeral store and executable | reusable libraries: none; network servers: `clap`, `uuid` and `time` |
| `go` | a Go module with the same system | standard library only |
| `web` | a WebAssembly bridge over the Rust target plus a page built at load time from an emitted `catalog.json` — no model is typed into its HTML | no build tool, no `wasm-bindgen` |
| `clap` | a command tree, shell completion support and a dispatcher with `Handler` seams for components declaring command-line reach and a CLI grammar | `clap` and `clap_complete` 4 |

Clap emits grammar rather than another type layer. Its handlers receive `clap::ArgMatches`; the
unimplemented handler names the obligation and refuses. The generated dependencies support parsing
and completion. Rust's semantic types and component libraries remain dependency-free. See the
[Clap emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/clap/mod.rs)
and [handler/completion tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/clap.rs).

All four full synthesis targets refuse unsupported modeled Binary64, as covered by the
[feasibility tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/feasibility.rs).
The `rust` target represents `Json` as `json::Value`, a module its types crate carries only when
the model uses `Json`. The generated server re-exports that module, so the wire codecs carry the
value unchanged: members keep their order and numbers keep their spelling. The `go` target
represents `Json` as `primitives.Json`, a wrapper over the document text that its primitives package
carries only when the model uses `Json`; the served surface reads it with its members in order and
its numbers as spelled, as covered by the
[Go Json tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/json_go.rs).
The `web` target's
bridge crate re-exports the same module and carries a `Json` value unchanged; its page holds one
as `JSON.parse` answers it, typed `JsonValue`, so on the page a number is a double and
integer-like member names come first. `TARGET.md` states that limit, as covered by the
[web Json tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/json_web.rs).
That every code target synthesizes a model using `Json` is covered by the
[Json tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/json_primitive.rs).
The `clap` target carries the same `json` module when the model uses `Json`: a `Json` flag
(bare, optional, repeated or through a newtype) takes one argument holding a JSON document, a
malformed one is a usage error, and a handler receives the `json::Value` and prints a `Json`
response unchanged with `json::push_value`, as covered by the
[clap Json tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/json_clap.rs).
This boundary is separate from the [structural data libraries](../reference/cli.md#adopter-owned-schema-contracts).
The [support matrix](../status/where-this-stands.md#support-boundaries) records current-source target
availability independently of the dated release observation.

`examples/gatepass/` is emitted to Rust and Go and deliberately not to the browser: it is a
component whose own words say its callers are not deployed with it, and a surface reached over a
network is one a page would *call* rather than contain.

## One crate instead of a workspace

The `rust` target's default layout, `--layout workspace`, is a workspace of crates: one for the
types, one per component, one for the system and one for the HTTP server, so one component can be
taken alone. An adopter that never deploys a component alone can ask for one crate instead:

```shell-session
$ ess generate synthesize --path examples/gatepass --target rust --layout crate --out out/
```

`Cargo.toml` and `src/lib.rs` land at `--out`, and nothing sits deeper than `src/<module>/<file>`:

| Module | Holds |
|---|---|
| `src/<context>.rs` | one per bounded context, as in the workspace's types crate, beside `primitives` and the `json`, `obligation`, `actor` and `behaviour` modules the model needs |
| `src/ports.rs`, `src/ports/<component>.rs` | each component's port |
| `src/system.rs` | the bindings and the transport |
| `src/server.rs`, `src/server/` | the HTTP surface: `http`, `wire`, `json`, `entry` and one route module per served component |

The HTTP surface is the `server` Cargo feature, off by default. Without it the crate has no
`std::net` in it; with `--features server` it is the same server the workspace builds, and passes
the same conformance suite. A bounded context named `ports`, `system` or `server` becomes
`ports_domain`, `system_domain` or `server_domain` in this layout only. `plan.json` records the
layout in its `scope`, and `PLAN.md` and every file header name `ess synthesize --layout crate` as
the command that rewrites the tree. The `go`, `web` and `clap` targets refuse `--layout crate`
with exit status 2 and write nothing. The workspace layout's bytes are unchanged. See the
[layout tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/single_crate_layout.rs).

## Realizations: the human's half

An **obligation** is implemented in a separate, hand-written crate or module — a *realization* —
that plugs into the generated seams. Three ship here: `examples/billing-realization/`,
`examples/gatepass-realization/` and `examples/gatepass-go-realization/`, one implementation per
obligation in the generated plan, linked into the generated tree. A generated behaviour keeps its
seam, so a realization may still supply one by hand in its place; billing's does, for each
behaviour and query its plan generates. The linker never chooses between candidate
implementations; ambiguity is an error.

## How the output is proven, not assumed

The generated code is judged by the suite the same specification generated
(see [Verify an implementation](./verify-conformance.md)), and the repository's `task check` gate
runs the source-workspace tests and realization checks on every commit. Use the public
`ess generate synthesize` command to write a target and review its output:

* The committed billing suite, **unchanged**, passes the generated workspace linked with the
  hand-written realization — 29 of 29 scenarios — and a deliberately corrupted linkage fails
  exactly the scenario that exists to catch it.
* Generator tests cover the structural contracts and deterministic bytes. The browser-specific
  `task site-build` gate additionally compiles the committed WebAssembly realization and runs a
  Node-driven boundary test that loads the module outside a browser and drives it through the
  page's own glue. Its last line is
  `browser boundary: 21 claims held — catalogue, dispatch, transport, view, refusal, redelivery`
  — the count is derived from the checks the script actually makes, not written down beside them.
  That gate fails when its toolchain is missing; it does not report a skipped check as passing.
* The **dual-target demonstration**: `examples/gatepass/` is synthesized to Rust *and* Go, both
  binaries are started on ephemeral ports, and their startup records, their answers to seven HTTP
  exchanges, and the `/openapi.json` and `/docs` documents they publish are compared. The seven are
  chosen to separate the kinds of "no": a registered visit (202), a visit of no length refused on
  domain grounds (422), two view reads (200), a body the schema refuses (400), an undeclared path
  (404) and a declared path under an undeclared method (405). The two applications must agree with
  each other and with the committed artifacts, byte for byte where bytes are claimed.

## Where the HTTP surface comes from

The gatepass model gained exactly one word to become a running server: a component may declare
`reached_by: network`, which states where its callers are and names no protocol. HTTP follows
because the one contract this project projects for a command surface is an OpenAPI document — the
transport is *derived*, which is the [typed projection boundary](../concepts/overview.md) doing its
job.

The same surface is reachable without a socket. Each served component's module has a public
`handle(system, name, input)` that runs a command or view by its qualified name and returns the
declared outcome as a `json::Value`, or an `entry::Refused`: an unknown name, input the declared
schema refuses (the route's `400`), or an unmet obligation (the route's `501`). The routes and
`handle` call the same decode-run-render function, so a conformance runner or an in-process caller
gets what the HTTP surface answers without writing its own dispatch table.

## Generated behaviour, over ports you provide

A command whose every outcome the specification spells out gets a generated behaviour in the Rust
target's `behaviour` module. It follows the conformance interpreter's semantics and the one
precedence order in
[the guard design](https://github.com/beyond10x/ess/blob/main/docs/design/cross-record-and-stored-field-guards.md):
input-guarded refusals first, then the addressed row and its subject guards, then the accepting and
`external:` branches in declaration order, then the default. It implements the command's existing
`…Behavior` trait on one bundle, `Generated<P>`, so the component ports take it unchanged.

Everything the specification leaves open is a port, and the ports are yours to provide:

| Port | What you provide |
|---|---|
| storage, one trait per entity a generated behaviour or query reads or writes (`InvoiceStorage`) | `get`, `put` and `delete` of a snapshot by identity, and `list` of every stored snapshot |
| `Context`, where a generated behaviour asks it anything | the caller's attributes, every identity and value the specification says the implementation assigns (a created identity, `{generated: true}`), and whether each `external:` branch is taken, given the executing command input as an `ExternalCommand` |

ESS preserves these ports for your implementations. For network-served components it also
generates in-memory stores whose `list` answers in identity order. These stores lose all data
when the process exits. Integer identities sort numerically; text and the text-backed primitives
sort by their stored rendering. Decimal `1` and `1.0` remain distinct identities, as do JSON
numbers with different spellings. Newtypes preserve the underlying comparison. Records compare
fields in declaration order, enum variants by their declared names, unions by tag then payload,
and lists lexicographically. Booleans sort false before true, bytes lexicographically, and each
optional layer absent before present. Maps compare sorted key/value pairs, independently of insertion
order. JSON objects retain their member order; they are not maps in the generated value model.
These comparisons govern lookup, replacement and deletion as well as listing. No numeric
normalization is added, and the existing HTTP decoder bounds and refusals still apply.
Memory-store identity compares decoded model values, rather than native Go pointer identity.
In particular, direct Go `Json` values that differ only in whitespace address the same row;
the zero `Json` value and explicit `null` also address the same row. This is the new store's
key contract; it does not change `Json` constructors or native type equality.
Malformed JSON supplied directly through a Go constructor retains a separate raw-text key,
ordered after valid decoded keys. It neither panics nor aliases a valid JSON value; HTTP still
refuses malformed JSON before it reaches the store.

`P` also supplies every behaviour and query the plan still owes, and `Generated<P>` forwards
them, so it is a complete bundle. To replace one generated behaviour, write a bundle that
implements that trait and delegates the rest to a `Generated`. `PLAN.md` names the same ports in
its **Ports — yours to provide** section.

An error the specification fully determines is generated too. Its fields come from the `payload:`
sources the outcome declares for them (see
[error value sources](./specify/commands-and-outcomes.md#an-errors-fields-can-have-a-declared-source)),
and a field without one is read from the row the refusal is answered for: its field of the same
name and type, or, for a field of the entity's state type, the state it rests in. A request no
declared branch answers, or a guard that is Unknown, is the typed refusal naming the command.

A `when_related:` guard is generated too, through an input reference, an Optional one and a
stored field of the addressed subject. The behaviour reads the related row by identity through the
related entity's storage port, and reads none for an absent reference. The order is the
conformance interpreter's. For an input reference: `existing_instance:`, then a missing row's
`exists: false` branch, then the input-guarded refusals. For a stored reference: the input-guarded
refusals, the addressed row's existence and held state, then the reference as the row held it
before the branch. Where a stored reference is read, or from `ess/22` where `wrong_state:` sits
beside a present-row refusal, the addressed row's existence and held state answer before the
present-row refusals, and those answer before every accepting branch. Otherwise the present-row
predicate branches are read in declaration order with the accepting branches. A `{related:}` value
still keeps its command owed.

A command stays a **whole** obligation when any one outcome uses a construct the generator cannot
express, and the plan names the first one it found. They are:

| Where | Constructs that keep the command an obligation |
|---|---|
| the command | a typed `response:`; `when_subject_state:` beside `external:`; more than one default branch |
| related rows | `when_related:` beside `external:`; a related row of a domain that a component accepting the command does not own (that component has no storage port for it) |
| subject guards | a subject guard with no supplied subject to read, or beside a branch addressing another subject; a subject predicate choosing between a move and an update |
| unknown identity | a supplied subject with neither `unknown_instance:` nor `wrong_state:` to answer an identity no record carries; a `wrong_state:` refusal with fields describing the rows of more than one subject |
| branches | `input_absent:`, `replays:`, `instances:`, `affects:`; a `wrong_state:` or `unknown_instance:` branch that acts or emits, except the creation of create-or-update |
| selection by existence | a creation that `existing_instance:` or a creating `unknown_instance:` decides, whose identity is not read from the input (for create-or-update, a required input field; beside `existing_instance:`, the same input field on every creation) |
| effects | `creates:` leaving a required field unset; a move, update or delete whose identity is observed; `sets:` without a subject |
| values | a declared conversion; a value of another type; `{subject:}` on a branch that holds no row; `{increment:}` with no previous value or on a field that is not an `Integer`; a struct source leaving a required member unset; `{related:}`; `{count: changed}`; a response field; `{cleared}` on an event or error |
| errors | an error field with no `payload:` source that the held row does not determine |
| guards | a path that does not resolve; a read into a union or a collection element (`.count` is read); an ordering over text; a truthiness test of a value that is not a `Boolean`; a comparison of two kinds of value or two literals; the current time |

The Go target generates the same behaviours, with the same order of evaluation, in a
`types/behaviour` package: one storage interface per entity (`InvoiceStorage`, with `Get`, `Put`,
`Delete` and, where a generated query reads it, `List` in an order the store keeps stable), a
`Context` interface asking only what the model asks (`Caller<Attribute>()`, `Generate<Type>()`,
`External(command ExternalCommand, outcome)`, where each `ExternalCommand` wrapper carries
the executing command input), and `Owed`, every behaviour and query the plan still owes. You hand
them to `behaviour.New(behaviour.Ports{…})`, and the `*Generated` it returns has the method of
every seam a component's bundle names, generated or forwarded to `Owed`, so it is a complete bundle
for every component port. Each package's `Unimplemented` stub covers only what the plan owes. Two
seams of one module whose Go method names coincide cannot both be methods of one type: none of
them is, each generated one keeps its seam owed, and `TARGET.md` names the weakening. See the
[generated behaviour tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/declared_behaviour.rs).

## Entity invariants are checked

An entity's `invariants:` are fully determined, so the Rust target generates their check. Each
entity that declares one gets `broken_invariant()` on its data type: the first declared invariant
the value breaks, as the specification spells it, or `None`. A behaviour asks it before storing a
value. An invariant is broken only when it is false. One that reads something absent, such as an
empty `Optional`, a list position past the end, or `state`, which the data type does not hold,
decides nothing, as the conformance interpreter reads it. An invariant the target cannot evaluate
is refused at synthesis by name. The Go target generates the same check as `BrokenInvariant()`,
answering the invariant's text and `true`, or `""` and `false`, over an evaluator in its own
`types/invariant` package; a generated behaviour refuses a write that would break one with the
typed refusal `entity invariant`. See the
[invariant check tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/invariant_check.rs).

## View queries are generated where the rows are determined

A view whose rows the specification fully determines gets a generated query in the Rust target's
`behaviour` module, on the same `Generated` bundle: a projection of the source entity's own fields
and `state`, a `filter:`, an `order_by:`, and an aggregation with `group_by`, `count`,
`count_distinct`, `sum`, `min`, `max` and `avg`. It reads through the storage port's `list`, which
answers every stored row in the order the store keeps them; that is the order an unordered view
answers in. A filter keeps a row where it holds and drops it where it is false or unknown. An
aggregation follows the conformance suite: an absent group key is one group, a `skip_absent`
aggregate skips absent values, and `avg` is rounded half-even to six fractional digits.

A view's query stays an obligation, and the plan names the construct as the reason, when the view
reads a parameter or pages; when its `filter:` uses a guard a command's behaviour could not; when a
field is not one its source holds at that type; when it orders by an optional field, a `Timestamp`,
an enum whose wire spellings are not its variant names, or a value with no order; or when a group
key or an aggregate reads a value the query does not compare, or an aggregate is declared at
another type than it computes. The Go target generates the same queries on its `Generated`, over
the storage interface's `List`, with exact decimal sums and means. See the
[view query tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/generated_view_queries.rs).

## Run a generated network component

A `reached_by: network` component gets `cmd/<component>-server/main.go` in Go and
`crates/<system>-server/src/bin/<component>-server.rs` in the Rust workspace. The Rust single-crate
layout puts the executable under `src/bin/` and requires `--features server`. Rust entries use
clap derive; their runtime dependencies are pinned to clap 4.6.7, uuid 1.26.1 and time 0.3.45,
compatible with Rust 1.85. These dependencies stay outside the reusable types and the default
single-crate feature set, which also build for WebAssembly. Go entries use the standard library.

Each entry accepts `--listen <addr>` (default `127.0.0.1:8080`), `--callers <mode>` (default
`none`) and `--static <dir>`. Port `0` chooses an available port; the ready record reports the
bound address. With `--static`, paths outside the API route table serve files from the selected
directory, including `index.html` at `/`. Filesystem aliases and encoded paths cannot escape
that directory. API routes keep their own method, authorization and error responses. This lets
a generated web app share the server's origin without CORS configuration.

The memory context supplies random v4 UUIDs and timestamps from the system clock. Before opening
the listener, an entry refuses startup if its reachable commands, views or bindings still owe
behavior, caller attributes, external decisions or assigned values of another type. Diagnostics
name the missing answers. Unrelated components' context requirements do not prevent startup.
To supply those answers, link a realization through the existing storage, context and `serve`
callback ports. The generated executable is for ephemeral use: restarting it clears its stores.
Rust's additive `TryContext` returns typed context errors and adapts existing `Context`
implementations automatically. Go's additive `NewWithContext(Ports, FallibleContext)` explicitly
selects the fallible companion ahead of `Ports.Context`. The existing `Ports` fields and
`New(Ports)` constructor remain unchanged. Missing context answers return a named
`UnmetObligation`. Generated commands prepare their outcome and event payloads before committing
storage changes, so an unavailable late context answer leaves stored rows unchanged.

An external decision receives the command input being executed, not only its name: Rust passes
the borrowed `ExternalCommand<'_>` enum, Go a typed `ExternalCommand` wrapper, through `Context`
and the fallible companion alike. Compare it with the request your proof was issued for and
refuse a mismatch. Regenerating changes this signature, so an existing names-only `external`
implementation stops compiling until it takes the command.

## Actor grants are generated as data, and a served surface enforces them

Which actors exist and which commands each may invoke is fully determined, so the Rust target
generates it as data: an `actor` module with an `Actor` enum, one variant per declared actor,
`may(actor)`, the qualified names of the commands that actor may invoke, and `Caller`, who a request
was authenticated as. The module is emitted only for a model that declares an actor.

Where the specification serves a component (`reached_by: network`), the generated server enforces
the grant. Its `dispatch` and `handle` (Rust) and `dispatch` (Go) take the caller your realization
authenticated the request as, or none, and `serve` takes the function that authenticates one. How a
request proves who sent it is yours; the `serve` callback remains available to your realization.
The generated executable defaults to `--callers none`, which authenticates nobody even if a
request carries an actor header. Explicitly selecting `--callers actor-header` enables a
demonstration mode: `Authorization: Actor <name>` names a declared actor by its qualified name
or an unambiguous short name. A request carrying more than one `Authorization` header is
authenticated as nobody. Startup identifies this demonstration mode. It is not production
authentication. Before the
command runs, a caller that is none, or is an actor without the grant, gets one standard refusal,
the same for every command: `403` with `{"refused": "not granted", "actor": <name or null>}`. The
contract declares it on every command. A `403` a caller-decided branch answers carries `outcome` and
the declared `error` instead, so a client tells the two apart by the members present. A command no
declared actor may invoke is refused to every caller. A view is not grant-checked unless an actor's
`may:` names it (below). The plan marks
the `actor grants` row generated. The Go server carries its own grant table, and both servers expose
the check (`admit` in Rust, `Admit` in Go) for code that drives the system in process.

Where the specification serves no component, nothing generated sees a caller, so the plan keeps the
`actor grants` row refused and enforcement stays with the caller. The Go target then emits no grant
table and lists that as a weakening.

For each command a served component accepts, the synthesized suite holds the grant to it:

- `<command>/grant/denied` sends the command as a declared actor the specification does not grant
  it, reusing the arrangement of one of the command's own scenarios whose send the command
  accepts. It requires the refusal, and that the target's event log holds no more of any declared
  event after the send than just before it. Where no scenario sends the command into an accepting
  branch, the denied scenario is withheld and a note names the command. A command every declared
  actor may invoke gets no such scenario, and synthesis says so in a note.
- A command no declared actor may invoke is refused to every caller, so no scenario sends it
  expecting it to run: those are withheld and named in a note, and its refusal is the only witness.
  An authored act sending it with no `actor:` is refused (`ESS-AUTHOR-040`).
- Every actor granted a command sends it at least once: the command's scenarios are sent by its
  granted actors in turn, and `<command>/grant/admitted/<actor>` is added only where it has fewer
  scenarios than granted actors. A command an actor carrying attributes holds keeps the actor
  synthesis chose, and a note names it.

A command no served component accepts gets none of these, and a specification that serves nothing
gets one note saying enforcement is the caller's. A server that skips the check runs the command
instead and fails the denied scenarios. See the
[actor grant tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/actor_grants.rs).

From source `ess/22` an actor's `may:` may also name a view (beyond10x/ess#286). The views it names
are read-granted: each generated server checks the caller before it reads one — `admit_read` and
`Caller::may_read` in Rust, `AdmitRead` and `Caller.MayRead` in Go, with the views each actor may
read in the types crate's `may_read(actor)` and the Go server's read grant table — and answers an
actor the grant does not name, or no actor, with the same standard `403` refusal a command answers.
The contract names the readers of each such view as `x-ess-may-read` and declares the `403` on its
operation. A view no actor names stays open to every caller, and a model that names no view keeps
its generated bytes. The synthesized suite holds the grant to each read-granted view a served
component answers:

- `<view>/grant/read/denied` reads the view as the lowest-named declared actor its grant does not
  name, then as no actor at all, and requires the standard refusal of each read. A view every
  declared actor names is still read as no actor, and a note says so.
- `<view>/grant/read/admitted/<actor>` reads it as each actor naming it, and requires it served.
- Every other read of the view in the suite is sent as the lowest-named actor naming it, with a
  `read_as` step before it.

A server that skips the read check serves the view to everyone and fails the denied scenario. See
the [view grant tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/view_grants.rs).

## Honest limits

* **What the specification cannot determine is not generated.** A decision or an algorithm the
  specification does not spell out is an obligation, and a command with one such outcome is an
  obligation as a whole. Storage remains a port; network-served components also get ephemeral
  generated stores, while durable storage stays with the realization.
* **Obligations are plan entries, not records** a task can own and evidence can close. Nothing
  blocks that extension; it is listed on the [roadmap](../status/roadmap.md#not-scheduled) as not
  scheduled.
* **The demonstration is not a deployment**: plain HTTP, no auth, no TLS, one connection at a time,
  no `servers` block because the model has no URL.
