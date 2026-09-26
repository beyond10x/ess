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

## ess/6 container review — 2026-09-21

Only `RawOutcome` and the new `RawSubjectField` change in this schema revision. The outcome gains
`when_subject` and `preserves`; the nested subject fact holds the declared field and enum value.
These are authored-domain semantics with their own behavioral obligations. None changes the three
CLI pipelines into consumers of the generated JSON Schema document. The root dialect, six literal
relationships, consumer profile fingerprints and executed metadata guard are unchanged.

The extracted root-definitions shape changes from `dcfd3383c4cc7082a423bfffafe0b74e27b5b30512bb5f9351eb544b0311ca0c`
to `68e68c7cb8cc7b6a3a0dbe854c1b74ab54c9da825009a85770efa2d4f3e23322`.
Exactly the three RootDefinitionsContainer rows are re-reviewed at that shape. No descendant
is exempted and no behavior coverage is claimed. The separate pre-existing missing classification
for `ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript` still refuses consumer accounting;
this container review does not repair or conceal that registry debt.

## ess/7 container review — 2026-09-22

The generated schema adds only the optional `RawOutcome.replays` property, a
reference to the existing `OutcomeName` definition. The source7 retained-result
relation is authored-domain behavior; it does not make these CLI pipelines
consumers of the generated schema document. Its descendants keep their ordinary
behavioral obligations.

The unchanged wire extractor measured the definitions-container shape changing
from `68e68c7cb8cc7b6a3a0dbe854c1b74ab54c9da825009a85770efa2d4f3e23322`
to `fb4c0b29d046e5f20a7379422ae41f77ebc0caed42167716ed130e38de329873`.
Review updates only the three existing RootDefinitionsContainer shape pins.
The root dialect, six literal relationships, consumer profiles, guard source,
baseline and behavioral accounting remain unchanged. The metadata guard and
fresh-provider qualification still have to execute; this review grants no new
exemption or runtime conformance claim.

## ess/9 container review — 2026-09-26

