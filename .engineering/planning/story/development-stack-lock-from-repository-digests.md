---
format: aep.planning-md/3
id: story:development-stack-lock-from-repository-digests
kind: story
status: draft
title: A development environment reaches a deployment without an executor's release catalogue
tags:
- adopter-report
- deployment-chain
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A repository can reach `ess generate deployment compile` for a bench or development environment
from what it knows (source commit and semantic, build and runtime digests), without an
executor-produced release catalogue.

## Evidence

An adopter on ess 0.55.0 could not get a stack lock: `ess generate stack resolve` requires
`--catalog` of executor-produced `ess-release/1` entries (artifacts, provenance, SBOM, signature,
conformance evidence) and uses only that catalogue (`crates/edge/ess-cli/src/main.rs:1319`,
`release.rs:11`); `release` only verifies (`main.rs:1299`). Still so on main.

## Acceptance

- A design page under `docs/design/` decides the development path (for example a local release
  entry marked unsigned and unattested, admitted only by an environment that says so).
- A production environment still refuses a lock whose entries lack provenance or signature.
- A development environment compiles a deployment from a repository checkout, checked by an
  ess-cli test.
