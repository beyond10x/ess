---
title: Generate contracts and documentation
sidebar_position: 5
description: Derive repository docs, static-site source, JSON Schema, OpenAPI and AsyncAPI from a specification, keep the committed output drift-checked, and know what a projection cannot carry.
---

# Generate contracts and documentation

Once a specification validates, five projections derive from it. Everything on this page is
deterministic: the same source produces byte-identical output, so generated artifacts can be
committed, reviewed and drift-checked like source.

## The five projections

```shell-session
$ ess generate --path examples/billing --kind openapi
openapi/email-service.yaml — 4608 byte(s)
openapi/invoice-service.yaml — 21077 byte(s)
2 artifact(s), nothing written
```

| `--kind` | Output | Why it exists |
|---|---|---|
| `docs` | Markdown with Mermaid diagrams (lifecycles as state diagrams, bindings as flowcharts) | the cheapest completeness check: a construct with no rendering is a hole in a page a person reads |
| `site` | HTML pages, navigation, CSS and a locally bundled Mermaid renderer | a browsable projection of the same documentation model |
| `schema` | JSON Schema for command inputs, messages, named types, and entities | the type system projected without losing its distinctions — newtypes stay separate definitions |
| `openapi` | one OpenAPI 3.1 document per component | the specification *is* the HTTP contract, not a document beside it |
| `asyncapi` | one AsyncAPI 3.0 document per component | the same for messaging, including what happens when a binding fails |

Omit `--kind` and all five are produced together. The CLI prints the exact artifact inventory; it
does not hide a projection that produced nothing.

Without `--out` you get a listing and nothing is written — a command that looks read-only does not
write into whatever directory you happened to be in. **`--out` names the root of the tree, not one
projection's directory**: each artifact's path already begins with its projection, so the committed
output of the whole set is one command.

```shell-session
$ ess generate --path examples/billing --out target/projections
```

Every artifact carries provenance: specification version, the digest of the resolved model, a
separate digest of the model slice it derives from, and the regeneration command. The model digest
is over the *model*, not the source files, so it does not move when a comment does — a digest that
moves for no reason is one every reader learns to ignore.

See [the worked example](../examples/specification-to-contracts.md) for one command's source next to
each generated document.

## What `site` means

`site` renders the same document IR as `docs`, without parsing generated Markdown back
into a second description of the model. It emits HTML pages and local presentation assets.
For example:

```shell-session
$ ess generate --path examples/billing --kind site --out target/projections
```

produces `target/projections/index.html`, the domain pages, the interaction and topology pages,
and `target/projections/assets/` with CSS, the bundled Mermaid renderer and its licence.

The `README.md` beside the specification supplies the front-page introduction. Additional
authored Markdown pages are explicitly included with `--include <page-id>=<path>`,
for example `--include operations/runbook=docs/runbook.md`. The page id determines
the output location, in this case `operations/runbook.html`.

Authored `mermaid` fences and model-generated diagrams use the same local renderer.
Their escaped source remains readable when JavaScript is disabled; non-Mermaid fences
remain code listings. This authored-fence support is an unreleased change.

The unreleased publisher resolves relative links using the included documents' source
locations, even when their output paths differ. `--front-page docs/README.md` overrides
the default front page without moving it. Declare UTF-8 downloads explicitly, for example
`--asset contracts/api.json=contracts/api.json`; they are copied verbatim, and their
destinations cannot collide with generated pages or assets. Binary downloads are refused.

Use `--strict-links` to fail on local targets that are not included pages or declared
downloads. Diagnostics name the authored file and line, and no output is written on this
failure. Without that flag, unpublished links retain their original destinations for
compatibility. The publisher never discovers or copies unselected sibling files. Queries
and fragments survive source-to-output mapping; strict mode checks fragments against
the generated and authored page headings. Download fragments retain their file-format
meaning and are not interpreted as HTML anchors.

This is **specification to documentation**, not documentation to specification. ESS still accepts a
typed ESS YAML document or directory as input; it does not parse prose or Markdown into model
semantics. The projection emits a static site, but does not deploy it or configure hosting.
Use the `docs` projection instead when another documentation system owns presentation.

## Schema-only Bundles

The unreleased `schema import-bundle` adapter selects named structural contracts from
`components/schemas` without asserting that the envelope is a valid HTTP service.
Its structural dialect must be supplied explicitly:

```sh
ess generate schema import-bundle --path components.json --component Settings \
  --dialect draft-2020-12 --out settings.bundle.json
ess generate schema project-bundle --bundle settings.bundle.json --root Settings \
  --schema-id urn:example:settings:1 --out settings.schema.json
ess generate schema validate-bundle --bundle settings.bundle.json --root Settings value.json
```

Repeat `--component` to select additional roots. The import retains the original source
bytes, their SHA-256, declared and selected dialects, explicit adapter accounting and
the complete reference closure in `ess-schema-bundle/1`. Reload reimports that source
and checks the entire result; changing a projected definition or dropping qualifications
does not become a successful checked import. This checks integrity, not an author's signature.

