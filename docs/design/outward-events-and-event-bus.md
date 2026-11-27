# Outward events, an event-bus contract, routing, transports and ingress

Status: design, 2026-11-26. The owner decided the seven forks the first draft held; they are
listed under "Decided" at the end. Nothing on this page is built. Planning:
`epic:outward-events-and-an-event-bus-contract`.

## Need

The owner, 2026-11-26 about 12:32 CET (relayed): "there should be the concept of outward events
each system can declare (ess) - when an eventbus is configured (contract) - then other system can
receive these events, this eventbus needs to be implemented by something then, so there are
probably multiple enterprise patterns (think camel) - here maybe its pub/sub or fan out [...] we
want to support multiple patterns later [...] then most likely in our current setup we wire this
together in [the operator console], eg inject some conform implementation (nats, unix-socket, etc)
<- i mean this is just the transport ... the other is the messaging semantics ... similar to ess
protospec thing where we seperate the exchange patterns/logic from the transport".

About 12:35 CET: "this eventbus could have a simple routing logic - if (condition) forward ->
sink_x".

About 12:45 and 12:46 CET (relayed, not word for word): one producer emits periodic aggregated
statistics (requests, tokens, latency and cost per route, not one event per token) to a configured
bus endpoint; the producers of the lower layer all emit through the bus, and one consumer shows
them on one architecture page.

About 12:47 CET: "the b10x rfc based or conformance based "inputs" ingresses? could also include
mcp,a2a,etec ....)".

Four questions follow: how a system declares the events it offers to other systems; what a bus
promises about them, apart from the wire that carries them; how a route forwards an event to a
sink under a condition; and how a system declares the protocols it accepts input on.

## What exists

In this repository (`origin/main` `14248d409e`):

- **A component publishes events.** `publishes:` lists them
  (`crates/specify/ess-domain/src/component.rs:135-137`, IR `:474-475`); an event nothing declares
  is refused (`:14`). Nothing says whether an event is internal to the system or offered to
  others.
- **A binding reacts to an event.** "A binding is `on event → invoke command`"
  (`crates/specify/ess-domain/src/binding.rs:1-6`), with a cause that is an event, an event an
  external channel delivers, or a period (`binding.rs:173-188`). Its `where:` is a finite typed
  predicate over the event payload (`binding.rs:317-320`); it is three-valued: "True invokes,
  False skips this binding occurrence and nothing else, Unknown is an unmet obligation that
  invokes nothing" (`crates/specify/ess-domain/src/binding/condition.rs:28-33`). That is a
  content-based router inside one system, with a command as its only sink. The delivery words
  `at_most_once` and `at_least_once` say who carries the risk (`docs/design/binding-delivery-guarantees.md:9-12`).
- **A periodic cause.** A binding may run a command every period, with a host contract and no
  fabricated event (`docs/design/periodic-binding-triggers.md:5-9`).
- **Aggregate views.** Counts, sums and extremes over one entity's rows
  (`docs/design/aggregate-views.md:1`).
- **An event's wire is a sibling document.** `ess-transport/1` binds the events of one exact
  specification to a broker; "An event is a fact; a channel is one way to carry it"
  (`docs/design/event-transport-binding.md:7`, decision `:19-24`). Its protocol set is closed at
  `nats` (`:59`; `crates/specify/ess-transport/src/lib.rs:46-52`); `ess-transport/2` adds subject
  tokens from the payload (`docs/design/parameterized-event-channel-addresses.md`). A consumer-side
  binding and brokers other than NATS are not decided (`event-transport-binding.md:104-109`).
- **Generated publishers** write one library per component behind a `Transport` seam
  (`docs/design/event-publishers.md:15-50`); consumer clients are not decided (`:79`).
- **Semantics apart from the wire.** The accepted record
  `architecture-decision-record:messages-are-exchanged-independent-of-the-wire` states an exchange
  with no wire word (§ 1, "The layer carries no wire word", line 91) and lets WebSocket, broker,
  HTTP upgrade and JSON-RPC bind it side by side, each its own typed format (§ 2, lines 93-110;
  § 4, lines 134-136). Every event is already a one-message exchange with no initiator message
  (lines 87-89). Conformance synthesizes from the layer, and a target adapter per binding carries
  the scenarios over its wire (lines 123-125).
