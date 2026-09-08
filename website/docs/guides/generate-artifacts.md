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

## Repeated generation and recovery

Output ownership is an unreleased source change. Generation records the files it owns beneath
the output root in `.ess-output`. Repeating a command replaces that owner's files and removes
its obsolete files, while preserving authored neighbours. A selected projection updates only
its own files; generating all five projections publishes their combined change as one
recoverable operation. Changing a synthesis or data-library target replaces that generator's
previous file set.

First generation refuses a destination file it does not already own, even when the bytes match.
For existing output, generate a reference in a separate fresh directory, then use
`ess output adopt` to enroll the chosen owner. Adoption requires exact reference bytes for every
existing file it enrolls; unrelated files remain unowned. Use `ess output adopt --help` for the
owner selector. Keep the reference and destination directories separate, with neither inside
the other. An edited or interrupted reference cannot authorize adoption.

For example, adopt only the documentation projection in an existing tree:

```sh
ess generate --path examples/billing --kind docs --out target/reference-docs
ess output adopt --ownership-root target/projections \
  --from target/reference-docs --owner projection:docs
```

Repeat adoption for each generator family you intend to manage. Adoption accepts first enrollment
or an exact repetition of that owner's inventory; it cannot discard an existing inventory by
using a smaller reference. Standalone file owners additionally require `--file NAME`, matching
the generated filename in both directories.

If a write is interrupted, generation identifies the pending output root and refuses a new
operation. Recover it explicitly before generating again:

```sh
ess output recover --ownership-root target/projections
```

Recovery uses the recorded operation, so it does not require the original model inputs. Before
commit it restores the actual previous files, including prior edits or missing owned files.
After commit it keeps the complete new result and finishes cleanup. Recovery can itself be
repeated after interruption. Preserve `.ess-output` and any reported recovery files until
recovery finishes; copying or deleting the state directory is not an ownership transfer.

This contract applies to cooperating ESS writers on one local mounted filesystem on Linux or
macOS, with controlled parent directories. Each file replacement is atomic; readers can see
intermediate changes across several files. Synchronization failures are reported and retained
for recovery. Storage must honor synchronization calls; this is not an unconditional guarantee
against hardware failure. Older ESS versions do not participate in this protocol.

No-output and check modes do not create ownership state or recover pending writes. The
repository's separate `cargo xtask generate` inventory refuses an intersecting ownership tree
before writing or pruning it.

## Generated schemas in a local registry

The generated contract schemas and `schemas/generated/ess.schema.json` have no root `$id`.
They are usable by schema tooling, but passing either original to `ess generate schema validate`
as a registry resource produces `missing_schema_id`. ESS does not assign them a public schema
endpoint. To use this registry, the adopter chooses resource IDs and supplies the resources locally.

This example runs from an ESS repository checkout with `ess` and `jq` installed. It reads the
committed generated schemas and creates separate copies, adding only a root `$id` to each.
The supplied application envelopes and instances live under
`crates/edge/ess-cli/tests/fixtures/schema-resource-identity`:

```sh
sample=target/schema-resource-example
fixtures=crates/edge/ess-cli/tests/fixtures/schema-resource-identity
mkdir -p "$sample/registry" "$sample/instances"
cp "$fixtures/registry/invoice-selector.schema.json" \
  "$fixtures/registry/source-selector.schema.json" "$sample/registry/"
cp "$fixtures/instances/create-invoice.json" \
  "$fixtures/instances/ess-source.json" "$sample/instances/"
jq --arg id urn:example:billing-create-invoice:1 '. + {"$id": $id}' \
  generated/schema/commands/billing.invoice.CreateInvoice.schema.json \
  > "$sample/registry/invoice-resource.schema.json"
jq --arg id urn:example:ess-source-syntax:1 '. + {"$id": $id}' \
  schemas/generated/ess.schema.json > "$sample/registry/source-resource.schema.json"
ess generate schema validate "$sample/instances" --schemas "$sample/registry" --format json
```

The command exits 0 with four schema resources, two accepted instances and no issues. The accepted
records name `urn:example:invoice-submission:1` and `urn:example:ess-source-submission:1`, with their
selector-schema filenames. They identify the selected envelopes; the payload resources are reached
through each envelope's `$ref`. Filenames are not registry identities.

