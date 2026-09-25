# Exact acquisition and aggregate accounting boundaries

Status: accepted after independent design review, 2026-09-15; review-result:consumer-accounting-applicability-pass-1.
Owner: story:consumer-accounting-baseline-never-extended, approved ESS evolution revision1.
This supplements [finite reconciliation](consumer-accounting-reconciliation.md) and amends only
the matching rules in [consumer coverage](review-consumer-coverage.md). It qualifies no behavior.

## Decision and preserved authority

Correct eight misclassified scenario-acquisition profiles while retaining mandatory executed
obligations. Add separately reported aggregate provenance for five exact identities, supported by
a structural guard and qualified same-consumer child obligations. Changed CLI profiles retain
ordinary behavioral qualification: no ProfileCompatibleUnknown bridge, old-mode alias or new
baseline row is admitted. No blanket applicability disposition is added.

The frozen initial-baseline.json SHA256 remains
3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de. Its current contents are1,813 model
identities,87 groups,157,677 eligible pairs and54 mandatory exclusions. The earlier157,068 figure
predates the accepted representation-only amendment. Preserve current bytes, owners and follow-ups;
that historical representation exception does not cover these semantic/wire changes.

These are extensions of existing closed Profile, Cell, Claim, PlannedCell and Baseline values in
consumer_coverage. Existing model/profile IDs and hashes remain their typed homes. No new product
entity, identity scheme, generic registry or ESS dependency on AEP is introduced.

## Eight owned scenario-acquisition profiles

The exact binding is ess_cli::bin(ess)::input_discovery::fn::authored. It receives an optional path
and role boolean and returns identity/origin/text inputs; it neither accepts nor parses an ESS
model. Its strict manifest and filesystem rules are still mandatory ESS-owned behavior.

| Literal profile | Role | Mode |
| --- | --- | --- |
| acquisition-authored-manifest | Authored | Manifest |
| acquisition-authored-legacy-directory | Authored | LegacyDirectory |
| acquisition-authored-direct-file | Authored | DirectFile |
| acquisition-authored-omitted-scenarios | Authored | Omitted |
| acquisition-coverage-manifest | Coverage | Manifest |
| acquisition-coverage-legacy-directory | Coverage | LegacyDirectory |
| acquisition-coverage-direct-file | Coverage | DirectFile |
| acquisition-coverage-omitted-scenarios | Coverage | Omitted |

Add ProfileClass::ScenarioAcquisition only for these eight IDs, exact entrypoint, role/mode pairs,
declaration hash and reviewed profile hashes. Keep all eight in the source/profile inventory and
exclude them only from the model-consumer Cartesian map. They are not ForeignContext. Reject a
ninth ID, prefix expansion, missing inventory row or source route that gains model consumption.

The closed ess-consumer-scenario-acquisition/2 authority has exactly eight ordered rows containing
existing profile IDs, exact current profile/entry hashes, closed role/mode, exact reviewed case
identities and AST/source fingerprints, and source/review references. A fresh executed structural
guard proves the source boundary and classification; it cannot replace the behavioral cases.
Qualification requires exact planned/proved row equality and normal deduplicated case execution.

Each row retains entrypoint_sha256 as the existing signature/declaration identity and separately
binds entrypoint_source to crates/edge/ess-cli/src/input_discovery.rs, entrypoint_ast_sha256 to the
existing source_item_sha256 for the complete authored function, and entrypoint_source_file_sha256
to that production module's raw bytes. The signature hash omits the function body and cannot
authorize its acquisition behavior. Candidate extraction must obtain the full source item and
module digests from the current production inventory; exact reviewed equality then binds this
bounded acquisition route, including its local helpers. Case AST/source identity remains separate.
Do not change global declaration/profile hashing or frozen baseline eligibility to add this guard.
Native freshness checks around compilation and each execution remain mandatory, but agreement
between a new provider and its new source is not approval of that new production body.

