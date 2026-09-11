---
format: aep.planning-md/1
id: story:output-ownership-tolerates-platform-xattrs
kind: story
status: implemented
title: Publication must not refuse platform-imposed extended attributes (SELinux, overlayfs)
summary: flistxattr non-empty refuses every ess generate on CI runners that label files; distinguish platform attributes from foreign ones.
tags:
- adopter-feedback
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: .engineering/planning/story/output-ownership-tolerates-platform-xattrs.md
revision: 5
---
## Context

`ess generate` publishes through the output-ownership layer, which refuses a destination that carries
any extended attribute: `crates/edge/ess-cli/src/output_ownership/filesystem.rs:428-445` —
`ordinary_metadata` calls `flistxattr` and bails with `output has extended metadata outside the
ordinary snapshot contract` when the name list is non-empty.

On CI runners every file under the project directory carries an attribute the generator did not
write — `security.selinux` on SELinux hosts, `trusted.overlay.*` on overlayfs upper layers — so a
publication into the checkout can never pass there. Observed 2026-09-10, consumer-application pipeline
408652 job 748546 (GitLab Docker executor, image node:22-bookworm-slim, project dir `/builds/…`),
ess 0.22.0 at 0de935d92c49df1147f8196e2424f49634ec2925:

```
$ scripts/ess-pinned.sh generate --path consumer-specification --kind site --out public --include …
error: output has extended metadata outside the ordinary snapshot contract
ERROR: Job failed: command terminated with exit code 1
```

The same command succeeds on a developer machine (no xattrs on the tree), with umask 000, and under
a setgid parent — the refusal is specific to labelled filesystems. The adopter now generates on tmpfs
and copies the tree into `public/`, which is a workaround, not a contract.

## Acceptance

Reconcile the adopter request with the landed exact-name contract in 391b2651c0f45bbf39db08b47c84f220579b3d0f, already on remote main 6b666e58. Linux security.selinux and security.SMACK64 access labels do not cause foreign-metadata refusal. Other attributes, including capabilities, ACLs, execution labels and overlay control metadata, remain refused by name. The original request for entire security.*, system.* and trusted.overlay.* namespaces is superseded by this precise contract; the original draft snapshot remains retained in Wave source provenance.

Verify the existing targeted xattr tests and commit ancestry; distinguish executed tests from source inspection and report any absence of a real labelled-filesystem witness. Do not run the full/ownership gate, broaden namespace acceptance, duplicate landed implementation or claim the original blanket policy was implemented.

## Notes

- Adopter record: consumer-application `.gitlab-ci.yml` `pages` job comment and the `getfattr -d -m - .`
  line that logs the runner's attributes on every run.
- Related: the docs projection differs by generator version (0.18.0 vs 0.21.0), which is why that
  adopter pins a commit and builds it in CI — a release archive per main commit would remove the
  build step.

## Wave source provenance

Imported through the AEP CLI from the operator-selected primary-checkout draft, revision 1. Original bytes retained as local-evidence:ess-evolution-20260910/priority-source-output-ownership-tolerates-platform-xattrs.md (SHA256 3503c5c73942edbe04851f8c0450ecb5c9b83fff42baa672c9ce4da4468616f8). This branch starts a truthful import record; it does not invent the source draft's earlier command history. The primary draft and journal remain untouched.

## Scope

Derived 2026-09-11 by aep-drive:story-scoper. Tracking reconciliation is the only confirmed required change — cited. The existing implementation is crates/edge/ess-cli/src/output_ownership/filesystem.rs: ordinary_metadata, platform_xattr, foreign_xattrs and three xattr_tests — cited. CHANGELOG.md already records the fix under 0.22.2 — cited. Confidence high: 391b2651 is on main 6b666e58 and its implementation has not changed since — cited. Any missing positive labelled-admission evidence is to be reported, not invented — inferred. Would collide with planning edits to this story or output-ownership filesystem code — cited.

## Public import correction

The first unpublished import was refused by the coordinated private-identifier check. Its exact rejected patch is retained privately in the local wave evidence. This replacement was created through AEP from the same source snapshot with the private organization identifier generalized before any journal event was written. Source acceptance and source snapshot hashes are preserved; no scanner policy or exception changed.

## Verification and reconciliation evidence

The verifier ran cargo test --offline --locked -p ess-cli --bin ess output_ownership::filesystem::xattr_tests -- --nocapture once at 59afcf8caec5230a88aadfd7c590a703e0b5500b. Exactly three tests passed, none failed or ignored, 13 filtered out; exit 0. Build 42.36 seconds, tests 0.00 seconds. Full output and report: local-evidence:ess-evolution-20260910/priority-wave/xattrs/{tests.log,report.md}.

Git ancestry confirms both 391b2651c0f45bbf39db08b47c84f220579b3d0f and remote main 6b666e58f2e87dd8798d27f935e9a012203296a3 are ancestors. The filesystem implementation is byte-identical to the landed fix. No source or test file changed; the assigned worktree is clean and the verifier ended its lease. Only tracking reconciliation was necessary.

Evidence is bounded: accepted SELinux/SMACK names are tested through the policy predicate; no actual platform-labelled destination publication was exercised. The existing real user-attribute test may return early on a filesystem reporting NOTSUP; the harness's passing count alone cannot exclude that branch. No real labelled-filesystem witness is claimed. The original broad namespace acceptance remains intentionally superseded by the landed exact-label contract. No full or ownership suite ran.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 01982c139838c1e91cc4871b171a89ee72b72c66242435dd3cbc272fefc66728, retained as local-evidence:runtime-gaps/publication-replay/snapshots/01982c139838c1e91cc4871b171a89ee72b72c66242435dd3cbc272fefc66728.md. Source creation recorded at 2026-09-10T23:37:05Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
