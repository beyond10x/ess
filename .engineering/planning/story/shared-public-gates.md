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
revision: 7
---
## Intent

The operator-approved Public shared gates, independent of Atlas plan adopts Eventlog, then ESS and AEP after verified producer publication. This story owns ESS adoption. Gates story:public-shared-gates owns producer implementation. Atlas authority documentation records the migration; Atlas availability and documentation reconciliation are not source integration requirements.

## Acceptance and scope

Install the immutable beyond10x/gates 0.1.0 reusable workflow and coordinated local hooks against the explicit adoption baseline 24d2fe714958c8cde63ea78122c31e28bcc682bc. This incorporates the concurrent ESS changes published after the originally proposed baseline; exact historical exceptions remain in protected policy. Require the shared check before subsequent integration, enable supported GitHub secret scanning and push protection, and retain ESS correctness, site-build and release requirements. Use the same bot identity without an Atlas checkout or organization-wide admission in commit/publish paths. Keep private policy outside public source. Document historical findings separately; do not rewrite history.

Scope: AGENTS.md, CHANGELOG.md, .github/workflows/shared-gates.yml, Taskfile.yml and this planning record. The test recipe runs the workspace normally and qualifies ess-xtask separately with the existing consumer-check native profile. Native linker flags must not reach nested WebAssembly builds. No guard or assertion is weakened. No ESS modeling behavior or downstream release changes.

## Evidence required

Published immutable producer and verified assets; local shared gate and receipt reuse; task check and task site-build; trusted base-branch workflow observation; selected-repository policy secret and required status/security settings readback. Source publication is authorized; ESS release is outside scope.

## Current verification

The three WebAssembly regressions that failed CI 34466577376 pass with the normal workspace profile. All 123 ess-xtask tests pass with the separately declared native consumer-check profile. The corrected candidate still requires the full repository CI before integration. Atlas documentation work does not block it.

## Operator-directed completion

After CI run 34468911188 passed all workspace tests and both macOS lanes but exhausted its 45-minute budget during consumer-check, the operator explicitly requested disabling consumer checks in CI and committing and pushing directly to main without another CI wait. CI now sets SKIP_CONSUMER_CHECKS=true; the omitted lane is reported explicitly. Local task check and task consumer-check retain consumer coverage. This instruction supersedes the earlier pre-integration CI wait for this delivery. Common security and privacy checks remain enforced. Concurrent main commit 833231633278f62aaf7841f85eb7e84891a730ef and both branches' original planning journal events are preserved.