- **`ess-protospec/1`** is a protocol sidecar of participants, bounded channels (ordering, loss,
  duplication), messages, timers and safety properties, checked by trace replay and bounded
  exploration, with model runs and target runs labelled apart (`docs/design/protocol-specification.md:5-9`,
  `:15`, `:21-25`, `:44-54`). This is the separation the owner names.
- **The broker-subject story** (`story:a-command-may-be-bound-to-a-broker-subject-as-a-request-with-one-reply`,
  draft): "Request/reply is a command and its response, and the broker is only the transport";
  correlation, timeout and no-responders live in the protocol document, the subject in the
  binding, in `ess-transport/3`. `story:a-broker-binding-carries-an-exchange` (draft) opens the
  same `/3`. `story:an-event-channel-may-be-bound-to-server-sent-events` (active) binds a
  component's events to an HTTP route with frames stated by a protocol document.
- **The component steps story** (`story:a-component-declares-level-and-time-steps-that-validate-checks-and-the-interpreter-runs`,
  draft, filed 2026-11-26): a time step, a command run every period, with its declaration still a
  fork (D1).
- No bus, no fan-out or recipient list, no aggregate pattern and no unix-socket transport exist in
  ESS (searched `docs/design/` and the planning store for `fan-out`, `pub/sub`, `event bus`,
  `unix socket`).

Elsewhere, public:

- **Communication topologies in the agent-session specification** (`beyond10x/agentspec`, main
  `847d8f8`): not found. Searched its `spec/`, `design/`, `docs/`, `queries/` and every branch for
  topology, pub/sub, fan-out, router and broadcast. What exists: a generated system whose "transport
  is derived from the specification, not chosen: [...] a pump delivers each event to every binding
  that reacts to it" (`generated/crates/sessionspec-system/src/lib.rs:7-9`), and `fanout` as a
  session-analysis query, not a messaging pattern (`queries/session.sq:34`).
- **RFC-derived specifications** (`beyond10x/rfcspec`, `f5e0124`): an RFC becomes a requirement
  catalogue, ESS documents, a wire sidecar, a generated core and a conformance suite
  (`README.md:3-6`). Each profile states axes per layer, among them `exchange`
  (`request-response`, `request-stream`, `session`, `publish`, `one-shot`), `delivery` (mapped to
  protospec channel ordering, loss and duplication) and `layering` (`over <profile>`)
  (`docs/design/rfcspec.md:239-251`). Profiles exist for four RFCs (868, 6455, 1035, 5905); no
  profile for HTTP, MCP, A2A, NATS or SSE was found.
- **Trigger rules** (`beyond10x/canon`, main `566ad6856`, `docs/design/trigger-rules.md`): "entities
  of kind K in state S where X holds start protocol P" (§ 1, line 12); `where` is one
  `canon-expr/1` expression over the row's declared fields (§ 4, lines 103-121); a row whose field
  is missing or mistyped gets the verdict `undetermined`, "UNKNOWN is not FALSE" (lines 130-139,
  190). The evaluation is a pure plan; Canon starts nothing (line 18).
- **Protocol ports from RFCs**: an unpublished design of the same day writes one
  `ess-protospec/1` port per wire protocol from its RFC sections, each MUST and SHOULD a vector row,
  with the transport (TCP, TLS, WebSocket, QUIC) an attribute of the entrypoint, apart from the
  protocol. Not citable here; the pull request names it.

The producers' present feeds:

| producer | feed today |
|---|---|
| `beyond10x/runbook` (`b634e080`) | three event views; the same path with `Accept: text/event-stream` is server-sent events with `id:` the position and `event:` the kind (`runbook/src/api/sse.rs:1-11`); a row is `{event_id, run_id, session_id?, sequence, position, kind, record, at}` (`runbook/src/state.rs:147-156`); `kind` is open text |
| `beyond10x/slot` (`3a0b985`) | `/api/events` server-sent events of `Job` on each state change and `Tick` once a second with pool size, limit and headroom (`crates/slot/src/types.rs:318-338`), a periodic statistics event already |
| `beyond10x/llm-gateway` (`0766824cd`) | `/dashboard/events` sends the dashboard snapshot as one server-sent event at once and after every change: calls, sessions, per-route totals, token rates, cache hit rate (`docs/gateway.md:1536-1560`); a hosted GPU backend and a local model server sit behind it |
| `beyond10x/cortex` (`f01fd55`) | no event feed; the one stream is the answer of `POST /ask` (`src/ask/serve.rs:1-6`) |
| `beyond10x/thalamus` (`8bbdef6`) | a perception layer that grades readings into verdicts (`README.md:5-9`); its feed was not read; later |