For example, `invoice-selector.schema.json` is a separate application resource:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:example:invoice-submission:1",
  "type": "object",
  "required": ["schema", "payload"],
  "properties": {
    "schema": {"const": "urn:example:invoice-submission:1"},
    "payload": {"$ref": "urn:example:billing-create-invoice:1"}
  },
  "additionalProperties": false
}
```

Its instance contains exactly `schema` and `payload`. Nest the original payload value unchanged:
injecting a selector into the strict invoice command object would violate that command schema.
This preserves the payload's JSON value, not its original whitespace or number spelling. It is an
application envelope example, not a universal ESS envelope or conformance-evidence format.

Keep both payload resources separate from their envelopes. The syntax resource retains draft-07
and its `#/definitions` references; the command resource retains draft 2020-12 and `#/$defs`.
Inlining an ID-less schema under `properties.payload` would change where those root fragments
resolve. Keep all constraints and `x-ess-*` provenance annotations when copying. Removing only the
added `$id` recovers the original parsed schema, but the copied file has different bytes: its model
and slice digests are not checksums of that new file.

These `urn:example:` IDs illustrate an adopter's immutable logical names. Choose distinct IDs for
payloads and envelopes, and define ownership, uniqueness, versioning and publication before sharing
them. The CLI detects duplicate exact ID strings in the supplied registry; it supplies no historical
immutability ledger, URI-alias equivalence guarantee, organization namespace or hosted resolver.
All resources are compiled offline before any instance is checked. A missing reference or invalid
registry leaves the report's `valid` list empty. Later instance failures can coexist with accepted
instances, so an exit of 1 does not always mean that list is empty.

Schema acceptance also does not assemble an ESS system. The syntax example deliberately names
`shop.cart` without defining that domain. Its envelope validates above, while this actual semantic
validation command exits 1 with an unsatisfied `system.domains` diagnostic:

```sh
ess specify validate --path "$fixtures/source/semantic-invalid.json" --format json
```

The separate `schema typescript` command supports a restricted structural vocabulary. The existing
`urn:example:widget:1` fixture can be projected, but the generated command's annotations and the
envelope's external `$ref` are refused before output is written. Do not strip annotations to make a
projection appear supported. Adding `$id` also supplies no original-text numeric admission or
`ModelTypes` authority; the bundle and normalization paths below keep their own contracts.

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
remain code listings. This authored-fence support is available in ESS 0.20.0.

The publisher resolves relative links using the included documents' source
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

The `schema import-bundle` adapter selects named structural contracts from
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
definitions. The document adapter assigns an explicit root identity:

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

The bundle realization path selects roots and their complete closure
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

#### Finite model codecs

Standalone Rust and Go model targets support compiler-owned `Binary64` fields
from `ess/2`. Rust emits `EssBinary64::new(f64)` with checked finite
construction and `.get()` access; Go emits `NewEssBinary64(float64)` and `.Float64()`.
Their JSON codecs retain the original numeric token through supported aliases,
recursive containers and union branches. They preserve signed zero, subnormals,
signed underflow and nearest-even rounding; wrong JSON kinds and overflow refuse.
Go's zero wrapper value represents positive zero.

Rust decoding requires original source text through `serde_json::from_str` or
`from_slice`, with the generated `raw_value` feature enabled. Deserializing an
already parsed `serde_json::Value` cannot recover original numeric spelling and
is outside this codec contract. A pure map with Binary64 values is supported;
a Rust record combining declared fields with Binary64-containing extra values
refuses as `rust_binary64_open_record` before files are written. Ordinary schema
refinements, union exclusivity and map-key constraints remain report obligations.
The helpers and required features appear only for selected Binary64 closures;
older non-Binary64 output maps stay unchanged at the same generator version.

### Explicit Normalization

The Rust library `schema_contract::realize::normalize` separates
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

The `ess-normalization/2` format adds these explicit operations to both
the reference engine and standalone Rust target:

