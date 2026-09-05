---
format: aep.planning-md/1
id: release-plan:consolidated-ess-019
kind: release-plan
status: active
title: Consolidate ESS worktrees and release 0.19.0
revision: 8
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

## Final Correction Verification

The final full review is now preserved as
review-result:review-boundaries-5-rust-adversary-pass-2, imported through AEP's
body-only interface from coordinator checkpoint
3770ef908f5e2123ccbc3516dd1c19158a81ceec. The imported body was compared byte-for-byte.
Both findings retain their measured pre-existing origins.

The integration candidate adopted the original implementor's exact working-tree
correction to delivery initializer scopes and borrowed map-key diagnostic paths,
together with both complete adversary test files. All four files matched the
implementor's source byte-for-byte after transfer. The binding design retains the
source-wire admission note and incorporates the final correction's lexical and
collection contracts. Release notes and the internal catalogue now account for
the repaired HTTP/Web map decoders. No independent alternative fix was introduced.

Observed on this combined source:

- 39 feasibility, 14 synthesis adversary and 4 CLI adversary tests passed.
- CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 task check exited 0.
- CARGO_BUILD_JOBS=4 task site-build exited 0, including the WASM browser lab.
- The existing site dependency audit still reports 9 moderate and 21 high findings.

Two fixed review outcomes are recorded against the immutable final review, with
the affected production paths as references. These observations establish the
combined correction result, not completion of the independent coordinator's
handoff. Its correction tree still had HEAD a2ff04d and uncommitted final changes
at the last check. Before publication, compare these files to its frozen final
commit, reconcile the final planning state and preserve that branch ancestry.
The agreed Wave 5-first publication order remains in effect. No version tag,
GitHub Release or worktree cleanup has occurred.

## Frozen Source Handoff

The original final correction is frozen as
028cd015877ce5c2255c840eb39a2764b159981e, with both bot identities verified.
All synthesis production files and both complete adversary test files match the
integration candidate exactly. The only synthesis test difference is the agreed
source-wire admission replacement plus three supplemental scope regressions.
The already successful combined gate and site build therefore cover these exact
production bytes. The native release build also exited 0 and reports ess 0.19.0.

Wave 5 locally merged the correction as
87338bdef898bf4e4a9a3a0e971c4ec822e3dacd. Its committed Rust story body and complete
wave narrative have now been reconciled into this candidate. Its final bounded
verification, exact SDK producer check and main publication remain owned by that
coordinator; final publication state must be synchronized before release.

An authoritative live session was identified for that coordinator. The existing
codex queue command accepted the release handoff and a reply address, followed by
confirmation of the exact frozen-source comparison. No daemon, restart, session
interruption or takeover was used. Earlier no-channel statements describe the
previous discovery state, not the current communication route.

The clean Atlas authority checkout was fast-forwarded to published
dfe353cba37ce072d118d0a67634fb78e59a5276; its instructions and bot tooling are
unchanged. A fresh documentation contract check with this ESS candidate selected
still refuses the previously recorded AgentIDE b10x-docs/v4 collection mismatch.
This is not an ESS site-build failure or an organization-wide green result, and
no unrelated source or documentation authority was changed to bypass it.

## Main Integration and Tag Verification

The consolidated source is published on main at
5201daf4ea6fe1e254d73a3e858abd06a8a71715, with published Wave 5 commit
87338bdef898bf4e4a9a3a0e971c4ec822e3dacd retained as an ancestor. The canonical
checkout is clean. Four original source/planning checkpoints remain reachable.

Exact-main GitHub observations:

- CI 33995345139: success.
- Documentation validation 33995345146: success.
- Documentation source bundle 33995345171: success.

The previous main CI failure was the missing wasm32-unknown-unknown standard
library, not a failing source assertion. The one-line toolchain prerequisite fix
is tracked by story:provision-wasm-for-gate, now implemented after the actual
corrected CI passed. Cache cleanup emitted ENOENT annotations in the successful
run; this is not represented as a warning-free run.

The annotated bare tag 0.19.0 resolves to 5201daf and was published with the
organization bot identity. GitHub draft release 383390739 was created by
b10x-bot[bot]. Release workflow 33995624881 has passed its full gate and public
documentation build; its four native package jobs are running. This paragraph
records preparation, not a completed public binary release.

Older renderer commits 4ca7b63e and a155edc2 are patch-equivalent to integrated
main changes. Their original bot-authored commits are now retained remotely as
archive/html-site-renderer and archive/document-ir-renderer, respectively,
because cleanup correctly refused them without advertised recovery proof.

