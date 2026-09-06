# Observation scope and topology coverage

Binding for `story:review-observation-completeness` (F06). The credential edge collects;
the domain validates claims; the compiler retains them; consumers qualify their conclusions.

## Acquisition and authority

Legacy cluster scans retain observation/1 bytes on success. A failed read terminates collection;
there is no retry into the current namespace. No failure publishes a partial observation.

An explicit namespace topology scan requires an explicit context. Namespaced kinds are read only
with that namespace. The Namespace is read by exact name. Nodes are read individually from the
observed Pods' node references, never listed cluster-wide. Each response is checked against the
requested identity/scope before use. Collection is sequential and is not an atomic API snapshot.
Claims describe the collector's request and successful responses, not cryptographic attestation
of the API server, and namespace names are not immutable cluster identities.

## Safe topology profile

Only explicitly selected structural fields cross the serialization boundary. Unknown Kubernetes
fields, annotations, managed fields, ConfigMap/Secret payloads, literal environment entries,
container commands/arguments and probes are omitted. Resource identities, labels, images,
selectors, references, resource envelopes and runtime state remain available. Labels are
structural inputs and are not a general secret-detection mechanism.

Coverage is a closed Rust type, not free-form prose or an arbitrary property map. It records the
namespace, the `namespace_topology` profile, and the fixed kind selection. Every required kind
must be present, including a present-empty collection. Nodes describe referenced nodes only.
Omitted payloads are unobserved, never an assertion that their values/keys/probes are absent.
Unknown selector terms, including unsupported matchExpressions, are refused before compilation;
they cannot become an empty conjunction that matches every Pod.

## Compatibility and identity

New scoped observations use `infra-observation/2`; the reader still admits observation/1.
New qualified models use `infra-ir/2`, with coverage in the canonical model digest. IR/1 rejects
coverage and retains its exact canonical bytes. Frozen old-reader checks precede the added field.
No implicit conversion strips qualifications or promotes a legacy bundle to complete collection.
Clones and checked transformations retain coverage; a changed qualification is not a model edit.

Graph and drift exports of qualified models use version 2 and retain coverage. Equal scope/profile
is required for comparison; legacy/qualified or different namespace comparisons refuse. A drift
report describes changes only in observed topology; omitted values remain unknown even when its
change list is empty. Referenced-node membership is not cluster node creation/deletion.
A changed qualified model digest without a detailed field change emits `TopologyDigestChanged`;
runtime or membership changes outside the detailed rules cannot produce a false unchanged result.
Diagnosis reports the limited observation instead of diagnosing missing omitted data. Simulation
returns unknown for these observations. Projection refuses before creating artifacts. Native
properties can report the structural fields collected for an observed workload; aggregate
invariant candidates are withheld for the qualified profile.

## Verification and rollout

Offline Rust fake kubectl tests cover exact scope, failure without fallback, response identity,
redaction sentinels and destination preservation. Admission, digest/round-trip, selector refusal,
graph qualification, drift comparison, unknown simulation and projection refusal have regressions.
Mutation verifies the redaction test detects bypassed protection. Existing unqualified fixtures
remain independently frozen and continue to pass. Full repository and site gates precede landing.

The ESS readers and writers ship together; callers must upgrade before opting into the new profile.
Legacy commands remain compatible. No automatic dependency promotion or release is authorized by
this design. The prior local inventory found no established foreign reader; it did not establish
that no deployed reader exists. Cross-repository consumers must pin the new producer before using
the new formats, and must not down-convert them to version 1.
