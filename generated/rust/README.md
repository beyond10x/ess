# Synthesised Rust workspaces

**Do not edit these files.** They are synthesised from the specifications under
[`examples/`](../../examples) by `ess generate synthesize`
([refresh procedure](../../docs/design/synthesis-fixture-maintenance.md)), and CI fails if they differ from what
the specifications determine — or if a workspace stops compiling.

A workspace here is the part of an implementation that was never anyone's to write: the
types, the states whose illegal transitions do not compile, the contracts, one crate per
component holding its port, and one system crate holding the bindings and the one transport
the specification's own delivery words require. What remains deliberately unwritten is each
workspace's `PLAN.md` — every capability of the specification with exactly one disposition:
generated, an obligation carrying its contract, or a refusal carrying its reason — and every
obligation is also a typed stub in the workspace, refusing with a value that names it.
`Cargo.lock` and `target/` inside a workspace are written by `cargo check` and are not part
of the committed tree.

The other half of the bargain is hand-written, and lives outside this tree because the
ownership boundary is absolute: [`examples/billing-realization`](../../examples/billing-realization)
implements each obligation against its contract, and its linker assembles components and
implementations into a runnable system without ever choosing — zero implementations for an
obligation is an unsatisfied obligation, two is an ambiguity error naming both (gap register D-2,
whose home is [`the linker never chooses`](../../docs/design/linker-never-chooses.md)).
The realization tests execute the committed conformance suite, unchanged, against
that linked system: 34 of 34 scenarios must pass, and the deliberately corrupted variant
beside the honest one must fail exactly the scenario that exists to catch it.

## The second transport, and the record two applications write

A component whose specification says `reached_by: network` has callers that are not
deployed with it, so its surface exists on a wire. *Which* wire is derived rather than
chosen: this repository projects exactly one contract for a command surface — the OpenAPI
document under [`generated/openapi/`](../openapi) — and an OpenAPI document is an HTTP
contract, so a server speaking anything else would contradict the document committed beside
it. The emitted surface answers exactly the paths that document declares, plus
`GET /openapi.json` and `GET /docs`, which serve the committed contract and the committed
prose byte for byte. A path the contract does not declare is a 404; a declared path under
another method is a 405; neither is a status the contract declares, because both are facts
about a transport rather than about a command.

**The startup record.** Every served application writes three lines of JSON to standard
output before it answers anything, and every member of them is derived from the
specification — except `runtime`, which is the process's own: the language it was
synthesised into, the address it bound and the port it took.

| line | carries |
| --- | --- |
| `system.starting` | the system, its version, the model digest, the contract digest, every component, and the plan's disposition counts |
| `surface.serving` | the served component, its declared reach, the transport, the number of routes, and every route as method, path, what it serves and the construct it serves |
| `system.ready` | the system, and how many surfaces this process serves |

Two applications synthesised from one specification must agree on every byte **outside**
`runtime`. The HTTP projection test compares the static startup records embedded in generated
Rust and Go source. It does not start the applications or compare their process output.

The generated plans record the current model/contract digests and disposition counts.

| workspace | specification | plan |
| --- | --- | --- |
| [`billing/`](billing) | [`examples/billing`](../../examples/billing) | [`billing/PLAN.md`](billing/PLAN.md) |
| [`gatepass/`](gatepass) | [`examples/gatepass`](../../examples/gatepass) | [`gatepass/PLAN.md`](gatepass/PLAN.md) |
