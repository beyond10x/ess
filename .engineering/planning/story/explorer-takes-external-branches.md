---
format: aep.planning-md/3
id: story:explorer-takes-external-branches
kind: story
status: implemented
title: The model explorer reaches external branches and reports reach per branch
relations:
- serves: vision:O2
- supersedes: story:external-mutation-explorer-and-toolchain
- decomposes: epic:retrofit-findings-20260927
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T07:58:33Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T08:00:41Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T14:57:29Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
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
