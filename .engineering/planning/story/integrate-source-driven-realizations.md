---
format: aep.planning-md/1
id: story:integrate-source-driven-realizations
kind: story
status: draft
title: Integrate source-driven realizations with the remediation baseline
relations:
- depends_on: story:review-format-catalog
- depends_on: story:review-rust-target-feasibility
- depends_on: story:types-only-realizations
- depends_on: story:source-pinned-data-normalization
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/feasibility.rs
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: docs/design/review-rust-target-feasibility.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 3
---
## Evidence

Read-only worktree audit on 2026-09-05 found the primary checkout clean at
`6616b26`, equal to locally cached `origin/main` (no fetch was performed). The
source-driven specification worktree is detached at `6cbe372`, 19 commits behind
main, with 78 changed/untracked paths at the audit snapshot.

Main and that working tree overlap on only `crates/edge/ess-cli/src/main.rs` and
`.engineering/planning/journal.jsonl`. Read-only three-way previews of `main.rs`
against both main and the pending Rust-feasibility source `f9a7cf7` exit 0. This is
textual compatibility, not evidence that the integrated source compiles.

The pending wave 5 coordinator is `wave/review-boundaries-5` at `90e1367`, with
two independent units: `impl/review-rust-target-feasibility` at `f9a7cf7` and
`impl/review-format-catalog` at `d8dedad`. The detached Rust-producer compatibility
tree is the exact `f9a7cf7` snapshot, not a second implementation. A live harness
process is associated with the coordinator worktree; individual authors are not
identified by the generic registry owner `agent`.

No competing schema-bundle importer, types-only realization, normalization engine
or wire-field validator was found in the other trees. Old renderer/release branches
that appear ahead are patch-equivalent to main; do not integrate them again. Three
older dirty trees retain separate configuration, architecture and invariant planning
work and are not cleanup candidates.

## Integration Gap

Wave 5's `website/docs/reference/formats.md` and
`docs/design/review-format-catalog.md` do not contain the formats introduced by
the source-driven specification work: `ess-schema-bundle/1`,
`ess-schema-bundle/2`, `ess-types-report/3`, `ess-normalization/1`, and
`ess-normalization-target/1`. This semantic omission must be reconciled at landing,
not dismissed because source merges cleanly.

The catalogue must distinguish raw-source SHA-256, complete canonical bundle SHA,
model contract/projection identities, canonical recipe SHA, and emitted schema/file
SHA. The normalization target report excludes itself from its artifact digest map.
Wave 5's `ess-target-failure/1` describes full synthesis refusal, not the same
contract as the structural-type or normalization-target reports.

## Outcome

Source-driven specification extensions integrate with the current remediation
baseline without lost source changes, AEP history, format identity or consumer
guarantees. This is a coordinated landing, not another implementation of either
workstream.

## Acceptance

The integrated branch contains both workstreams with reconciled format/digest documentation and planning history, and passes the full ESS gate, documentation build and affected consumer checks on that exact combined source before publication.

## Orchestration

1. Keep wave 5's existing coordinator responsible for synthesis feasibility and its
   baseline format catalogue. This session continues to own the schema-contract,
   structural realization and normalization additions. These are proposed edit
   reservations, not a claimed acknowledgement from another session.
2. Land wave 5 through its existing process first, then base the source-driven
   integration on the resulting main. Recheck ancestry before acting; the recorded
   hashes are an audit snapshot, not a permanent merge target.
3. With explicit commit authorization, checkpoint the accumulated source-driven work
   as focused commits: authored-site fixes; schema imports; structural type targets
   and model wiring; wire-name validation; normalization execution/CLI; normalization
   targets. Preserve their actual dependencies and do not label unfinished targets
   as implemented.
4. One integration owner handles CLI wiring, changelog and format-catalogue changes.
   Reconcile AEP artifacts through the CLI. Do not use a line-based last-writer-wins
   journal merge or rewrite another session's artifact state. Retain the original
   managed tree and its evidence while reconciliation is incomplete.
5. Run `CARGO_NET_OFFLINE=true task check`, `task site-build`, and the actual
   generated-library/consumer checks after integration. A green old-base gate or a
   clean textual preview does not discharge this step.
6. Publish and retire worktrees only under explicit authorization and managed
   worktree recovery rules. Preserve unrelated dirty planning/documentation trees.

## Scope

Cited shared surfaces: `crates/edge/ess-cli/src/main.rs`, `CHANGELOG.md`,
`website/docs/reference/formats.md`, `docs/design/review-format-catalog.md`, and
AEP planning records via `aep artifact`. The source implementations remain owned
by their existing stories; this story does not duplicate their acceptance.

## Coordination Boundary

No repository was fetched, rebased, merged, committed, pushed or cleaned by the
audit. The independent audit agent returned findings read-only. This session has
no established messaging channel to the other root coordinator, so a cross-session
handoff has not been acknowledged. The user must explicitly authorize commits
before the accumulated changes are checkpointed. Planning remains draft.

## Source Preview

