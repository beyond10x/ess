---
format: aep.planning-md/3
id: story:reusable-component-release-action
kind: story
status: implemented
title: Publish any ESS component through one release action
summary: Provide an ESS-owned action for building or adopting an image and publishing chart, evidence, and bundle artifacts.
relations:
- decomposes: epic:oci-component-delivery
scope:
- confidence: cited
  path: .github/actions/release-component/action.yml
- confidence: cited
  path: .github/actions/release-component/release.sh
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-04T02:56:53Z", actor: "human:timo", revision: 3, imported: true}
- {from: "proposed", to: "active", at: "2026-09-04T02:56:53Z", actor: "human:timo", revision: 4, imported: true}
- {from: "active", to: "implemented", at: "2026-09-04T03:02:37Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
## Outcome

A component repository invokes one ESS-owned composite action to build or adopt its exact image, package its chart, sign both, publish evidence, verify release manifests, and publish the digest-addressed component bundle.

## Acceptance

- Inputs are component-generic and contain no Service SDK package assumptions or registry coordinates.
- Generated services can ask ESS to build and push the image; mature services can adopt a previously published multi-architecture tag from an earlier job.
- The action runs a repository-supplied verification command before publication.
- Runtime, chart, SBOM, provenance, signatures, conformance log, release manifests, and bundle remain tied to exact digests.
- Callers retain their own permissions, registry credentials, and version/tag policy.

## Scope

- `.github/actions/release-component/action.yml`
- `.github/actions/release-component/release.sh`
- `Taskfile.yml`
- `CHANGELOG.md`
