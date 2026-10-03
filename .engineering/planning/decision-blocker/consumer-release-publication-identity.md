---
format: aep.planning-md/3
id: decision-blocker:consumer-release-publication-identity
kind: decision-blocker
status: open
title: Release publication awaits the operator identity decision
relations:
- blocks: task:consumer-backlog-20261002
withholds: approval
revision: 1
---
## Decision required

The release owner reports that an operator publication-identity decision still prevents tagging/publishing ESS0.52. The decision itself has not been supplied to this runtime/backlog lane. Clearance requires the release owner's acknowledged operator decision and its resulting publication authorization; passing source checks, bot Git identity or elapsed time supplies no answer. Do not invent the identity choice, change publishing authority or tag from this lane.

## Evidence and scope

User coordination on2026-10-03 confirms exact main e68684efb6a4ac22052c77d3ed8292fd44f9ace5 passed local task check and task site-build including site-lab, and reports main CI/Gate run37093196094 green, no open ESS PRs, no0.52tag and latest published0.51. Root independently read local final zero exits and rehashed check.log949920adfa37c81f1a1182c5632c758604719330a46821670a097233c08726a1 and site-build.loga41fb4900f775a6ed774acf532b9aab339a8b3c894541c232634e5d39c509224. Remote status is attributed to owner coordination in this record, not a fresh root API query. Exact release/tag/assets verification remains necessary after publication.

This blocker covers the release portion of task:consumer-backlog-20261002, not independent implementation or validation. The owner explicitly released its build reservation and permits owned work under existing resource limits. The full held ess/21 bundle remains on batch/ui-live-apps-complete-20261003 with one future bundle PR and the existing sole integrator. No partial bundle or separate runtime PR; all transport implementation remains with the third session. Source/evidence remain retained until published integration is explicitly verified.
