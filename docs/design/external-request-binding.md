# Request-bound external decisions

An external authorization may be valid for one expiry, payee, amount or evidence reference and
invalid for another. A context callback receiving only the command and outcome names cannot
distinguish two direct generated calls that differ in those fields. Checking a request around
one transport leaves another generated entry path with the same missing information.

ESS412 retains the existing authored external outcomes and gives the required Rust and Go
context ports the actual typed command input. This is a generated API change, not new authored
syntax, an IR or suite format change, a generic metadata registry, or authentication supplied by
ESS. Implementation and executable review must establish this design before adoption.

## Required context ports

Rust generates a closed borrowed `ExternalCommand<'a>` enum in the behavior module, with one
variant carrying a reference to each input type whose generated behavior asks an external
question. `name()` returns the canonical qualified command identity. The required method is:

```rust
fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool;
```

The callback sees the actual borrowed command without JSON conversion, allocation or loss of
integer, optional or nested values. It cannot mutate that input through the shared reference.
The context implementation compares it with its trusted request-bound decision and refuses
missing or mismatched proof as required by the declared external outcome. ESS does not choose
an authorization policy or supply a permissive default.

Go generates a closed `ExternalCommand` interface with `Name() string` and an unexported marker.
Concrete generated wrappers carry each command's typed input value. The required method is:

```go
External(command ExternalCommand, outcome string) bool
```

A context can switch on the concrete wrapper and inspect its typed input. Go's ordinary nested
pointer, map and slice aliasing remains; this interface does not promise Rust borrow checking
or an immutable deep copy. Implementors must not mutate shared request state through aliases.

Both targets retain the existing mutable context receiver and external branch order, including
input-guarded external branches and first-true selection. Generated local input, addressed-row
and stored-field guards retain their existing precedence. No extra callback asks about a
command that returns at an earlier local refusal. The current request accompanies every
external callback that actually runs.

The fallible companion port carries the same typed command. Rust's `TryContext::try_external`
and Go's `FallibleContext.TryExternal` take `ExternalCommand` and the outcome name; generated
behaviour calls only the companion. The Rust blanket adapter and Go's `readExternal` pass the
command to `Context.external` / `Context.External` unchanged. The generated server's
`MemoryPorts` and `MemoryContext` accept it and still refuse with `external branch answer`. Neither
port keeps a names-only form.

## Names and emission

Allocate request variants/wrappers once in a deterministic map keyed by canonical qualified
command name. Flattening dotted or underscored qualified names into PascalCase is not injective;
reserve candidates before allocating collision suffixes, and account for the generated namespace.
Use that same allocation for declarations, canonical names and callback call sites. Payload paths
come from the existing Rust Layout and Go Emit/lookup mechanisms, including `naming.code`, rather
than a second reconstruction of package or type names. Go call sites use the already allocated
input local so authored package names cannot shadow it.

Only commands whose actually generated implementation calls an external branch appear in the
request type. A model with no such calls emits neither this type nor the required external
method, including one whose external command remains wholly owed. This avoids an unused Rust
lifetime and avoids imposing a callback on unrelated implementations. Mixed generated/owed
models include only the generated calls. Rust crate and workspace layouts obey the same rule.

## Adoption and compatibility

Regeneration intentionally breaks existing Rust and Go context implementations until they
migrate to the required input-bearing signature. There is no old-signature fallback that silently
treats a missing decision as false or allows a request. Existing generated artifacts are unchanged
until regeneration. Adding an external command may also require an exhaustive Rust match or a Go
consumer's explicit handling policy to change; an unrecognized case must not become implicit
authority.

Passing typed input supplies facts, not trust. Consumers still establish actor authority,
freshness, immutable evidence, command-local clock policy and atomic serialization against the
state they authorize. A context intended for request A must compare the actual executing request
B before answering. A callback implementation that deliberately ignores input remains capable
of implementing an unsafe local policy; ESS cannot authenticate the outside world for it.

Web hosts generated Rust behavior and therefore shares this port contract. There is no separate
TypeScript product-behavior target in this generator; TypeScript schema and conformance emitters
serve different contracts. No conformance-synthesizer, interpreter, schema or authored behavior
semantics change is implied by this API correction.

## Deciding evidence

A brand-free lease renewal fixture must first compile against the old names-only API and fail
an outcome/storage/event assertion because a present decision for one request admits a substituted
request. A future API compile failure is not this semantic RED. Preserve those exact assertions
while explicitly migrating the fixture context to compare the current typed request.

Matching proof succeeds; missing or changed expiry/evidence/adjacent-large-integer/optional/nested
values refuse without storage or event effects. Exercise direct generated calls and HTTP paths in
Rust's two layouts and Go. Check local guard precedence, ordered multiple external callbacks,
collision suffixes and code aliases, no-use and owed-only emission, and deterministic generation.
Separately establish that old context signatures fail compilation with the specific API mismatch.
Existing compiled harness behavior and assertions remain intact while their signatures migrate.

## Supported profiles and component locals

The callback allocator is tested with colliding canonical names and distinct payload aliases on
an embedded/direct surface. The existing Rust network surface explicitly refuses those names
when its codec identifiers collide (ESS415); this change does not claim to admit that surface.
A separate served alias-only control and the original served request-binding tests retain HTTP
coverage. No codec or target-feasibility rule changes here.

Go component handlers reserve every local name against generated package names before emitting
receiver, input, result and event-extraction expressions (ESS414). They use the existing ordered
package-name contract and underscore allocator. A real-domain matrix exercises all five local
bases; an explicit synthetic reservation test exercises repeated suffixes, because the current
package-name normalizer removes authored underscores. Ordinary unreserved component bytes stay
unchanged. Full generated modules and real handler/outbox behavior remain part of the check.
