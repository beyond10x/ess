---
format: aep.planning-md/3
id: design:ess-protospec
kind: design
status: draft
title: 'ess-protospec: communicating state machines and protocol conformance'
summary: Draft protocol semantics for peers, messages, exchanges, channels, timers and layered assertions.
relations:
- serves: vision:O2
- informed_by: epic:message-contract-clients
revision: 2
---
# ess-protospec design proposal

Status: draft for design review. Date: 2026-10-03.

`ess-protospec` is a proposed ESS capability for specifying and checking conversations between
independently evolving participants. A conversation comprises local state machines, typed messages,
exchanges, channels, timers and observable effects. Its conformance result says which protocol
obligations an implementation satisfied, at which observation boundary, under which assumptions.

The intended benefit is to replace duplicated behavioral specifications and test orchestration with
one typed contract, while retaining independent wire fixtures, protocol vectors and integration
evidence. A passing generated test establishes agreement with the encoded model; it does not prove
that the model correctly or completely translates an RFC.

This document proposes semantics and two proving cases. The user authorized the first implementation on 2026-10-03. Its adopted subset and explicit experimental formats are specified in docs/design/protocol-specification.md and tracked by story:ess-protospec-foundation; the remaining proposals below are a roadmap, not implemented capabilities. This work does not reopen deferred ESS evolution work.

## Need and existing evidence

Two peers can each implement a plausible state machine while their combined conversation is wrong.
Examples include a response resolving another request, a retransmission reaching the application
twice, shutdown overtaking a terminal response, and a canceled timer affecting a replacement
exchange. Payload schemas and ordinary command outcome tests do not by themselves describe those
relationships.

The general requirement is to express the legal observable executions of a protocol, then hold
implementations to them at several boundaries: semantic machines, wire codecs, transport drivers,
and composed sessions. A linear exchange script is one witness, not the complete protocol.

The assessment used these snapshots:

| Source | Revision | Relevant evidence |
|---|---|---|
| ESS | `d0f22461dae326f59664bc9427efad3589641961` | Authored declarations, conformance target, timer observations and message-contract epic |

ESS already provides types, entities, commands, outcomes, lifecycles, events, views, bindings,
predicates, deterministic compilation and conformance reporting. Its Rust runner is available as
the `ess-conformance` library through `ConformanceTarget`; the absence of a generated Rust test
package is not the absence of a Rust runner. Existing target capabilities include periodic-host
observations, event redelivery, elapsed-time observations and clock-reading provenance. [E1–E4]

The remaining gap is an explicit, compositional model of participant-local protocol behavior and
channel delivery, with exchange-scoped correlation, timer lifetimes and obligations over traces.
One can encode selected cases as ordinary commands and entities today. Doing so does not make
network causality, channel fault assumptions or complete negative observations native semantics.
The source review establishes the missing first-class constructs; this draft does not claim that
every possible encoding in existing ESS has been experimentally ruled out.

## Fit review and proposed decision

The proposed disposition is **accept the need, redesign the integration**, subject to review. This
is a design recommendation, not an accepted AEP artifact or an implementation commitment.

| Fit question | Answer |
|---|---|
| Need apart from syntax | Describe independent peers and determine whether their observable message, state and timer histories satisfy a protocol. No YAML spelling is assumed. |
| Classification | A semantic gap in conversation modeling and observation. Individual message types and many local state transitions are already expressible. [E1–E4] |
| Existing idiom | Reuse commands, outcomes, types, views, bindings and current observation mechanisms. Avoid parallel copies of them. A minimal executable comparison against the existing idiom is a prerequisite to adopting new source keys. |
| Composition | Protocol roles bind to component instances; application actions bind to existing commands; predicates retain their type rules; effects refer to existing message/event types. Network transmission does not silently inherit an application event binding's delivery guarantee. |
| Independent use | A device controller and a device can exchange numbered operations, acknowledgements and status notifications while handling reconnects and stale replies. This needs the same semantics without telephony-specific concepts. |
| Cost | New source declarations, resolved references, trace operations, suite admission, target capabilities, compatibility classifications and documentation. Every affected target must implement or explicitly refuse them. |
| Alternatives | Keeping current harnesses, lowering entirely to existing commands, a separate protocol language, and an integrated protocol capability are compared below. |

