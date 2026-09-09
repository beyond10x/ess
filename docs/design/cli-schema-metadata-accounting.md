# Exact schema metadata accounting for CLI pipelines

Accepted bounded decision, 2026-09-09. Owner: story:cli-presentation-binding.
This is a scoped amendment to review-consumer-coverage.md. The earlier
cli-consumer-pipeline-coverage.md decision did not authorize a new disposition.
Independent review: review-result:cli-schema-metadata-decision-r1-20260909.
The coordinator accepts the independently approved amendment under the existing
authorization to close ESS dependencies of the local CLI contract milestone.

## Purpose and limit

The authored CLI pipelines consume ESS documents, not the generated JSON Schema
document. Two inventory nodes describe the latter: its root dialect identifier
and definitions container. An unknown authored-field refusal does not test those
nodes. Keep them in the universal matrix, but account for exactly six reviewed
relationships as SchemaDocumentMetadata, separately from Supported, Refused and
BaselineUnknown. This establishes schema representation bookkeeping, not CLI
behavior or runtime conformance.

This closes a bounded dependency of Connectors' local CLI contract fixture.
It adds no general applicability registry, prefix exemptions, automatic future
coverage, new CLI behavior, provider handlers, or changes to ESS model formats.

## Exact relationships

The complete admitted set is the Cartesian product of these two literal IDs:

- wire:RawSpecFile#/$schema — RootDialectIdentifier.
- wire:RawSpecFile#/definitions — RootDefinitionsContainer.

And these three literal consumers:

- cli-binding-resolution.
- cli-binding-rust-emission.
- cli-binding-process-execution.

There are exactly six rows. No descendant, root document, authored property or
fourth consumer qualifies. Preserve all 1,813 existing model IDs, wire shapes,
the 87 pre-existing profile records and their fingerprints, existing behavior
cases and requirements, and the accepted baseline bytes. The baseline SHA256 is
3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de.
Definitions descendants retain all their ordinary obligations. A descendant
change changes the container's conservative shape hash and invalidates its row.

## Source claims and executed proof

A separate closed source manifest, reviewed-schema-metadata.json, has format
ess-consumer-schema-metadata/1. Each row pins the exact model ID and shape hash,
consumer ID and profile fingerprint, closed relationship, guard ID and source
digest, precise nonbehavioral reason, and this decision reference. The guard
must compare the row set to the six literal relationships, not trust an arbitrary
manifest allowlist. Missing, duplicate, extra, stale and contradictory rows fail.

A small private metadata.rs module owns the executed guard and a proof value
that cannot be constructed by deserializing saved receipts. Under check_at's
current compiled provider/source/build authority, the guard must:

1. Compare the fresh schema provider output with the inventoried schema and run
   the existing wire extractor; compare its obligations and reference inventory.
2. Assert the exact supported Draft 7 root dialect, an object definitions map,
   and absence of both metadata spellings from the root authored properties.
   Preserve existing failures for unresolved references and unsupported shapes.
3. Match exact model/profile fingerprints and the real authored-pipeline entry
   lists. Match the reviewed guard source digest. The definitions subtree hash
   retains its existing meaning; use the existing canonicalization implementation.
4. Return proof only after those checks pass, bound to the current run authority.
   Preserve pre/post source-authority verification. A saved or forged JSON receipt
   is diagnostic evidence and can never substitute for this proof.

The qualification plan records required metadata guards separately from required
behavior cases. Qualification requires equality of planned and proved pair/guard
sets, including their model/profile identities and fingerprints. One guard may
verify all six cells: report one executed guard, not six invented behavioral tests.
The receipt records source/provider/build authority, manifest/guard digests and
its bookkeeping-only claim limit. Avoid circular source pins: whole-source and
provider hashes belong in the generated receipt, not the source manifest.

Declaration fingerprints establish the reviewed entry boundary, not a whole
call graph. The semantic relationship is explicitly reviewed. Do not claim that
the gate proves no private future implementation could inspect JSON Schema.
If a future boundary changes, that change requires review of these exact rows.

## Accounting and migration

Keep behavioral requirements and Behavior/ClaimedBehavior unchanged. Metadata
candidates travel separately; candidate and execution-plan stages are unqualified.
A metadata cell cannot carry behavior cases, a refusal label or an unknown owner.
All Cartesian cells remain present. Qualified totals satisfy:

total = Supported + Refused + BaselineUnknown + SchemaDocumentMetadata

A successful checkpoint for this exact decision has six metadata cells. Candidate
and refused runs report zero qualified metadata, with pending candidates explicit.
Keep executed case and guard counts separate. Total accounting completeness does
not mean complete behavioral coverage.

The fourth disposition changes persisted accounting meaning. Introduce the first
explicit accounting envelope, ess-consumer-accounting/1, with closed candidate,
execution-plan and qualified stages, for unaccepted-cells.json,
execution-plan.json and qualified-cells.json. Identify this same format in
checkpoint-summary.json, summary.json and refusal diagnostics. Reject absent or
unknown formats and incompatible stages at typed reading/qualification boundaries.
The old unversioned files remain historical receipts; never convert or replay them
into qualified cells. Do not invent a prior named /0 or silently interpret them
using the new algebra.

Preserve ess-consumer-initial-eligibility/1 and its baseline bytes. Preserve wire
inventory, extraction products, behavioral case identities and existing profile
fingerprints. No ess/1, ess/2, ess-cli/1, schema, OpenAPI or native case-protocol
migration occurs. No new dependency or native test target kind is required.

## Implementation and verification

Scope is consumer_coverage's metadata module/manifest, proposal.rs, account.rs,
enforce.rs, mod.rs and focused tests, plus this decision and an additive amendment
to review-consumer-coverage.md. Keep inventory, executor/native protocols and
baseline source unchanged. Planning mutations remain coordinator-owned.

Meaningful tests must prove actual fresh-provider admission and exact six-cell
counting, plus refusal for stale or missing hashes/rows, duplicates, extra pairs,
wrong roles, descendants, a fourth profile, changed dialect/definitions/reference
inventory, a retained provider differing from the fresh one, wrong run authority,
and mismatched planned/proved sets. Check conservation and distinct case/guard
counts in successful and refused outputs. A container row cannot absorb a new
descendant obligation. Demonstrate old closed readers rejecting the new variant
and new readers rejecting legacy/unknown envelopes and wrong stages.

Execute focused gate tests, preserve the existing inventory/baseline/profile
checks, then run fresh extraction, planning and qualification under one source
authority. Full task check and task site-build remain mandatory before landing.
The exact final ESS source is then pinned, rebuilt and used to regenerate and
validate Connectors; focused tests alone do not complete the CLI epic.
