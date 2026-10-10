---
title: Where this stands
sidebar_position: 1
description: Current-source ESS capabilities, a dated release observation, and their evidence boundaries.
lede: ESS is experimental and standalone. What the current source does, the latest release observed, and the evidence behind each claim.
source: Capabilities generated from website/data/status.json by cargo xtask site-data; the support block checked by cargo xtask support
---

# Where this stands

## Latest release

Latest published release observed on 1 October 2026:
[0.51.0](https://github.com/beyond10x/ess/releases/tag/0.51.0). Its release record lists archives
for Linux and macOS on x86-64 and ARM64, plus SHA256SUMS. This is a dated observation of the asset
list: it does not claim that the archives were downloaded, their checksums verified, or the binary
installed or run.

The last five releases:

| Release | What it added |
|---|---|
| 0.51.0 | Generated Rust servers select an `existing_instance:` refusal or a creating `unknown_instance:` branch by looking the input identity up in storage before dispatch; generated output committed with its `.ess-output` regenerates in another checkout when its owned files still have their recorded bytes |
| 0.50.0 | Synthesis witnesses an aggregate view whose creating command has a `when_related` guard, instead of refusing it with ESS-SYNTH-017 |
| 0.49.0 | Generated Rust and Go servers enforce actor grants: they take the authenticated caller and answer `403 not granted` before a command runs, and served suites add `grant/denied` and `grant/admitted` scenarios; source format `ess/20` (a related row's lifecycle `state` in `when_related`); synthesis witnesses copies from a related row, `when_related` over an `owns` via field and overlapping `when:` branches; `verify diff` leaves no residual for a one-sided declaration |
| 0.48.0 | `ess ui test` runs `ess-ui-test/1` tests headless and emits a Playwright spec from the same file; `verify conform report` turns an outside runner's per-scenario results into a report/2 that says ESS executed nothing; a served `501` carries `committed`; `ess ui check --model <dir>` reads `ess-inputs.yaml`; a delegating `ess` names the release it runs on every command |
| 0.47.0 | The generated Rust and Go servers answer with the events a command published, keep request headers and name and encode every system event, and bindings are delivered per binding; `ess-ui/1` UI documents load, check, render as a terminal application or a React project and document themselves (`ess ui`, `ess generate ui`); `verify diff` no longer reports an added or regrouped aggregate view as `unclassified-changed` |

[Format version history](../reference/spec-versions.md) says what every format version changed and
which release introduced it, and the
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every release.

## Current source capabilities

[ess-status-record-begin]: #

Generated from `website/data/status.json`, the file the landing page's status section reads; statuses as of 2026-10-04.

### Specify

- `shipped` [Validation and canonical compilation](../guides/write-a-specification.md) — Name resolution, reusable shapes, entity relations, total handle lookup, inspection and graphing.

### Generate

- `shipped` [Documentation and interface projections](../guides/generate-artifacts.md) — docs, site, schema, openapi and asyncapi; docs-ir on request.
- `shipped` [JSON Schema registries](../guides/generate-artifacts.md#generated-schemas-in-a-local-registry) — Offline validation and deterministic TypeScript projection for adopter-owned registries.
- `shipped` [Structural synthesis](../guides/synthesize.md) — rust, go, web and clap targets, with explicit obligations and refusals.
- `planned` [Obligations as trackable records](roadmap.md) — Today an obligation is an entry in the generated plan, not a record evidence can close.

### Verify

- `shipped` [Conformance suites and runs](../guides/verify-conformance.md) — Authored and component-scoped suites, outcome-to-entity assignments and parameterized views; built-in targets billing, oracle-fixture and interpreted; generated Go and TypeScript runners with standalone reports.
- `shipped` [Mutation audit](../guides/verify/mutation-audit.md) — Mutates the specification to find rules the synthesized suite does not pin; seeded command exploration and linearizability checks of recorded histories.
- `shipped` [Semantic diff and impact](../guides/track-change.md) — verify diff and verify impact.
- `planned` [A conformance target in another process](roadmap.md) — verify conform run reaches only the targets built into ess.

### Infrastructure

- `shipped` [OpenAPI and Kubernetes import](../guides/check-infrastructure.md) — OpenAPI 3.0 and 3.1 service subset; sanitized Kubernetes observations to infrastructure IR, analysis and manifest projection.
- `shipped` [Component delivery](../guides/deliver/deploy-an-environment.md) — Build, runtime, release, stack and deployment models; generated Services, stateful workloads and volume claims; digest-pinned OCI bundles and affected-only Helm reconciliation behind explicit executor commands.

### UI

- `shipped` [UI documents](../reference/ess-ui.md) — ess-ui/1 documents load, check, render and test headless.

[ess-status-record-end]: #

### Support boundaries

These rows describe the source checkout. Dated conformance evidence remains scoped to its original
release observation; publishing a newer source tag does not re-run that evidence.
Output and CLI metadata checks establish kinds, versions and availability; the linked owners and
tests establish the bounded support and refusals. The offline `cargo xtask support --check` compares
this complete maintained block. It does not verify remote release records.

[ess-source-support-begin]: #

The source checkout’s workspace version is `0.57.0` and includes separately documented unreleased changes.

| Capability | Current source | Limits and evidence |
|---|---|---|
| Default projections | `docs`, `site`, `schema`, `openapi`, `asyncapi` | Generator inventory and actual CLI artifacts; [ess-gen](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/lib.rs). `docs-ir` is an additional explicit choice. |
| Documentation | `docs`: Markdown with Mermaid diagrams | A projection of the document model; [docs emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/docs.rs) establishes rendering, not implementation behavior. |
| Site | `site`: HTML and local stylesheet/Mermaid assets; explicit output at `index.html` and `assets/`, combined output under `site/` | Explicit authored pages and downloads are supported; ESS does not host the site. [authored-site tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/authored_site.rs). |
| Document IR | Explicit `docs-ir`: `docs-ir/document.json`, `ess-docs/1` | A document projection, not HTML or a general persisted EssIr reader. [document emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/document.rs). |
| JSON Schema | `https://json-schema.org/draft/2020-12/schema` | Named types, entities, command inputs, events and errors; structural validation does not establish behavior. [schema emitter/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/schema.rs). |
| Native API projections | OpenAPI `3.1.0`; AsyncAPI `3.0.0` | Projection directions; [OpenAPI emitter/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/openapi.rs) and [AsyncAPI emitter/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/asyncapi.rs) own their structural coverage. |
| Adapter directions | `infra import`: `kubernetes`, `openapi`; `generate project`: `buildkit`, `helm`, `kubernetes`, `openapi` | Availability comes from CLI help. No AsyncAPI importer is declared; the following rows qualify each adapter. |
| OpenAPI adapter | Supported 3.0 and 3.1 service/interface import to `ess-openapi-import/1`, retaining source and accounting; checked projection | External references refuse. Semantic gaps, unresolved references and legacy interface-only inputs block checked projection; annotation normalization alone may be allowed. [accounting tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-openapi/tests/accounting.rs) and [import/refusal owner](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-openapi/src/lib.rs). |
| Kubernetes import | Sanitized observation bundle or explicit live context to infrastructure IR | The live scanner is the credential edge. A fixed category list and empty `coverage_gaps` do not prove complete observation. [import/redaction owner](https://github.com/beyond10x/ess/blob/main/crates/infra/ess-kubernetes/src/lib.rs). |
| Kubernetes projection | Intent plus observed IR to patches, new objects and obligations | No apply operation; unstated decisions remain obligations and unsupported conditions may refuse. [projection/refusal tests](https://github.com/beyond10x/ess/blob/main/crates/infra/infra-project/tests/projection.rs). |
| BuildKit and Helm projection | Checked build IR to Dockerfile/Bake inputs; runtime IR to a configuration-neutral Helm chart | These projections neither execute BuildKit nor apply a chart or establish live resource availability. [deployment projection tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/tests/deployment.rs). |
| Structural synthesis | `rust`, `go`, `web`, `clap` | Generated structure plus obligations/refusals; `rust` also generates the command behaviours and view queries the specification fully determines, over implementor-provided storage and context ports. All four full targets refuse modeled Binary64; separate structural data libraries have their own support boundary. [feasibility tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/feasibility.rs) and [synthesis guide](../guides/synthesize.md). |
| Clap synthesis | Command grammar, completion support and handler seams receiving `clap::ArgMatches`; generated `clap` and `clap_complete` 4 dependencies | No additional type layer or implemented command behavior. [Clap emitter](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/clap/mod.rs) and [handler tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/clap.rs). |
| Typed CLI presentation | `specify cli` validates `ess-cli/1` to `ess-cli-plan/1` and `ess-cli/2`, whose `config` and `output` globals are optional, to `ess-cli-plan/2`; `generate cli` emits a Rust/Clap package, help, Bash completion and reference with `ess-cli-artifacts/1` and `ess-cli-generation/1`; `--check` compares generated bytes | Typed inputs, results and declared errors remain model-owned; unsupported types and invariants refuse. Process context is separate from payloads. Application behavior requires `Handler`, dynamic native validation requires `DynamicValidator`, and the generated default handler is unavailable. [binding admission tests](https://github.com/beyond10x/ess/blob/main/crates/specify/ess-cli-contract/tests/binding.rs) and [projection and process tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-cli-project/tests/projection.rs). |
| Conformance targets | `billing`, `oracle-fixture`, `interpreted` | Built-in reference implementations. A production adapter must establish its own execution boundary; these targets do not prove independent deployment. |
| Conformance formats | Defaults: `ess-conformance/34`, the default report refused before execution without explicit `--report-format 2`. Explicit count surfaces: `ess-conformance-report/2`, `ess-conformance-run/2`. CLI suite choices: `4`, `5` (default `4`); report choices: `1`, `2` (default `1`). | Actual report markers and CLI metadata; all-pass legacy execution can still mean inconclusive conformance. [count-report tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/count_reports.rs). |
| Coverage qualification | Current-source `ess-conformance/35` requires explicit report/2 before execution | Only a nonempty all-pass selection with complete inventory and no in-scope refusal can qualify. Suite/5, carrier and paired replay were introduced in 0.21.0. [coverage CLI tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/coverage_cli.rs) and [conformance guide](../guides/verify/runners.md#opt-into-declared-coverage). |
| Mutation audit | `verify conform mutate` against `billing`, `oracle-fixture`, `interpreted`; classes `from-drop`, `transition-to`, `guard-boundary`, `sets-retarget`, `guard-negate`, `guard-connective`, `error-swap`, `emit-drop`, `order-flip`, `sets-drop`, `precedence-swap`, `emit-swap`; writes `ess-mutation-report/3`, or `/4` for a component, a known-failure declaration or an `emit-swap` site with no alternative | Mutates the specification, not the implementation, and runs no authored scenario: a survivor is a rule synthesis does not pin, answered by the model or a synthesis gap. [mutation audit tests](https://github.com/beyond10x/ess/blob/main/crates/verify/ess-conformance/tests/mutation_audit.rs) and [conformance guide](../guides/verify/mutation-audit.md#audit-the-suite-with-specification-mutants). |
| Browser conformance | Replay presentation with no execution report | A green replay is not independent execution evidence; digest comparison does not authenticate the publisher. [browser admission tests](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/tests/coverage_browser.rs). |
| Runtime compilation | Checks supplied identities, component coverage, replica bounds and stateful storage | Does not establish live provisioning or all resource requirements. [runtime checks/tests](https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/runtime.rs). |
| Explicit executors | `execute`, `publish`, `fetch`, `reconcile` invoke external clients; reconciliation requires `--authority` naming a protected registry entry and refuses without one, compares an admitted baseline desired deployment with the desired one, and attempts at most one admitted mutation per release | Caller-supplied state, authority and credentials remain material; a supplied baseline is admitted intent rather than proof of application, and a stopped invocation leaves the affected release unknown rather than absent or rolled back. [CLI executor owner](https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/src/main.rs). The support check invokes none of these verbs. |
| Schema commands | `import-bundle`, `import-document`, `project-bundle`, `validate-bundle`, `types-bundle`, `normalize-check`, `normalize-run`, `normalize-generate`, `validate`, `typescript` | Current-source command inventory for import, validation, types and normalization; [CLI reference](../reference/cli.md#adopter-owned-schema-contracts) describes the selected operations. Availability is independent of the dated release record. |

[ess-source-support-end]: #

The CLI presents five areas: `specify`, `generate`, `verify`, `infra`, and `ui`. Earlier flat
spellings remain hidden aliases with the same accepted-command output and exit status. Guidance for
an agent using ESS in another repository is the `ess` plugin in
[`beyond10x/agentplugins`](https://github.com/beyond10x/agentplugins).

Compilation and projection remain deterministic and offline. Live Kubernetes import and the
commands named `execute`, `publish`, `publish-conformance`, `fetch`, and `reconcile` are explicit
credential edges; they do not turn ESS into a continuously running deployment control plane.

The offline repository gate is `task check`. The documentation gate is `task site-build` because
installing the pinned npm dependency graph requires network access.

## Compatibility posture

Persisted v1 formats retain their current identity, reference, and canonicalization rules. Internal
Rust types do not by themselves create a new format. A strict old reader must be tested before an
additive field is written to a format that rejects unknown keys.
