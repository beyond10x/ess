---
title: Where this stands
description: Current-source ESS capabilities, a dated release observation, and their evidence boundaries.
---

# Where this stands

ESS is experimental and standalone.

Latest published release observed on 10 September 2026:
[0.22.1](https://github.com/beyond10x/ess/releases/tag/0.22.1). Its release record lists archives
for Linux and macOS on x86-64 and ARM64, plus SHA256SUMS. This is a dated asset-list observation;
it does not claim that the archives were downloaded, their checksums verified, or the binary
installed or executed. The release repairs the release gate and retains the `at_most_once`
bindings introduced in 0.22.0.

The dated conformance observations below remain scoped to their original release and execution.

## Current source capabilities

- validation and canonical compilation of typed system specifications;
- name resolution, reusable shapes, entity relations, total handle lookup, inspection, graphing,
  semantic diff, and impact analysis;
- outcome-to-entity assignments and parameterized views that make generated scenarios assert the
  values written by commands;
- repository Markdown documentation, HTML sites, JSON Schema, OpenAPI, and AsyncAPI generation;
- offline validation and deterministic TypeScript projection for adopter-owned JSON Schema registries;
- structural Rust, Go, browser, and Clap synthesis with explicit obligations;
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

### Support boundaries

These rows describe the source checkout. Dated conformance evidence remains scoped to its original
release observation; publishing a newer source tag does not re-run that evidence.
Output and CLI metadata checks establish kinds, versions and availability; the linked owners and
tests establish the bounded support and refusals. The offline `cargo xtask support --check` compares
this complete maintained block. It does not verify remote release records.

[ess-source-support-begin]: #

The source checkout’s workspace version is `0.22.1` and includes separately documented unreleased changes.

| Capability | Current source | Limits and evidence |
|---|---|---|
| Default projections | `docs`, `site`, `schema`, `openapi`, `asyncapi` | Generator inventory and actual CLI artifacts; [ess-gen](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/lib.rs). `docs-ir` is an additional explicit choice. |
| Documentation | `docs`: Markdown with Mermaid diagrams | A projection of the document model; [docs emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/docs.rs) establishes rendering, not implementation behavior. |
| Site | `site`: HTML and local stylesheet/Mermaid assets; explicit output at `index.html` and `assets/`, combined output under `site/` | Explicit authored pages and downloads are supported; ESS does not host the site. [authored-site tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/authored_site.rs). |
| Document IR | Explicit `docs-ir`: `docs-ir/document.json`, `ess-docs/1` | A document projection, not HTML or a general persisted EssIr reader. [document emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/document.rs). |
| JSON Schema | `https://json-schema.org/draft/2020-12/schema` | Named types, entities, command inputs, events and errors; structural validation does not establish behavior. [schema emitter/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/schema.rs). |
| Native API projections | OpenAPI `3.1.0`; AsyncAPI `3.0.0` | Projection directions; [OpenAPI emitter/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/openapi.rs) and [AsyncAPI emitter/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/asyncapi.rs) own their structural coverage. |
| Adapter directions | `infra import`: `kubernetes`, `openapi`; `generate project`: `buildkit`, `helm`, `kubernetes`, `openapi` | Availability comes from CLI help. No AsyncAPI importer is declared; the following rows qualify each adapter. |
| OpenAPI adapter | Supported 3.1 service/interface import to `ess-openapi-import/1`, retaining source and accounting; checked projection | External references refuse. Semantic gaps, unresolved references and legacy interface-only inputs block checked projection; annotation normalization alone may be allowed. [accounting tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-openapi/tests/accounting.rs) and [import/refusal owner](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-openapi/src/lib.rs). |
| Kubernetes import | Sanitized observation bundle or explicit live context to infrastructure IR | The live scanner is the credential edge. A fixed category list and empty `coverage_gaps` do not prove complete observation. [import/redaction owner](https://github.com/beyond10x/ess/blob/main/crates/infra/ess-kubernetes/src/lib.rs). |
| Kubernetes projection | Intent plus observed IR to patches, new objects and obligations | No apply operation; unstated decisions remain obligations and unsupported conditions may refuse. [projection/refusal tests](https://github.com/beyond10x/ess/blob/main/crates/infra/infra-project/tests/projection.rs). |
| BuildKit and Helm projection | Checked build IR to Dockerfile/Bake inputs; runtime IR to a configuration-neutral Helm chart | These projections neither execute BuildKit nor apply a chart or establish live resource availability. [deployment projection tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/tests/deployment.rs). |
| Structural synthesis | `rust`, `go`, `web`, `clap` | Generated structure plus obligations/refusals, not business behavior. All four full targets refuse modeled Binary64; separate structural data libraries have their own support boundary. [feasibility tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/feasibility.rs) and [synthesis guide](../guides/synthesize.md). |
| Clap synthesis | Command grammar, completion support and handler seams receiving `clap::ArgMatches`; generated `clap` and `clap_complete` 4 dependencies | No additional type layer or implemented command behavior. [Clap emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/clap/mod.rs) and [handler tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/clap.rs). |
| Typed CLI presentation | `specify cli` validates `ess-cli/1` to `ess-cli-plan/1`; `generate cli` emits a Rust/Clap package, help, Bash completion and reference with `ess-cli-artifacts/1` and `ess-cli-generation/1`; `--check` compares generated bytes | Typed inputs, results and declared errors remain model-owned; unsupported types and invariants refuse. Process context is separate from payloads. Application behavior requires `Handler`, dynamic native validation requires `DynamicValidator`, and the generated default handler is unavailable. [binding admission tests](https://github.com/beyond10x/ess/blob/main/crates/specify/ess-cli-contract/tests/binding.rs) and [projection and process tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-cli-project/tests/projection.rs). |
| Conformance targets | `billing`, `oracle-fixture` | Built-in reference implementations. A production adapter must establish its own execution boundary; these targets do not prove independent deployment. |
| Conformance formats | Defaults: `ess-conformance/4`, `ess-conformance-report/1`. Explicit count surfaces: `ess-conformance-report/2`, `ess-conformance-run/2`. CLI suite choices: `4`, `5` (default `4`); report choices: `1`, `2` (default `1`). | Actual report markers and CLI metadata; all-pass legacy execution can still mean inconclusive conformance. [count-report tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/count_reports.rs). |
| Coverage qualification | Current-source `ess-conformance/5` requires explicit report/2 before execution | Only a nonempty all-pass selection with complete inventory and no in-scope refusal can qualify. Suite/5, carrier and paired replay are unreleased relative to the dated 0.20.0 observation. [coverage CLI tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/coverage_cli.rs) and [conformance guide](../guides/verify-conformance.md#opt-into-declared-coverage). |
| Browser conformance | Replay presentation with no execution report | A green replay is not independent execution evidence; digest comparison does not authenticate the publisher. [browser admission tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/coverage_browser.rs). |
| Runtime compilation | Checks supplied identities, component coverage, replica bounds and stateful storage | Does not establish live provisioning or all resource requirements. [runtime checks/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/runtime.rs). |
| Explicit executors | `execute`, `publish`, `fetch`, `reconcile` invoke external clients; reconciliation requires `--authority` naming a protected registry entry and refuses without one, compares an admitted baseline desired deployment with the desired one, and attempts at most one admitted mutation per release | Caller-supplied state, authority and credentials remain material; a supplied baseline is admitted intent rather than proof of application, and a stopped invocation leaves the affected release unknown rather than absent or rolled back. [CLI executor owner](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/src/main.rs). The support check invokes none of these verbs. |
| Schema commands | `import-bundle`, `import-document`, `project-bundle`, `validate-bundle`, `types-bundle`, `normalize-check`, `normalize-run`, `normalize-generate`, `validate`, `typescript` | Current-source command inventory for import, validation, types and normalization; [CLI reference](../reference/cli.md#adopter-owned-schema-contracts) describes the selected operations. Availability is independent of the dated release record. |

[ess-source-support-end]: #

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