Projection emits only the chosen root's transitive closure. URI/JSON Pointer references
are rewritten only at schema positions, never inside defaults or examples. Constraints,
nullability, unions, tuple schemas and object openness retain JSON Schema 2020-12 semantics.
Defaults and formats are annotations; validation neither fills missing fields nor enables
the optional format-assertion vocabulary. `x-ess-source` records the projected contract's
qualification. Unknown keywords, nested resource/anchor declarations and unresolved or
references outside the selected source collection are refused without a partial successful artifact.

When the source is a complete JSON Schema document, keep its root as well as its
definitions. The unreleased document adapter assigns an explicit root identity:

```sh
ess generate schema import-document --path record.schema.json --root Record \
  --definition Settings --dialect draft-2020-12 --out record.bundle.json
ess generate schema project-bundle --bundle record.bundle.json --root Record \
  --schema-id urn:example:record:1 --out projected.schema.json
```

`--definition` is optional and repeatable; it admits additional independently
selectable `$defs` roots. References from the document root are included automatically.
Local `#` recursion and `#/$defs/Name` references retain source identity. The root name
must not collide with a source definition. `$id`, anchors, nested resources, external
references and contradictory declared dialects refuse rather than change resolution.
This adapter writes `ess-schema-bundle/2`, whose explicit `document_root` distinguishes
the source root from its definitions. Component imports still write unchanged `/1`
envelopes; old strict readers refuse `/2`. Both formats support replay, root-selected
validation and all three data targets. Findings point to original schema positions,
including the empty JSON Pointer for the document root.

This does not generate application behavior or claim the separate TypeScript projector
supports every imported construct. It also does not reinterpret the existing OpenAPI
service importer, which keeps its own strict dialect and interface subset.

## Accounted Data Types

The unreleased bundle realization path selects roots and their complete closure
before emitting declarations:

```sh
ess generate schema types-bundle --bundle settings.bundle.json --root Settings \
  --target typescript --out target/settings-types
ess generate schema types-bundle --bundle settings.bundle.json --root Settings \
  --target rust --package settings_types --out target/settings-rust
ess generate schema types-bundle --bundle settings.bundle.json --root Settings \
  --target go --package settingstypes --module example.org/contracts/settings \
  --out target/settings-go
```

Each target emits declarations (`types.ts`, `types.rs` or `types.go`),
`types-report.json` and the replay-checked `source.bundle.json`. Native targets add
`Cargo.toml` or `go.mod` for standalone library builds.
Repeat `--root` for multiple admitted roots. The report carries source and bundle
digests, generator version, typed target configuration, declaration names, retained annotations and source-located
runtime obligations. Name collisions, unsupported shapes and unguarded alias cycles
refuse before output is written. This path does not change the legacy
`schema typescript` command or full application synthesis.

TypeScript preserves required versus nullable fields, JSON wire keys, recursive
objects, open objects, union/intersection shapes and prefix tuples. Use `strict`
and `exactOptionalPropertyTypes`. The recursive JSON-value helper appears only as
the meaning of unrestricted source JSON, not a fallback for unsupported semantics.
Numeric precision, integer membership, exact object closure, exclusive unions and
schema refinements remain explicit validation obligations. Defaults are not applied.

Rust and Go supply typed JSON codecs with independent presence/null representations,
exact JSON numbers, recursive records, retained typed extra fields, finite string enums,
unions and fixed prefix tuples. Native unions select the first structurally decodable
branch; schema exclusivity and refinements remain report obligations, not application
dispatch. Unsupported native intersections, non-string literals and variable prefix
layouts refuse before publication. Go uses only its standard library; Rust pins Serde
and serde_json with arbitrary-precision number support in the generated manifest.

The `ess-types-report/3` envelope records typed input provenance, the language and explicit native package/module
configuration. These are data libraries, not complete schema validators or application
decoders: defaults, aliases and external module dispatch are not applied.

### Model Types

Select qualified roots directly from a resolved ESS model using the same target options:

```sh
ess generate types --path examples/billing --root billing.invoice.Money \
  --target rust --package billing_types --out target/billing-types
ess generate types --path examples/billing --all-types \
  --target typescript --out target/billing-typescript
```

Repeat `--root` for a shared closure, or explicitly choose `--all-types`, never both.
The output directory must be outside the input specification tree. The model path
reuses ESS's existing JSON wire mapping: decimal strings, wire field names, optional
object properties, nullable collection values, map-key spellings and adjacent tagged
unions. Missing roots and colliding wire field names refuse before output.

`source.schema.json` retains the exact selected definitions and model provenance;
the report identifies the system, specification version and model/contract/projection
digests separately from bundle identity. This is a definitions document, not a root
instance validator: choose the relevant `$defs` reference for schema validation.
Invariant predicates and map-key grammars remain explicit validation obligations.
TypeScript's structural aliases do not enforce nominal model identities, which its
report names for each selected newtype. Full synthesis remains unchanged. Model-field
bindings to imported schemas and application-specific decoder semantics remain pending.

### Explicit Normalization

The unreleased Rust library `schema_contract::realize::normalize` separates
transformations from structural declarations. `Root::pin` identifies an admitted
schema-bundle root by the complete canonical bundle digest. `Plan::read` checks a
strict `ess-normalization/1` recipe against supplied checked bundles; `Plan::run`
selects an explicitly named external branch and evaluates its ordered stages.

