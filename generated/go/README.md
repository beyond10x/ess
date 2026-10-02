# Synthesised Go modules

**Do not edit these files.** They are synthesised from the specifications under
[`examples/`](../../examples) by `ess generate synthesize`
([refresh procedure](../../docs/design/synthesis-fixture-maintenance.md)), and CI fails if they differ from what
the specifications determine — or if a module stops being `gofmt`-clean, stops compiling, or
stops passing `go vet`.

This tree is the **second emitter** behind the synthesis seam, and the reason it exists is
that a claim about language-neutrality is worth exactly one test. Go was chosen because it
has no sum type: every tagged union, every enum and every command outcome has to be encoded
by hand — as a sealed interface, one unexported marker method and one struct per variant — or
refused out loud. The plan did not change to admit it: each module's `PLAN.md` and `plan.json`
are **byte-identical** to the ones in [`../rust`](../rust).

What Go holds more weakly than Rust, and what it cannot represent at all, is in each module's
`TARGET.md` — never in the plan, because a weakening is a fact about a language and the plan
is a fact about the model. Standard library only, and a module path under the reserved
`example.invalid` domain, so nothing here can be mistaken for something publishable.

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

The Go half of a served surface is `net/http` and `encoding/json`, both standard library, and
generated codecs beside them: a generated type carries an unexported field, which
`encoding/json` cannot see, and exporting it would undo the distinctness the newtype encoding
exists for. The hand-written realization that links into it is a module of its own —
[`examples/gatepass-go-realization`](../../examples/gatepass-go-realization) — reaching this
tree through a filesystem `replace`, so nothing here resolves over a network either.

The generated plans and target notes record current digests, disposition counts and target gaps.

| module | specification | plan | target notes |
| --- | --- | --- | --- |
| [`billing/`](billing) | [`examples/billing`](../../examples/billing) | [`billing/PLAN.md`](billing/PLAN.md) | [`billing/TARGET.md`](billing/TARGET.md) |
| [`gatepass/`](gatepass) | [`examples/gatepass`](../../examples/gatepass) | [`gatepass/PLAN.md`](gatepass/PLAN.md) | [`gatepass/TARGET.md`](gatepass/TARGET.md) |