ESS's draft `epic:message-contract-clients` covers broker subjects, streams, envelopes, AsyncAPI and
typed publishers. Reuse compatible transport and message declarations from that work; do not
create a competing catalog. Conversation state and scheduled delivery are additional concerns,
not implied capabilities of that epic. `task:deferred-protocol-ui-bindings` remains deferred and
is not made a prerequisite or silently reopened. [E5]

## Semantic model

The global execution configuration is conceptually:

```text
configuration = (
    participant instances and their local machines,
    channel contents and delivery state,
    pending exchanges,
    timers and local clock domains,
    observation history
)
```

This is an interpreter and checker model. It does not introduce shared mutable memory into the
implementation. A participant transition can read its local state and its delivered input; it
cannot inspect another participant's unobserved state to decide what to do.

### Participants, roles and machine instances

A role describes protocol responsibilities. A participant is an instance playing that role in a
scope. Neither is an authorization actor or a transport client/server position. One endpoint may
initiate an exchange and answer a different exchange on the same connection.

A machine declares typed local data, states, accepted inputs, guards and effects. Inputs include
local application actions, delivered messages, timer expirations, transport observations and
signals from other local machines. Effects include local state updates, sends, timer operations,
application notifications, machine creation and termination.

A local semantic step has an explicit atomic boundary. Ordered effects do not make their external
execution atomic: queuing a write, completing a write, flushing and receiving are separate facts.
Failures between effects remain observable and can enable further transitions. Illegal inputs
must have declared behavior, such as ignore, reject or close; absence of a transition is not an
implicit success.

Instances form a typed graph, not necessarily an ownership tree. Containment, lifetime ownership,
correlation and reference relationships are distinct. A connection may carry several sessions; a
session may contain many exchanges; one initiating operation may create several related dialogs.
Each creation rule declares identity, multiplicity and cleanup responsibilities.

### Messages and exchanges

Message content uses ESS's type system. Routing, peer identity, wire identity and observation
identity are separate. The model distinguishes:

- an exchange identity;
- a logical message within the exchange;
- each transmission occurrence;
- each delivered copy of that occurrence.

Network duplication and application retransmission therefore remain distinguishable even if they
carry identical bytes. Correlation rules declare their scope, including direction and connection
epoch where required. There is no universal assumption that a request ID alone is unique.

An exchange describes how messages relate: initiating request, zero or more provisional results,
completion, acknowledgement, cancellation, failure and timeout as applicable. A one-shot RPC is a
restricted exchange profile. Multiple final responses, streaming results and branching exchanges
require their own explicit rules rather than exceptions to a universal one-response assumption.

Typed semantic messages and wire bytes remain separate layers. A codec binding declares encoding,
framing, required presence, correlation extraction and supported normalization. Golden bytes
remain an independent authority where byte identity is required. Equivalent legal encodings must
not be rejected merely because they differ from a generator's preferred encoding.

### Channels and delivery assumptions

A channel declares endpoints, direction, ordering scope, capacity and failure semantics. Sending
enqueues an occurrence; delivery later provides an input to the receiver. The selected profile
decides whether loss, duplication, reordering, disconnect and delayed delivery are permitted.

Faults are applied at the declared layer. A reliable byte-stream profile cannot arbitrarily reorder
complete application messages while claiming to model the stream's ordinary behavior. It can
exercise byte fragmentation, coalescing, disconnect and application scheduling. A datagram profile
can permit packet loss and reordering. Malformed-message injection is a separate hostile-peer or
wire-input capability, not ordinary channel delivery.