| Operation | Meaning |
|---|---|
| `concat {parts}` | Concatenate required strings in order; no escaping or normalization. |
| `join {list, separator}` | Join required string elements, retaining duplicates and empty strings. |
| `integer_string {value}` | Render an exact signed-64-bit integer as decimal text; negative zero renders as `0`. |
| `concat_lists {lists}` | Concatenate lists in order, without recursive flattening or deduplication. |
| `item_index` | Read the original zero-based index in the innermost collection scope. |
| `select_map {list, condition, value}` | Map matching items in source order, without renumbering their indices or evaluating rejected values. |
| `find {list, condition, value, otherwise}` | Return the first matching value, without evaluating later items; evaluate fallback only on no match, in the enclosing scope. |

Empty text/list constructions return empty values. All branches and operand types
still check before execution, including unselected branches. Nested collection scopes
rebind the index; an unbound index refuses. Version 1 recipes cannot opt into these
operations without explicitly changing their format. Numeric admission is unchanged:
binary64 decoding must be explicitly declared as described below.

#### Binary64 Inputs and Integer Conversion

Version 2 optionally declares `binary64_inputs`, mapping exact branch names to
paths through their first input root. A path uses `{kind: field, name: KEY}` for
an object member and `{kind: items}` for every array element. An empty path selects
a numeric root. Only selected numeric values round to binary64, nearest with ties
to even; undeclared numbers retain the exact admission policy. Missing/null values
are neither defaulted nor coerced. Unknown branches, duplicate paths, undeclared
fields and nonnumeric leaves refuse. Underflow preserves signed zero; infinity
refuses. The declaration is forbidden in version 1, even when empty.

`binary64_to_integer {value, steps, out_of_range: reject}` explicitly converts a
required numeric value to binary64, applies ordered `multiply`, `minimum` and
`maximum` steps, then truncates toward zero into signed 64-bit representation.
Each step has a `value` containing a finite JSON numeric token **as a string**,
retaining authored decimal provenance. Multiplication rounds after each step;
intermediate infinity refuses. Minimum/maximum retain signed-zero semantics.
The final range is `[-2^63, 2^63)`, not host-dependent saturation or wrapping.
Subsequent integer arithmetic separately declares its overflow policy.

For example, a branch whose first input root is numeric can declare
`"binary64_inputs": {"primary": [[]]}` and use this stage value:

```json
{
  "op": "binary64_to_integer",
  "value": {"op": "read", "scope": "input", "path": []},
  "steps": [{"op": "multiply", "value": "1000"}],
  "out_of_range": "reject"
}
```

Input `1.001` yields integer `1000`: binary64 multiplication precedes truncation.
Exact decimal multiplication would yield a different result and is not substituted.
Schema boundaries validate the decoded value, not the original decimal token.
The value-based API cannot restore precision already lost by a caller's parser;
use the raw JSON API for declared decoding. All constants and paths check before
execution, including those in an unselected branch.

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
arithmetic. Only explicitly declared version 2 paths opt into binary64 rounding.
Raw JSON library consumers use `Plan::run_json`; `Plan::run` accepts an
already-decoded value and cannot recover precision lost by the caller's parser.

The library API `Plan::rust(package)` emits a standalone Cargo normalization crate,
with checked root schemas and the same recipe/execution/input logic as the reference
engine. It has no ESS runtime dependency. `Normalizer::new()` prepares the embedded
validators; `normalize(branch, input_json)` checks and executes one explicit branch.
`normalize_value(branch, value)` also accepts decoded values with the same
caller-owned precision qualification as the reference `Plan::run` API.
The `ess-normalization-target/1` report records the canonical recipe, source roots,
schemas and emitted-file digests, excluding the report itself. Generated-crate checks
cover default serde_json features and consumer-enabled arbitrary precision.

The library API `Plan::go(package, module)` emits a standalone Go 1.26
module with the same source-pinned report. `New()` prepares offline validators;
`Normalize(branch, input)` accepts JSON bytes and returns a complete `json.RawMessage`
or a typed `Refused`. It preserves exact integer tokens, explicitly declared
binary64 decoding, ordered collection operations and lazy first-match evaluation.
Schema findings are sorted by escaped instance pointer, retaining duplicates;
Rust retains its validator's traversal order. Go uses pinned JSON token and schema
libraries. The exact base64 pattern below is qualified against the pinned reference
for bundle schemas and model `Bytes` wire projections:

