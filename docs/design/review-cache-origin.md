# Checked OCI cache origin

Status: accepted for wave 12 under the standing remediation implementation approval. Owns
story:review-cache-origin and both cache consumers named by F11. The integrated coverage CLI
was independently refreshed at d1fe6755; subsequent 8e3c8ab5 changes only closing records.
Source observations and their limits are in the sealed cache scope report, SHA256
e6cd5709142582cc406fe8583c23045c71b0fcb3b0ce411119fb0e68ad6da69a.

## Acceptance and ownership

A self-consistent release bundle or Helm chart substituted under another requested OCI manifest
digest is refused before its bytes are returned, written as the requested output, or passed to
Helm. A valid proved cache hit works with the external fetch client unavailable. A legacy entry
has no proof and causes cold acquisition; corrupt proof causes explicit refusal.

The CLI package owns this transport/cache boundary. Existing ess-deployment checked types,
canonical bundle admission and strict SHA-256 Digest are dependencies, not edit targets. Own
crates/edge/ess-cli, docs/design/review-cache-origin.md and
website/docs/concepts/component-delivery.md. Root owns planning, changelog, integration and
publication. No release action, manifest/lock, general output recovery or execution-recovery
change is selected. A concrete dependency need must be re-scoped before adding it.

## Identity before interpretation

The caller's pinned repository and SHA-256 manifest digest are the authority. Retrieve the entire
original manifest, compare its SHA-256 with that requested digest, then decode it. Never hash a
JSON reserialization or accept an identity learned from downloaded bytes as the requested one.
Retain the original manifest bytes, including whitespace and annotations. The same rule applies
on cold acquisition and every warm read. A valid descriptor supplies both exact byte size and
SHA-256; both must match the original referenced blob before any payload semantics or executor.

JSON admission is closed and rejects duplicate object keys, including annotations, throughout
the manifest. schemaVersion must be integer token 2. Descriptor size must be an exact nonnegative
integer token representable as u64, within the relevant limit; negative, fractional, exponent,
overflowing or rounded forms refuse. Hashes use the existing strict lowercase SHA-256 grammar.
Manifest fields are schemaVersion, mediaType, optional artifactType, config, layers and optional
annotations. Descriptor fields are mediaType, digest, size, optional annotations, and only the
specific bundle-config data allowance below. Other fields, subject, URLs, platform, indexes,
Docker manifest lists and unsupported media types refuse with a profile reason.

Annotations are inert string-to-string maps. They never select a local path, payload, client
argument or endpoint. Reject duplicate keys and non-string values. Allow at most 64 pairs per
map, keys at most 4,096 UTF-8 bytes and values at most 8,192; the complete manifest bound also
applies. These are chosen local resource limits, not OCI specification limits.

## Two finite profiles

Both profiles require schemaVersion 2 and mediaType
application/vnd.oci.image.manifest.v1+json. They cover every admitted referenced blob, not merely
the executable payload. No signature, publisher authorization or installed-state guarantee is
made. The local publisher-shape probe used synthetic bytes and did not prove ESS cache behavior.

**ESS bundle:** require artifactType application/vnd.beyond10x.ess.release-bundle.v1, exactly one
layer of application/vnd.beyond10x.ess.release-bundle.v1+json, and config media type
application/vnd.oci.empty.v1+json. Config must identify exactly the two bytes {} with size 2 and
their existing SHA-256. Its optional data must be exactly e30=, as emitted by measured ORAS 1.2.3;
otherwise refuse. The fixed verified {} bytes are retained as config without a client call,
whether data is present or absent. Layer data is unsupported. After descriptor verification,
the selected bundle must pass the existing checked-reader, graph verification and exact canonical
JSON check. Preserve the publisher's current flags and bytes. Older unmeasured OCI 1.0 bundle
forms are explicitly unsupported rather than guessed from their cache contents.

**Helm:** require config media type application/vnd.cncf.helm.config.v1+json, exactly one chart
layer of application/vnd.cncf.helm.chart.content.v1.tar+gzip, and zero or one provenance layer of
application/vnd.cncf.helm.chart.provenance.v1.prov. artifactType may be absent or equal the config
media type; any other value refuses. No descriptor data field is admitted in this profile.
Either layer order is admitted, but duplicate chart/provenance layers and other layers refuse.
Fetch and verify config and optional provenance as opaque content; their presence does not
establish chart metadata correctness or signature verification. Helm remains the chart semantic
consumer. These media types are the chosen contract from the official Helm registry documentation,
not a claim about an observed installed 3.8.1 publisher or every historical remote artifact.
Tests must include an independently assembled original-byte fixture for each allowed form.

## Acquisition and resource limits

Use the already measured installed ORAS manifest fetch and blob fetch surface, with separate
argv and explicit owned output files. Fetch the manifest at the requested repository@digest;
fetch blobs only at that same repository plus their checked descriptor digest. Never use oras
pull, descriptor URLs, title-based extraction, an unpinned tag or a shell interpolation. Preserve
existing credential discovery; no new credentials, registry query or live executor is required
for offline tests. Local fake clients must distinguish exact requested operations and identities.