Transport guarantees, scenario fault choices and protocol obligations are recorded separately.
The report states the combination tested. Queue limits are explicit so exploration and runtime
acceptance can include admission and backpressure without inventing an unbounded channel.

### Timers and clocks

A timer instance identifies its owner, name, generation, clock domain and deadline. Arming,
rearming, cancellation and expiration are distinct actions. A replacement timer cannot inherit
the identity of an earlier queued expiration accidentally.

The profile states what cancellation guarantees. Some drivers prevent an expiration from being
queued; others permit an already queued callback. Tests must exercise that actual contract and
the machine's response to a stale callback. Generations can exist only in the observation/model
layer when the implementation has no corresponding public field; the adapter must not claim an
implementation provides generation validation that it does not perform.

Durations and arithmetic are typed, unit-bearing and checked for overflow. Timer rules can require
a constant deadline, multiplication, bounded backoff or a ceiling. The first implementation should
admit only the arithmetic required by its proving cases and refuse the rest.

Logical time is controlled at a declared test boundary. Advancing time schedules due events; it
does not silently choose the response-versus-timeout race winner. Equal-deadline orderings are
explored where the protocol permits both. Real-clock observations carry their source and epoch;
cross-peer timestamp comparison needs calibration or a causal relation.

Each scenario also has an execution budget independent of protocol time, so advancing logical time
cannot create an unbounded loop. Long-term clock drift, process restart and durable timers are
later capabilities, with explicit refusals in the first implementation.

## Properties and observation

Properties name their scope, trigger, applicability, required observations and normative source.
Cross-peer properties can inspect a consistent observed configuration or related causal events;
they do not let either participant access remote state.

| Property family | Example | What constitutes evidence |
|---|---|---|
| Local safety | A pending operation resolves at most once | Actual completions scoped to that operation |
| Correlation | A reply resolves only its matching exchange | Sent/received identities and the observed resolved operation |
| Relational agreement | Both committed peers agree on negotiated parameters | Both local commitments and the negotiated values |
| Causal order | Terminal response flush completes before teardown | Ordered observations from the actual transport boundary |
| Bounded progress | A request completes or times out by its deadline | A complete observed interval and the terminal result |
| Quiescence | No owned work occurs after close acknowledgement | A complete tail interval or a deterministic settled boundary |
| Resource bound | Pending work never exceeds admitted capacity | Occupancy/admission observations at the specified boundary |
| Cross-layer consistency | A declared timer cancellation reaches the driver | Core effect and corresponding driver observation |

Agreement is conditional on a protocol milestone. An accepter may commit before its response
reaches the initiator, so equality of both peers' state at every instant is generally false.

Observer bookkeeping may derive pending exchanges and message ancestry from an independently
defined model. It must not fabricate implementation state. Internal-state assertions require an
actual observation seam; a packet-only target cannot claim to have checked hidden timer tables.
Paired snapshots require a consistent cut or causal evidence, not two unrelated reads labeled
simultaneous.

### Allowed executions and verdicts

The monitor retains the set of model states consistent with the observations. A legal
nondeterministic choice must not fail because it differs from one preferred scripted execution.
An observation fails safety when no admitted model execution can explain it. Internal steps may
be abstracted only by an explicit observation mapping.

Missing output becomes a failure only after its obligation's deadline or settled boundary is
established. Before then it is pending. A completed finite trace cannot prove an unbounded
eventually claim. A property whose trigger never occurred is unreached, not positive evidence.

Reports distinguish pass, violation, unavailable capability, incomplete observation and exhausted
exploration budget, mapped explicitly to the existing ESS report vocabulary at implementation.
There is no new synonym silently mapped to pass. Required scenarios with unresolved capability
or coverage gaps prevent a complete-conformance claim.

Liveness assumptions are explicit. Under arbitrary permanent loss, successful connection cannot
be required; bounded timeout and cleanup can still be required. Fairness assumptions used by model
exploration are reported and are not inferred facts about a deployed network.

## Execution and adapter contract

The same protocol model supports four modes:

1. **Model exploration:** search bounded schedules and input classes for contradictory obligations
   or reachable forbidden configurations. This checks the specification.
2. **One implementation and a modeled peer:** exercise either role with valid behavior and selected
   hostile inputs. This checks a particular implementation boundary.
3. **Two implementations and an independent monitor:** execute both roles and check observed
   histories against the contract. Agreement between the two implementations alone is insufficient.
4. **Trace replay:** evaluate recorded observations and state which properties the capture cannot
   establish. Absence in an incomplete capture is not evidence of non-occurrence.

Extend the existing conformance target with typed optional capabilities only where the new
semantics demand them. Candidate capabilities include binding participant instances, injecting a
local input, controlling delivery, collecting ordered observations, and settling a controlled
clock interval. These are conceptual responsibilities, not proposed public method signatures.

The adapter drives and observes. It does not calculate expected states, select verdicts, silently
repair a message, remove unexpected output, or echo an expectation as a result. Unsupported
capabilities are explicit. The same adapter should expose all observable effects, including those
the scenario did not expect, within the selected boundary.

The runner must distinguish a modeled send from an implementation send. In implementation tests,
the channel receives actual output from the tested code; the reference model cannot emit the
packet on its behalf. Likewise, a timer test observes the implementation's requested duration
before deciding when to fire it.

A failed run retains the model and suite digests, implementation identity, profile, fixture values,
seed, schedule choices, actual observations, source obligation and the shortest reproduced failing
trace found within the reduction budget. Reduction must preserve the failure's assumptions and
correlation relationships. The original trace remains available.

## Worked application to a session protocol

A session protocol can separate payload catalog, envelope, session runtime and transport. A linear request/response/event script is insufficient to specify deferred response ownership, pending requests, response dispatch and terminal response flushing.

The initial session proving case is a terminal request racing an unrelated pending request:

```text
given: two active peers; one ordinary request remains pending
when:  a supported terminal request is received
then:  its successful response is flushed before local transport teardown
and:   each still-pending request resolves exactly once on local closure
and:   a late response cannot resolve a later session's request
and:   close acknowledgement leaves no session-owned work running
```

Explore the ordinary reply before terminal completion, while the terminal response is queued, and
after closure. A write or flush failure follows a separately specified failure path; it must not
be reported as successful delivery. Flush completion proves the local transport boundary, not
remote receipt. The late-response case requires an explicit connection/session epoch boundary.

The second session case tests a nested request made by a handler: the response must remain
processable while ordinary request/event dispatch is occupied. That checks scheduler behavior
which a payload catalog cannot establish.

A later media case can specify buffer clearing. It must distinguish locally queued frames,
already transmitted frames and frames admitted after the clearing boundary. A blanket assertion
that no audio exists after the request would be an invented requirement.

## Worked application to SIP transactions

A SIP transaction adapter can consume messages and fired timers and return ordered effects without sockets or clock reads. The protocol requirements below follow RFC 3261 section 17. [N1]

The protocol model should compose these levels without flattening them:

| Level | Contract |
|---|---|
| Wire | Framing, parsing, encoding and typed message constraints |
| Transaction | Matching, state changes, retransmission, timer effects and delivery upward |
| Dialog | Dialog identity, sequencing, routing and termination |
| Offer/answer | Negotiation state and permitted changes to media parameters |
| Call | Coordination of transactions, dialogs, media and application events |
| Runtime | Delivery of core effects, actual timer lifecycle, cancellation and resource release |

A connection, transaction, dialog and call are distinct identities. Neither a strict ownership
tree nor a universal client/server role captures their relationships. A fork may create multiple
dialogs, and an established endpoint can initiate a new transaction in the opposite direction.

### Initial transaction witness

Use an unreliable transport and an INVITE receiving a non-2xx final response:

