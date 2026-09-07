# Observed component bindings

The first consumer is a legacy monolith whose existing semantic ESS can be pinned, whose
implementation source is known, and whose Kubernetes workload template is observable. The
connection must be repeatable without an agent, must not invent build provenance from a tag,
and must not turn an observed snapshot into desired state.

## Existing authorities and boundary

`ess-realization/1` already binds exact semantic ESS components to immutable implementation
artifacts. Reuse its compiler and expose shared read-only queries for compiled implementations
and their artifacts. `infra-ir/1` and `infra-ir/2` retain their existing meanings. The CLI already
consumes both libraries and owns this first cross-context comparison; put the pure comparison
in its own native Rust module. No dependency is added from the infrastructure context to ESS,
and no existing persisted format or runtime one-workload-per-component rule is relaxed.

## Authored input

`ess-observed-bindings/1` is a closed document with `format`, stable `id`, exact
`realization_digest`, `scope` (explicit Kubernetes context and namespace), and nonempty `bindings`.
Each binding has a stable `id`, an existing `implementation`, a `workload` (kind and name), an
explicit `container`, and the declared `image` reference. Optional source evidence records name
repository, exact Git revision and file path; they are attribution, not proof of file contents.
The semantic component membership is read from the compiled realization, never duplicated here.
Several components can share an implementation and several explicit deployment roles can reference
that implementation. A container binding does not assert which HTTP operations it exposes.

Reject unknown fields, invalid identifiers or namespace/workload/container coordinates, duplicate
binding IDs and duplicate target tuples, empty selections, incompatible realization digests and
unknown implementation references before live collection. Scope is exact; there is no ambient
context fallback. Authored image values are exact expectations for the workload template.

## Native command and results

`ess verify bindings --spec <ess-source> --realization <document> --bindings <document>` takes
either `--infra <observation-or-ir>` or `--live --observation-out <new-external-file>`.
Live acquisition reuses the native namespace-topology collector, then the same infrastructure
reader as offline verification. Require an explicit unused output outside Git checkouts; do not
clobber observations or accepted models. `--format json` writes a single deterministic report;
`--markdown-out` optionally writes a generated reference table after report admission.

The closed `ess-observed-bindings-report/1` report carries the exact binding digest, realization
digest, observation digest/provenance/coverage when available, per-binding semantic component and
implementation identities, checks, and explicit exclusions. It is a result document, not an
admissible authority for another check. The report states observed workload UID and image reference.
No clock is consulted for offline checks; the timestamp is the input observation's timestamp.
Live mode performs a new collection. Sequential reads are not an atomic runtime snapshot.

Check the scope, workload, container and image reference. Wrong context or absent/unequal namespace
coverage makes the binding unknown: missing targets outside admitted coverage are not evidence of
absence. Within admitted coverage a missing workload/container or unequal expected image is a
violation. Reuse admitted infrastructure identities; never join by Pod name prefixes.

Image provenance has separate semantics. An exactly digest-pinned container artifact and an
identical digest-pinned workload reference establish artifact-reference agreement. Source artifacts
and tag-only workload references leave source-to-image or immutable-image identity unknown. A
Git tag, registry tag resolution, copied URL, or an authored evidence note cannot discharge this
check. The first legacy adopter is expected to expose this limitation honestly. The check does not
attest running Pod image IDs, build authenticity, profile-specific operation exposure, endpoint
behavior, health, replicas, secret values or omitted payloads.

Overall status is violated if any check violates, otherwise unknown if any is unknown, otherwise
satisfied. Exit codes: 0 satisfied, 1 violated/invalid authored contract, 2 unknown/uncheckable.
Malformed input or failed collection produces a machine-readable unknown report and nonzero exit;
no green empty result. Invalid references produce a refused contract before acquisition. JSON
reports retain all applicable findings and never print raw credential-adapter stderr.

## Verification

Exercise exact-digest success; legacy source/tag unknown; missing workload and container; changed
image; wrong context and namespace; old incomplete observation coverage; duplicate/empty bindings;
unknown implementation and realization digest mismatch; immutable deterministic outputs; same
implementation bound to distinct roles; and failed live collection returning a report rather than
silently reusing a baseline. Keep all real adopter observations outside this public repository.

## Realization admission discovered by the adopter

The released 0.19.0 reader refuses the honest backend declaration with EmptyDeclaration at
entrypoints and PrimaryEntrypoint (zero primary entries). There is no evidenced executable
semantic surface in this partial backend model. Do not invent one. Introduce explicit
`ess-realization/2` and compiled `ess-realization-ir/2` for this wider meaning. V2 admits no
entrypoints only when no actor-access or conformance claim accompanies it; component and
implementation coverage must still be nonempty and exact. Nonempty entrypoints retain all current
rules. V1 admission and canonical bytes remain unchanged. V2's realization digest hashes the
version tag together with the existing identity tuple so its meaning cannot alias V1.

The backend keeps released 0.19.0 for its established structural/schema gate. Its implementation
selection validation uses the same exact published ESS source pin as the consuming system, with
that separate prerequisite explicit. Old readers reject V2; this is an opt-in, not a silent
reader widening or a change to existing deployment cardinality.