The first consumer, an operator console, reads one producer's server-sent events through one
subscriber seam today and polls another; it reads nothing from the other three.

## 1. A system declares its outward events

An **outward event** is a published event the system offers to other systems. It is an event as
today: one declared payload type, published by a component. Being outward adds a promise to
readers the system does not know: its name and payload are a contract, so `ess verify diff`
reports removing it or changing its payload incompatibly as breaking, and every bus contract may
carry it. A published event that is not outward stays the system's own; a bus contract that names
it is refused.

- **Relation to events.** Nothing about the event changes; outwardness is one more fact about it,
  declared on the component's `publishes:` in the specification itself, at the next `ess/N`.
- **Relation to views.** A view answers when asked; an outward event is pushed. A periodic
  aggregate (§ 2) is where the two meet: the answer of an aggregate view, published every period
  as an outward event.
- **Relation to bindings.** A binding of the same system may still react to an outward event;
  outwardness does not route anything.
- **Relation to the broker-subject story.** That story binds a command to a broker subject as a
  request with one reply. An outward event is a one-message exchange with no initiator message
  (the exchange record, lines 87-89), so it needs no request half. Both reuse the same split: the
  wire address (a subject, a socket path) is the transport binding's; correlation, order and
  timers are a protocol's. The bus is bound by its own format (§ 4), so an outward event adds
  nothing to `ess-transport`; the broker-subject story keeps `ess-transport/3` to itself.

```yaml
components:
  - name: gateway.Router
    publishes:
      events: [gateway.calls.CallFinished, gateway.usage.RouteStats, gateway.internal.Retried]
      outward: [gateway.calls.CallFinished, gateway.usage.RouteStats]
```

## 2. The event-bus contract: messaging semantics

A **bus contract** says, for outward events of one or more systems, who receives what and with
which promise. It names no subject, no socket and no host. It is to a bus what an exchange is to
a conversation: the transport binding (§ 4) maps it to a wire, and several bindings of one
contract may coexist.

The pattern catalog, named after the enterprise integration patterns. The first four are designed
here; the rest are named so later stories add them one at a time, each by its own decision:

| pattern | promise | first |
|---|---|---|
| publish-subscribe channel | every subscriber attached to the channel receives each event published on it; subscribers come and go without the producer knowing them | yes |
| fan-out (recipient list) | the contract names the sinks; each named sink receives a copy of each event; a sink that is down does not hold back the others | yes |
| content-based router | each route has a condition over the event and a sink; an event reaches the sinks of the routes whose condition is True (§ 3) | yes |
| periodic aggregate | once per period the producer publishes one event whose payload is an aggregate over the period (counts, sums, extremes per group), never one event per occurrence | yes |
| point-to-point with competing consumers, message filter, splitter, correlating aggregator, resequencer, wire tap, dead-letter channel, idempotent receiver, claim check, scatter-gather, throttler | named for later | no |

Request and reply is not in the catalog: it is a command, and the broker-subject story binds it.

Every channel carries a delivery word with the meaning bindings give it
(`binding-delivery-guarantees.md:9-12`): `at_most_once` (a lost copy is allowed, a duplicate is
not) or `at_least_once` (a duplicate is allowed, the receiver must survive it). "Exactly once" is
not a word here either.

```yaml
type: ess-bus/1
name: platform
systems:
  - {system: pool, version: v1, source_digest: "sha256:<hex>"}
  - {system: gateway, version: v1, source_digest: "sha256:<hex>"}
  - {system: runs, version: v1, source_digest: "sha256:<hex>"}
channels:
  - name: activity
    pattern: publish_subscribe
    events: [pool.jobs.JobChanged, runs.runs.RunEvent, gateway.calls.CallFinished]
    delivery: at_most_once
  - name: stats
    pattern: publish_subscribe
    events: [gateway.usage.RouteStats, pool.slots.PoolStats]
    delivery: at_most_once
  - name: audit
    pattern: fan_out
    events: [runs.runs.RunEvent]
    sinks: [{system: archive, input: archive.records.Store}]
    delivery: at_least_once
```

