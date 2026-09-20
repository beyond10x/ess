# An enum variant carries its own wire spelling

`story:an-enum-variant-carries-its-own-wire-spelling`.

## What was missing

`TypeBody::Enum` and `RawTypeBody::Enum` each carried `variants: Vec<String>`. `Naming` was
reachable from a domain, a command, an event, an error, a view and a field, and not from a variant.
A variant whose wire form is not derivable from its name therefore could not be declared at all.

The case that asked for it, measured against a downstream conformance target: one command whose
enum selector chooses between three route shapes. `Stop` is the empty path suffix
(`PUT /recordings/{id}`), `Flag` is `/flag`, `Tags` is `/tags`. With a declared spelling per
variant, `PUT /recordings/%s%s` is derivable. Without one, the alternatives are splitting the
command to satisfy a path, or writing the table again in every target — and a table in a generator
script is one no target can read.

Where the derivation holds nothing was wrong: `Unhold` -> `unhold` completed
`POST /outbound/%s` with no code change.

## The shape

`EnumVariant { name: String, naming: Naming }`, in `crates/specify/ess-domain/src/types.rs`.

Authored either way:

```yaml
variants: [Stop, Flag, Tags]
```

```yaml
variants:
  - name: Stop
    wire: ""
  - name: Flag
    wire: flag
```

`Naming` is reused rather than a new per-variant type being introduced, so a variant gains `wire`,
`display`, `summary` and `code` with the meanings those already have everywhere else.

### Why the bare form stays

`Serialize` writes a bare name when the variant declares nothing, and the mapping only when it
declares something. Every specification and every IR document written before this existed therefore
keeps its bytes, which is what the repository's determinism rule requires of a change that is not a
coordinated format migration.

`Deserialize` is untagged over the two forms. The mapping form denies unknown fields, so a
misspelt key is refused by name rather than dropped.

### Why the empty string is a spelling

`EnumVariant::wire()` returns an authored empty string rather than treating it as absent. `Stop`
is the route with no suffix; falling back to the variant's name there would spell `/Stop`, which
is the route that does not exist. This is the one place the type deliberately does not behave like
`Naming::wire_or`, which has no such case to serve.

## Format admission

`ess/5` — `FormatVersion::V5`, added to `SUPPORTED_FORMATS`.

A variant that declares naming under `ess/1` … `ess/4` is refused with
`ValidationCode::UnsupportedFormatVersion` at `types.<type>.variants.<variant>`, by the check in
`crates/specify/ess-domain/src/primitive_admission.rs`. This follows `story:error-wire-codes`,
which added optional `naming.wire` to errors and cost `ess/4`; its gate is the one directly above.

A bare variant list is admitted by every format, including the four that predate this.

## What reads the spelling

| Reader | Reads | Why |
|---|---|---|
| `ResolvedBody::Enum` in the compiled IR | the whole variant | a conformance target loads `ir.json` and needs the mapping the generator table used to hold |
| `ess-gen` JSON Schema `choices` | `wire()` | a generated schema describes the document on the wire |
| `ess-cli-contract` | `name()` | that shape is what an operator types at a command line |
| `ess-synth`, `ess-conformance` | `name()` | unchanged behaviour; consuming the spelling in the emitters is follow-up work |

For a variant that declares nothing all four are the same string, so no existing model moves.

## What a revision comparison reports

`ess-diff/5`, and three cases on `TypeChange`:

| Case | Reported when |
|---|---|
| `VariantWireNameChanged` | a variant both revisions declare is called something else on the wire |
| `VariantDisplayNameChanged` | its display name moved |
| `VariantSummaryChanged` | its one-line summary moved |

A variant's own name does not move when its wire spelling does, so the variant set and the variant
order both say nothing about it. Before these cases existed the comparison returned an empty delta
for a change every deployed consumer breaks on.

`SemanticChange::minimum_format` maps all three to 5, so a delta carrying one is refused by a
reader of `ess-diff/1` … `ess-diff/4` rather than being read with a case it does not know. This is
the mechanism `story:error-wire-codes` installed for `ErrorChange::WireNameChanged` at
`ess-diff/4`.