The generated schema changes `RawOutcome.when_subject` from a reference to `RawSubjectField`
to a reference to the new `RawSubjectFact`, an `anyOf` of the unchanged `RawSubjectField` and
the new `RawSubjectPredicate` (`{predicate}`, beyond10x/ess#75). The ess/9 subject predicate is
authored-domain behavior; it does not make these CLI pipelines consumers of the generated schema
document. Its descendants keep their ordinary behavioral obligations.

The unchanged wire extractor measured the definitions-container shape changing
from `fb4c0b29d046e5f20a7379422ae41f77ebc0caed42167716ed130e38de329873`
to `ce18e2b8593d657226635ec33a36698ece87fa78309ef47971e04399574a1bdc`.
Review updates only the three existing RootDefinitionsContainer shape pins.
The root dialect, six literal relationships, consumer profiles, guard source,
baseline and behavioral accounting remain unchanged. The metadata guard and
fresh-provider qualification still have to execute; this review grants no new
exemption or runtime conformance claim.

## ess/10 container review — 2026-09-26

The generated schema changes `RawViewSpec.fields` from items of `Field` to items of the new
`RawViewField` (`Field`'s members plus an optional `aggregate`, a `RawAggregate` one-key map whose
`count` takes the new `Empty`), and adds `RawViewSpec.group_by` (beyond10x/ess#96, aggregate views).
`Field` itself is unchanged at every other position. Aggregate views are authored-domain behavior;
they do not make these CLI pipelines consumers of the generated schema document. Their descendants
keep their ordinary behavioral obligations.

The unchanged wire extractor measured the definitions-container shape changing
from `ce18e2b8593d657226635ec33a36698ece87fa78309ef47971e04399574a1bdc`
to `dd6345942de597f9d076b7168bcae88de36332f873ff8c2daf53b6b6794b22c1`.
Review updates only the three existing RootDefinitionsContainer shape pins.
The root dialect, six literal relationships, consumer profiles, guard source,
baseline and behavioral accounting remain unchanged. The metadata guard and
fresh-provider qualification still have to execute; this review grants no new
exemption or runtime conformance claim.

## Wave 2 container review (ess/11, ess/12, typed literals) — 2026-09-26

Three changes merged together move the generated schema, and none of them re-reviewed the
container on its own. ess/11 (beyond10x/ess#103, beyond10x/ess#104) adds `alphabet` to a
`String` newtype and the new `InputField` (`Field` plus `example`) for command inputs. ess/12
(beyond10x/ess#105) adds the root `outcome_groups` list of the new `RawOutcomeGroup`,
`RawGroupOutcome` and `OutcomeGroupName`. The typed literals of beyond10x/ess#113 widen
`RawPayloadSource` with boolean, integer and number alternatives beside the string. All three are
authored-domain behavior; they do not make these CLI pipelines consumers of the generated schema
document. Their descendants keep their ordinary behavioral obligations.

The unchanged wire extractor measured the definitions-container shape changing
from `dd6345942de597f9d076b7168bcae88de36332f873ff8c2daf53b6b6794b22c1`
to `ff756782d8262e4ad1a77fda6cc355fa12f7323652a069c0c0f92bbba3f75d92` on the merged tree
(the unit branches had pinned `593740573912b57d912b08faad77c72d88fc8659d9ede47b8840493236cae68b`
after ess/11 and `41ad2644870036b36151f12d657c1c860b0dc6293e92eecc9a3c81ab9b5cd1ca` after
ess/12, without a review here). Review updates only the three existing RootDefinitionsContainer
shape pins. The root dialect, six literal relationships, consumer profiles, guard source, baseline
and behavioral accounting remain unchanged. The metadata guard and fresh-provider qualification
still have to execute; this review grants no new exemption or runtime conformance claim.

The same merge moves two invocation guards in `macro-guards.json`, and only their invocation
digests; both definition digests are unchanged. Each invocation gains exactly one code for the
unset-at-creation refusal of beyond10x/ess#112, and no existing code changes:

- `ess_compiler::resolve::codes::codes` gains `CREATION_LEAVES_INVARIANT_FIELD_UNSET`, from
  `95e1f6bb7c382d3c9d17b0f1ef6a00e18894c74d7191aeedfbd3219f28a829ad`
  to `325198be42e23426e0ba7fecbd06aba639ca261feb12ad92fe0b6b3711a31b04`;
- `ess_primitives::error::validation_codes` gains `InvariantReadsUnsetField`, from
  `0e7a4d5e372e92fbefc48c99d57d3dad876e21b5e9ca522358d2ee46d933c365`
  to `d466e6623ca525b125419a2811f566b71b2dc092a9e0d6fd4900757203191577`.

## ess/13 container review (fixture inputs) — 2026-09-26

The schema gains `RawCommandSpec.fixture_inputs` and its `FixtureName` declaration
(beyond10x/ess#58). These are authored-domain semantics with their own descendant obligations;
they do not make the three CLI pipelines consumers of the generated schema document.

The unchanged wire extractor measured the definitions-container shape changing
from `ff756782d8262e4ad1a77fda6cc355fa12f7323652a069c0c0f92bbba3f75d92`
to `fb292f6d49799efe775923c52a35307cedc8266d25a68c4ee3eb6b5e0a764ce5`.
Review updates only the three existing RootDefinitionsContainer shape pins. The root dialect, six
literal relationships, consumer profiles, guard source, baseline and behavioral accounting remain
unchanged, and no invocation guard in `macro-guards.json` moves: the change adds no refusal code.
This review grants no new exemption or runtime conformance claim.

## Serialized-string container review — 2026-09-26

The schema's `ExternalRef` and `Period` definitions change from the Rust shapes (an object with
`provider` and `reference`; a `uint32` integer) to the strings ESS actually reads and writes: a
`provider:key` string with the parser's pattern and URL refusal, and a `PT<seconds>S` string
bounded to `u32::MAX` (story:schema-describes-serialized-strings). No document ESS reads changes
and no source format moves; the schema stops describing documents ESS refuses. Neither change
makes the three CLI pipelines consumers of the generated schema document.

The unchanged wire extractor measured the definitions-container shape changing
from `fb292f6d49799efe775923c52a35307cedc8266d25a68c4ee3eb6b5e0a764ce5`
to `aea620213fc71de3aa8a89cb1afb0af0d265f86f9c75137cac8267ee3c517475`.
Review updates only the three existing RootDefinitionsContainer shape pins. The root dialect, six
literal relationships, consumer profiles, guard source, baseline and behavioral accounting remain
unchanged, and no invocation guard in `macro-guards.json` moves: the change adds no refusal code.
This review grants no new exemption or runtime conformance claim.
