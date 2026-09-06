---
format: aep.planning-md/1
id: review-result:cache-origin-binding-pass1
kind: review-result
status: active
title: Cache origin candidate binding review
relations:
- reviews: story:review-cache-origin
revision: 1
---
unit: cache-origin candidate binding v1, SHA256 719c29e9f36a67990133c74f33a71b372e4b6ed90c23dd41bfa640af39ef7985
verdict: nothing found
cases: executed not run→not run, red not measured
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: No binding correction identified; retain fresh-source/selection requirements and complete the separately acknowledged story Scope wording correction.

Scope/diff proof: this is pass 1 of the explicitly assigned document-only adaptation of aep-drive:adversary 0.8.0. The full installed charter and fixed draft were read. No Git/diff command, test, build, gate, ESS/ORAS/Helm execution, network/registry operation, source/store mutation or active-writer inspection occurred. Only this report, its manifest and two exact story snapshots were written under the assigned cache-binding-review-1 scratch directory. There is no executed failing case or source approval to report.

No concrete blocker or warning was found in the fixed candidate binding. Its profile and cache choices can meet the stated content-binding acceptance within the three paths. This conclusion is static document/source correspondence, not evidence that a cache implementation, race test or timeout guard has passed.

## What was attacked

All draft line references below name target/review-boundaries-11/cache-origin-preparation/binding-draft-v1.md.

| Boundary | What was checked | Static assessment and limits |
| --- | --- | --- |
| Both F11 consumers | Draft :10–13 names release bundles and Helm charts, withheld output/Helm use, offline proved hits, legacy cold acquisition and corrupt-proof refusal. | It resolves the old bundle-only ambiguity. Current source still has two weak cache routes, at main.rs:1469–1497 and :3135–3175; the draft does not claim they are already fixed. |
| Requested identity | :24–38 hashes original manifest bytes before interpretation and then checks exact selected descriptor size/digest on every cold/warm read. Duplicate keys, exponent/fractional/overflowing sizes and unsupported fields fail admission. | No route was found that can choose the expected manifest identity from downloaded proof or a local sidecar. A complete self-consistent replacement manifest still has to match the caller's requested digest. |
| Actual ORAS profile | :52–60 admits the measured artifact type, exact layer type, empty-config media type/digest/size and optional literal e30= data; :40–43 admits inert annotations. | It fits the retained local ORAS1.2.3 synthetic publisher manifest, including config.data, title and created annotations. Fixed {} bytes are already available and their digest/size can establish that config's content without a fetch; no registry availability or signer claim follows. Older unmeasured OCI1.0 forms are explicitly outside the profile. |
| Helm profile | :62–71 binds one chart, an optional provenance layer, both permitted orders, the documented config media type and a finite artifactType rule. Config/provenance remain opaque verified content. | No conflict with the coordinator-supplied documented profile was found. The draft explicitly distinguishes this selection from an actual installed3.8.1 or historical publisher measurement. Helm remains the semantic chart consumer; a matching blob digest does not prove chart metadata or a signature. |
| Complete admitted graph | :48 and :92–96 require config and every admitted layer, with exact order/call counts. Bundle config is the explicit fixed-byte exception to fetching, not to verification. | An optional provenance blob cannot be silently omitted or used as the chart. Two, three and four cold calls follow from the chosen bundle, plain Helm and provenance Helm profiles respectively. |
| Resource handling | :82–90 fixes manifest/config/bundle/chart/provenance limits, limits reads to limit+1, checks descriptor sizes before fetching, uses one monotonic acquisition deadline and drains diagnostics with bounded retention. | The framing can be read with bounded lengths rather than allocating according to unchecked u64 values. ORAS disk use and internal memory are explicitly excluded from the resource guarantee. Kill/reap and diagnostic-drain behavior remain tests to execute. |
| Single-file cache admission | :100–113 binds a new namespace, exact ESSOCI1+LF magic, big-endian lengths/count, ordered original blobs, exact count and end-of-file. | The complete entry is self-contained; profile/magic/trailing-byte checks prevent legacy or differently profiled files being treated as proved hits. The original caller digest and original manifest remain the admission authority, rather than self-declared proof hashes. |
| Publication and corruption | :115–128 requires unique same-filesystem staging, full staged re-admission, atomic no-replace hard-link publication, valid-winner reuse or refusal, and no repair/deletion of a corrupt winner. | It separates completeness from mere path existence. An interrupted staging name is not a hit; a cooperative losing writer cannot merge partial bytes with a winner. Unsupported filesystems explicitly refuse. Power-loss durability and garbage collection are not promised. No actual filesystem race or platform test was run. |
| Consumed bytes | :124–135 requires an owned read handle/verified bytes, no later shared-data reopen, and an invocation-private chart whose lifetime spans Helm completion. | The contract closes the specific shared-cache replacement window: replacing the cache later cannot choose the chart bytes passed to Helm. It does not use a hostile-local-process exclusion to excuse ordinary symlink or cooperative-writer cases. |
| Earlier guards and rollout | :137–141 preserves desired/current validation, dry-run/removal refusal before acquisition, existing Helm options and sequential release order. | It matches the source's pre-executor ordering at main.rs:1628–1703. A bad later chart stops that release/later operations without inventing rollback of earlier Helm calls. No whole-plan recovery expansion is required. |

The content graph is anchored in the supplied digest, not in publisher authenticity. The caller's repository still selects the fetch endpoint; complete locally retained bytes can establish content equality without proving that an authorized publisher signed them, that a particular registry remains available, or that a release was applied. The draft maintains these distinctions at :24, :48–50 and :137–141.