```text
client sends INVITE                         -> server receives INVITE
server sends final non-2xx                  -> client receives final response
client emits ACK                           -> channel drops that occurrence
server retransmission timer expires
server repeats final response              -> client receives repeated response
client repeats ACK                         -> server receives ACK
absorption timers expire                   -> local transaction resources retire
```

Required observations and assertions:

- The final response moves the client transaction to Completed and produces the proper ACK.
- The repeated final response produces another ACK without another final-response notification
  to the transaction user.
- The server's timer-driven retransmission uses the specified backoff and ceiling.
- Receipt of the matching ACK moves the server to Confirmed and clears the relevant timers.
- Timer ownership remains correct when another transaction uses the same timer names.
- Eventual retirement is tested through the distinct client/server absorption timers, rather than
  asserting both sides terminate simultaneously.
- Codec and driver variants repeat the behavior at their own boundaries; a core pass is not
  evidence that a real socket sends the ACK or that a driver cancels the timer.

RFC 3261 section 17 supplies the transaction rules. RFC 6026 must be incorporated for the separate
successful-INVITE case: Accepted states remain alive, repeated 2xx responses reach the transaction
user, and successful-response ACK behavior is checked at the appropriate higher layer. [N1–N2]

Subsequent cases should include provisional-response versus timeout races, CANCEL versus final
response races, simultaneous independent transactions, forks and overlapping offer/answer activity.
Their acceptance must distinguish protocol requirements from application policy.

### What can eventually be replaced

Generated transition scenarios can replace repetitive hand-transcription of state tables. Typed
contract projections can replace duplicate enumerations. Requirement-linked execution results
can strengthen or replace evidence-path heuristics for the specific claims they actually cover.

Existing fuzzing can reuse typed actions and the new evaluator, while retaining its search engine
and independent invariants. RFC fixture recovery, cryptographic vectors, malformed-byte fuzzing,
feature/MSRV gates and live interoperability checks remain independent. No existing check is
removed solely because a generated suite passes.

## ESS integration and compatibility

The preferred placement is within ESS's existing compile, generate and verify pipeline. Reuse
qualified names, type references, predicates, source spans, diagnostics, digests and conformance
evidence. Keep protocol execution distinct from infrastructure observation. No `InfraIr` merge,
generic facet registry, arbitrary property bag or blanket `ess-ir/2` redesign is proposed.

Concrete protocol types belong at the narrowest owner established by the first consuming
compiler or evaluator. The design should first test reuse of existing entity lifecycle machinery
without duplicating states in two authored places. A protocol-specific machine wrapper is
justified only where local inputs/effects and composition need semantics the existing lifecycle
does not supply.

New persisted semantics require coordinated source, resolved representation and suite-version
decisions. Exact numbers are allocated when implementation begins. Old documents retain their
canonical bytes. Older readers must reject new vocabulary before execution; unknown fields and
required capabilities must never be ignored. Updating `RawSpecFile` requires regenerating its
schema through the existing projection workflow.

| Surface | Required disposition |
|---|---|
| Validation/compiler | Resolve every role, machine, message, timer, correlation and property reference; reject invalid ownership and inaccessible remote reads |
| Rust conformance | Execute the proving cases through typed native observations and the existing runner |
| Go/TypeScript conformance | Implement supported operations or refuse suites containing unsupported protocol operations before running them |
| Interpreter/explorer | Execute the supported finite model with explicit state, queue, step and time bounds |
| Semantic diff | Expose changed roles, permitted traces, correlation, timer bounds, channel assumptions and property applicability; return undetermined when compatibility cannot be established |
| Docs and scenario UI | Render declared behavior, actual traces, assumptions and gaps; a playback animation is not evidence of implementation execution |
| Code synthesis/entity runtime | Generate only semantics the target can preserve; otherwise emit named obligations/refusals |
| OpenAPI/AsyncAPI/schema | Project supported message/transport facts and disclose omitted behavioral semantics |

Source translation preserves obligation strength and scope: MUST, SHOULD, MAY, role, profile,
exceptions, amendments and applicable errata. Do not promote a SHOULD into an unconditional failure
or a MAY into required behavior. Unsupported obligations remain accounted for. Extraction from
RFC prose may assist drafting, but review must establish the interpretation.