The earlier unadopted acquisition/1 proposal bound only the signature and is retained as historical
provisional evidence. The new persisted row fields require version2 for this acquisition family:
authority, candidate, plan, execution proof and qualified acquisition output. Refuse the old/1
envelopes rather than reinterpret them; preserve closed old-reader rejection evidence. The parent
accounting/2 continues to name the explicit nested acquisition format; no unrelated ESS format,
canonical byte stream or historical accounting/v1 reader changes. This is a concrete correction
of the already required source-boundary guarantee, not new unknown eligibility.

Cases assert identity, origin and original bytes; role-specific manifest/discovery, containment
and symlink rules; deterministic selection; refusal before output; and Omitted returning no inputs
without probing a default. Shared cases may prove explicitly attributed rows only. Wrong role,
stale source, missing/ignored/zero-selected cases and early return before assertions refuse.
Preserve source/binary/native-runtime checks. Candidate output has zero qualified acquisition;
final v2 reports its eight obligations separately and cannot pass with one absent or unproved.

Reconciliation records exactly14,504 old eligible tuples (8 x1,813) as RetireConsumerProfile.
Every old tuple must exist in the frozen file; its consumer must be absent from the current model
map and present in the guarded acquisition inventory. This grants no model support. The80 tuples
involving removed models are included here once; remaining removed-model retirements are790
(10 x79), giving15,294 historical retirements. Missing/extra/duplicate or revived tuples, changed
digests and absent acquisition obligations refuse. Retirement never silently drops source coverage.

## Five exact aggregate identities

Wire extraction hashes whole canonical subtrees, so child changes alter ancestor hashes. Only the
following old/current transitions may use AggregateClosure:

| Identity | Frozen old shape | Current shape |
| --- | --- | --- |
| rust:ess_domain::command::RawOutcome | a2072ea0863f3bfdaa92b6ab4b30eb1857bf23af0538db9747ea3b3efc3505b0 | f167bcf3a4fd9ea3f40d41885bf457433f1601dd39353efb7883626a38c4b160 |
| wire:RawSpecFile# | 687cebb230483729bb9de45eeec08e14b3f8961cfc977c1f470001cc1b41d6a5 | 35e9219da6bb5023ef23090ace368b30f8f52f17b28a8c9d0a9bafaf14f69963 |
| wire:RawSpecFile#/definitions | 3515c009028136bab26f2c1e31a6a5a15f8a44eba3f3ae6444e47a3aa5aab244 | 3c1ce721245397ae759f2f3c6fb91ce4039d0aee9c211043c996c73657e6f79a |
| wire:RawSpecFile#/definitions/RawOutcome | eed5afcaac752aae6182c13a784afdc777ff207dd5404c20b320a619d7683722 | 09922c5e6ca0034e702f9ff81caa1dcb7a2a4430701ddbef9f4ac6b12e4afc3b |
| wire:RawSpecFile#/definitions/RawOutcome/properties | c72e70a9700e2b90445a761f78283670c50659dffa3bee63ff0bb5878c7297ab | c2da7f3d928ba82ca11ccfceaf9be30cf83736b0845abd372784c5774ca544c5 |

AggregateClosure is provenance, not Supported, Refused, metadata or new eligible unknown. Its row
records the old/current parent tuple, baseline/reconciliation digests, any unresolved unchanged
provenance, a finite same-consumer child frontier with exact dispositions/cases, structural witness
and review reference. The closed ess-consumer-aggregate-closure/1 authority has two proof modes:

- ShapeDelta: the consumer/profile is unchanged and the exact old parent is a frozen eligible
  unknown. The listed frontier covers every added/changed/removed structural path exactly once;
  outside it, old/current canonical structure is equal. Every current frontier child is Supported,
  Refused or an already qualified nested AggregateClosure under the same exact profile. Every
  removed child has an exact retirement. Preserve the old parent owner, follow-up and unresolved
  unchanged portion as visible residual provenance. This does not claim whole-parent support.
- CompleteCurrentSubtree: when the consumer/profile changed, no old unknown is inherited. The
  finite frontier covers the entire current parent structure, including unchanged children, with
  qualified same-current-consumer behavior or recursively complete closures. BaselineUnknown,
  metadata and ShapeDelta residuals discharge no part. An incomplete frontier remains unaccounted.

