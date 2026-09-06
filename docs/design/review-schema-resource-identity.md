# Generated schemas as adopter registry resources

Accepted coordinator interpretation for story:review-schema-resource-identity, 2026-09-06.
The standing remediation approval covers this bounded explanation of existing behavior. It does
not select an ESS namespace, hosted schema service, automatic ID projection or new wire format.
The implementation must demonstrate the worked pair through the actual CLI before publication.

## Existing authorities

The syntax schema is a projection of RawSpecFile, not whole-system semantic validation
(`crates/edge/ess-xtask/src/main.rs:423`). The CLI's specification path separately loads,
assembles and compiles source (`crates/edge/ess-cli/src/load.rs:65`). Generated contract
schemas carry self-contained definitions and model/slice provenance
(`crates/generate/ess-gen/src/schema.rs:154`). Neither producer emits a root `$id`.
The internal and public format catalogs must correct their contrary contract-schema cells.

The existing registry indexes explicit root IDs and compiles the supplied resources offline
before checking instances (`crates/generate/schema-contract/src/validate.rs:124`). It selects
an instance's nonempty `schema` string by exact ID and validates the entire instance, including
that selector (`:228`). Resource filenames are not identities. Successful records identify the
selected schema ID and file (`:274`); they do not enumerate every referenced payload resource.

## Accepted local representation

Read the exact frozen syntax and CreateInvoice contract schemas without changing their files:

- `schemas/generated/ess.schema.json`
- `generated/schema/commands/billing.invoice.CreateInvoice.schema.json`

Parse a separate copy of each and add only a root absolute `$id`, chosen by the adopter. Keep
its dialect, definitions, restrictions, local references and all provenance annotations. The
test must show that removing only the added ID recovers the original parsed value and that
both original files remain byte-identical. The separately serialized copy has different bytes;
its model or slice digest is not a checksum of that new file.

Use illustrative immutable logical IDs under `urn:example:` for this local workflow. Each
payload resource and each envelope has a distinct ID. The adopter owns uniqueness, versioning
and publication policy. The CLI detects exact-string duplicate IDs within the supplied registry;
it has no historical immutability ledger or promised URI-alias equivalence policy. No concrete
consumer requires a digest-ID scheme here. Such a scheme would need a separate definition of
the hashed bytes and how it avoids an ID containing its own digest.

The selector belongs to a separate application envelope schema. That schema requires exactly
`schema` and `payload`, constrains `schema` with `const` to its own ID, assigns `payload` an
absolute `$ref` to the copied resource and sets `additionalProperties: false`. The original
payload value is nested unchanged; no member is injected into a strict domain object. This
does not promise preservation of the payload's original lexical JSON bytes. The representation
also permits non-object payloads when their referenced schema permits them, and does not
introduce a universal ESS envelope.

Keep the two resources separate: the syntax resource retains draft-07 and `#/definitions`,
and the generated contract retains draft 2020-12 and `#/$defs`. Inlining an entire ID-less
schema beneath `properties.payload` would change root-relative fragment resolution. Use
distinct, deliberately unrelated filenames ending in `.schema.json` to demonstrate ID lookup.
Keep the ID-less originals and malformed registries in isolated negative cases. Supply all
referenced resources locally; missing references refuse without a network retriever.

## Projection and model limits

Registry validation is distinct from the restricted structural TypeScript projector. The
existing widget fixture remains a positive projection control. Generated annotations and
external references can be refused by that projector even when schema validation succeeds.
Do not strip annotations or widen the projector's vocabulary to hide that boundary.

The legacy registry reads parsed JSON values. Adding an ID supplies neither original-text
numeric admission nor model authority. Preserve the current normalization versions 1–6,
model selection, native Binary64 codec limits, Rust/Go targets and TypeScript limitations in
the surrounding public guide. This story changes none of their contracts. An accepted schema
copy is not admitted ModelTypes, a replay identity or conformance evidence.

## Required execution evidence

The new Rust CLI integration test must run the real ESS executable and retain the commands,
statuses and observations. Both original schemas must refuse with `missing_schema_id` when
an instance is supplied. Each copied resource plus its envelope must accept its independently
specified valid instance and report the envelope's ID/file. Assert payload ID/reference
identity separately. Invalid nested values must yield `invalid_instance` with a payload path;
missing or unknown selectors, missing payload and extra envelope members must refuse at
their corresponding boundaries. Injecting a selector directly into the strict contract payload
must remain invalid.

Exercise missing offline references and duplicate exact IDs, retaining the registry-stage
empty acceptance list. Do not infer whole-run transactionality for mixed instance results.
Demonstrate a source document accepted by the syntax envelope but refused by actual
specification assembly for an unsatisfied domain roster.

Retain the widget TypeScript ID lookup, generation and equal/stale check controls. Missing or
duplicate selected IDs, unsupported generated annotations and external-reference envelopes
must refuse before writing, preserving an existing sentinel and an absent output parent.
This makes no rollback claim for later filesystem failures.

These are examples and regressions over existing behavior; no manufactured production red
run is required. Preserve the first observations. If a promised positive cannot run, retain
its failure and return the smallest necessary production scope to the coordinator. Do not
change generated schemas, dependencies or registry semantics under this document's authority.

The source inspection supporting this decision is the refreshed story-scoper report for
4777c1de3a80ea7645a255e4e229ad30a8a5cc8e, retained with wave9 preparation. Its decisive
producer, registry and frozen-schema hashes still match the wave10 opening source. It is
preparation evidence, not an executed registry result.
