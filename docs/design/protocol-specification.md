# Protocol specifications

Status: experimental first implementation. Planning: [ess-protospec](../../.engineering/planning/design/ess-protospec.md).

## Boundary

`ess-protospec/1` is an explicitly versioned protocol sidecar. It describes finite communicating machines with participant-local state, message delivery and logical time. It does not change `ess/N`, `ess-ir/1` or `ess-conformance/N`; existing projections cannot accidentally ignore a new protocol declaration. The authored model belongs to `ess-domain`, admission and canonical identity to `ess-compiler`, execution and checking to `ess-conformance`, and filesystem operations to the CLI. No infrastructure or planning dependency is introduced.

This is a distinct transition model because entity lifecycle transitions are caused by commands and their outcomes, whereas these transitions are caused by local inputs, received messages and owned timer firings. Message transmission and delivery are distinct facts. A protocol participant is a modeled role, not an authentication identity or necessarily a transport client.

## Execution contract

The admitted model resolves every participant, channel, message, state and timer reference. Finite bounds and strict field admission are mandatory; unsupported syntax is refused. Persisted collections have deterministic ordering and canonical digests include all semantic content. Missing references, unknown fields, duplicate identities and arithmetic overflow are errors.

Execution is pure: no sockets, wall-clock reads or background tasks. An explicit action drives a local input, a selected message delivery, a permitted channel fault or advancement of logical time. A send creates an observable transmission in a bounded channel. Delivery consumes that transmission or a permitted duplicate occurrence and invokes the receiving machine. FIFO and unordered channels differ in which delivery actions are enabled. Loss and duplication are possible only when declared. Channel exhaustion refuses execution rather than silently dropping traffic.

Timers belong to a participant and a name, carry a generation and a checked deadline, and are canceled or replaced explicitly. Stale generations cannot fire a replacement timer. Advancing time does not silently suppress an intervening enabled timeout. Duration arithmetic must not wrap. Retransmissions remain distinct transmission occurrences even when they represent one logical exchange.

The initial subset uses finite declared state and typed message fields. Arbitrary executable predicates, unbounded model checking, binary parsing, cryptographic semantics, durable restarts and universal distributed clocks are outside this format's admitted capability. Unsupported observations or capabilities cannot produce a passing result.

## Checking and evidence

A recorded trace identifies the compiled model and retains exact actions and actual observations. Trace checking preserves every model configuration consistent with those observations; it does not select one preferred schedule. Contradictory observations fail. Incomplete capture, exhausted bounds and unresolved future obligations are inconclusive. A complete prefix alone is not evidence that a future deadline was satisfied.

Bounded exploration enumerates enabled actions reproducibly and reports the bounds and explored counts. A clean bounded search is evidence about that model within those limits, not a proof of arbitrary executions or of any implementation. A target interface supplies independently produced observations; its adapter must not use the reference executor to fabricate expected output. Model execution and target conformance are labeled separately.

## Proving cases

A terminal-response session must flush the response before closing, settle a pending request once, and avoid accepting a response from a different session generation. The first finite example focuses on observable flush/close ordering; unsupported dynamic lifetime semantics are documented rather than inferred.

A rejected INVITE transaction follows RFC 3261 section 17. A lost ACK permits retransmission of the final response. The client re-ACKs without notifying its user twice; the server cancels its retransmission timer when ACK arrives. Client and server absorption timers are independent. Retransmission backoff is capped. A finite example and deliberately faulty independent target provide evidence for the supported subset; byte parsing and actual stack interoperability remain separate obligations. Successful INVITE responses have different semantics (RFC 6026) and cannot reuse this machine unchanged.

Keep byte-level golden fixtures, RFC vectors, fuzzing and real transport interoperability tests when adopting this facility. They test boundaries this model does not observe.

## First format details

The first release uses scalar payload/local fields (`String`, `Boolean`, signed 64-bit `Integer`),
explicit equality guards and ordered typed effects. Source expressions read a local field, an input
field or a literal. Effects assign locals, send, arm/cancel timers, emit application events,
acknowledge transport flushing and close. Safety properties prohibit a state, bound an event count,
require a flush before close or relate two participant states. These are bounded safety checks;
there is no general temporal predicate language yet.

`ess-prototrace/1` persists an explicit `origin` (`model` or `target` with implementation identity),
model digest, exact action/observation steps and capture completeness. Origin is adapter-supplied
provenance, not cryptographic attestation. Replay reports that origin; a model trace can never
supply implementation conformance evidence merely by passing replay. Distinct transmission IDs
start at 1; duplicate queue copies retain `original`, exchange and logical message identity.

A trace passes only if every observation has a permitted successor and every surviving state has
no queued messages, armed timers or declared outstanding flush obligations. This is a finite
observation-boundary verdict, not proof that all protocol states are globally terminal. Flush
obligations are created on sends only for channels named in `flush_before_close` properties.
Property violations are checked at initialization and after each ordered effect, and remain sticky.

The native adapter operates one isolated lifetime per capture. Capability admission precedes open;
all drive calls return ordered observations through their causal boundary, and cleanup is called
after either successful drive or a drive error. Runtime inability to interpret a transition and
exhaustion of execution bounds remain inconclusive, even if another schedule could have completed.
Dynamic role creation, session epochs, received-envelope guards, trace reduction and asynchronous
cursor-based observation are future extensions; this format does not imply that they exist.

The first exploration timing profile offers inputs initially and at scheduled deadlines. Its report
names this policy; it does not claim coverage of arbitrary intermediate-time inputs. Explicit replay
can exercise other logical instants. Invalid input witnesses refuse exploration rather than being
silently discarded. The logical-time bound limits clock advancement: a future timer may exceed that
horizon and be canceled before it, so arming alone does not exhaust the time bound.
