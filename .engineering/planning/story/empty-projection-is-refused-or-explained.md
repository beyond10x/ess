---
format: aep.planning-md/3
id: story:empty-projection-is-refused-or-explained
kind: story
status: active
title: A projection that writes nothing says why
refs:
- provider: github
  reference: beyond10x/ess#102
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:33:16Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:33:49Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

A projection that writes nothing says why.

## Why

GitHub issue beyond10x/ess#102; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#102 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.

## Delivery reconciliation 2026-10-02

Source audit identifies delivered behavior in 69abcd25a, included in released 0.51.0: default empty projection names unowned domains on stderr and exits zero; --strict refuses before writing output. Tests in crates/edge/ess-cli/tests/empty_projection.rs:76,93,109,125 cover both projection families, aliases, owned controls and refusal. This selects the issue's explanatory alternative. Historical red-first execution evidence has not been recovered, so the active story's red-first acceptance is not silently discharged. No new implementation is warranted for this already-delivered behavior; remaining work is evidence reconciliation.
