# Infrastructure IR ownership and checked transformations

Status: binding for `story:review-infra-ir-invariants`.

## Problem and ownership boundary

`InfraIr.model` currently lets a consumer remove a referenced map entry after obtaining one of
the six private handles. The owning IR then cannot honor its total lookup contract. The owning
model becomes crate-private, with `model(&self) -> &InfraModel` as the public query. Detached
`InfraModel` values remain ordinary editable data. Neither a mutable model query, unchecked
public constructor/replacement, nor `Deserialize` on `InfraIr` or its handles is introduced.
Provenance remains public: editing it cannot invalidate a handle.

The six classes are node, service, configmap, secret, service account and persistent volume claim.
Handles belong to the owner that minted them; the existing prohibition on using a handle against
another IR remains. This change does not add generative lifetimes or cross-owner branding.

## Checked transformation

Add `InfraIr::try_transform(&self, edit: impl FnOnce(&mut InfraModel)) ->
Result<InfraIr, infra_domain::ValidationErrors>`. It clones the model, invokes the edit on that
detached candidate, preserves source provenance, computes the candidate document using the existing
writer, and admits it through `read_document`. That reader remains the single owner of closed
document shape and reference membership validation and remints all six handle classes. The source
owner is never modified, including when candidate admission fails. No candidate owner is returned
before successful admission.

Four narrow compiler capabilities for replicas, resources, probes and disruption budgets would
avoid serialization but duplicate projector concepts and introduce four privileged mutation paths.
The checked detached transformation uses existing model types and admission without importing
`infra-project::Change` into the compiler. Its cost is a model clone plus a serialize/read pass per
candidate; this is acceptable for the existing offline projection loop. No performance improvement
is claimed.

This admission protects reference membership, not every original domain-value rule. In particular,
`read_document` does not rederive unresolved facts or observation completeness. Callers editing those
semantics retain those obligations; this story does not silently expand the reader's meaning.

## Projector transaction and failure

The projector keeps its four existing `Change` variants and applies each to a detached candidate.
Candidate admission must succeed before recording an artifact, generated disposition, slot,
created object or fixed-point progress. Only then does the workbench replace its working owner and
record the corresponding change. The same `Change` still drives the model edit and emitted patch.

Make `project(&InfraSpec, &InfraIr)` return `Result<Projection, infra_domain::ValidationErrors>`.
An admission error aborts the projection and returns the existing typed reader errors. The CLI
propagates that error before writing any projection files. There is no partial successful
projection, silent fallback to the old working model, or new serialized refusal reason. Existing
successful projection documents and artifact bytes remain unchanged. This is a deliberate Rust
API migration for all current callers in `infra-project` and `ess-cli`.

## Persisted compatibility

Keep `infra-ir/1`, its envelope, field order, reference encoding, canonical model bytes and digest
algorithm unchanged. `InfraIr` continues to serialize identically despite field privacy; its
persisted document retains the existing borrowed model representation. Successful simulation,
drift and projection products likewise retain their bytes. This is an ownership/API change, not
an observation-format or unresolved-accounting migration.

Freeze output from the unmodified base writer before implementation and compare the corrected
writer and reader against it. Retain the committed example products and their actual package
determinism tests. A new writer reading itself is not the old-byte comparison.

## Verification

External Rust probes exercise direct clearing, replacement, nested mutation, shared-query mutation,
unchecked owner construction and deserialization; valid public read/detached-data controls must
compile with the same compiler and dependency selection. The direct old-model mutations must
demonstrably compile before correction, causing their rejection tests to fail.

Runtime tests call all six total lookups after compilation, persisted reading, cloning and a valid
transformation. For each referenced target family, candidate deletion must be refused without
changing the source document, digest, provenance or already obtained handles. Successful and failed
edits exercise the actual public transform rather than a second test validator.

Projector tests retain real emitted patch/object application to observation bundles, recompilation,
outcome comparison and corrupted-patch controls. Cover replicas, resources, explicitly stated
probes, disruption budgets and induced fixed-point gaps. Test that rejected candidate admission
leaves all workbench recording state unchanged. Run the five assigned package suites, their
formatting and strict Clippy; integration and site gates belong to the coordinator.
