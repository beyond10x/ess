# OpenAPI import accounting — binding design

Status: accepted for implementation on 2026-09-06 under standing ESS review remediation authorization for `story:review-openapi-semantic-accounting` (F04). Source implementation and verification remain pending.
Source: incoming ESS `e113a65a0bac63e77cd17f43fa280a5bf56c93f9`. Intended durable design:
`docs/design/review-openapi-accounting.md`. Source paths/line numbers below refer to this pin.
The existing detailed Scope remains the audit; this document chooses its open boundaries.
Compared with the initial refresh pin `1b1c4a3a424e3304e503747422a0c3d10d86928a`, the
OpenAPI crate and import/project implementation are unchanged. Incoming normalization generation
adds a CLI command and updates shared reference documents; preserve those bytes and the
51-command leaf count. See `scope-acceptance-delta.md` for the precise overlapping reservations.

## Decision proposed for binding

Keep `ess-service-interface/1` unchanged and introduce one distinct, closed
`ess-openapi-import/1` envelope. Its successful value contains the retained source, unchanged
typed interface and durable accounting. It is not a new interface schema, an ESS lifecycle model,
a `schema-contract::Bundle`, or an executable `ess-normalization/2` recipe.

The proposed fields, in canonical order, are:

| Field | Typed meaning |
|---|---|
| `format` | Exactly `ess-openapi-import/1`. |
| `normalization` | Exactly `ess-openapi-service-subset/1`: this adapter's bounded interpretation/accounting rules, fixed by this design. Not an executable normalization recipe or a runtime-selected registry. |
| `source` | Closed `{text: String, sha256: String}`; retain the original UTF-8 input exactly. |
| `schema_dialect` | Exactly `https://spec.openapis.org/oas/3.1/dialect/base` for this first profile. Original authored declarations remain in source. |
| `interface` | The existing `ServiceInterface` with `format: ess-service-interface/1`; `source_openapi` remains the exact admitted source version, independent of service version. |
| `accounting` | Closed object with always-present ordered arrays `normalizations`, `coverage_gaps`, `unresolved_references`. |

Normalization/gap entries are closed `{pointer, code, detail}` records: normalization code
`annotation-omitted`, gap codes `constraint-unpreserved` and `feature-unpreserved`.
Unresolved entries are closed `{pointer, target}` records
identifying the reference site and decoded component name. Sort/deduplicate by typed field order;
do not collapse different sites into one component-name count. No persisted duplicate counts,
timestamps, filesystem paths, compiler digest, fabricated build Git revision, or self-hash is needed.
A package version is not a semantic identity; the fixed normalization profile identifies the
importer rules. Unsupported profile/format versions refuse. Future changes to those meanings need
an explicit coordinated version decision, not silent reinterpretation under this profile.

Canonical output is typed `serde_json::to_string_pretty` plus exactly one LF. Source SHA-256 is
bare 64 lowercase hex over `source.text.as_bytes()`, including original whitespace/newlines;
never hash a parsed/re-emitted YAML/JSON value instead. It is not ESS's compiled-model
`source_digest`. Whole-import identity, if a caller needs one, hashes complete canonical envelope
bytes; do not add a redundant stored digest. Equivalent YAML and JSON may have different source
and envelope identities.

## Checked admission, not stored claims

The public import result is sealed. Parse a private closed DTO, check the exact format/profile,
reimport its retained source under the recorded fixed dialect/profile, and compare the entire
derived interface/accounting/source digest with the stored result before returning the public
value. Unknown fields, missing accounting, malformed digest, altered interface, removed gaps,
changed unresolved sites or inconsistent source/dialect refuse. Preserve strict duplicate-field/map
refusal at this new wire boundary; a generic Value decode must not erase duplicate fields before
the closed DTO sees them. Replacing the source and every derived fact consistently declares a new
input, not an authenticated historical producer.

This is the same *admission pattern* already established by
`schema-contract/src/bundle.rs:173–209` and `docs/design/schema-bundle-import.md`, without taking
a dependency on that separate component adapter or changing its format. The present loss is explicit:
`ess-openapi/src/lib.rs:231–240` has only an in-memory report; CLI
`main.rs:2859–2861` persists only its interface.

For the source hash, add `sha2 = "0.11"` to this crate's manifest and update its lockfile
dependency list. That version already exists in the lockfile and in other local crate manifests;
there is no workspace `sha2` dependency to inherit. No root manifest change is proposed.

