---
format: aep.planning-md/3
id: story:diff-classifies-an-added-domain-entry
kind: story
status: active
title: ess verify diff classifies a domain added to system.yaml as compatible
refs:
- provider: github
  reference: beyond10x/ess#469
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T11:19:52Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-07T11:19:52Z", actor: "human:timo", revision: 3}
---
## Outcome

Adding a domain to `system.yaml`'s `domains:` list is classified `compatible`, not
`system/<name>/unclassified-changed` with verdict `unknown`
(https://github.com/beyond10x/ess/issues/469). Measured on 0.55.0 with the issue's `warehouse`
reproducer: `fails --fail-on breaking-or-unknown: system/warehouse/unclassified-changed`.

## Acceptance

- The reproducer exits 0 under `--fail-on breaking-or-unknown`, with the added domain named as a
  compatible change; removing a domain stays breaking; reordering stays unreported.
