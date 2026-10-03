---
format: aep.planning-md/3
id: decision-blocker:consumer-release-publication-identity
kind: decision-blocker
status: open
title: Release publication awaits the operator identity decision
relations:
- blocks: task:consumer-backlog-20261002
withholds: approval
revision: 3
---
## Decision required

The release owner reports that an operator publication-identity decision still prevents tagging/publishing ESS0.52. The decision itself has not been supplied to this runtime/backlog lane. Clearance requires the release owner's acknowledged operator decision and its resulting publication authorization; passing source checks, bot Git identity or elapsed time supplies no answer. Do not invent the identity choice, change publishing authority or tag from this lane.

## Evidence and scope

User coordination on2026-10-03 confirms exact main e68684efb6a4ac22052c77d3ed8292fd44f9ace5 passed local task check and task site-build including site-lab, and reports main CI/Gate run37093196094 green, no open ESS PRs, no0.52tag and latest published0.51. Root independently read local final zero exits and rehashed check.log949920adfa37c81f1a1182c5632c758604719330a46821670a097233c08726a1 and site-build.loga41fb4900f775a6ed774acf532b9aab339a8b3c894541c232634e5d39c509224. Remote status is attributed to owner coordination in this record, not a fresh root API query. Exact release/tag/assets verification remains necessary after publication.

This blocker covers the release portion of task:consumer-backlog-20261002, not independent implementation or validation. The owner explicitly released its build reservation and permits owned work under existing resource limits. The full held ess/21 bundle remains on batch/ui-live-apps-complete-20261003 with one future bundle PR and the existing sole integrator. No partial bundle or separate runtime PR; all transport implementation remains with the third session. Source/evidence remain retained until published integration is explicitly verified.

## Additional owner-held delivery ancestry refusal

Release-owner coordination on 2026-10-03 adds an independent delivery blocker to the pending publication-identity decision. Gates refused an optional prebuild publish of main e68684efb6a4ac22052c77d3ed8292fd44f9ace5 with "merged pull request missing or ambiguous". The owner reports that its GitHub update-branch call on PR 404 created 258594c86 (bot author, GitHub committer); this commit is in main ancestry but is not a completed pull-request merge, so the full-DAG published_merge verifier cannot admit it. No queue branch, workflow or tag was created. These refusal details are attributed to the owner's explicit coordination, not a reproduced publish attempt in this lane.

The release owner records dependency-blocker:release-0-52-delivery-ancestry in its integration planning store. This lane preserves that ownership and does not create a competing resolution. Passing source checks do not resolve provenance. Publishing descendants requires an approved provenance resolution as well as the separate publication-identity decision. Do not retry through another tool, bypass the refusal, rewrite main, or change trust policy. No delivery attempt is authorized from this lane.

Existing held-bundle boundaries remain: batch/ui-live-apps-complete-20261003 is the sole future bundle delivery branch, its existing root is sole integrator, no partial ess/21 release or independent PR, and transport implementation stays with the third session. Local source validation may continue under the released build reservation and existing resource guards. All frozen source handoffs and evidence remain retained until explicit published-integration verification.

## Local ancestry confirmation without publication

Root read local Git objects without invoking a publication path. Commit 258594c8602fa9e6d372ed5a869ad478d7e0c382 has parents 1998a0a870240c613722bf9881f29da9cfe71b41 and 27b1ef5075666a52e256c06c2dee47a172c3d95f, bot author, GitHub committer, and subject "Merge branch 'main' into fix/395-publisher-cap". Candidate e68684efb6a4ac22052c77d3ed8292fd44f9ace5 has parents 27b1ef5075666a52e256c06c2dee47a172c3d95f and 258594c8602fa9e6d372ed5a869ad478d7e0c382, with subject "Merge pull request #404 from beyond10x/fix/395-publisher-cap". git merge-base --is-ancestor for the former against the latter exits 0. Retained local object output SHA256 057f35cc1d3e3afd74a286971ac8a7a569f6aeec117b58e951a1bb4edd513a68.

This verifies the local ancestry and metadata, not remote admission or an approved remedy. The reported Gates refusal and pending identity decision remain owner-held and unresolved. This lane made no publication attempt and changed no source refs or trust policy. All four frozen runtime handoff documents were independently rehashed unchanged, including the two already acknowledged by the integrator and the later nested/explorer addenda.