Both modes require an acyclic, nonempty exact frontier and a fresh compiled-provider proof of
current canonical structure, source/build identity, reference closure and path partition. Frozen
old structural witnesses must hash to the baseline shape using the existing canonical extractor;
they are reviewed data, never guesses from hashes. Rust uses the actual declaration and field
delta. Wire uses canonical schema positions and exact references, not plain prefix grouping.
Saved current schemas and arbitrary JSON proof objects cannot construct the private fresh proof.

Reject direct parent changes outside the frontier, missing/overlapping paths, cycles, wrong hashes,
changed child dispositions, wrong-consumer cases, unqualified children, unrecorded removals, a
sixth aggregate or stale source. The guard grants no child evidence. Required behavior cases are
the deduplicated union of child cases executed normally; one generic schema case cannot qualify
downstream consumers. Existing behavioral claims and the six metadata relationships stay unchanged.

The possible replacement set after reclassification is395 (5 x79). Only385 have unchanged profiles
and may use ShapeDelta; the other10 require CompleteCurrentSubtree under the two changed CLI
profiles. These limits are not claimed qualifications.

## Changed profiles, formats and conservation

coverage-final-merge and conformance-go-runner still need finite behavioral qualification for their
3,482 unchanged-model rows because the profile hashes changed. Added/changed models, noncompact
behavior, compact IR bytes and compact/Go refusal remain explicit obligations. A refusal before
model input proves that command boundary, never a model cell. An old-mode source comparison alone
admits no compatibility-unknown disposition.

Use closed ess-consumer-accounting/2 candidate, execution-plan and qualified stages. AggregateClosure
is the fifth cell disposition; acquisition is a separate mandatory set. Retain a genuinely closed
v1 reader with its original four dispositions. Before adding v2, add old-reader rejection cases for
v2 envelopes, new dispositions and acquisition fields. A shared enum must not silently extend v1.
Preserve unrelated ESS formats, canonical bytes and native-case protocols. All new authority readers
reject unknown fields. Bind exact authority digests and current execution proof in v2 receipts.

Current totals conserve Supported + Refused + BaselineUnknown + SchemaDocumentMetadata +
AggregateClosure. Retired historical cells and acquisition qualifications are separate totals.
Before reconciliation, reclassification predicts198,522 current cells:134,003 unchanged eligible
unknowns,50,646 missing,8,380 stale current cells,5,487 existing behavioral candidates and6 metadata
candidates. This arithmetic proves no execution. Do not subtract the3,482 changed-profile rows as
old unknowns or count the80 overlapping retirements twice.

## Fault sensitivity and acceptance

Mutations cover ninth-profile exclusion, missing acquisition rows, wrong role/mode, forbidden
traversal, changed identity/bytes, default probes on omission and missing/reused retired tuples.
For omission, use a real authored-body mutation that adds a discarded directory probe and retains
the same signature. Send that body through the production AST extractor and candidate constructor:
the previous signature-only authority admits the gap, while the corrected reviewed body/module
binding refuses it. An arbitrary edited digest string alone is not that causal experiment.
Also retain an on-disk source mutation through actual compiled extraction and normal guard entry,
restore production bytes, regenerate the current provider and execute all eight unchanged cases.
Aggregate mutations cover parent/ref/child changes, incomplete/overlapping partitions, wrong-consumer
evidence, stale provider proofs, changed-profile ShapeDelta, unknown residual in a complete frontier
and one missing executed child. Wrong v1/v2 envelope/disposition combinations refuse. A parsing test
alone cannot prove an executed guard; retain causal failures for each meaningful guard.

Independent design review and exact finite implementation scoping precede dispatch. The owner still
requires every current cell reconciled, actual case/guard execution, zero stale/unaccounted cells,
default task consumer-check and task check, plus task site-build. A framework or smaller passing
submatrix does not close the story or ESS evolution.
