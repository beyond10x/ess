---
format: aep.planning-md/2
id: story:change-fragment-upgrade-obligation
kind: story
status: draft
title: A change fragment states the upgrade a release asks of adopters
relations:
- serves: vision:O2
revision: 1
---
# A change fragment states the upgrade a release asks of adopters

`changes/*.yaml` moves to `b10x-change/v2` with an `upgrade:` block naming the source-format bump and the expected `ess verify diff` delta families; the WHATS-CHANGED check enforces that a release with a format bump carries one. Today fragments are prose only (`whats_changed.rs:53-67`), and `component.rs:55-58` admits an incompatibility recorded only in prose. Filed by ESS-EVOLUTION.md revision 3.