**Periodic aggregate.** The producer states the period and the aggregate; the bus carries the
result like any other event. The outward event is declared as the answer of an aggregate view the
system already has, sampled every period. The view is windowed: it aggregates what happened since
the last emission, and the interpreter samples it on logical time.

```yaml
outward:
  - event: gateway.usage.RouteStats
    every: PT60S
    from_view: gateway.usage.RouteTotalsSinceLast   # requests, input and output tokens,
                                                    # latency extremes, cost, grouped by route
```

The payload type is the view's row type, a list of one row per group; a period with no rows
publishes an empty list, so a consumer can tell "nothing happened" from "the producer is gone".

**What compiling a bus contract checks:** every pinned specification resolves; every event a
channel names is outward in its system; a channel's pattern is one of the catalog's built members;
a fan-out names at least one sink and every sink is a declared input of a pinned system whose input
type the event can fill; names are unique; output is a deterministic `ess-bus-ir/1`.

## 3. Routing

A **route** is a condition over one event and a sink:

```yaml
routes:
  - name: slow-hosted-routes
    from: stats
    event: gateway.usage.RouteStats
    where: [event.backend == hosted, event.p95_latency_ms > 5000]
    to: {system: alerts, input: alerts.pages.Raise}
```

- An event reaches the sinks of every route on its channel whose condition is True. False skips
  the route. Unknown (a comparison reading an absent field) routes nothing and is counted as
  undetermined on the route, never silently dropped. That is the binding condition's rule today
  (`condition.rs:28-33`) and Canon's `undetermined` verdict.
