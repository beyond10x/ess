---
title: Where this stands
description: Shipped ESS capabilities and the evidence behind them.
---

# Where this stands

ESS is experimental and standalone. `0.5.1` is the current tagged release. Version releases publish
checksum-pinned `ess` archives for Linux and macOS on x86-64 and ARM64; a locked source build remains
the fallback for other targets. Entity relations shipped in `0.5.0`; the static-site source
projection shipped in `0.4.0`.

## Shipped

- validation and canonical compilation of typed system specifications;
- name resolution, total handle lookup, inspection, graphing, semantic diff, and impact analysis;
- repository documentation, static-site-ready Markdown/sidebar, JSON Schema, OpenAPI, and AsyncAPI generation;
- offline validation and deterministic TypeScript projection for adopter-owned JSON Schema registries;
- structural Rust, Go, and browser synthesis with explicit obligations;
- semantic conformance-suite generation and reference execution;
- standalone conformance reports;
- OpenAPI import and projection for the declared service/interface subset;
- sanitized Kubernetes import, infrastructure analysis, and manifest projection;
- deterministic fixture and generated-byte checks across the workspace.

The site projection is intentionally narrower than a site builder. It consumes an ESS
specification—not prose—and emits Markdown, YAML frontmatter, and `sidebar.json`; another system
owns HTML rendering, theme, navigation shell, and hosting.

The offline repository gate is `task check`. The documentation and browser-lab gate is
`task site-build` because installing the pinned npm dependency graph requires network access.

## Compatibility posture

Persisted v1 formats retain their current identity, reference, and canonicalization rules. Internal
Rust types do not by themselves create a new format. A strict old reader must be tested before an
additive field is written to a format that rejects unknown keys.
