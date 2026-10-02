# Event payload roots for standalone data libraries

An event already declares the typed record a producer publishes. Realizing that record must not
require a second, manually synchronized struct. This capability selects existing event payloads;
it changes no authored ESS syntax, command outcome, binding, broker or batching contract.

`ess generate types --event QUALIFIED_EVENT` selects an event payload. Repeat `--event` or combine
it with existing `--root QUALIFIED_TYPE` to share a transitive closure. `--root` remains type-only.
`--all-types` remains exclusive and selects only named types. Missing and wrong-kind names refuse
before output publication. There is no implicit selection of every event.

The shared `ModelTypes` selection gains typed `ModelRoot::Type` and `ModelRoot::Event` identities.
The existing `select` API remains the type-only compatibility entry point. Event payloads reuse
the same `Message::of_event` and schema mapping as JSON Schema and AsyncAPI. Selected event fields
contribute their complete reachable type closure, preserving wire names, closed-object behavior,
optional-field presence, nested structure, nominal newtypes and Binary64 codec metadata.

Schema definition keys remain qualified names. Current source admission gives each qualified name
one owner across construct kinds (`ess-domain/src/system.rs`, `Claim::refuse`); an older AsyncAPI
module comment saying names can collide across kinds does not override this checked boundary.
Projection still detects conflicting definitions before insertion. Event and reachable struct
wire-name collisions, and shared host declaration-name collisions, refuse before publication.
No order-dependent suffixes or synthetic type declarations are introduced.

Provenance starts with real event references and the selected type closure. A change to a selected
event or a reached type changes its contract digest; an unrelated declaration does not. The source
digest still identifies the source input, and the schema projection digest binds the projected bytes.

Type-only and imported-bundle outputs retain `ess-types-report/3` and their existing bytes.
Selections containing an event write `ess-types-report/4`, adding `model_roots`: the ordered list
of selected `{kind: type|event, name: QUALIFIED_NAME}` identities, excluding merely reached types.
The existing `roots` array still holds selected schema-definition keys. `declarations` maps the
complete closure to host names. The report remains serialize-only accounting; it is not a new
specification input or a declaration that every target runtime constraint has been discharged.

Go, Rust and TypeScript consume the same sealed structural plan. Rust and Go codec tests must
compile emitted libraries and exercise actual event wire serialization, optional absence/null
boundaries and exact numeric behavior. TypeScript must compile with strict optional-property
semantics and retain its explicit validation and numeric-precision obligations. Event Binary64
fields use the same checked original-token codec authority as type-root fields.

Acceptance includes exact agreement with the existing event schema, shared mixed-root closure,
unchanged old type selection bytes, missing/wrong-kind roots, output preservation on refusal,
wire/host collisions, typed provenance sensitivity, direct and nested Binary64 metadata and actual
generated libraries on all three targets. This tooling projection adds no conformance vocabulary;
an existing model and synthesized-suite control remains byte-identical.
