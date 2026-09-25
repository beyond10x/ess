---
format: aep.planning-md/2
id: story:infrastructure-acceptance-against-independent-observation
kind: story
status: implemented
title: 'Infrastructure acceptance: a placement is compared against an independent observation'
relations:
- decomposes: initiative:ess-evolution
- serves: vision:O2
revision: 4
---
# Infrastructure acceptance: a placement is compared against an independent observation (ESS evolution M8)

## Outcome

A selected service placement is linked to a deterministic infrastructure projection and compared against an
observation something other than the projection produced. The comparison names every difference; a desired
declaration is never counted as an observation, and nothing in ESS deploys a projection.

## Delivered

ess#83 (`601ed2c68`), released in 0.32.0: `task infra-acceptance` runs the generated billing service in a
disposable k3d cluster. Rows (final run, round 3, cluster `ess-m8-6d13`):

1. the bindings document validates;
2. the projection is produced twice, byte-identical;
3. only harness manifests reach the cluster (static guard, 12 bypass fixtures);
4. the live read outside Git passes, and the harness manifest offered as an observation is refused (`OBS-BIND-006`);
5. three sensitivity cases fail as intended (`OBS-BIND-004`, `-002`, `-008`);
6. no Secret value reaches a written file (plain and base64 scan, planted-copy control);
7. teardown verified.

Three adversary passes; every finding fixed test-first. Evidence: `.ess-evolution/waves/0020-close-c-g/m8/`.
Selection: `.ess-evolution/waves/0017-phase-f-infra/story-body.md`.
