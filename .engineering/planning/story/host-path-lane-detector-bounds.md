---
format: aep.planning-md/1
id: story:host-path-lane-detector-bounds
kind: story
status: implemented
title: 'Three constructed bounds on the host-path gate lane: what precedes a marker, non-ASCII account names, and an unknown CI platform'
relations:
- serves: vision:O2
- informed_by: story:fixtures-carry-workstation-paths
scope:
- confidence: cited
  path: crates/edge/ess-xtask/tests/host_paths.rs
revision: 6
---
## Finding

Three bounds on `crates/edge/ess-xtask/tests/host_paths.rs`, each **constructed** by
`aep-drive:adversary` pass 2 in ESS wave 22 (`review-result:adversary-wave22-unit2-pass-2`) and each
reaching nothing in the repository today. They are recorded rather than fixed because the attack
budget for that unit was spent and none of them is a present leak.

| # | The bound | Measured |
|---|---|---|
| 1 | The detector never asks what **precedes** a marker, so a relative path or URL with a component named `home`, `root` or `Users` is collected — and reported as an absolute path that exists on no host. A tracked path whose second segment is `home` is collected as though the marker began it, which names a directory that exists on no host. The literal is left out of this sentence because the repository's own secret scanner reads it as a workstation path — which is the bound being described, arriving from the other side. | 9 constructed false positives; 0 of the 571 live findings are of this shape |
| 2 | `continues_a_path` accepts ASCII only, so a home path whose account name **starts** with a non-ASCII byte is collected as nothing: `home-path:sha256:907967c2d6267512aad75534888116dd1b9bd286e9462d8d4e5f18731b8ae147` → nothing. Only the first byte decides — `…/jösé/…` is still caught, truncated. | constructed; no such tracked path |
| 3 | `ci_runner_labels` is documented as failing when CI adds a platform with no home root recorded, but it searches only for families `RUNNER_HOME_ROOTS` already names, so an added platform yields no label and the case passes on the runners it knows. | constructed workflow text; all seven workflow files run only ubuntu or macos |

Bound 1 matters most of the three: `website/` is a Docusaurus tree where a route named `home` is
ordinary, so the day one is added the gate refuses a clean file and names a path that exists
nowhere. A gate that refuses a clean file is a gate somebody switches off.

## Acceptance

Each of the three either holds or is stated in the lane's module doc as a bound it does not cover,
beside the one already stated for non-standard home roots. A red-first case for each one that is
fixed.

## Not in scope

The blockers pass 2 found — the surviving `home-path:sha256:05292b25783aaf4e4f66300d2a83cc809bae6ec68927bf615cedbfd4ca174e6f` mutant, the `$HOME` dependency and the panic on a
tracked file the working tree lacks — are corrected in wave 22 itself and are not this story's.
