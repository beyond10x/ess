---
title: CLI reference
description: The canonical ESS command and its model, schema-contract, adapter, generation, and conformance surfaces.
---

# CLI reference

`ess` is the canonical command. It exits `0` on success and non-zero on invalid input, unresolved
semantics, unsupported projection, or a failed check.

## Model and analysis

| Command | Purpose |
|---|---|
| `ess validate [--path PATH] [--format text\|yaml\|json]` | Load, resolve, and validate one specification. |
| `ess compile [--path PATH] [--out FILE] [--format …]` | Produce canonical typed IR. |
| `ess inspect --path PATH NAME [--format …]` | Resolve and render one declaration. |
| `ess graph [--path PATH] [--format dot\|mermaid\|json\|yaml]` | Render the interaction graph. |
| `ess diff --from PATH --to PATH [--format text\|json]` | Compare two revisions semantically. |
| `ess impact --from PATH --to PATH [--suite PATH] [--format …]` | Name invalidated scenarios and generated artifacts. |

## Generation and synthesis

| Command | Purpose |
|---|---|
| `ess generate --path PATH --kind docs\|site\|schema\|openapi\|asyncapi --out PATH` | Generate deterministic projections. `site` is Markdown plus frontmatter and `sidebar.json`, not HTML. |
| `ess synthesize …` | Emit supported structural implementation artifacts plus obligations. |
| `ess conform synthesize …` | Generate the semantic suite required by a specification. |
| `ess conform run …` | Execute a suite against a supported target and emit a standalone report. |

Run `ess synthesize --help` and `ess conform <command> --help` for target-specific arguments.

Omitting `--kind` generates every projection. Omitting `--out` lists or serializes artifacts
without writing them. The repository-only `cargo xtask generate` command reconciles the committed
`generated/` projection tree; `cargo xtask generate --check` compares it without writing.

## Adopter-owned schema contracts

| Command | Purpose |
|---|---|
| `ess schema validate PATH… --schemas DIR [--format text\|yaml\|json]` | Validate JSON instances against the offline `*.schema.json` registry they select by stable `schema` identity. |
| `ess schema typescript SCHEMA_ID --root TYPE --schemas DIR [--out FILE] [--check]` | Project deterministic structural TypeScript from one authoritative JSON Schema. |

Both operations are offline. Schema identity comes from `$id`; filenames only locate documents.
`--check` compares an existing generated module byte for byte without rewriting it.

## Import and projection

| Command | Direction |
|---|---|
| `ess import openapi …` | OpenAPI → supported ESS service/interface IR. |
| `ess project openapi …` | ESS service/interface IR → OpenAPI. |
| `ess import kubernetes …` | sanitized bundle or explicitly selected live cluster → infrastructure IR. |
| `ess project kubernetes …` | infrastructure intent and observation → manifests and obligations. |

Projection writes artifacts only. It does not call `kubectl`, apply a manifest, or mutate a target.

## Infrastructure analysis

`ess infra` contains `diagnose`, `graph`, and `diff` operations over sanitized infrastructure IR.
Live or bundle scanning is under `ess import kubernetes`; manifest generation is under
`ess project kubernetes`. Run `ess infra --help` for their current arguments.
