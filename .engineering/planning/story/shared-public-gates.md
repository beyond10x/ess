---
format: aep.planning-md/1
id: story:shared-public-gates
kind: story
status: active
title: Adopt independent common source gates
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: .github/workflows/shared-gates.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
revision: 5
---
## Intent

The operator-approved Public shared gates, independent of Atlas plan adopts Eventlog, then ESS and AEP after verified producer publication. This story owns ESS adoption. Gates story:public-shared-gates owns producer implementation; Atlas task:eventlog-source-gate-reuse coordinates the authority migration.

## Acceptance and scope

Install an immutable beyond10x/gates reusable workflow and coordinated local hooks against the explicit current-main baseline 988f219e90c9a7b105daec2345f77d6035075e92. Require the shared check before integration, enable supported GitHub secret scanning and push protection, and retain ESS correctness, site-build and release requirements. Use the same bot identity without an Atlas checkout or organization-wide admission in commit/publish paths. Keep private policy outside public source. Document historical findings separately; do not rewrite history. Scope: AGENTS.md, CHANGELOG.md, .github/workflows/shared-gates.yml and this planning record. No ESS modeling behavior or downstream release changes.

## Evidence required

Published immutable producer and verified assets; local shared gate and receipt reuse; task check and task site-build; trusted base-branch workflow observation; selected-repository policy secret and required status/security settings readback. Source publication is authorized; ESS release is outside scope.
