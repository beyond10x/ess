---
title: Where this stands
description: Shipped ESS capabilities and the evidence behind them.
---

# Where this stands

ESS is experimental and standalone. `0.13.2` is the current release. Version releases publish
checksum-pinned `ess` archives for Linux and macOS on x86-64 and ARM64; a locked source build remains
the fallback for other targets.

## Shipped

- validation and canonical compilation of typed system specifications;
- name resolution, reusable shapes, entity relations, total handle lookup, inspection, graphing,
  semantic diff, and impact analysis;
- outcome-to-entity assignments and parameterized views that make generated scenarios assert the
  values written by commands;
- repository documentation, static-site-ready Markdown/sidebar, JSON Schema, OpenAPI, and AsyncAPI generation;
- offline validation and deterministic TypeScript projection for adopter-owned JSON Schema registries;
- structural Rust, Go, and browser synthesis with explicit obligations;
- semantic conformance-suite generation, component-scoped suites, and reference execution;
- standalone conformance reports from the Rust and generated Go runners;
- component descriptors and deterministic build, runtime, release, stack, and deployment models;
- canonical build IR that round-trips through release verification, digest-pinned OCI release
  bundles, and affected-only Helm reconciliation behind explicit executor commands;
- generated Services, stateful workloads, persistent-volume claims and mounts, with schema-valid
  configuration-neutral Helm defaults;
- OpenAPI import and projection for the declared service/interface subset;
- sanitized Kubernetes import, infrastructure analysis, and manifest projection;
- deterministic fixture and generated-byte checks across the workspace.

The site projection is intentionally narrower than a site builder. It consumes an ESS
specification—not prose—and emits Markdown, YAML frontmatter, and `sidebar.json`; another system
owns HTML rendering, theme, navigation shell, and hosting.

The CLI presents four areas: `specify`, `generate`, `verify`, and `infra`. Earlier flat spellings
remain hidden aliases with the same accepted-command output and exit status.

Compilation and projection remain deterministic and offline. Live Kubernetes import and the
commands named `execute`, `publish`, `fetch`, and `reconcile` are explicit credential edges; they do
not turn ESS into a continuously running deployment control plane.

The offline repository gate is `task check`. The documentation and browser-lab gate is
`task site-build` because installing the pinned npm dependency graph requires network access.

## Compatibility posture

Persisted v1 formats retain their current identity, reference, and canonicalization rules. Internal
Rust types do not by themselves create a new format. A strict old reader must be tested before an
additive field is written to a format that rejects unknown keys.
