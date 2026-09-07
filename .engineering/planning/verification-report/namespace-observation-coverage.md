---
format: aep.planning-md/1
id: verification-report:namespace-observation-coverage
kind: verification-report
status: draft
title: Verify scoped Kubernetes collection and retained coverage
relations:
- verifies: story:review-observation-completeness
revision: 3
---
## Verified implementation

The binding is docs/design/review-observation-completeness.md. Native namespace collection
preserves exact scope, omits value-bearing content before writing, and emits observation/2.
IR/2 includes coverage in its digest and validates namespace keys and referenced-node membership.
Graph and drift retain the qualification. Scope mismatches refuse comparison; a differing
qualified model outside detailed drift rules produces TopologyDigestChanged. Diagnosis and
simulation withhold conclusions, and projection refuses before artifacts are written.
Unsupported selectors and ambiguous environment sources are refused rather than guessed.
Legacy version 1 canonical bytes remain compatible.

## Executed checks

- task check: exit 0; 192 reported test groups, 2073 passing tests, zero failures or ignored tests.
  Log SHA-256: 4e98e2d058fb2b7d256148cb9900dc00e48d704a793f9c28ce2dd5a8f5937a12.
- task site-build: exit 0, including the Rust/WASM browser boundary and Docusaurus build.
  Log SHA-256: 57a99715ddff518c25bdd9c266008b91a1e660b977712e448791d86900b1e404.
- Redaction mutation: bypassing the legacy Secret sanitizer made
  valid_secret_observation_bytes_remain_compatible fail (exit 101); restoring the guard passed
  (exit 0). Mutation log SHA-256: 3c6ed57841618c9b546717ddb9808203b6c2dc9fd2976c5dbd230794f46cfed3;
  restored log SHA-256: a26d4e7b8dd129ffc61b0b4ceceafe6bb53536d95a8b057183f0f98903e62435.
- The released ESS 0.20.0 reader refused a new synthetic IR/2 document. The frozen old-envelope
  tests independently reject version 2 and the version 1 field extension.
- git diff --check: exit 0.

## Regression witnesses

crates/infra/ess-kubernetes/tests/secret_boundary.rs tests exact subprocess arguments, no
scope-changing fallback, payload sentinels, cross-scope responses, incomplete pagination and
preservation of existing output on refusal. crates/infra/infra-compiler/tests/coverage.rs covers
admission, digest/round-trip, all three workload key spellings, incompatible qualifications,
unknown selectors and environment sources. The coverage tests in infra-spec and infra-project
cover downstream uncertainty, topology digest changes and projection refusal. The public command
path is exercised by crates/edge/ess-cli/tests/infra_scope.rs in both import spellings.

## Limits

This verifies implementation, not a released binary or downstream adoption. No cluster
credentials, observations or operational traces are included in the repository. No dependency
promotion, release, or public documentation delivery is claimed by this record.

## Integration with current main

The merge integrates main `9d84a425e3a0c052bb08975766c5dbd600d04ef0`. Source and changelog changes merged without conflict. The only conflict was the append-only planning journal: native Git union merged additions affecting disjoint artifact sets. Both original event streams were compared byte-for-byte against the result, with no event lost, altered or newly duplicated. The combined 188-artifact store validates; existing prose-only review warnings remain unchanged.

The combined branch passed `task check` (194 test groups, 2,108 passed) and `task site-build`, both exit 0. Check-log SHA-256: `aa0d93330f7820450faee0d645c4fdeeb21366d9fc5a9ca9eca80ea483f3ca03`; site-log SHA-256: `8597e6e6b46825863ea4ce4a7f80260d5485869d8d5c2eb998fbdcb179384abd`. An earlier attempt stopped with ENOSPC while writing a native fixture log; disposable incremental build cache was freed and the complete gate rerun successfully. No source or test assertion was changed to address that environment failure.

Workflow bytes remain identical to current main. The operator requested merging PR #12 on 2026-09-07. This record does not assert a release tag, binary publication or completed documentation delivery.

## Final integration after concurrent documentation changes

Main advanced during verification to `57e242e8a0eaa721968c3970099b4bc561cb91aa`. Integrating it changes the documentation/support checker and its public source, while the collector and all runtime library/CLI source trees remain unchanged from the previously tested integration. Native Git union again preserved all original journal records exactly and admitted only disjoint artifact additions; the resulting 195-artifact AEP store validates.

The complete gate was repeated against this combined tree: `task check` exits 0 with 194 test groups and 2,125 passed tests, including the new support-matrix check. `task site-build` also exits 0. The original collector commit remains an ancestor so existing exact source pins retain their identity.
