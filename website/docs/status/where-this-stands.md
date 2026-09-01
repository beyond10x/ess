---
title: Where this stands
description: Shipped ESS capabilities and the evidence behind them.
---

# Where this stands

ESS is experimental, standalone, and released as `0.1.x`.

## Shipped

- validation and canonical compilation of typed system specifications;
- name resolution, total handle lookup, inspection, graphing, semantic diff, and impact analysis;
- documentation, JSON Schema, OpenAPI, and AsyncAPI generation;
- structural Rust, Go, and browser synthesis with explicit obligations;
- semantic conformance-suite generation and reference execution;
- standalone conformance reports;
- OpenAPI import and projection for the declared service/interface subset;
- sanitized Kubernetes import, infrastructure analysis, and manifest projection;
- deterministic fixture and generated-byte checks across the workspace.

The offline repository gate is `task check`. The documentation and browser-lab gate is
`task site-build` because installing the pinned npm dependency graph requires network access.

## Compatibility posture

Persisted v1 formats retain their current identity, reference, and canonicalization rules. Internal
Rust types do not by themselves create a new format. A strict old reader must be tested before an
additive field is written to a format that rejects unknown keys.