```text
^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$
```

It admits empty strings and checks the ASCII alphabet and padding placement; it
does not decode bytes or require zero unused pad bits. Every other selected pattern
refuses generation with `go_schema_pattern` at its source pointer, including
equivalent spellings of this expression. `contentEncoding` alone does not qualify
a pattern. General ECMA-262 matcher compatibility remains unfinished.

Generate a normalization library directly with the CLI:

```shell-session
$ ess generate schema normalize-generate --recipe normalization.json \
    --bundle input.bundle.json --bundle output.bundle.json \
    --target rust --package settings_adapter --out generated/rust
$ ess generate schema normalize-generate --recipe normalization.json \
    --bundle input.bundle.json --bundle output.bundle.json \
    --target go --package settings_adapter --module example.invalid/settings-adapter \
    --out generated/go
$ ess generate schema normalize-generate --recipe normalization.json \
    --bundle input.bundle.json --bundle output.bundle.json \
    --target go --package settings_adapter --module example.invalid/settings-adapter \
    --out generated/go --check
```

Every branch and target checks before files are written. The command protects its
recipe and bundle inputs and preflights the complete generated destination set.
`--check` compares every planned file without writing; missing or stale files return
a nonzero exit. Neither mode deletes unrelated or obsolete files. Output preflight
assumes controlled parent directories and does not provide rollback for later I/O
failure. The output includes `normalization-report.json` with the exact library-API
provenance and file identities.

Model-owned records use the `ess-normalization/3` format.
`Root::pin_model` pins a root explicitly admitted by a sealed
`ess_gen::schema::ModelTypes` selection; `Plan::check_with_models` accepts these
selections alongside replay-checked bundles. Identity includes the system,
specification version, source/contract/projection digests and selected root set.
Matching JSON field shapes alone cannot substitute for any of those coordinates.

The same CLI commands accept repeated `--model PATH` inputs, alone or mixed with
`--bundle`. Each model specification is compiled and the recipe's exact selections
are recreated and checked. Do not import generated model schemas after stripping
their annotations, or maintain a parallel schema just for normalization. For example:

```shell-session
$ ess generate schema normalize-generate --recipe model.normalize.json \
    --model specifications/settings --target rust --package settings_adapter \
    --out generated/adapter
```

Outputs cannot replace a model input or reside inside a model input directory.
Version 3 targets retain complete selected model schemas and provenance in a
version 2 normalization target report. Wire names, optional properties, newtype
definitions and refinements come from the existing model projection. Selected
model invariant statements refuse normalization planning because the engine does
not execute those predicates. Go's pattern refusal also applies to model-derived
patterns outside the exact qualified `Bytes` expression above. Bundle-only versions
1 and 2 keep their canonical recipe representation and version 1 report envelope.

The `ess-normalization/4` format adds explicit lexical capture through
`raw_json_inputs`. Each branch maps to a list of field/items selectors; an empty
path selects the root token:

```json
"raw_json_inputs": {
  "primary": [[{"kind":"field","name":"payload"}]],
  "retain_document": [[]]
}
```

This is a recipe-member excerpt; branches and source-pinned stages are still
required. The first input schema describes the captured representation: a selected
leaf must be string-shaped, normally model `Bytes`. At the text edge, the complete
selected token becomes canonical standard base64 before validation. Whitespace
outside the token is excluded; interior whitespace, quotes, escapes, member order,
duplicate members and number spelling remain exact. Huge numbers are retained
without numeric conversion. Capturing `null` produces `bnVsbA==`; an absent member
stays absent, and null intermediate containers are not traversed.

Capture requires valid JSON, strict Unicode and at most 64 levels of nesting,
including the captured content. Outside captures, existing duplicate-key and
numeric rules still apply. Selectors use declared wire fields and array items;
duplicate or ancestor-overlapping captures and capture/binary64 overlaps refuse
planning. Capture is never inferred from a `Bytes` type or field name.

Use `Plan::run_json`, Rust `Normalizer::normalize`, Go `Normalizer.Normalize`,
TypeScript `Normalizer.normalize`, or the existing CLI `normalize-run` text edge. `Plan::run` and Rust `normalize_value`
refuse a branch with active capture selectors because decoded values have lost
token provenance. An empty selector list does not disable those value APIs.

