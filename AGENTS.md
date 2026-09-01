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

The adopter-facing Docusaurus site lives under `website/`; repository-root `docs/` remains the
engineering record and is never published directly. A documentation change, release, or Pages
change must additionally pass:

```console
task site-build
```

This check is separate because `npm ci` fetches the exact public `docs-system` Git revision and
therefore cannot be part of the offline gate. Pages publishes only `website/build/`.

## Commits

- Use conventional prefixes and a body explaining what changed and why.
- Use organization bot tooling outside this public repository for commits and pushes.
- Never commit credentials, tokens, kubeconfigs, or unsanitized observations.
