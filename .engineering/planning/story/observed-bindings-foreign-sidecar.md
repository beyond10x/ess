---
format: aep.planning-md/3
id: story:observed-bindings-foreign-sidecar
kind: story
status: implemented
title: A binding document can acknowledge a container ESS does not realize
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/observed_bindings.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T00:24:48Z", actor: "human:timo", revision: 3, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T00:26:29Z", actor: "human:timo", revision: 4, imported: true}
- {from: "active", to: "implemented", at: "2026-09-26T03:25:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
# A binding document can acknowledge a container ESS does not realize

OBS-BIND-008 (ESS 0.32.0) refuses any container or native sidecar no binding names. A third-party proxy or agent sidecar can then only be silenced by declaring it as a realization implementation it is not. Add an explicit acknowledgement for foreign containers to `ess-observed-bindings` (a format change), found by the M8 adversary review (observed_bindings.rs:499).