## Alternatives and scope limits

**Change nothing.** Existing harnesses remain the least risky immediate solution. They retain
duplicated state tables and bespoke scheduling/correlation logic. This remains the fallback if
the proving cases cannot demonstrate a net improvement.

**Encode everything as ordinary ESS commands/entities.** This is preferred wherever sufficient.
Making each delivery and timer a command can represent selected witnesses, but without explicit
channel and observation semantics it leaves the protocol rules in handwritten adapters. The
first experiment must measure exactly which rules require new constructs.

**Build an independent protocol language and runner.** This offers freedom but duplicates types,
predicates, reporting, versioning and target support. It also makes ESS integration an importer
maintenance problem. The current proposal does not justify that cost.

**Generate complete protocol implementations immediately.** Deferred. Shared generation can
produce a common error in implementation and oracle. First establish independent observations,
frozen vectors and mutation-sensitive tests; generate only demonstrated structures later.

The first implementation excludes arbitrary executable predicates, a general binary grammar
language, cryptographic verification, unbounded model checking, durable restart recovery and a
universal distributed-clock model. It supports bounded in-memory cases before live networking.
These exclusions must be refusals or explicit coverage gaps wherever a specification requires them.

## Proving the design before adoption

The proposed sequence is experimental acceptance, not a dispatched backlog:

1. Write minimal brand-free cases using existing ESS constructs. Validate and synthesize them;
   retain exact outputs and identify rules still living only in the adapter.
2. Introduce only the typed semantics those cases cannot express, with an interpreter and native
   Rust target. Prove separate send/delivery events, correlation scope, timer cancellation and
   complete observation windows.
3. Run the terminal-response/shutdown case and the lost-ACK transaction case against real code,
   under bounded deterministic schedules. Preserve existing independent tests alongside them.
4. Inject faults: wrong correlation, duplicate completion, early close, missing timer cancellation,
   incorrect backoff, retransmission delivered upward twice and missing observation data. Each
   fault must produce a relevant failure or non-passing incomplete result.
5. Replay and minimize a failure, then run one real transport variant to establish the adapter
   boundary. Report what the real transport run still cannot observe.
6. Measure the maintenance result: duplicated assertions removed, handwritten adapter size,
   remaining custom semantics, reproducibility and runtime. Retire a checker only after its
   specific guarantees have an equal or stronger replacement.

The first implementation decisions are recorded in the binding design and the AEP foundation story. Broader lifecycle reuse, asynchronous observation completeness, transport ownership and compatibility classification remain open for later work. No downstream conformance result is claimed.

## Source references

- E1: [ESS authored model at the inspected revision](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/crates/specify/ess-domain/src/spec.rs).
- E2: [ESS Rust runner](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/website/docs/start/runners/rust.md).
- E3: [Conformance target and observation capabilities](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/crates/verify/ess-conformance/src/target.rs).
- E4: [Bindings, periodic causes and clock provenance](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/website/docs/guides/specify/bindings-and-components.md), [authored observation scenarios](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/website/docs/guides/verify/author-scenarios.md).
- E5: [Message-contract epic](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/.engineering/planning/epic/message-contract-clients.md), [deferred protocol work](https://github.com/beyond10x/ess/blob/d0f22461dae326f59664bc9427efad3589641961/.engineering/planning/task/deferred-protocol-ui-bindings.md).
- N1: [RFC 3261 section 17](https://www.rfc-editor.org/rfc/rfc3261.html#section-17).
- N2: [RFC 6026 sections 7–8](https://www.rfc-editor.org/rfc/rfc6026.html#section-7).
- N3: [RFC 3264 offer/answer model](https://www.rfc-editor.org/rfc/rfc3264.html).

Repository adoption uses generic proving cases and normative RFC references; application-specific research provenance remains outside the public source.