## Source fit and preservation

The CLI already has the dependencies needed for strict JSON decoding and the existing SHA-256 type. ess-deployment/src/identity.rs:72–105 provides strict lowercase SHA-256 parsing and exact-byte hashing. The pure library remains read-only: ReleaseBundle's checked readers and validation in component.rs:245–310/:398–401 still own nested model/graph consistency. New private manifest/descriptor and cache-framing types can remain in the CLI package; no general OCI SDK or new public ESS envelope is required by the selected semantics.

The bundle route must adapt its current verified_bundle path helper (main.rs:3050–3070) to operate on the exact already checked bytes while retaining the canonical JSON and graph checks. The draft explicitly requires that at :57–59 and :132; it does not authorize replacing semantic validation with hashes. The existing publisher flags and canonical bundle bytes at main.rs:1432–1466 are preserved.

The old Helm path returned a shared cache filename after checking a local checksum (main.rs:3135–3152). The selected private snapshot at draft :133–135 requires the implementation to keep that snapshot alive through the actual executor call. The return type/lifetime adaptation fits the CLI package and is part of the required implementation, not an inferred existing guarantee.

The package reservation also contains the affected test files. persisted_delivery.rs:93–105 currently uses placeholder chart digests, and its successful test at :163–175 expects exactly oras, helm, helm. tests/support/fake_delivery.rs:16–26 only creates a chart in an output directory and does not implement manifest/blob retrieval. Draft :154–156 explicitly replaces this positive fixture with a linked original-byte graph and the exact three-ORAS/Helm/warm-Helm sequence for the selected shared plain chart. Retaining the obsolete one-pull assertion would be wrong; weakening all call assertions would also be wrong. The five existing invalid-plan/duplicate/order groups remain preservation controls.

Draft :145–162 requires actual CLI cold and offline-warm checks, each relevant mismatch/profile/limit case, output/cache preservation, verified chart snapshots, deterministic I/O failure seams, a real two-writer contention case, stalled-client kill/reap and targeted red mutations. These are meaningful obligations against the actual failure surfaces. They remain unexecuted. The previous four-command local ORAS probe proves only synthetic publisher transport shape; its success cannot stand in for any of these CLI/cache tests.

## What this review does not establish

- No implementation exists in this review, so this is not a passing source gate, mutation result, filesystem portability result, process-timeout proof or cache-substitution reproduction.
- The documented finite Helm profile is a chosen compatibility boundary. Actual installed/historical publisher output remains separately qualified; the draft is explicit about that limit and requires fixtures for admitted forms.
- No guarantee was inferred for hard disk quotas inside ORAS, power-loss durability, signature authorization, chart metadata validity or applied-state recovery.
- Exact numeric-token parsing, bounded stream drainage, staging cleanup/error handling, hard-link winner admission, snapshot lifetime and independent fixture expectations still need the implementation evidence the draft assigns.
- No extra reservation for the deployment library, release action, root manifests/locks or whole-plan recovery was found necessary. A concrete dependency need still requires re-scope as :19–20 says.
- The final integrated CLI must be freshly inspected before later implementation selection. An unchanged relevant-file hash is not a substituted integration gate.

## Story pins and coordinator reconciliation

The original scope inspected draft revision2: 2,425 bytes, SHA256 ffde3b201bef237d66cfbf7dc1b4a6f1482d1f7c703d5b0190072a248a7db281. Root then applied the three scope paths and explicit both-cache acceptance through AEP. This review separately pinned revision8: 6,355 bytes, SHA256 8c3ebee3b23fa5e2f27a6050996ce9535487299ed13038be4a13cd5cf6b9b4e7. Both original byte sequences are retained under inputs/story-revision2.md and inputs/story-revision8.md; the first pin was not overwritten.

Revision8's copied Scope acceptance bullet still uses the older “current story acceptance is bundle-specific” wording although its actual Acceptance now includes both caches. This residue was reported to root, which acknowledged a separate CLI wording correction after reconciliation. It is outside the fixed candidate binding, whose acceptance already names both consumers; no source or binding finding is assigned to it here.

While root replayed the canonical store, this review used its preserved revision8 snapshot and did not read transient live revisions. Root subsequently confirmed completion at supplied coordinator3811e902cd42bffbafbf4fa345613e6905b6a7a4 and paused its wording correction. The final live readback then matched revision8 exactly. No Git or planning command was invoked by this reviewer; the commit identity is coordinator supplied.

## Integrity and relinquishment

The 12,319-byte candidate retains its requested SHA256. All 48 current input pins matched final readback, including the final live revision8; all 18 previously sealed local ORAS evidence files remained unchanged. The seven inspected production inputs represented in the retained 504-file source baseline also match. The two separately retained story snapshots match their respective original hashes.

The first scratch snapshot copies had one extra trailing LF introduced by patch construction. Their initial hashes are retained in source-manifest.json. Only those scratch copies were corrected, after checking that removal of exactly one LF restored each original pin. No original story, source, prior report or local evidence file was altered.

Source-manifest.json: 32,957 bytes, SHA256 **56fa6defb56c42620f617800cbb1f8cb57b92b71809fe97a7dc09a9feeecc35a**. The report hash is returned separately. This report, that manifest and the two input snapshots are the complete new write set; nothing was written outside the assigned worktree scratch. All writes are relinquished after final readback. Root owns the remaining metadata wording correction, binding acceptance, fresh-source checks, later selection, implementation and delivery.

```findings
[]
```