The independent coordinator and three related verification trees remain reserved
pending an ownership handoff. No adopter specification has been changed in this
release phase. The full source-specific mapping and Go/TypeScript normalization
targets are still unfinished and are not claimed as delivered by 0.19.0.

## Historical Release Ancestry Audit

The first post-tag release-status check exited 201 because historical tag 0.17.0
at 1c85689d77acdae2ffb2fc68579104b98d65c424 was not an ancestor of remote main.
Only that one commit is absent from main's ancestry. The tag and its public
GitHub release predate this consolidation; neither is moved or rewritten.

The feature was revised and integrated at 43c0b79, the 0.18.0 release baseline.
`git range-diff 1c85689^..1c85689 43c0b79^..43c0b79` accounts for the changes:
version/feed/changelog updates, removal of device-specific skin CSS and stronger
neutrality assertions. The intervening early-stop assertion explains the
ess-conformance/4 CLI help, additional ordered-scan exports and older-base
planning journal difference. The epic, binding design, player JavaScript and
vendored Vue asset/license are byte-identical between the two feature commits.
The revised feature and subsequent repairs are already in the released source.

Preserve 1c85689 as an additional ancestor using an explicitly tree-preserving
merge after the final evidence commit. This is not permission to restore its
old workspace version, retired CSS, old public feed identity or divergent
journal snapshot. Verify tree equality and remote-main reachability afterwards.

The same check incorrectly called the then-draft 0.19.0 release published.
That separate defect is tracked by story:release-status-publication-state.

## Published Release and Cleanup Result

Release 0.19.0 is public at https://github.com/beyond10x/ess/releases/tag/0.19.0,
published 2026-09-05T22:36:33Z by b10x-bot[bot]. GitHub readback confirms
isDraft=false and all four native archives plus SHA256SUMS. Release workflow
33995624881 passed all jobs. All four downloaded archive checksums match; the
downloaded x86_64 Linux binary reports ess 0.19.0. The annotated tag remains on
5201daf4ea6fe1e254d73a3e858abd06a8a71715 and is not moved by later evidence edits.

The final evidence checkout passed task check and task site-build again. The
documentation handoff ba43fda contributes the exact final Wave 5 narrative and
320-character feed summary. Its Rust story body/status and new feed story,
scope and observed validator evidence were reconciled through AEP, not by
concatenating its journal. The feed story remains active because Website
delivery is separately owned and not proven by an ESS site build.

Managed GC removed these ten exact reviewed ESS ids after refreshing remote
recovery proof:

- wt-37a7d77f8f76
- wt-ece544778ed1
- wt-b04819074c6e
- wt-b2a2189b9ecc
- wt-ce6e143e1fe5
- wt-34496b6990aa
- wt-e46db550dce9
- wt-340546c4af89
- wt-780a9b306a76
- wt-46ef382d9f07

Five trees initially refused cleanup because of ignored Cargo/Docusaurus output.
Only exact dry-run-reviewed generated paths documented by .gitignore and Taskfile
were cleared before retrying finish. No source or active coordinator evidence
was removed. Original source and planning checkpoints remain published.

Three old trees remain explicitly retained: wt-3726210974f1,
wt-1f0716ada0f4 and wt-8850a8418f1f. Worktree 0.3.4 repeatedly reports
worktree-locked at finish, while its own repository listing reports locked=false;
Git's locked files are absent and historical operation metadata remains. No
metadata was deleted and no refusal was bypassed. All three heads are already
ancestors of main, so this is a cleanup limitation, not missing source integration.

The independent coordinator wt-752828a285ba remains reserved for public delivery.
Its three implementation/compatibility trees were retired by that owner during
this operation, not selected by this session. The release checkout wt-6525314b6daa
is retained only until its final evidence commits are published and checked; its
own finish/GC is the last local operation. The temporary clean Atlas authority is
likewise reserved until bot publication is complete. ACD and IVR specs remain
unchanged throughout this release phase.

The historical 0.17.0 ancestry reconciliation and final handoff are audited above;
their original commits will be retained without replacing the integrated tree.
Re-run task release-status against the resulting remote main. The initial exit201
and draft-reporting defect remain recorded rather than replaced by a clean-history
claim. Full IVR mapping, Go/TypeScript normalization and the separate Website
delivery are not claimed complete by this release plan.