To inspect tokens inside a retained document, use a second checked recipe and an
explicit decoding boundary: `Plan::run_base64_json`, Rust
`Normalizer::normalize_base64_json`, Go `Normalizer.NormalizeBase64JSON`, or
TypeScript `Normalizer.normalizeBase64Json`.
These accept unquoted base64 text, require canonical standard encoding and UTF-8,
then pass unchanged JSON text to normal execution. Failures occur in order:
`input_base64`, `input_utf8`, then normal text-edge findings. They do not parse and
reserialize the retained document or automatically decode a nested field. Each
recipe retains its own source identities; callers own their composition.

Format 4 uses target report version 3. Versions 1–3 refuse `raw_json_inputs`, even
an empty map, and retain their canonical recipes, generated code and file maps.
Across package releases, reports still record the actual generator version.
Expanded recursive shapes, tuples and
intersections refuse in the earlier checker profiles; format 6 adds only the
explicit closed-tuple support described below. Schema bounds and other refinements are
checked at runtime boundaries, not proven by structural checking. Integer arithmetic
requires exact signed-64-bit integral JSON tokens. Equality is scalar-only and has
the format-5/6 floating-pair limitation described below. Source-adapter adoption
remains a separate consumer task.

#### Modeled finite floating values

The `ess-normalization/5` format admits modeled `Binary64` inputs and
outputs from an `ess/2` specification. Every Binary64 leaf in the first input
requires an explicit `binary64_inputs` field/items/root selector, including unused
optional fields. Map-value and tagged-union selectors are unsupported and refuse
planning. Versions 1–4 refuse selected Binary64 model closures, including types
used only for outputs. A plain imported `type: number` schema or annotation cannot
grant model Binary64 identity.

Use explicit floating construction for defaults and computed outputs:

```json
{
  "op": "fallback",
  "value": {"op": "read", "scope": "input", "path": ["ratio"]},
  "fallback": {"op": "binary64_literal", "value": "-0.0"},
  "on_null": false
}
```

`binary64_literal` keeps its numeric token string in recipe identity and checks
grammar and finiteness in every branch. `binary64` takes a numeric `value` and
ordered `steps` using the existing multiply/minimum/maximum vocabulary; it returns
a finite floating value without truncating. An empty steps list is an explicit
integer/general-number conversion. Float-to-integer conversion still requires
`binary64_to_integer`. An integral floating result does not become an integer
operand. A later conversion cannot recover an input token refused at the text edge.

Reference and generated Rust/Go preserve positive/negative zero, signed underflow,
subnormals and nearest-even rounding. Overflow and nonfinite values refuse.
Format-5 `equal` compares two typed Binary64 operands using IEEE equality, so
positive and negative zero compare equal; it does not cast mixed scalar kinds.
Explicit raw capture and retained-document helpers from format 4 remain available.
Format 5 uses target report version 3. Frozen templates preserve the complete
emitted Rust/Go file maps of formats 1–4.

#### Fixed positional input arrays

Use `ess-normalization/6` when the source contract explicitly constructs
a fresh fixed string array. Declare the prepared field as a positive-length closed
tuple: `prefixItems` contains exactly that many string schemas, `minItems` and
`maxItems` equal the length, and `items` is `false`. The input policy records the
source decoder behavior instead of asking schema validation to infer it.

This excerpt adds a two-slot policy to an otherwise complete checked recipe:

```json
{
  "format": "ess-normalization/6",
  "positional_inputs": {
    "decode": [{
      "path": [{"kind": "field", "name": "operands"}],
      "kind": "fixed_string_array",
      "length": 2,
      "missing": "preserve",
      "null": "zero",
      "short": "zero_pad",
      "extra": "discard",
      "null_element": "zero"
    }]
  }
}
```

Every displayed declaration member is required; this version admits only these
policy values. A missing field stays missing, null produces empty strings, short
arrays are padded and excess elements are discarded. Consumed elements must be
strings or null. Discarded tokens still pass complete JSON grammar, Unicode and
the global 64-level depth check, without converting their numeric values.
Required fields remain required after preparation.

