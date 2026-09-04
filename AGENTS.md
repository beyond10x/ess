# AGENTS.md — ess

The contract for changing this repository. Organization-wide rules live in `atlas/AGENTS.md`.

## Serves

- **O2 — decisions as data, with evidence.** ESS makes system structure, semantics, projections,
  conformance requirements, and infrastructure intent executable as deterministic typed data.

## Boundaries

- ESS has no AEP dependency. Workflow evidence adaptation belongs on the AEP side of the seam.
- `EssIr` and `InfraIr` remain separate until a concrete comparison requires a shared envelope.
- Do not introduce a generic facet registry, arbitrary JSON property bags, or an `ess-ir/2` plan.
- Add concrete Rust types only when an importer or projector establishes their required semantics.
- Imports never guess. Projections never apply infrastructure or mutate an external system.
- Each adapter declares supported directions and reports coverage gaps, obligations, unresolved
  references, and refusals.
- Anything executable is Rust. Do not add Python or shell checkers.

## Crate tree

`crates/<area>/<crate>/`. The area is the directory only; a crate's name is its identity and is
never changed to follow a move.

- **`crates/specify/`** — `ess-primitives`, `ess-domain`, `ess-compiler`, `ess-composition`,
  `ess-realization`: an authored system becomes a validated, resolved IR.
- **`crates/generate/`** — `ess-gen`, `ess-synth`, `ess-openapi`, `schema-contract`,
  `ess-deployment`: that IR becomes artifacts, and nothing here applies one to a running system.
- **`crates/verify/`** — `ess-conformance`, `ess-diff`: an implementation held to the
  specification, and one revision of a specification against another.
- **`crates/infra/`** — `infra-domain`, `infra-compiler`, `infra-analyze`, `infra-spec`,
  `infra-project`, `ess-kubernetes`: the observed cluster, a separate bounded context whose only
  dependency on the rest is `ess-primitives`.
- **`crates/edge/`** — `ess-cli`, `ess-xtask`: the `ess` binary an adopter runs, and this
  repository's own tooling.

`ess-deployment` sits under `generate/` rather than `infra/` because its dependencies are
`ess-compiler` and `ess-realization` and it has no `infra-*` dependency at all — an environment
lowered to a deployment document is a projection of the specification, not an observation of a
cluster.

## Determinism and formats

- Ordered collections only in persisted or generated data.
- Compiler-minted handles must have total lookup functions.
- Preserve canonical bytes unless a coordinated format migration explicitly changes them.
- A new format version is required when meaning, identity, references, canonicalization, names, or
  the persisted envelope changes. Internal Rust capabilities alone do not require one.
- `infra-ir/1` rejects unknown fields. Before adding any persisted field, add an old-reader
  compatibility test and decide the format consequence explicitly.

## Kubernetes credential edge

- Raw Secret `data`, `stringData`, and last-applied configuration values never reach serialized
  bytes or disk. Preserve the redaction tests and verify the guard by mutation.
- Live-cluster tests require explicit credentials and remain outside the offline gate.
- The libraries downstream of `ess-kubernetes` never select kubeconfig authority or reach a cluster.

## Gate

```console
task check
```

The gate is offline and runs formatting, strict Clippy, all workspace tests, rustdoc, command smoke
tests, and the dependency boundary test. Land nothing until it exits zero.

The adopter-facing Docusaurus source lives under `website/`; repository-root `docs/` remains the
engineering record and is never published directly. A documentation, release, or validation
workflow change must additionally pass:

```console
task site-build
```

This check is separate because `npm ci` fetches the exact public `docs-system` Git revision and
therefore cannot be part of the offline gate. `.github/workflows/pages.yml` preserves the same
Rust/WASM, browser-lab and site-build checks without Pages authority; the unified Website publishes
the collected source and the Atlas-generated façade owns the project redirect.

Cutting a release pushes a tag, and the release workflow gates the tag only after it exists. Run
`task check` and `task site-build` on the commit being tagged before pushing the tag: 0.5.0 was
tagged eleven minutes after `task site-build` had already gone red on `main`, its release was never
published, and no workflow can withdraw a tag here. After cutting a release, and after any release
run that fails:

```console
task release-status
```

It asks the remote and GitHub whether every pushed version tag is on `origin/main` and has a
release behind it, and fails while one is not. `.github/workflows/release-record.yml` runs it after
every release run and daily.

**Merge the branch, then tag.** `0.10.0` was tagged on `feat/outcome-sets-entity-fields`, published,
and `main` did not have a line of it — every other check reads the workspace and the remote's tag
list, and neither says which line a commit is on. AEP hit the same shape one version later and cut a
*newer* release that silently dropped the older one's features.

## Where work is tracked

| What | Where |
|---|---|
| The plan: epics, stories, blockers | `.engineering/planning/`, mutated only through `aep artifact` — never by editing a store file, never by writing `status:` |
| The protocol tree the store obeys | `.engineering/project.yaml` (a pinned `git+…#<40-hex>` source) |
| Binding designs | `docs/design/` — a construct is a design page before it is code |
| What shipped | `CHANGELOG.md`, and `git tag -n99` |

## Commits

- Use conventional prefixes and a body explaining what changed and why.
- Use organization bot tooling outside this public repository for commits and pushes.
- Never commit credentials, tokens, kubeconfigs, or unsanitized observations.

<!-- b10x-docs-operations:start -->
## Public documentation operations

This repository owns the public source and presentation allowlist in `b10x.docs.yaml`. The generated credential-free `.github/workflows/b10x-docs-bundle.yml` passively packages only those declared files for the exact successful `main` commit; it must never run repository code. Atlas selects the latest successful bundle with every other catalog source, and Website plus Docs System own rendering, shared components, search, and feeds. Do not add a standalone docs deployer or put App credentials in this public repository. If Atlas catalogs a former Pages workflow, that file remains repository-owned validation: preserve its bespoke checks while keeping exact read-only permissions, an unconditional pull-request trigger, and no deployment primitives. Project Pages at `/ess/` is only the generated stable redirect façade in `.github/workflows/b10x-docs-pages.yml`; content-only publication never rebuilds it.

From the complete organization workspace, verify the contract with a clean Atlas checkout at the current remote `main`. Set `B10X_ATLAS_CHECKOUT` to a managed Atlas worktree when the primary checkout is dirty or stale; never infer command availability from the primary alone.

```bash
atlas_checkout="${B10X_ATLAS_CHECKOUT:-atlas}"
atlas_head="$(git -C "$atlas_checkout" rev-parse HEAD)"
atlas_main="$(git -C "$atlas_checkout" ls-remote origin refs/heads/main | awk '{print $1}')"
test -z "$(git -C "$atlas_checkout" status --porcelain)"
test "$atlas_head" = "$atlas_main"
cargo run --manifest-path "$atlas_checkout/Cargo.toml" --locked -q -- \
  --store "$atlas_checkout/catalog/store" docs reconcile --workspace . --check
```

Keep internal plans, stories, ADRs, decisions, worklogs, security material, and research out of the public allowlist unless a repository authority explicitly declares them public.
<!-- b10x-docs-operations:end -->
