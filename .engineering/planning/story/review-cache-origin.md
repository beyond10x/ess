---
format: aep.planning-md/1
id: story:review-cache-origin
kind: story
status: implemented
title: Verify cached bundle bytes against their OCI identity
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-persisted-delivery-validation
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: inferred
  path: docs/design/review-cache-origin.md
- confidence: cited
  path: website/docs/concepts/component-delivery.md
revision: 15
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:1457`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A self-consistent cached release bundle or Helm chart substituted under another requested OCI manifest digest is rejected before its bytes are returned, written as the requested output, or passed to Helm. A valid proved hit works without a fetch client. Legacy entries lacking manifest proof cause cold acquisition; corrupt proved entries refuse.

This explicitly includes both cache consumers named by F11; it clarifies the original bundle-only wording rather than treating the chart finding as implicitly closed. The reviewed profile, publication and consumption binding is accepted for wave 12 below.

## Implementation boundary

Both the release-bundle fetch and Helm reconciliation cache must retain and revalidate the requested manifest-to-blob chain before content use. Preserve canonical bundle semantics, persisted-plan validation, dry-run/removal guards and sequential rollout order. Keep external fetch clients injectable. Do not silently upgrade legacy local consistency into registry identity. A concrete manifest/profile, bounded acquisition, cache publication and verified consumption binding precedes implementation.

## Validation

Fake registry/process fixtures cover correct manifest/layers, wrong manifest digest, altered layer, self-consistent replacement bundle and incomplete old cache; test cold and warm paths with no live registry.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This establishes content binding, not publisher signature trust.

## Opening scope (retained)

Derived 2026-09-07 by story-scoper 0.8.0 from draft revision 10, its implemented prerequisite, the reviewed candidate binding and current integrated source — cited.

- **Owning package:** `crates/edge/ess-cli`; both release-bundle fetch and Helm reconciliation own acquisition, cache admission and external-process dispatch here — cited.
- **Contained implementation:** add the shared checked OCI manifest/blob reader, bounded acquisition, proof-entry publication and verified consumption within the owning package; exact helper and test filenames remain implementation choices — inferred.
- **Internal binding:** `docs/design/review-cache-origin.md`, currently absent; proposed home for the reviewed bundle/Helm profiles, original-byte verification, bounds, cache framing/publication and consumption contract after coordinator acceptance — inferred.
- **Public explanation:** `website/docs/concepts/component-delivery.md`; its digest-verified-cache diagram and complete-chain explanation directly describe this boundary and need the supported profiles, legacy/corrupt behavior and offline-hit limits — cited.
- **Acceptance:** both cache consumers must refuse a self-consistent replacement under another requested OCI manifest digest before returning bytes, writing requested output or invoking Helm; valid proved hits work without a fetch client, legacy entries cause cold acquisition, and corrupt proved entries refuse — cited.
- **Tests:** retain all existing persisted-delivery assertions, replace its placeholder cold-fetch fixture with an independently linked OCI graph, and add actual CLI cold/warm, substitution, profile, bound, interruption, concurrency and verified-snapshot cases inside the owning package — cited.
- **Preservation:** retain canonical bundle/model/plan bytes, checked desired/current-plan admission, dry-run/removal guards, sequential rollout behavior and injectable clients; cache content identity establishes neither signer trust nor applied-state evidence — cited.
- **Dependencies:** existing checked delivery types and strict Digest APIs are sufficient reusable dependencies; this assessment establishes no additional library, release-action, root manifest/lock, public-format or executor-recovery write reservation — inferred.
- **Current-source refresh:** coverage integration changed the CLI file but left the release command, release execution, deployment execution and cache/helper sections byte-identical to the source used by the prior binding review; existing delivery fixtures and the public concept page are also unchanged — cited.
- **Would collide with:** any write anywhere in the owning CLI package, to the proposed internal binding, or to the exact public concept page; broader or nested reservations require manual overlap checking because scope tokens are literal — cited.
- **Delivery:** publication and normal Website/Atlas delivery remain coordinator-owned; no external repository is reserved for this implementation unit — inferred.
- **Confidence:** high for source ownership and reservation sufficiency, because both consumers, their test fixtures and documentation owners are directly established by current source and the unchanged reviewed candidate contract — cited.
- **Pending selection:** the candidate binding remains unaccepted and this story remains draft; this refresh authorizes neither implementation nor a new wave — cited.

## Candidate binding review

The 2026-09-06 independent scope report is SHA256 e6cd5709142582cc406fe8583c23045c71b0fcb3b0ce411119fb0e68ad6da69a. Root verified all 42 scope input and 18 local evidence files. Candidate binding v1 is SHA256 719c29e9f36a67990133c74f33a71b372e4b6ed90c23dd41bfa640af39ef7985. Its independent document review found no binding issue; review-result:cache-origin-binding-pass1 preserves the report verbatim. Root verified all 48 review inputs, both historical story snapshots and all 18 prior local evidence files. The separately acknowledged obsolete Scope acceptance sentence is corrected here; the reviewed binding is unchanged.

The local ORAS evidence establishes an installed synthetic publisher shape, not cache correctness or actual Helm publisher bytes. No implementation or gate has run. Binding acceptance and fresh integrated source/scheduling remain pending; this story is not selected by wave 11.

## Coverage-integrated scope refresh

The independent scope report is retained at
`target/review-boundaries-12/preparation/scopers/cache-report.md`, SHA256
`73c689fd02b3b5530ece77f4dd427e6b8634a52472653dfbd3b24f163cbf23bd`. Root read back every inspected input hash before applying this section
(31 inputs). The report was captured from the original final message without
rewriting it. The machine reservations remain unchanged. This read-only refresh ran no source
tests or build and did not itself select an implementation.

## Accepted implementation binding — wave 12

Root accepts `docs/design/review-cache-origin.md` under the standing implementation approval
for the remediation epic. Original reviewed candidate SHA256 is
`719c29e9f36a67990133c74f33a71b372e4b6ed90c23dd41bfa640af39ef7985`; the recorded independent
document review has no findings, and the fresh source comparison leaves its semantics unchanged.
Only status/source references change when promoting that document. Earlier candidate/preparation
statements, including the scoper's pending-selection line, describe the state before this decision.

Select this story alone for wave 12 with the same three reservations. The implementation owns
both cache consumers and all bound refusal, process, publication and consumption controls.
No manifest, dependency, public wire format, signer trust, execution recovery or release change
is selected. The normal source attack, full gate and exact public delivery remain required.

## Scope

Confirmed from the implementor's sealed report, section 1 (lines 8–28), against unit
`6fd6e796c580656b65d9199f9367ebe3b662f806`. These are source/package observations; the
independent attack, integrated gate and public delivery are recorded separately below.
The opening hypotheses remain above so their corrections are visible.

| Opening reservation or hypothesis | Confirmed scope |
|---|---|
| CLI package owns both cache consumers — cited | `crates/edge/ess-cli/src/main.rs` dispatches both consumers into the new package-local `oci_cache.rs`. All source and permanent test changes stay inside the reserved package. |
| Shared reader and proof publication can stay package-local — inferred | Confirmed: original manifest/blob checks, finite profiles, framing, bounded acquisition and no-replacement publication are implemented in `oci_cache.rs`; `main.rs` supplies verified bytes to the bundle reader and private Helm snapshot. |
| Internal binding path initially absent — inferred | Corrected at acceptance: `docs/design/review-cache-origin.md` exists and is the unchanged root-owned work order. The implementor read it without editing it. |
| Public concept page owns the cache explanation — cited | Confirmed: `website/docs/concepts/component-delivery.md` documents the admitted profiles, original-byte identity, old/corrupt cache behavior, local limits, publication and consumption boundaries. |
| Existing dependencies suffice — inferred | Confirmed: existing Digest and checked bundle APIs suffice; no dependency, manifest, lockfile or downstream library changed. |
| Persisted-delivery assertions remain — cited | All existing invalid-plan assertions remain. The authorized positive fake fixture now checks exactly three ORAS calls and two Helm calls for a reused chart. |
| Delivery, planning, changelog and integration are coordinator-owned — inferred | Confirmed: the unit has no planning/Git/lifecycle/publication authority and made no source changes outside the two reserved owners. Its assigned external temporary root is separately inventoried. |

The same three machine reservations remain. Source review added two test files inside the
reserved CLI package: `tests/cache_origin_adversary_pass1.rs` and
`tests/support/cache_origin_attack_client.rs`. Root recorded the returned report byte-for-byte
as `review-result:cache-origin-source-pass1` and the no-op review outcome. No production or
existing test file changed during that attack. Root added the normal integration changelog
entry. No public wire migration, signer authority, general cache cleanup, rollback or execution
recovery is included.

## Source review and integrated subject

Implementation commit `6fd6e796c580656b65d9199f9367ebe3b662f806` passed 226 package cases,
strict Clippy and formatting. Its original substitution regression and all three required
mutations produced assertion failures and were restored; the implementor report SHA256 is
`e929825bf3e7686c2dadb4e0f762d9721025b8e7062dac435057f12e0b52a873`.

The first source attack added nine actual-CLI regressions and found zero defects. Final package
validation executed 235 cases with zero failed or ignored; strict Clippy and formatting both
exited zero on the same 1,132 repository inputs. The immutable report SHA256 is
`21290e42530fa5547722d4b0aeb617168bc0babe7758ab86cd84705f28178da8`. Root verified
the complete sealed inventory and retained the executed test/CLI binaries before integration.
The two new test files are bot commit `e6d9a7a3e3e440c5f77769bcaa293d5fdbc7f401`.

The integrated subject is `dbe78c5b15df478ec2cd4883c67d0012cdf90e17`. The first full gate
stopped at doctests with exit 201 after 2,077 passing workspace cases and zero assertion
failures: the mutable stable toolchain path had changed from Rust 1.98.0 at preflight to
1.98.1 rustdoc against already-built 1.98.0 libraries. The retained test log records E0514.
Root selected the already-installed versioned 1.98.0 compiler and rustdoc, verified that the
compiler bytes match the first preflight, and restarted all declared lanes without changing
source, installing tools or switching defaults. The original partial run remains evidence;
only a successful complete retry can establish the gate.


## Completed integration and delivery

All ten integration lanes passed at unchanged source
`dbe78c5b15df478ec2cd4883c67d0012cdf90e17`: fmt, strict Clippy, workspace tests,
rustdoc, examples, projection drift, release check, action check, site build and planning validation.
The successful workspace test lane executed 2,093 cases, zero failed or ignored, across 189
native summaries in 339.464207763 seconds. Its retained log SHA256 is
`f0d85c54426707b2520fafa4decec040caa37487e7b7995f911b825f09966555`.
This is the successful lane count; partial attempts and setup probes are not added to it.
`target/review-boundaries-12/gate-dbe78c5b15df-attempt3/complete.json` records all ten
zero exits; SHA256 `eab2f8aa0fd31201d54e17809166bd00e93548dd6011459f8aae82a791d02f7c`.

The versioned-toolchain retry described above also failed: that installed host lacked the
WASM standard library, producing seven synthesis failures after 1,553 passing cases. Root
then copied and verified the complete already-installed Rust 1.98.1 toolchain, including WASM,
into an owned immutable snapshot. Native, WASM and doctest setup probes passed before the third
attempt. Every lane checked the same 378 snapshot entries and unchanged repository inputs.
After six successful lanes, the runner stopped before the seventh when disk space crossed its
8 GiB floor. Verified retention and retirement of the two failed attempts' temporary roots,
plus deduplication of identical owned Go/Node copies, restored headroom. A recorded continuation
ran only the four remaining lanes against the same source, tools and environment. No source
correction or repeated successful lane was needed. All failed and interrupted outputs remain
retained with their direct statuses; this was not one uninterrupted gate invocation.

The exact integrated source was published to main. GitHub CI
[34072064491](https://github.com/beyond10x/ess/actions/runs/34072064491), documentation
validation [34072064460](https://github.com/beyond10x/ess/actions/runs/34072064460), and
source bundle [34072064499](https://github.com/beyond10x/ess/actions/runs/34072064499)
all succeeded at that exact source. Both recorded actors are the organization bot.

Atlas publication [34073139680](https://github.com/beyond10x/atlas/actions/runs/34073139680)
succeeded using exact ESS `dbe78c5b15df478ec2cd4883c67d0012cdf90e17`, AEP
`658cf76e6371b1628f6de69548e724b52803f5c2`, and Website runtime
`fc4571534765c098ed861bc326da4d3da0d1df63`. Root verified artifact 10001174083 against its
API digest and length: 32,760,306 bytes, SHA256
`6dbc2ea26b1f933be1141b39ec73517185218277fbe6f73bc2d37dd7b0f75290`.
The source-set SHA256 is `af87dac187661f4f9724d20655578c58df25449f66a152fea7f4c43eda44c2d6`.
Independent local artifact verification and the full Website gate both exited zero;
99 tests passed with zero failed, cancelled, skipped or todo. The gate took 138.725500964
seconds. The immutable artifact remained unchanged and covered 357 routes and 1,331 site files.
Both live provenance endpoints, the ESS component-delivery page and the corrected Devcenter
acceptance page returned HTTP 200 and matched the artifact byte for byte.

The earlier Atlas run 34072441317 failed before publication on two ambiguous Devcenter console
fences. Another session had already corrected them at `ba8cf660bdded7872df3512548d33eef22e61555`
after that run selected inputs. Root verified the corrected bundle and reused the newer successful
publication; it made no duplicate Devcenter fix or planning mutation. The failure remains recorded.

This completes cache-origin acceptance. The separate execution-recovery obligation remains open:
OCI artifact proof does not establish target ownership, fresh observation or durable applied history.