- A sink is closed to three kinds: a channel of the same contract, a declared input of a pinned
  system (an input is § 5's: a command it accepts over a declared ingress), and the contract's
  dead-letter channel once that pattern exists.
- **The condition language** is ESS's binding `where:` fragment, extended by numeric comparison
  and list membership. ESS's binding `where:` and Canon's `where` are both three-valued, both typed
  against declared fields, both refuse an undeclared field. They differ in reach: the ESS fragment
  compares String, enum and Boolean leaves for equality and tests `defined`
  (`condition.rs:22-26`); `canon-expr/1` adds numeric comparison, list literals and `contains`
  (trigger-rules.md lines 103-121), and a statistics route needs numeric comparison. ESS keeps one
  typed language against its own payload types. A stated translation of the shared fragment into
  `canon-expr/1`, with a test per operator, lets Canon's evaluator read a route's condition
  without a second parser.
- **"Start protocol P" is not a kind of sink.** A trigger rule is about the state of an entity row
  in a snapshot, with owner, mode, cap and refire policy (trigger-rules.md § 3); a route is about
  one event in flight. A trigger evaluator is one more system with a declared input; a route whose
  sink is that input hands it the event, and the rule still decides whether P starts. The bus
  learns nothing of Canon's protocol identity, mode, cap or refire.

## 4. Transport bindings that conform

A **bus binding** maps the channels of one bus contract to one wire. It is a typed document with a
closed vocabulary, pinned to the contract's digest, and it never restates a promise of the
contract. There is one binding per bus, so one injection point: the operator console injects one
implementation by choosing one binding. It reuses `ess-transport`'s subject and stream checks as a
library.

```yaml
type: ess-bus-binding/1
bus: "sha256:<compiled digest of platform>"
transport:
  protocol: nats                     # closed: nats | unix_socket
channels:
  - {channel: activity, subject: platform.activity}
  - {channel: stats, subject: platform.stats}
  - {channel: audit, subject: platform.audit, stream: AUDIT}
```

```yaml
type: ess-bus-binding/1
bus: "sha256:<compiled digest of platform>"
transport:
  protocol: unix_socket
  framing: ndjson                    # closed: ndjson | length_prefixed
channels:
  - {channel: activity, publish: run/bus/activity.pub, subscribe: run/bus/activity.sub}
```

- **NATS.** Subjects follow `ess-transport`'s rules 4 and 6-7 (`event-transport-binding.md:73-81`):
  publishable tokens, and on JetStream exactly one stream per subject. Publish-subscribe is a
  plain subject. Fan-out to named sinks is one subject per sink, or one subject and a durable
  consumer per sink when `at_least_once`. `at_least_once` needs JetStream; core NATS admits only
  `at_most_once`.
- **Unix socket.** A path relative to a declared runtime directory, the framing, and a maximum
  message size. The bus process accepts subscriber connections and writes each message to every
  connection of the channel; it keeps nothing, so a unix-socket channel admits only
  `at_most_once`.
- A channel the binding leaves unbound, a pattern the transport cannot carry, or a delivery word it
  cannot keep is refused by name.

**How conformance is checked.** Three levels, labelled apart as protospec labels model and target
runs (`protocol-specification.md:21-25`):

1. **The pattern's model.** Each pattern of the catalog has a reference machine written as an
   `ess-protospec/1` document: participants publisher, bus and sinks; channels whose ordering,
   loss and duplication follow the delivery word; safety properties such as "each subscriber
   receives each event at most once", "an event whose route condition is False reaches no sink of
   that route", "one aggregate per period". Bounded exploration checks the machine itself.
2. **The contract's scenarios.** Synthesis writes scenarios from the compiled contract alone:
   publish each outward event on each channel and expect it at each subscriber and sink; for each
   route an event that makes the condition True, one False and one Unknown; for each aggregate a
   period with rows and one without.
3. **A bus implementation over a binding.** A target adapter per transport (NATS, unix socket)
   publishes the scenarios' events over the bound wire, records what each sink receives as an
   `ess-prototrace/1` trace with origin `target`, and the trace is checked against the pattern's
   machine (`protocol-specification.md:44-54`). Incomplete capture is inconclusive, never a pass.

Which wire bytes a NATS or unix-socket message carries (the event's payload in ESS's JSON, one
payload per message) is the binding's; generated publishers (`event-publishers.md`) gain a bus
seam that writes them.

## 5. Ingress: a system declares its inputs as protocol bindings

The same separation, on the way in. Four layers, from the inside out:

| layer | what it states | where it lives today |
|---|---|---|
| semantics | the commands and views a caller may invoke, or a named exchange | the specification; the exchange record |
| protocol | the protocol's own behaviour: its messages, order, correlation, lifecycle, errors | an `ess-protospec/1` port written from an RFC (as RFC-derived specifications produce them) or from a published specification with its own conformance suite |
| binding | which command or view each protocol operation carries, and in which wire form | `ess-http/1` (built); `ess-jsonrpc/1` and `ess-grpc/1` (`docs/design/rpc-bindings.md`, draft) |
| transport | standard streams, HTTP with server-sent events, a unix socket, a WebSocket | the protocol port's layering axis (rfcspec.md line 250) |

- **MCP** carries JSON-RPC 2.0 messages: a command is a tool (`tools/call` with the tool name, its
  input schema the command's input schema), a view a resource (`resources/read`), an outward event
  a notification. Its lifecycle (initialize, capability negotiation) is the port's, not the
  binding's.
- **A2A** carries JSON-RPC 2.0 over HTTP with server-sent events: a command is a skill a caller
  sends a message to; a task's updates are outward events of that exchange.
- **HTTP** is `ess-http/1` as it is.
- An ingress binding pins the protocol port's digest as the broker-subject story pins
  `protocol:`, so a binding cannot claim a behaviour its port does not state.
- Conformance: the port's requirement vectors check the protocol (RFC MUST and SHOULD rows, or the
  published suite); the specification's scenarios run through a target adapter per binding, as
  `ess-http/1` runs its routes today.

Each protocol has its own binding format with a closed vocabulary: `ess-mcp/1` and `ess-a2a/1`,
beside `ess-http/1`, each pinning its protocol port. The exchange record keeps every binding its
own format (§ 4, lines 134-136).

## 6. First producers and the first consumer

| producer | outward events (proposed) | pattern | today |
|---|---|---|---|
| `beyond10x/slot` | a job changed state; pool statistics | publish-subscribe; periodic aggregate (its `Tick` already is one) | server-sent events |
| `beyond10x/llm-gateway` | a call finished; statistics per route (requests, tokens, latency, cost) with the backend (hosted GPU, local model server) as a group field | periodic aggregate; publish-subscribe | snapshot as server-sent events |
| `beyond10x/runbook` | its run events, with `kind` a closed enum of the kinds the daemon writes | publish-subscribe | server-sent events per view |
| `beyond10x/cortex` | none yet; an answer given, a review opened and closed are candidates | publish-subscribe | no feed |
| `beyond10x/thalamus` | later: a verdict changed | publish-subscribe | not read |

Each producer declares its outward events in its own specification; one bus contract pins the
five specifications; the consumer, an operator console, chooses one binding (NATS, or a unix socket
on one host), subscribes to `activity` and `stats`, and draws every producer on one architecture
page from the contract: the systems, their channels and the routes between them are data, not a
hand-drawn diagram. Each producer's present feed becomes one adapter that publishes its outward
events to the bus until it publishes natively; the consumer keeps its subscriber seam and gains a
bus implementation behind it.

ESS ships none of the producers or the consumer. It ships the formats, their checks, the
projections and the conformance suite, and one worked example with generic names that holds the
end-to-end path.

## Decided

The owner decided each fork of the first draft on 2026-11-26 (relayed by the operator session).
The picked option is the design above. Each rejected option is one line.

- **F1, where outwardness is declared: A.** `outward:` on a component's `publishes:`, at the next
  `ess/N`. Rejected: B, nowhere, the bus selects (no contract boundary for `diff`); C, a sibling
  `ess-outward/1` (one more document per system for one list).
- **F2, where the bus contract lives: A.** A sibling `ess-bus/1` pinning several specifications by
  digest, compiled to `ess-bus-ir/1`, with each pattern's reference machine in `ess-protospec/1`
  for conformance (§ 4). Rejected: B, inside each producer's specification (a producer names its
  consumers); C, one `ess-protospec/1` per bus (no pattern vocabulary).
