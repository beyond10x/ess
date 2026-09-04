---
title: CLI reference
description: The canonical ESS command, the four areas its first level is made of, and the flat spelling every verb keeps.
---

# CLI reference

`ess` is the canonical command. It exits `0` on success and non-zero on invalid input, unresolved
semantics, unsupported projection, or a failed check.

Its first level is the four areas ESS is built out of, one per crate directory, and `ess --help`
lists exactly those:

| Area | The verbs it holds |
|---|---|
| `ess specify` | `validate`, `compile`, `compose`, `inspect`, `graph`, `realization`, `runtime` |
| `ess generate` | `generate`, `synthesize`, `project`, `schema`, `build`, `release`, `stack`, `deployment` |
| `ess verify` | `conform`, `diff`, `impact` |
| `ess infra` | `infra`, `import` |

## Flat spellings

Every verb is also spelled flat at the top level, exactly as it was before the areas existed:
`ess validate --path .` is `ess specify validate --path .`, `ess conform run …` is
`ess verify conform run …`, and `ess import openapi …` is `ess infra import openapi …`. A flat
spelling is the same command with the same arguments, and it prints no notice of any kind: when the
command runs, both spellings produce the same stdout, the same stderr and the same exit status, so
a caller that reads the output is unaffected. For a refusal `ess` itself did not write — a missing
required argument, which the argument parser answers — and for `--help`, only the `Usage:` line
differs: it names the path you typed. It is left out of `--help` only so the listing stays the four
areas. Nothing is deprecated and no pinned caller needs changing.

Two verbs share the name of the area they sit in, so their flat spelling is one level shorter
rather than one word: `ess generate --path …` is `ess generate generate --path …`, and
`ess infra diagnose …` is `ess infra infra diagnose …`. `ess generate --help` therefore offers the
verb's options beside the area's subcommands, and the two cannot be written together —
`ess generate --path PATH synthesize` names a specification for a verb that takes one and is
refused with exit 2 rather than run against the current directory.

## `ess specify` — a system, resolved

| Command | Purpose |
|---|---|
| `ess specify validate [--path PATH] [--format text\|yaml\|json]` | Load, resolve, and validate one specification. |
| `ess specify compile [--path PATH] [--out FILE] [--format …]` | Produce canonical typed IR. |
| `ess specify compose --path PATH --service KEY=PATH… [--out FILE]` | Compile exact component surfaces into composition IR and generated clients. |
| `ess specify inspect --path PATH NAME [--format …]` | Resolve and render one declaration. |
| `ess specify graph [--path PATH] [--format dot\|mermaid\|json\|yaml]` | Render the interaction graph. |
| `ess specify realization validate …` | Resolve a physical realization against one exact ESS digest. |
| `ess specify realization compile …` | Emit deterministic `ess-realization-ir/1`. |
| `ess specify realization generate …` | Render a run-mode guide from the resolved realization. |
| `ess specify runtime compile …` | Compile `ess-runtime/1` against exact semantic, realization, and build inputs. |

## `ess generate` — artifacts, and never an applied one

| Command | Purpose |
|---|---|
| `ess generate --path PATH --kind docs\|site\|schema\|openapi\|asyncapi --out PATH` | Generate deterministic projections. `site` is Markdown plus frontmatter and `sidebar.json`, not HTML. |
| `ess generate synthesize …` | Emit supported structural implementation artifacts plus obligations. |
| `ess generate project <adapter> …` | Project typed IR into concrete artifacts. |
| `ess generate schema validate …` | Validate adopter-owned JSON Schema contracts. |
| `ess generate build compile\|graph …` | Validate and compile `ess-build/1`, or render its DAG. |
| `ess generate release verify …` | Verify an `ess-release/1` against exact build and runtime IR. |
| `ess generate stack resolve\|validate …` | Resolve generic product stacks from an offline release catalogue. |
| `ess generate deployment compile\|diff …` | Bind an exact stack lock to an environment, or compare two deployments. |

Run `ess generate synthesize --help` and `ess generate <command> --help` for target-specific
arguments.

Omitting `--kind` generates every projection. Omitting `--out` lists or serializes artifacts
without writing them. The repository-only `cargo xtask generate` command reconciles the committed
`generated/` projection tree; `cargo xtask generate --check` compares it without writing.

### Adopter-owned schema contracts

| Command | Purpose |
|---|---|
| `ess generate schema validate PATH… --schemas DIR [--format text\|yaml\|json]` | Validate JSON instances against the offline `*.schema.json` registry they select by stable `schema` identity. |
| `ess generate schema typescript SCHEMA_ID --root TYPE --schemas DIR [--out FILE] [--check]` | Project deterministic structural TypeScript from one authoritative JSON Schema. |

Both operations are offline. Schema identity comes from `$id`; filenames only locate documents.
`--check` compares an existing generated module byte for byte without rewriting it.

## `ess verify` — held to what was declared

| Command | Purpose |
|---|---|
| `ess verify conform synthesize …` | Generate the semantic suite required by a specification. |
| `ess verify conform run …` | Execute a suite against a supported target and emit a standalone report. |
| `ess verify diff --from PATH --to PATH [--format text\|json]` | Compare two revisions semantically. |
| `ess verify impact --from PATH --to PATH [--suite PATH] [--format …]` | Name invalidated scenarios and generated artifacts. |

Run `ess verify conform <command> --help` for target-specific arguments.

## `ess infra` — an observed cluster, and what reads one

| Command | Direction |
|---|---|
| `ess infra import openapi …` | OpenAPI → supported ESS service/interface IR. |
| `ess infra import kubernetes …` | sanitized bundle or explicitly selected live cluster → infrastructure IR. |
| `ess generate project openapi …` | ESS service/interface IR → OpenAPI. |
| `ess generate project kubernetes …` | infrastructure intent and observation → manifests and obligations. |

Projection writes artifacts only. It does not call `kubectl`, apply a manifest, or mutate a target.

`ess infra infra` contains `diagnose`, `graph`, and `diff` operations over sanitized infrastructure
IR — the same three as `ess infra diagnose`, `ess infra graph` and `ess infra diff`, which are their
flat spellings. Live or bundle scanning is under `ess infra import kubernetes`; manifest generation
is under `ess generate project kubernetes`. Run `ess infra infra --help` for their current
arguments.
