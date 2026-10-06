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

## Typed attributes on the variant object form (ess/23, beyond10x/ess#450)

The object form gains `attributes:`, a typed value per attribute the enum declares:

```yaml
- name: demo.plans.Plan
  kind: enum
  attributes:
    - {name: max_seats, type: Integer}
    - {name: sso, type: Boolean}
  variants:
    - {name: Basic, attributes: {max_seats: 5, sso: false}}
    - {name: Team, attributes: {max_seats: 50, sso: true}}
```

- **Declaration.** `attributes: [{name, type}]` on the enum, the shape of `fields:`. A type is a
  `Boolean`, `Integer`, `Decimal` or `String`, a newtype of one, an enum, or an `Optional` of these;
  a `List` or `Map` is refused by name. Below `ess/23` the declaration is
  `unsupported_format_version`.
- **Values.** Each variant fills every attribute that is not `Optional`, by the rule `sets:` types
  a literal by (`literal_representation`, reached through the `LiteralSource` the hint names): a
  missing value is `missing_declaration`, a value for an undeclared attribute
  `undeclared_reference`, a value that is not one of the type `type_mismatch`.
- **Where it is held.** `EnumVariant::attributes` carries every declared attribute in declaration
  order with the variant's value, so the declaration is read off any variant and neither
  `TypeBody::Enum` nor the IR's `ResolvedBody::Enum` changed shape. A variant that declares no
  attributes keeps its bytes: bare, or the naming map it always was.
- **Reading.** A predicate reads `<fact>.<attribute>` and is lowered to membership over the
  variants that satisfy it where each authored predicate is resolved
  (`ess-domain/src/expression/attributes.rs`): a comparison with a literal, `any_of`, `none_of`,
  `defined(…)` and truthiness become `any_of` over variants; a comparison with another fact
  expands to one branch per variant within 128 predicate nodes. Which variants a read holds for
  is the predicate evaluator's own answer for each variant's value, truthiness included; a variant
  that leaves an `Optional` attribute unfilled is `Unknown` and in neither a read nor its
  negation, which is pushed to the reads by De Morgan. A literal that is no value of the attribute
  (another scalar, a word naming no variant of an enum-typed attribute) and an ordering over a
  `Boolean` are refused by name, as over a plain fact; what else cannot be lowered is refused by
  name too. Reading an attribute as a value (`input.plan.max_seats` in `sets:`) is refused in this
  cut.
- **Projections.** `x-ess-attributes` on the enum's JSON Schema and OpenAPI node (one entry per
  attribute: name, type, kind, and each value by the variant's wire spelling; a value of an
  enum-typed attribute is that enum's wire spelling, as its own schema lists it), a table in the
  generated documentation, and one accessor per attribute in the types-only Rust, Go and TypeScript
  outputs.
- **Revision comparison.** No released delta format has a case for an attribute, so an attribute
  declared, removed or revalued stays in the residual and is `unclassified-changed`; a guard whose
  lowered membership moved is its own `outcome-condition-changed`.
