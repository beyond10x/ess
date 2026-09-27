---
format: aep.planning-md/2
id: story:explorer-takes-external-branches
kind: story
status: active
title: The model explorer reaches external branches and reports reach per branch
relations:
- serves: vision:O2
- supersedes: story:external-mutation-explorer-and-toolchain
- decomposes: epic:retrofit-findings-20260927
revision: 3
---
## Scope

- #156: the generated model explorer takes `external:` branches as choices the target's double
  arranges, and reports reach per branch.

Split from `story:external-mutation-explorer-and-toolchain` (archived). Cited sites:
`crates/verify/ess-conformance/src/go/explore.go:518-590`, `src/ts/explore.ts:390-420`. Scoper
reading (not run): the explorers exclude any condition other than `when`, `otherwise` and
`wrong_state`, so this is a new condition kind in both.

## Acceptance

An exploration of a specification with external branches reaches them through the double and
reports, per branch, whether it was reached.
