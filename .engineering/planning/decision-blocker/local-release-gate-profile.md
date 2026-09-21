---
format: aep.planning-md/1
id: decision-blocker:local-release-gate-profile
kind: decision-blocker
status: cleared
title: Choose the local qualification profile for the prepared release
relations:
- blocks: task:release-0-28-0
withholds: test_result
revision: 3
---
## Decision needed
The operator requested integration into main and passing checks. The prepared release passes the workspace and tooling tests, source and projection checks, fuzz lane, release metadata and site build. Local task check still refuses consumer-accounting registry drift; the existing CI/release workflow explicitly excludes that lane.

## Concrete refusal
unclassified concrete consumer entry ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript; finite review required

Accounting additionally reports an unknown claimed NamedType wire path. Read-only extraction finds 140 unclassified entries, including inherited additions; new model semantics also need qualified cells. No unknown eligibility or blanket behavioral support has been added.

## Options presented
Use the already configured CI/release profile for this release while preserving the local refusal, or finish the entire finite consumer-accounting qualification before release. A direct question is pending with the operator. No answer or exception is inferred from elapsed time.

## Evidence
See docs/conformance-core-checkpoint-2026-09-21.md. Release version and both Cargo locks are prepared; publication remains pending.

## Resolution

The operator explicitly instructed merging and tagging PR #56 on 2026-09-21, after being told that every CI check passes, the full local consumer-accounting gate still refuses, and release under the existing CI profile awaits approval. This authorizes the existing CI/release profile for this bounded 0.28.0 release. Local qualification will use task check SKIP_CONSUMER_CHECKS=true plus task site-lab on the exact tag commit. The ordinary default task check retains consumer accounting; its refusal remains recorded and no accounting baseline or assertion is relaxed. Integration and annotated tagging use the organization bot, followed by verification of the exact tag, required checks, published release and four archives plus checksums.