A refused import still exits 1 and writes no successful import file, leaving an existing output
unchanged. Representable partial imports may succeed and persist the envelope with nonempty
semantic gaps or unresolved references. Terminal presentation derives from that same accounting;
it is not the durable authority.

## Defaults, old readers and projection

Implement the new checked reader before switching the CLI writer within this unit. Both flat
`ess import openapi` and area `ess infra import openapi` write the new envelope through `--out`;
`--format` continues to select terminal presentation, not the persisted wire format.

Keep the old `ServiceInterface` DTO, canonical writer and explicit `read_interface` reader intact
(`lib.rs:12–80,275–287`). Old strict readers must reject the new wrapper rather than silently
discarding accounting. The new import-document admission path recognizes legacy /1 as
**accounting unavailable** and gives an actionable “reimport the original OpenAPI source”
refusal; it must not manufacture empty arrays or a complete status. No automatic upgrade can
recover absent source facts. No legacy-output downgrade switch is proposed.

CLI `generate project openapi --ir` uses the checked import-result boundary. It refuses legacy
unqualified input, any semantic coverage gap, or unresolved reference before opening/writing
the output. A known annotation normalization alone need not block the existing structural
projection. No acknowledgement/bypass flag is proposed: an accurate partial-projection envelope
would be another contract and is unnecessary for F04.

Retain the existing low-level `project(&ServiceInterface)` API as an explicitly structural,
unqualified projection with unchanged supported YAML bytes; it cannot claim historical import
coverage. The checked projection API/CLI is separate and never silently extracts an interface from
a partial wrapper. Native `generate project openapi --path` remains the compiler-owned path.
The three actual CLI boundaries are `main.rs:2825–2900,3165–3205` and the native `--path` branch;
the source-only adapter still supports the existing subset, not broad OpenAPI validation.

## Variant and dialect accounting

| Input case | Bound behavior proposed |
|---|---|
| Valid OpenAPI 3.1 patch version; omitted dialect or explicitly supported OAS base dialect | Admit the existing service subset. Validate the version token, not merely a `3.1.` prefix. Preserve the authored version. |
| 3.2/unknown version, Swagger 2.0, unsupported `jsonSchemaDialect` or schema `$schema` | Refuse before interpreting schemas under a guessed dialect. No alternate-dialect support is added. A matching dialect declaration is not itself a coverage gap. OpenAPI 3.0 is admitted through the rewrite in the amendment below. |
| Explicit string, nonempty all-string enum and/or string const | Preserve the existing representable constraints. Infer string only when type is genuinely absent and the admitted literal constraint proves a string-only domain. A type array must not be mistaken for absent type. |
| Integer/number/boolean/object/array enum or const | Record a semantic gap at each unpreserved keyword; do not globally mark enum/const consumed. Mixed/untyped shapes without a faithful structural carrier may refuse. |
| Empty enum, a type array other than `[<type>, "null"]` (see the amendment below), or unrepresentable literal combination | Refuse as an unsupported adapter shape; never turn an empty enum into an unconstrained string. Do not call every valid-but-unsupported schema malformed. |
| 3.1 array without items; boolean/untyped items that the existing IR cannot represent | Explicit refusal at the schema/items boundary before any caller can silently drop the component, property or message schema. |
| Bare supported local component reference | Preserve it and record exact reference sites. Missing target remains durable unresolved accounting and blocks checked projection. |
| 3.1 schema reference with siblings | Inspect every sibling before returning a Reference. Report each unpreserved constraint as a gap; record explicitly known omitted annotations as normalizations. Do not apply the nonschema Reference Object's sibling rule here. |
| External/noncomponent/non-string reference; resource/anchor/base behavior outside this local subset | Refuse; no network or speculative lookup. Preserve existing supported pointer escapes; unsupported URI-fragment spellings must refuse before being misclassified as a different missing component. |
| Other reviewed keywords | A keyword is consumed only by the variant that actually preserves it. Known harmless annotation omission is an explicit normalization. Unsupported constraints/extensions get gaps; an unsupported interpretation-changing dialect/base feature refuses. Never recurse through literal annotation values as if they were schemas. |

For this profile, annotation-only omission is limited to `title`, `description`, `example`,
`examples`, `default` and `deprecated` in their admitted annotation positions. A field actually
preserved by that position is consumed instead. `default` never inserts a value. Do not classify
`readOnly`, `writeOnly`, arbitrary `x-*`, or `x-ess-invariants` as harmless merely by their names;
unpreserved contextual/extension meaning remains a gap. Existing non-schema feature diagnostics
remain accounted rather than silently disappearing during this split.