Read manifest at most 1 MiB, config at most 1 MiB, bundle at most 32 MiB, chart at most 64 MiB
and provenance at most 1 MiB. At most three blobs exist (config plus at most two layers); each
declared size is checked before launching its fetch. Bound decoding/read allocation by reading
at most limit+1 bytes, then refuse oversize, rather than trusting metadata before an unbounded
read. Use one monotonic 60-second deadline per cold acquisition, including all sequential client
calls; a timeout kills and reaps the owned client before returning. Bound retained diagnostic
output to 64 KiB per stream while draining it without unbounded buffering. These limits do not
claim a hard disk quota or memory bound inside ORAS; it writes only private staging until exit,
and excessive returned bytes cannot be allocated, admitted, published or consumed by ESS.

Cold call order is manifest, then config if not the fixed bundle config, then each layer in its
manifest order. Thus the measured bundle form uses two calls; a Helm chart without provenance
uses three, and with provenance four. Do not silently omit config/provenance checks or loosen
positive fixture assertions to accept arbitrary calls. Every client nonzero exit, timeout,
missing/truncated output, unsupported profile or proof mismatch refuses.

## Private cache proof and publication

Use a new private namespace beneath the supplied cache root:
oci-proof-v1/<bundle-or-helm>/sha256/<manifest-hex>.entry. Existing sha256/... bundle entries and
helm/sha256/... chart/checksum pairs are untouched and ignored for proved-hit admission. A cold
fetch failure preserves them and the requested --out destination. There is no trusted-local
import or automatic corrupt-entry repair route in this change.

One complete entry is one regular file, avoiding publication of a partially populated directory.
Its internal binary framing is the eight-byte magic (seven ASCII characters ESSOCI1 plus LF), a big-endian u64
manifest byte length, the exact manifest, a big-endian u32 blob count, then for config and each
layer in manifest order a big-endian u64 length and original blob bytes. Refuse wrong magic,
truncation, lengths/count outside the profile limits, missing or extra blobs, or trailing bytes.
The file need not carry self-declared proof digests: rederive each check from the caller and
original manifest on every read. This is a private cache interpretation version, not a public ESS
wire format. A profile mismatch cannot be healed by renaming the entry into another namespace.

Write the complete entry into a uniquely created staging file on the final directory's same
filesystem. Verify the full staged bytes using the same cache reader before publication. Publish
with an atomic no-replacement hard link to the final filename, then unlink only the invocation's
own staging name. If a final entry already exists, verify and reuse a complete valid winner;
otherwise refuse without replacing, merging or deleting it. Do not use a replacing rename.
If the filesystem does not support that operation, refuse explicitly. Private staging leftovers
after process death are never hits; this change does not sweep other invocations' leftovers.
The promise is process-interruption consistency, not power-loss durability or a general cache GC.

Observed symlink/nonregular cache entries refuse. Read original bytes through an owned open handle
and verify what was read; no later reopen of shared data supplies payload bytes. This boundary
does not claim to sandbox a hostile local actor who controls the executing process or its private
directories. Tests must still cover ordinary symlink/nonregular entries and cooperative concurrent
writers, not rely on that limitation to skip cache-substitution checks.

## Consumption and existing execution order

Bundle semantics are decoded from the exact verified bytes and preserve existing canonical output.
Helm receives an invocation-private chart snapshot written from verified owned bytes, with its
lifetime extending through child completion. Replacing the shared cache after verification must
not change the chart passed to Helm. No archive extraction is added here.

Both persisted desired/current plans still validate before any cache acquisition or executor;
dry run and removal refusal retain zero external calls. Sequential release reconciliation remains
sequential: a bad chart prevents that release's Helm call and stops later work, but does not undo
an earlier completed release. No whole-plan preflight, rollback or applied-state evidence is
implied. Preserve current Helm options and release order.

## Required implementation evidence

For both consumers, actual CLI fake-client tests exercise cold success, offline warm success with
a trap client, wrong raw manifest including whitespace-only identity change, changed config/layer,
size disagreement, self-consistent replacement under the wrong requested digest, legacy cold miss,
corrupt/incomplete proved entry, unsupported profile/ambiguous layers and URL/title tricks. Check
exact returned/consumed bytes, client sequence and unchanged output/cache sentinels on refusal.
Include actual duplicate JSON keys, exact-size token negatives and each boundary limit.

Exercise missing and existing --out destinations on refusal. For Helm, prove no call for the
failed release, preserve sequential behavior, and replace the shared cache between admission and
executor use to verify the invocation snapshot. Update the existing positive persisted-delivery
fixture with genuinely linked original manifest/config/chart bytes and exact calls (three ORAS
then Helm, warm Helm for a shared no-provenance chart); retain every invalid-plan assertion.

Deterministic I/O seams cover interrupted staging, failed publication and a concurrent complete
winner. Also exercise two real local writers contending for the same entry and a stalled fake
client: timeout must kill and reap it, with no admission/publication/output/executor afterward.
Retain red mutations that bypass requested-manifest hashing, descriptor verification and the
verified consumption snapshot; their targeted assertions must fail before restoration.

Keep full per-command outputs/exits and exact source/tool/fixture identities. No source gate or
cache-correctness test has run for this candidate. The implementation runs appropriate package
checks; the coordinator runs task check, site-build, independent source review and normal public
delivery. The source refresh and independent binding review are complete; implementation evidence is pending.
