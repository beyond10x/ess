---
format: aep.planning-md/1
id: decision-blocker:retained-result-release-qualification
kind: decision-blocker
status: open
title: Qualify the local gate for the retained-result release
relations:
- blocks: story:retained-command-result-replay
withholds: test_result
revision: 1
---
## Decision boundary

The retained-result release remains subject to AGENTS.md's local task check and
exact-tag qualification. Existing CI and release workflows explicitly use
SKIP_CONSUMER_CHECKS=true. The prior local-release-gate-profile resolution
authorized that profile for0.28.0 only; it does not authorize0.29.0.

## Observed refusal

A fresh task consumer-check on the prepared0.29.0 coordinator refused:

```text
unclassified concrete consumer entry ess_cli::bin(ess)::enum::SuiteTarget/variant/Typescript; finite review required
```

Accounting also reported an unknown claimed NamedType wire path. The retained
consumer-check-preflight.log, status and extraction inventory distinguish this
registry/accounting debt from the independent retained-result runtime findings.
The inventory includes inherited and newly introduced entries; no claim says
every outstanding entry predates this release. No consumer cases qualified in
that refused run. The final corrected candidate still requires a fresh gate.

## Current handling

Finish and independently review the source correction, and execute the remaining
repository, documentation and release checks before presenting a concrete
release decision. No request or answer is recorded yet. Continue independent
implementation and verification while preparing that candidate.

The bounded choice will be to authorize the existing CI/release profile for
this release, preserving the default local refusal, or to complete the finite
consumer-accounting qualification first. Do not weaken the gate, bulk-invent
applicability or rewrite the pinned historical accounting baseline. The existing
consumer-accounting-baseline-never-extended story retains that broader work.

This blocker concerns source integration/release qualification. Passing scoped
tests or publishing a review branch cannot clear it or establish EKR runtime
conformance, durability or phase completion.