- **F3, how a periodic aggregate is declared: A.** An outward event sampled from an aggregate view
  every period (`every`, `from_view`). Rejected: B, a time step emitting an event (its payload is
  checked against no view); C, the bus aggregating per-occurrence events (per-occurrence traffic
  the owner excluded).
- **F4, the route condition language: C.** ESS's binding `where:` fragment extended by numeric
  comparison and list membership, plus a tested translation into `canon-expr/1`. Rejected: A
  without the translation (two unrelated languages); B, `canon-expr/1` unchanged (ESS depends on
  Canon's crate or reimplements it).
- **F5, "start protocol P" as a sink: B.** No sink kind of its own; a trigger evaluator is a system
  with a declared input, the evaluator's declared input being the sink. Rejected: A, a sink kind
  that teaches the bus Canon's mode, cap and refire; C, protocol starts outside the bus.
- **F6, where a bus's transport binding lives: B.** A sibling `ess-bus-binding/1` per bus, reusing
  `ess-transport`'s subject and stream checks. Rejected: A, in `ess-transport` per producer (N
  documents must agree); C, transport keys inside the contract (wire words in the semantics).
- **F7, how an ingress binding is cut: A.** One format per protocol, `ess-mcp/1` and `ess-a2a/1`.
  Rejected: B, one `ess-ingress/1` with a closed `protocol:` set; C, MCP and A2A as methods of the
  draft `ess-jsonrpc/1` (no typed home for tool listing or agent card).

## What it is not

- Not a broker, a bus implementation or a deployment: ESS states and checks; it runs nothing and
  creates no subject, stream or socket.
- Not a generic binding document with a free `transport:` field, and not a property bag: every
  pattern, sink kind and protocol is a closed member added by its own decision.
- Not request and reply (the broker-subject story) and not a conversation (the exchange record).
- Not Canon's trigger rules: a route forwards an event; a trigger rule decides whether a protocol
  starts.
- Not "exactly once".

## Stories

| story | delivers | decisions |
|---|---|---|
| `story:a-system-declares-which-of-its-published-events-are-outward` | § 1 | F1 |
| `story:an-event-bus-contract-states-publish-subscribe-and-fan-out-channels-over-outward-events` | § 2 | F1, F2 |
| `story:a-periodic-aggregate-event-publishes-an-aggregate-view-s-answer-every-period` | § 2, periodic aggregate | F3 |
| `story:a-bus-route-forwards-an-event-to-a-sink-when-its-condition-holds` | § 3 | F4, F5 |
| `story:a-bus-contract-binds-to-nats-or-a-unix-socket-and-a-bus-implementation-is-checked-over-each` | § 4 | F6 |
| `story:a-system-declares-its-ingress-as-mcp-and-a2a-protocol-bindings-beside-http` | § 5 | F7 |
| `story:a-worked-example-puts-several-producers-and-one-consumer-on-one-bus-and-checks-it-end-to-end` | § 6 | F1-F7 |
