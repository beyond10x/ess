---
format: aep.planning-md/2
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
---
# A binding document can acknowledge a container ESS does not realize

OBS-BIND-008 (ESS 0.32.0) refuses any container or native sidecar no binding names. A third-party proxy or agent sidecar can then only be silenced by declaring it as a realization implementation it is not. Add an explicit acknowledgement for foreign containers to `ess-observed-bindings` (a format change), found by the M8 adversary review (observed_bindings.rs:499).
