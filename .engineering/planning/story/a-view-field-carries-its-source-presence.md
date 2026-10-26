---
format: aep.planning-md/3
id: story:a-view-field-carries-its-source-presence
kind: story
status: implemented
title: A view field carries its source field's presence
summary: 'A projected view field mirrors or declares presence: so a null_when_absent member is sent as null'
tags:
- ess-0.69.0
refs:
- provider: github
  reference: beyond10x/ess#677
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T19:58:22Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T19:58:23Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:27Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---

- An entity field `Optional<…>` declares `presence: null_when_absent`.
- A view field that projects it cannot declare `presence:`: `RawViewField` refuses the key (`crates/specify/ess-domain/src/primitive_admission.rs:80-88` records this).
- So the view's published shape leaves the member out when absent, while the served API sends `"embed_token": null`. A conformance suite holds the implementation to the wrong spelling.

## Expected

A view field that projects a stored field carries that field's `presence:`, or accepts `presence:` itself (ess/15 rules: only on `Optional<T>`, `omitted_when_absent` refused on `Json`).

## Note

A view that projects a named struct already carries the struct's presence. The gap is a view field written field by field.