Primary meaning: OAS 3.0 Reference Object siblings are ignored and its array shape requires items,
but this adapter continues to refuse 3.0.
[OAS 3.0.3 Reference Object](https://spec.openapis.org/oas/v3.0.3.html#reference-object),
[Schema Object](https://spec.openapis.org/oas/v3.0.3.html#schema-object).
In 3.1, schema dialect declarations govern interpretation, and schema references differ from
nonschema Reference Objects.
[OAS 3.1.0 dialects](https://spec.openapis.org/oas/v3.1.0.html#specifying-schema-dialects),
[Reference Object](https://spec.openapis.org/oas/v3.1.0.html#reference-object).
JSON Schema 2020-12 permits reference siblings and omitted items imposes no item assertion.
[Core §8.2.3.1](https://json-schema.org/draft/2020-12/json-schema-core#section-8.2.3.1),
[§10.3.1.2](https://json-schema.org/draft/2020-12/json-schema-core#section-10.3.1.2).
Enum/const apply to any JSON value; an empty enum matches none.
[Validation §6.1.2–3](https://json-schema.org/draft/2020-12/json-schema-validation#section-6.1.2).
These facts motivate refusal/accounting; they do not authorize new broad support.

## Verification and remaining binding choices

Retain the existing supported 3.1.0 fixture's /1 bytes and projected YAML. Measure red/after cases for
each reviewed enum/ref/array hole at component, property and message sites; preserve partial
accounting through CLI write→checked read; reject tampering; verify legacy admission and an actual
frozen old strict reader against the new wrapper. Verify checked-projection refusals leave both
stdout artifact output and existing destination untouched, plus complete-source positive controls.
No tests/builds were run to prepare this proposal.

Root accepts the concrete wrapper/profile names and exact source retention, replay admission,
new CLI writer default, strict checked-projection refusal for partial/legacy input, and the bounded
dialect/variant matrix above. These decisions precede implementation; no new support or test result
is claimed. Preserve both enum and const faithfully when combined, or explicitly refuse an
unrepresentable combination; never drop one constraint because the other can be represented. No external persisted
consumer or cross-repo byte verifier has been established; no new ADR, consumer migration wave or
cross-repo verifier is a prerequisite inferred by this proposal.

## Amendment 2026-09-25: the OpenAPI 3.0 subset (beyond10x/ess#73)

A valid `3.0.N` document is no longer refused at `/openapi`. Each Schema Object is rewritten to its
3.1 form before the unchanged 3.1 reading applies, so the envelope, `schema_dialect`, profile
name and every 3.1 import's bytes stay as they were; `source_openapi` keeps the authored `3.0.N`
and replay admission reimports the retained 3.0 source under the same rewrite. An older reader
refuses such an envelope on replay, which is the safe direction.

| 3.0 construct | Rewrite |
|---|---|
| `exclusiveMinimum: true` with numeric `minimum` (likewise maximum) | 3.1 numeric `exclusiveMinimum: <minimum>`; `minimum` is folded in. The bound remains an ordinary `constraint-unpreserved` gap, as in 3.1. |
| `exclusiveMinimum: false` | Dropped; it has no meaning. |
| `nullable: false` | Dropped. |
| `nullable: true` beside an explicit `type`, with no `enum` or with `null` in its `enum` | 3.1 `type: [<type>, "null"]`. The interface IR has no `null`, so the adapter keeps `<type>` and records a `feature-unpreserved` gap at `…/nullable`; checked projection then refuses as for any gap. A `null` enum member is removed from the string values. |
| `nullable: true` without `type`, or with an `enum` lacking `null` | No `null` is admitted (OAS 3.0.3 Schema Object), so it is dropped exactly. |
| `nullable: true` beside `$ref`; non-boolean `nullable`; `exclusiveMinimum: true` without a numeric `minimum`; numeric `exclusiveMinimum` | Refused at that keyword's pointer. 3.0 ignores `$ref` siblings, so reading `nullable` there would be a guess. |

The 3.1 spelling of the same meaning agrees with it: `type: [<type>, "null"]` (either order)
imports as `<type>` with the identical `feature-unpreserved` gap at `…/type`, and an `enum` or
`const` that excludes `null` makes it exact. Any other `type` array still refuses as an
unsupported adapter shape. Swagger 2.0 (`swagger:`) and every other version still refuse.
