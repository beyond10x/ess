---
format: aep.planning-md/1
id: release-plan:consolidated-ess-019
kind: release-plan
status: active
title: Consolidate ESS worktrees and release 0.19.0
revision: 3
---
## Authorization

The operator explicitly requested integration of all ESS worktrees, a new release,
and managed cleanup before further adopter-specification planning. This authorizes
source commits, publication, the release tag and exact reviewed cleanup operations.
It does not authorize unrelated repository implementation or adopter pin changes.

## Scope and Sequence

Release 0.19.0 consolidates the remediation baseline, Rust/Web target feasibility,
the source/digest catalogue, authored-site corrections, checked schema imports,
structural Go/Rust/TypeScript libraries, wire-key validation and the currently
implemented normalization reference/CLI and standalone Rust library surface.
Go/TypeScript normalization and full source-specific adoption remain active work.

Preserve four outstanding configuration, invariant and architecture planning
artifacts and the maturity outlook. Preserve existing Git ancestry and original
governed records; reconcile current AEP state through the CLI, never concatenate
or overwrite a divergent journal. Published recovery refs retain source snapshots
where their history is not the integrated store's event sequence.

Resolve the independently measured Web json-dependency and HTTP Out-binding
compiler failures before release. Preserve distinct-name positive controls and
the source-level wire-key admission decision in the integrated design.

Refresh the format catalogue from its latest reviewed commit. Run the complete
offline gate and site build on the exact release source. Commit with organization
bot tooling, land on main before creating the annotated bare-version tag, then
verify the release workflow and assets. Do not claim a source tag is a completed
binary release while packaging is pending or failed.

## Cleanup

Publish wanted commits before worktree finish. Review a fresh gc dry run and pass
only exact reviewed ESS ids to gc apply. Preserve any tree with unaccounted content,
live ownership or failed recovery proof; no force removal or manual deletion.
The temporary clean Atlas authority checkout is not an unrelated source change.

## Completion

Main contains the consolidated source and reconciled plan; 0.19.0 has a verified
release; every ESS worktree is either safely retired or listed with its exact
remaining lifecycle/refusal reason. Adopter specifications are unchanged.

## Candidate Verification

The 0.19.0 integration candidate includes frozen Rust/Web correction
a2ff04d70f70a691288b661e4d027eee5da74e42. Its three production files and both
original adversary test files match that commit exactly. Combined tests retain
both regression sets and the source-wire admission invariant.

Observed checks on this candidate:

- CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check: exit 0.
- CARGO_BUILD_JOBS=4 task site-build: exit 0, including the WASM browser lab.
- Pinned optional schema-contract all-features lane: exit 0 with Go 1.26.5 and
  TypeScript 6.0.3. Its first invocation omitted required compiler-path environment
  variables and failed at setup; the corrected explicit invocation passed.
- Combined targeted feasibility/adversary tests: 39 + 5 + 2 passed.
- Strict package Clippy, formatting and git diff --check: exit 0.
- Site dependency audit: 9 moderate and 21 high existing findings, not resolved
  or represented as a clean security audit by this release.

The planning store has 112 artifacts after CLI reconciliation of fourteen absent
source/older artifacts, three immutable Wave 5 reviews, fourteen historical test
observations and five Wave 5 review outcomes. Immutable review bodies were compared
byte-for-byte after creation. Original event sequences remain in the published
checkpoint histories, while canonical state is represented by new CLI operations.
Validation exited 0 with twelve historical prose-only review warnings.

The independent Wave 5 coordinator is still conducting the final Rust adversarial
review. Its latest observed committed checkpoint is
11bbaeff7398c7a73e785c199e335a5bdd6beac8. Final review, source publication and the
last canonical planning-state reconciliation remain pending. No release tag or
GitHub Release has been created and no worktree has been removed.
