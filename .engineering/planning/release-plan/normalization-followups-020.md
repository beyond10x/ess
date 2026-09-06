---
format: aep.planning-md/1
id: release-plan:normalization-followups-020
kind: release-plan
status: active
title: Release ESS normalization followups as 0.20.0 and qualify IVR adoption
relations:
- depends_on: story:typescript-normalization-target
- serves: vision:O2
revision: 10
---
## Purpose and authority

Prepare a concrete 0.20.0 ESS release containing the completed normalization followups, then adopt its verified binaries in the relocated `specs/services/ivr` consumer. The continuing operator instructions authorize owned integration, a version containing the ESS changes, publication, managed cleanup and a final personal status message. This record is agent-authored release preparation, not a new human approval, a claim that a tag exists, or a waiver of the exact-source gates.

The current public release is 0.19.0 at `5201daf4ea6fe1e254d73a3e858abd06a8a71715`. Main checkpoint `b55efa4cd6c379217f60b98fb30f28f70e526ee6` has successful local workspace/site gates and remote CI; its binary still prints 0.19.0 and is explicitly unreleased. TypeScript normalization is active in its isolated unit and must pass implementation, independent review and integration gates before release preparation changes the version.

## Release scope

Include source-pinned normalization formats 2–6, Rust/Go execution and the reviewed native TypeScript target; model-owned roots, retained JSON capture and strict base64 composition; finite Binary64 model/normalization semantics and supported structural Rust/Go codecs; checked positional arrays; the conformance/admission corrections already integrated since 0.19.0. Preserve every other published main change. The final changelog is rewritten from the actual integrated diff and review outcomes, and explicitly names the fallible pre-1.0 API changes and admitted target limits.

No released source-specific decoder parity is inferred from generated types or a green ESS suite. IVR candidates still require concrete regeneration and reference/native evidence after the released binary is downloaded and verified. ACD currently remains on its own 0.18.0 specification pin; changing that pin requires an actual adoption delta and validation, not a namespace change.

## Producer-version compatibility qualification

The read-only release witness audit identifies exactly four proposed test paths. Its proposal patch SHA-256 is `0d33730c6afefb768af6997e6563e07e0912c5c038b1d4445456ae8b0c51b9b7`; the manifest is `3356fae023e4f1cd7ffb4ed84df46de1d9ba2786e24fccb8b1c142e6ac1f8912`. That original audit was an unapplied, uncompiled proposal; the later pre-bump qualification below records its actual application and checks. Before changing producer versions, retain actual raw 0.19 output for the four structural maps and all normalization maps. Structural raw output must reproduce the established `02312f45fadac5e16b54aa1fce13d8f68cd8aaf14336a2180b85cd923811c79f` witness. The TypeScript unit independently freezes twelve complete Rust/Go format-1–6 maps before shared implementation edits.

Qualify a test-only projection that first asserts the actual current producer version, then changes only the exact report producer line and structural generator header to the historical version for comparison. Preserve all remaining raw bytes, paths, support files and map membership. Replace only the ten older normalization report hashes that had used a synthetic placeholder with their independently reproduced raw 0.19 report hashes; keep the other 162 entries exact. Add mutation controls for stale producer identity, body changes and membership drift. Never globally replace versions, discard report formatting, or bless a different structural aggregate digest.

After the actual version bump and actual generator rebuild, retain separate raw 0.20 output. Differences must consist only of the declared producer-version slots for these compatibility witnesses. Qualify the TypeScript unit's twelve-map witness with the same strict current-version rule in a separately reviewed delta. Synthetic input labeled 0.20 does not substitute for actual post-bump generation.

## Concrete procedure

1. Integrate the independently reviewed TypeScript unit and common guidance; finish its workspace, explicit native and site checks and publish the source checkpoint.
2. Capture and qualify all pre-bump compatibility witnesses. Apply only reviewed test changes through managed source work.
3. Change root workspace version and its eighteen internal path constraints to 0.20.0. Let Cargo update workspace package versions in Cargo.lock without dependency upgrades. Supply a dated, complete 0.20.0 changelog. Use authoritative generation commands only for real projection drift; preserve semantic provenance unaffected by build version.
4. Run release verification, the qualified post-bump native/witness lanes, the full `task check` and `task site-build` on the exact commit to be tagged. Record each actual exit and any setup/error correction. Do not tag an untested metadata followup.
5. Publish the bot-authored release commit to main, confirm remote ancestry, create the annotated bare tag 0.20.0 on that main commit, and push it. Observe the release workflow through completion. On completion or failure run `task release-status`; preserve failures rather than calling a tag a binary release.
6. Verify release assets/checksums and an actual downloaded binary. Regenerate and qualify the relocated IVR adopter with that binary, including model roots and recipes, then publish its validated source and successful GitLab pipeline. Record the precise ACD outcome separately.
7. Refresh the bounded ESS release observation in Atlas, regenerate projections, run its required fences, preserve external workspace findings and publish owned records. Verify the exact ESS source bundle reaches the normal Atlas documentation publication and live provenance.
8. Publish remaining owned evidence; preserve wanted scratch and use managed finish, reviewed GC and exact IDs for remaining task worktrees. Send Timo the authorized Slack DM with concrete canonical GitLab links, release/pipeline evidence and any remaining mapping boundaries.

