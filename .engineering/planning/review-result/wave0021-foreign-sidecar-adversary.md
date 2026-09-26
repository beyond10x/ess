---
format: aep.planning-md/2
id: review-result:wave0021-foreign-sidecar-adversary
kind: review-result
status: active
title: 'Wave 0021: adversary review of foreign-container acknowledgements'
relations:
- reviews: story:observed-bindings-foreign-sidecar
revision: 1
---
# Wave 0021: adversary review of foreign-container acknowledgements

Unit: `story:observed-bindings-foreign-sidecar`, ess#115, merged with main after ess#109 (merge `6a92c7c8`; 28/28 `observed_bindings` tests on the merge).

- Two adversary passes. Every finding was fixed with a mutation check; ess-cli tests 615 -> 627.
- Result: `ess-observed-bindings/2` adds `foreign_containers` (name and reason); report `/3` lists them as acknowledged, not bound.
- An acknowledgement cannot hide the realization: its own image, artifact locator, `@sha256:` digest under another name, or artifact identity still violates `OBS-BIND-008`.
- A stale acknowledgement leaves `OBS-BIND-008` unknown; `/1` is read unchanged and refuses the key.
