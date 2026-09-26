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

`ess-observed-bindings/1` (and `/2`, below) is a closed document with `format`, stable `id`, exact
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

The closed `ess-observed-bindings-report/3` report carries the exact binding digest, realization
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

A bound workload is bound as a whole. Within admitted coverage, a container or native sidecar (an
`initContainers` entry with `restartPolicy: Always`) in the observed template that no binding for
that workload names is a violation (`OBS-BIND-008`) that names it: it runs code the declared
placement does not account for. A binding may name a native sidecar exactly as it names a
container. Plain init containers run to completion before the pod starts and are not checked; the
finding's detail says so rather than claiming every container was bound. The check reads only the
bound workload's template, so workloads no binding selects remain uncovered rather than violated.

The report moves to `ess-observed-bindings-report/2`: the same fields, a new check and therefore
new semantics, so a reader of `/1` must not read `/2` as `/1`. The input stays
`ess-observed-bindings/1` with the same fields, and its meaning widens: a document that binds some
containers of a workload now claims that the workload runs nothing else. A document whose bound
workload carries an unbound sidecar, which was satisfied under report `/1`, is violated under `/2`.

### Acknowledged foreign containers

A third-party proxy or agent in a bound workload is not an implementation of this realization, and
binding it to one would misstate what the service builds. `ess-observed-bindings/2` adds optional
`foreign_containers`: per bound workload, the containers the document acknowledges as foreign, each
with a DNS-label `name` and a nonempty `reason`, and no image. The format moves because the
document can now say something `/1` could not, and what a satisfied `OBS-BIND-008` means depends on
it (the rule in `website/docs/reference/spec-versions.md`, "When the number moves", and the
repository's `AGENTS.md`). An optional field under `/1` would have been refused by older readers
only through `deny_unknown_fields`, a refusal that blames the document rather than naming the
version.

Before collection, an acknowledgement is refused when its workload is bound by no binding (it
would never be read), when it names a container a binding also names (a container is built here
or foreign, never both), when a workload appears twice or a container twice in one workload, when
its container list is empty, or when a reason is blank. A `/1` document carrying the key is
refused, even as `[]`; a `/1` document without it is read unchanged, acknowledges nothing, and
keeps its binding digest, since an absent field is omitted from the digested bytes.

At evaluation, an observed container or native sidecar that is acknowledged counts as accounted
for, and the binding result lists it under `acknowledged` with its reason; it is not bound, so no
artifact check covers it. Its image is compared with this document's own: one running a binding's
declared `image` or any implementation's artifact locator is this realization's code under another
name, and so is one whose `@sha256:` digest equals the digest pinned by either, or a container artifact's `identity` (the image digest `OBS-BIND-005` compares): a digest names the
artifact, and the repository and tag in front of it only say where it was fetched. Either violates
`OBS-BIND-008`, naming it, without being listed as acknowledged. Tags are not compared, since the
same tag under another repository is not evidence of the same bytes.

An acknowledgement naming no observed container or native sidecar cannot be shown stale. The
infrastructure model keeps containers and, from producers that collect them, native sidecars; it
never keeps a plain init container's name (`native_sidecars` in `infra-domain` filters on
`restartPolicy: Always`, and `infra-ir` has no field for the rest). So the observation
distinguishes present containers and native sidecars from everything else, and "everything else"
always includes plain init containers it did not keep. Such an acknowledgement leaves
`OBS-BIND-008` `unknown`, never satisfied, and the detail names it and says why; where init
containers went unrecorded altogether it may also be an unrecorded native sidecar. Turning a stale
acknowledgement back into a violation needs plain init container names in the infrastructure
model, which is an `infra-ir` change and not part of this version. The report moves to
`ess-observed-bindings-report/3`: the `acknowledged` field is new, and a satisfied `OBS-BIND-008`
no longer means every entry is bound, so a `/2` reader must not read `/3`.

Native sidecars reach the comparison through an optional `native_sidecars` field of the
infrastructure model (name and image only). Unknown differs from false, so the field has three
states: absent means the observation is not taken to have recorded init containers, an empty list
means it recorded them and none is a native sidecar, and a list names each one.

What ESS 0.31.0 wrote differs by path. Its namespace-topology collector dropped `initContainers`
from every template. Its full scan copied each API object verbatim, so a template with init
containers carries the key there; the API server omits an empty list, so no 0.31.0 document carries
`initContainers: []`. From this release on the namespace-topology collector writes the key for
every template, `[]` when there are none, as its statement that it looked.

The compiler writes `native_sidecars` for a workload when, and only when, one of these holds:

1. the template's `initContainers` holds at least one entry with `restartPolicy: Always` — the field
   lists them;
2. the template's `initContainers` is the explicit empty list `[]` — the field is `[]`;
3. the observation's `scout_version` is a plain release at or after 0.32.0, the first that collects
   init containers — the field is `[]` when neither of the above lists anything.

In every other case the field is absent: no key, a list of plain init containers only, or a
producer version that is older, a pre-release or a label such as `synthetic`. A document in any of
those cases compiles to exactly the bytes and digest ESS 0.31.0 computed for it, since the field is
the only addition and it is omitted. A 0.31.0 full scan with a native sidecar does gain the field,
and its digest moves; ESS 0.31.0 silently dropped a running container there, which is the reason for
the field. An IR document is read as written: absent stays absent, whatever its provenance says,
because only the compiler that wrote it knew.

Where the field is absent and every container is bound, `OBS-BIND-008` is `unknown` and its detail
names the producer version and the first release that collects init containers. An unbound plain
container still violates, since the evidence of it is in the containers list itself. A binding
that names something not among the containers is `unknown` under `OBS-BIND-003` rather than
violated, because it may be an unrecorded native sidecar.

No format version moves. A reader older than the field refuses a document that carries it, because
the model's mirrors deny unknown fields.

## Live acceptance

`cargo xtask infra-acceptance` (`task infra-acceptance`) runs the comparison against a disposable
k3d cluster that the harness creates and deletes. It builds a generated service into one container
image, sets the cluster up from its own manifests (never from an ESS projection), reads the
namespace live, projects the placement intent twice over the same observation and requires identical
bytes, runs wrong-tag, missing-workload and extra-container sensitivity cases with a restore after
each, checks every observation and report for a Secret's value, and reads back the absence of the
cluster, its images, containers, volumes and scratch files. It needs Docker and is not part of the
offline gate.

Two guards hold the no-projection property. A syntax-level test reads the harness source: every
program it starts must be one that function may start, no argument names kubectl outside its one
wrapper, a writing verb other than the one `apply`, `docker cp`, a k3d volume or the node's
manifest directory, and the only non-template input to a rendered manifest is a random value
only the harness can mint. At run time every command is refused before it starts if its program is
not one the harness runs (`k3d`, `kubectl`, `docker`, `tar`, cargo and the built `ess`, named
exactly), if an argument lies under a projection output, or if it names a cluster, context or
node that is not this run's own.
A run name that does not select an `ess-m8-*` cluster is refused, and so is creating a cluster
whose name already exists.

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