## Current state

ESS 0.20.0 is published from exact tagged source c90ca1b2a3a5db02d7580dab63be6cbc56679e0b. Before publication, literal `task check` passed 2,003 tests with no failures or ignored cases across 176 result groups (exit 0, 144.946 seconds); `task site-build` passed (exit 0, 16.217 seconds). Annotated tag 0.20.0 and main were pushed. [Release workflow 34043573262](https://github.com/beyond10x/ess/actions/runs/34043573262) succeeded. [Release 383642123](https://github.com/beyond10x/ess/releases/tag/0.20.0) is published by the existing workflow as github-actions[bot], with draft and prerelease both false. These observations supersede the earlier pending checkpoints retained below.

All four platform archives and SHA256SUMS were downloaded. Their sizes and hashes match both API digests and the checksum file. The Linux x86_64 archive SHA256 is 569a34b0c2263e663140579b8511c321514fd6428634d6e62ddf2e2b15c66232; its executable SHA256 is eff3b1c8fe8834b2700681267906c768444a6e38e703ebe5349dd7e472da4980. The downloaded executable returns `ess 0.20.0`, exit 0. Literal `task release-status` exits 0. The actual post-bump native and producer-witness qualification is recorded below.

Atlas commit 78d96cc706f735c3a160369e3ce2f2133f4b1d41 publishes the bounded local-tag observation, validated catalog, generated version projections and separate remote-release evidence. Its full unscoped `scripts/fences.sh` completed in 199.141 seconds with exit 1: all 149 Rust tests, catalog, live Pages, projections, Markdown and brand checks passed; the existing AgentIDE collector-v4, primary Website Docs System pin and Widgets Serves findings remain. This does not establish organization-wide convergence. The final evidence-log append separately passed the 181-file Markdown fence.

Normal documentation publication 34043900364 succeeded. [Live provenance](https://beyond10x.github.io/PROVENANCE.json) observed at 2026-09-06T16:11:04Z selects newer ESS main 239996d846460aee342ce42514378c25b2be5152, a release descendant with workspace version 0.20.0. Producer 34044137573 and artifact 9992578091 carry 40 files that match exact Git and artifact bytes. Two public documentation files differ from c90 because an independent schema-resource review wave subsequently landed. Both 40-file bundles were verified. The coordinator fast-forwarded to that main commit without rewriting either the planning journal or the release tag.

The selected consumer has the verified released binary pin, CLI-owned common bundle import, five repinned recipes and all 15 generated normalization packages. Native qualification, independent review and publication remain in progress. Broader decoder, default, Flow and family boundaries remain outside these canonical subsets; no gapfree service-conformance claim is made. Final consumer evidence, owned-tree cleanup and operator notification remain required.

## Module identity during adoption

The published namespace relocation at IVR cf1a9657298760af53b50fe9f5abd0eea3a1c222, docs/namespace-relocation.md:12–15, explicitly retains generated Go module declarations under gitlab.stack.babelforce.com/specs/ivr/... for compatibility. These are import identities, while origin fetch/push and navigation use specs/services/ivr. The ESS release adoption preserves those module identities. A later module cutover must coordinate generator and consumer pins together; an earlier private preparation note suggesting a generator-only namespace replacement is superseded by this source-backed rule.

## Orthogonal pre-bump qualification

After TypeScript unit commands became terminal, the coordinator received the exclusive compiler lane to prequalify the four reviewed historical witness test paths on unchanged 0.19 source. This test-only preparation can run while the implementor freezes its handoff; it does not bump the version, include mutable TypeScript source, or replace the later integrated 12-map/new-release qualification. Capture raw structural and normalization files before comparison, preserve pre-existing scratch, and keep this qualification distinct from final release gates.

## Actual pre-bump witness result

At coordinator ba3c962c5862437be0822086b35cd3b7e321ec86, the reviewed four test paths were applied and scoped formatting passed. Five focused runner tests passed (two existing complete-map witnesses plus three producer/body/membership controls), with zero failures or ignored cases, followed by strict Clippy exit 0. The qualified source patch after formatting is SHA256 a4aa5a052dc63848cde9b23d850a17c79181b37b50bfc51906d638527a49e2c4. No production emitter or Cargo version changed.

The actual 0.19 generator's raw four-map structural payload hashes to the original 02312f45fadac5e16b54aa1fce13d8f68cd8aaf14336a2180b85cd923811c79f. Every report and declaration header carries 0.19.0 before projection. All ten raw normalization maps were retained and independently checked against the qualified 172-entry witness: exactly ten report expectations move from their previously reproduced placeholder representation to raw historical report hashes; all 162 other expectations remain unchanged. The prior scratch capture was copied before the test wrote its new canonical-map record.

Exact raw old/current files, commands, exit times and source hashes are preserved under release-020-preparation/qualification-019, including qualification-result.json and the complete artifact manifest. This is pre-bump witness qualification only. Actual 0.20 generator output and the separately qualified twelve-map TypeScript witness are still pending; synthetic producer controls are not evidence of those future executions.

## TypeScript twelve-map pre-bump qualification

After the frozen source and independent tests were integrated at 0f2dcd71e28f852de54af3ba01c62b3b22e6cc9b, the reviewed one-file producer projection patch 88d607dd037da8dd3459a24cf7db7f1036789801da3a33aaf44cab6efd018be8 was applied. The actual complete-map test and strict scoped Clippy exited zero on unchanged 0.19.0 source. All 218 raw files across 12 Rust/Go maps match the original literal fixture, including all 12 reports carrying actual producer 0.19.0; the fixture bytes are unchanged. Raw output and exact command exits are retained in release-020-preparation/qualification-typescript019. This only qualifies the test projection before changing the actual generator version; 0.20 output and final gates remain pending.

## Version-edit scope and command qualification

The release delta is Cargo.toml, Cargo.lock, CHANGELOG.md, docs/design/typescript-normalization.md, website/docs/guides/generate-artifacts.md and website/docs/reference/formats.md, together with this CLI-owned plan and closing verification evidence. The separately qualified TypeScript witness file is already in the published checkpoint. The installed AEP scope verb refused this release-plan kind because typed wave scope belongs to stories; no unsupported frontmatter was inserted and no fake story was created merely to bypass that rule. The release procedure is single-writer coordinator work, not a newly dispatched concurrent unit.

The read-only projection audit found no further fixed producer golden requiring migration: 46 normative sample projections and the authored document schema omit the ESS build version. Actual checks still decide drift. Only cargo xtask generate or cargo xtask schema may refresh their owned output after an actual failing check; historical example provenance and dependency fixtures remain untouched.

## Actual 0.20 producer and native qualification

The rebuilt 0.20.0 source passed the selected structural Rust/Go wire, historical map and TypeScript native/adversary checks with 24 printed passing runner results across eight result groups, including nested generated Rust test groups, zero failures and zero ignored cases. The TypeScript corpus again executes 30176 inherited controls plus 22 independent checks; those inner controls are separate from runner counts. Strict Clippy with the actual external-tool features and explicit release verify 0.20.0 also exited zero. The initial command mistakenly requested a nonexistent rust-typecheck feature and was refused before compilation; that exit 101 is retained. Rust wire tests are unconditional, so the corrected command uses only the actual go-typecheck and typescript-typecheck features. No test/source was changed to obtain these results.

Actual raw post-bump generation was compared to the retained actual 0.19 producer output. The ten-map 172-file witness changes only ten report producer lines; the twelve-map 218-file witness changes only twelve report producer lines. All other bytes, paths and literal fixtures are exact. The four structural maps change only four producer report lines and four first-line declaration headers; their remaining bodies/support files are exact. Raw old structural map SHA256 remains 02312f45fadac5e16b54aa1fce13d8f68cd8aaf14336a2180b85cd923811c79f; raw actual 0.20 map SHA256 is b628c84541f549efd1b388df24d15244effae9b5eff6bc66d28a3b298c1b810c. No synthetic producer input is offered as a post-bump observation.

Complete raw captures, initial refusal, corrected command exits and independent file comparisons are retained in release-020-preparation/qualification-020. The next commit contains the actual version/changelog/documentation and this qualification record. Literal task check and task site-build must both run on that exact commit before its main publication and annotated tag; no untested metadata followup may become the release tag target.