An uncommitted managed source preview, `wt-6525314b6daa`, is based on frozen
Rust-feasibility commit `f9a7cf7fcca79448a34b2754adb12f1a411573bd`. It overlays
four non-planning catalogue files from `d8dedad` and 68 source-driven implementation,
test and documentation files from the original worktree. All 72 files apply without
textual conflicts using exact Git blob three-way comparison. No `.engineering/`
or coordinator `docs/plan/` content was mechanically transplanted. Existing branches,
primary checkout and original feature worktree remain intact and uncommitted.

The combined gate found a semantic conflict despite clean source merging:
`ess-synth/tests/feasibility.rs::wire_aliases_are_checked_only_at_an_emitted_codec_surface`
expects duplicate effective JSON keys to remain compiler-admitted for pure Rust,
while `story:unique-wire-field-identity` rejects them before schema projections can
overwrite declared fields. Initial combined `task check` exited 201 (nested test
exit 101): 30 of 31 feasibility tests passed, with this case failing during assembly.

Preview reconciliation retains source-level rejection, replaces that obsolete
assumption with a source-admission regression, and records the explicit integration
decision in `docs/design/review-rust-target-feasibility.md`. Target-only codec symbol
checks, representation checks and distinct-wire positive controls remain. The focused
`cargo test -p ess-synth --test feasibility --locked` then passed all 31 tests.
This reconciliation is a proposal for joint adoption, not an acknowledged change to
the independent feasibility branch's binding decision.

Both catalogue pages now include schema bundle /1 and /2, types report /3,
normalization recipe /1, normalization target /1, and the feasibility unit's target
failure /1. They distinguish exact source bytes, complete canonical bundle bytes,
model semantic/projection identities, recipe bytes and emitted-file bytes; target
report self-exclusion is explicit. No format is represented as already released.

The coordinator was rechecked at `ae1e70f`, with new uncommitted review results and
story/journal updates. Main remains `6616b26`. Do not overwrite that active review
work, reapply patch-equivalent old branches or assume the frozen preview is the
eventual landing commit. Cross-root handoff is still unacknowledged. No commits,
pushes, branch merges, rebases or worktree retirement are authorized or performed.

Full combined gate and documentation build results will be recorded separately.
Planning-history reconciliation and actual adopter regeneration remain outstanding.

## Preview Verification

After the source-wire admission reconciliation, the combined managed preview
`wt-6525314b6daa` passed `CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check`
(exit 0) and `CARGO_BUILD_JOBS=4 task site-build` (exit 0). The latter includes
the WASM/browser lab and Docusaurus production build. Its dependency audit reports
30 existing findings: 9 moderate and 21 high. `git diff --check` also exited 0.

An actual adopter check with the preview binary validated and compiled its model,
verified all contract snapshot checksums and regenerated 73 files. Its strict drift
check exited 1: 51 files differ from the old-base generated tree. Full byte comparison
accounted for every difference: 45 contain only the new `slice-sha256/2:` provenance
prefix; the remaining 6 propagate the resulting model-schema projection digest into
three target reports and three source headers. Both old and new projection hashes
were recomputed from actual emitted bytes; removing only those accounted identity
changes leaves byte-identical artifacts. No shape or generated behavior changed.

Adopter snapshots were not overwritten with an unreleased preview. Regeneration and
an exact released producer pin remain part of coordinated adoption. A green combined
source gate is not a clean adopter drift result, a merged planning journal, acceptance
of the cross-workstream design reconciliation, a release or completion of the
three-language normalization objective. All existing working branches remain intact.


## Integration Provenance

Reconciled through AEP from wt-46ef382d9f07 at original revision 5 and status draft. Source artifact SHA-256: 0e93bc1bb333770c896874cb07968316c1d3f95096d81c6a17ac7be38e141718. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.

## Authorized Consolidation

The operator subsequently authorized consolidation, publication, release 0.19.0 and managed cleanup before further adopter work. The release owner is release-plan:consolidated-ess-019. Earlier statements in this artifact that commits or release were unauthorized describe the earlier audit only.

The independent Wave 5 coordinator recorded acknowledgement in docs/plan/2026-09-05-review-boundaries-5.md: Wave 5 lands first, followed by source-driven integration, with source-level duplicate-wire admission retained. Correction a2ff04d70f70a691288b661e4d027eee5da74e42 is frozen for its final independent review. No competing source writer or release tag is authorized by this integration owner.

Published recovery checkpoints retain the complete source and original governed histories: integrate/source-driven-checkpoint at ddbe42a; integrate/configuration-checkpoint at 06653a6; integrate/enum-invariant-checkpoint at 662cb1f; integrate/outlook-checkpoint at a3f5ea9. All four source trees were clean after bot-authored publication.

Fourteen absent artifacts were reconciled through AEP into the integration preview, restoring source lifecycle states where supported by observed evidence. Original revision identities and journal event sequences remain in their published snapshots; the integrated store records its own CLI operations, not concatenated journals. Current Wave 5 review records and final story state still require reconciliation after its handoff.

The versioned 0.19.0 preview passed the complete offline task check, including release version/changelog validation. A fresh full gate and site build will run after final Wave 5 source reconciliation. No release tag has been created, no release claimed, and no worktree removed at this checkpoint. Structural Go/Rust/TypeScript generation is implemented; source normalization has a Rust library target but Go/TypeScript normalization and its target-generation CLI remain pending.
