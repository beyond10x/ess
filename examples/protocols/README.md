# Experimental protocol specifications

These finite examples use `ess-protospec/1`. They exercise protocol state and observable effects;
they do not test a real SIP stack, network, parser or session runtime.

From the repository root, with a binary built from this checkout:

```console
cargo run -p ess-cli -- specify protocol validate --path examples/protocols/terminal-response.yaml
cargo run -p ess-cli -- specify protocol compile --path examples/protocols/terminal-response.yaml
cargo run -p ess-cli -- verify protocol run --path examples/protocols/terminal-response.yaml --actions examples/protocols/terminal-response.actions.json --out terminal.trace.json
cargo run -p ess-cli -- verify protocol replay --path examples/protocols/terminal-response.yaml --trace terminal.trace.json
cargo run -p ess-cli -- verify protocol explore --path examples/protocols/terminal-response.yaml
cargo run -p ess-cli -- verify protocol run --path examples/protocols/lost-ack.yaml --actions examples/protocols/lost-ack.actions.json --out ack.trace.json
cargo run -p ess-cli -- verify protocol replay --path examples/protocols/lost-ack.yaml --trace ack.trace.json
```

Output files are create-new: an existing file is never overwritten. Validation and replay return
exit 0 on success, 1 on refusal or contradiction, and 2 on inconclusive observation or exploration.
A simulated trace is model evidence. To test an implementation, implement
`ess_conformance::protocol::ProtocolTarget`, then use `capture_target` or `run_target`. The adapter
must supply actual ordered observations at the stated causal boundary, including timer and transport
facts. It must not call the reference executor to produce its answers. Opening and closing that
adapter lifetime does not inject a protocol close action. A failure after successful open still
calls adapter cleanup.

`terminal-response` makes flush and close separate application inputs. Its flush observation means
that the transport finished prior writes, not that the remote application received the response.
Delivery may occur after the sender closes. The example observes one pending resolution and checks
that it occurs at most once.

`lost-ack` is a bounded rejection path from RFC 3261 section 17, with T1=500 ms, T2=4000 ms,
T4=5000 ms and an unreliable transport. The first ACK is lost. Timer G retransmits the rejection,
the client re-ACKs without another notification, and the ACK cancels G/H. Timer I and Timer D end
the two sides independently. This finite example deliberately omits provisional responses,
INVITE retransmission before a final response, transaction matching from wire bytes and successful
INVITE behavior (RFC 6026). It is not a complete SIP conformance specification.

The authoring subset includes String, Boolean and signed 64-bit Integer fields, typed local
assignments, equality guards, finite states, fixed timers and capped backoff, channel FIFO/order
and permitted loss/duplication. Message exchange/logical identities are recorded separately from
transmission occurrences. Dynamic session epochs and guards over envelope identities are deferred;
an adopter needing them must explicitly model typed payload fields or extend the admitted format.

Exploration uses all enabled zero-field inputs and, for typed inputs, only the finite witnesses
provided by `--inputs` as a JSON action array. It searches schedules and preserves nondeterministic
transitions. Bounds exhausted, unsupported target capabilities, empty evidence and unfinished queue,
flush or timer obligations are inconclusive. A passing trace proves the specified finite observation
boundary only. There are no unbounded liveness or universal distributed-clock claims.

See [the binding design](../../docs/design/protocol-specification.md) for ownership and compatibility.

The exploration report names its timing policy: inputs are offered initially and at scheduled timer
deadlines. Intermediate-time inputs are not exhaustively enumerated; exercise those with explicit
`advance_to` actions in replay scenarios. The time bound limits clock advancement, not future timer
deadlines: an armed timer beyond the horizon may still be canceled inside it.