A checked `position` expression reads one slot and can map it to a named field:

```json
{
  "op": "position",
  "value": {"op": "read", "scope": "input", "path": ["operands"]},
  "index": 0
}
```

The operand must have one proven exact tuple arity and the index must be in range.
Missing tuples propagate missing. Heterogeneous closed tuples can be read with the
type of their selected slot; nullable tuples, ambiguous source tuple unions,
homogeneous arrays and open or variable prefixes refuse. Homogeneous list
operations retain their earlier behavior.

Policies select only field, homogeneous-item or root boundaries and run at the
external original-text edge before first-stage schema validation. Overlap with
raw capture, Binary64 policies or other positional declarations refuses. Active
positional policies require an original-text entrypoint (`Plan::run_json`, Rust
`Normalizer::normalize`, Go `Normalizer.Normalize`, or CLI `normalize-run`) or a
retained-base64 helper. `Plan::run` and Rust `normalize_value` refuse them even
when the selected field is absent.
The existing normalize check/run/generate commands and Rust/Go/TypeScript targets apply.
Format 6 keeps `ess-normalization-target/3`, its report fields and digest domains;
formats 1–5 keep their complete generated maps at the same generator version.

#### Standalone TypeScript normalization

The `Plan::typescript(package)` API and CLI emit a private ES2022 ESM
package with strict TypeScript build inputs and no runtime dependencies:

```shell-session
$ ess generate schema normalize-generate --recipe normalization.json \
    --bundle input.bundle.json --bundle output.bundle.json \
    --target typescript --package settings-adapter --out generated/typescript
$ cd generated/typescript
$ tsc --project tsconfig.json
```

Generation installs nothing. TypeScript 6.0.3 and Node 22.23.1 are the qualified
compiler/runtime. Package names use lowercase ASCII parts beginning with a letter,
optionally `@scope/name`; parts can also contain digits, `-`, `_` and `.` and the
complete identity is at most 214 bytes. `--module` is Go-only and refuses here.
Import the emitted root entrypoint after compiling:

```typescript
import { Normalizer } from './generated/typescript/dist/index.js';

const adapter = new Normalizer();
const result = adapter.normalize('primary', '{"value":9007199254740993}');
if (result.ok) {
  console.log(result.json);
} else {
  console.error(result.findings);
}
```

The branch and input must match the checked recipe. Successful output is compact
JSON text; refusals carry `pointer`, `rule` and `detail`, with no partial output.
`normalizeBase64Json(branch, encoded)` accepts strict canonical base64 containing
UTF-8 JSON. A decoded BOM is preserved and then fails normal JSON grammar; invalid
UTF-8 and lone Unicode surrogates are rejected. No decoded-object API is offered.
Internal bigint preserves integer precision; ordinary JSON.parse on the returned
text would transfer responsibility for that precision to the caller.

The fixed `schema-profile.json` is hashed with the runtime and retained sources.
It includes tuple `prefixItems` and array/string lengths, exact numeric bounds,
source enums and the frozen Bytes pattern. It does not widen the existing Plan
shape checker: some numeric type/enum and allOf intersections still refuse there.
`uniqueItems: true`, unqualified patterns and other unsupported constraints refuse
before files are returned. Formats/content encodings are annotations and defaults
do not run. Schema findings retain the full reference multiset, sorted by escaped
Unicode-scalar pointer; non-schema findings retain their located rule and detail.

Recipe formats 1/2 use target report 1, format 3 uses report 2, and formats 4/5/6
use report 3. The new language needs no new recipe/report version. Report files
include all emitted runtime/profile inputs; consumer-built `dist/` is not hashed.
Existing Rust/Go emitted maps remain unchanged at the same generator version.

Static equality checking adds two typed Binary64 operands in formats 5/6 and keeps
Binary64 output provenance distinct. The frozen evaluator also compares any two
floating representations numerically in these formats, even when an Integer
source schema admitted integral floats. For example, an admitted Integer equality
can compare `1.0` with `1.0`; mixed `1`/`1.0` still refuses integer eligibility.
TypeScript preserves this runtime limitation. It does not make a general Number
expression comparable or allow implicit Binary64 output conversion. A separate
core-contract change would need its own compatibility decision.

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