Each stage validates input, checks authored requirements, constructs a new value and
validates output. Missing fields remain distinct from null. Defaults apply only via
explicit fallback or conditional expressions; zero, false and empty strings are not
absence. Scalar equality, exact case-sensitive string-prefix selection,
signed-integer arithmetic with explicit overflow policy,
ordered choices, list mapping and distinct string-category counts have defined
semantics. There is no trial dispatch or implicit default branch. Duplicate authored
map keys refuse rather than overwrite earlier operations.

Check a recipe and run one branch through the same library:

```sh
ess generate schema normalize-check --recipe settings.normalize.json \
  --bundle stored.bundle.json --bundle runtime.bundle.json --out checked.normalize.json
ess generate schema normalize-run --recipe checked.normalize.json \
  --bundle stored.bundle.json --bundle runtime.bundle.json \
  --branch primary --input settings.json --out normalized.json
```

Omit `--out` for JSON-only stdout. Repeat `--bundle` for all source identities used
by the recipe; bundles are replay-checked, and missing or changed identities refuse.
No successful value is written on a failed check, requirement or schema boundary.
Outputs cannot replace the recipe, a bundle or the input instance. Existing output
link/alias containment checks apply.

The input JSON boundary rejects duplicate object keys, excessive nesting, trailing
data, and numeric tokens that cannot round-trip through the reference representation
without changing their decimal value. Integer literals within signed range retain
their exact representation; fractional/exponent tokens are not silently made eligible
for integer operations. This is precision-loss refusal, not arbitrary-precision
arithmetic. Raw JSON library consumers use `Plan::run_json`; `Plan::run` accepts an
already-decoded value and cannot recover precision lost by the caller's parser.

This is a checked recipe workflow and reference evaluator, not yet a
Go/Rust/TypeScript adapter generator. Expanded recursive shapes, tuples and
intersections refuse in its initial checker. Schema bounds and other refinements are
checked at runtime boundaries, not proven by structural checking. Integer operations
require exact signed-64-bit integral JSON tokens; equality is scalar-only, with no
floating-point or cross-kind coercion. Full language-target normalization and
source-adapter adoption remain unfinished.

## The graph, without generating a tree

`ess specify graph` prints the actor/command/event picture the generated docs open with:

| `--format` | Output |
|---|---|
| `dot` | Graphviz, for `dot -Tsvg` |
| `mermaid` (default) | a `flowchart`, unfenced — redirect into a Markdown file or paste into a PR |
| `json`, `yaml` | the nodes, edges and groups themselves — 13 nodes, 7 edges and 3 groups for `examples/billing` |

One renderer produces both the CLI's diagram and the documentation's: `ess specify graph --path
examples/billing --format mermaid` emits exactly the bytes fenced under *The system as a graph* in
`generated/docs/index.md`, and a test compares them, so the two cannot drift.

## Drift-checking in CI

Commit the generated output and regenerate in CI:

```shell-session
$ cargo xtask generate --check    # committed projections still match the specification?
projections are up to date
```

The check runs the public `ess generate` command, compares every owned file byte for byte, and also
fails on a committed file no projection produces any more. Run `cargo xtask generate` to reconcile
the tree, review the resulting diff, and commit it with the specification change. Structural
synthesis, conformance, schema, and infrastructure fixtures are exercised by their Rust tests in
`task check`; this repository does not advertise repository commands it does not ship.

## What a projection can quietly destroy

Two questions to ask of any generated artifact, answered honestly for these:

* **A newtype collapses on the wire.** `billing.invoice.Email` and `billing.email.EmailAddress`
  stay separate schema definitions — each carries `"x-ess-kind": "newtype"` and its own name — so
  code generators emit two types. But both are `"type": "string"` on the wire, and a payload with
  the two values swapped validates clean. JSON Schema constrains structure; it cannot carry nominal
  identity.
* **A command's HTTP path is a convention.** The model has no `exposures:` construct, so
  `/invoices/commands/create-invoice` is a shape the generator chose — written into the generated
  document's own `info.description` rather than left for a reader to infer.

And one check that is scoped rather than total, stated per projection:

| projection | what is checked | what is not |
|---|---|---|
| `schema` | every document is validated against the real JSON Schema 2020-12 meta-schema and built into a validator | — |
| `openapi` | every **embedded** schema is validated against the same meta-schema, because OpenAPI 3.1's dialect *is* 2020-12 | the envelope, checked against an enumerated list by hand |
| `asyncapi` | the envelope is checked as a skeleton: version, `info`, `channels`, `operations`, and every operation's `action` | the payloads. They are AsyncAPI Schema Objects and declare no `schemaFormat`, so validating them against 2020-12 would assert a dialect the document does not claim |

What closes the two gaps is vendoring the OpenAPI 3.1 and AsyncAPI 3.0 meta-schemas, which is an
open decision rather than an oversight: neither ships with anything here, and a test may not fetch
one — the validator is built with `default-features = false` and has no retriever, so it could not
reach the network if it tried.
