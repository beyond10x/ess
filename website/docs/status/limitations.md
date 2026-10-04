---
title: Limitations and trust assumptions
sidebar_position: 2
description: What ESS deliberately does not infer, apply, or attest.
lede: What ESS deliberately does not infer, apply, or attest.
---

# Limitations and trust assumptions

## Specifications and generation

- Import coverage is adapter-specific. Unsupported input is reported rather than placed in an
  arbitrary property bag. There is no AsyncAPI importer.
- Projection is not universal reversibility. Supported IR can round-trip semantically; concrete
  source formatting may normalize.
- Generated structural code contains obligations where behavior still needs an implementation.
  ESS never chooses between two implementations of one obligation.
- `EssIr` and `InfraIr` are intentionally separate until a concrete comparison requires otherwise.

## Conformance

- The built-in conformance targets (`billing`, `oracle-fixture`, `interpreted`) demonstrate the
  contract. A production adapter must establish its own independent execution and evidence
  boundary.
- An implementation outside this repository is held to its suite through the generated Go or
  TypeScript package. A Rust implementation needs a checkout of this repository, because the
  `ess-conformance` crate is not published. No runner reaches a target in another process.
- A scenario a target cannot answer is never a pass. The Rust runner fails the run; the generated
  Go and TypeScript packages report it as skipped and the run as `inconclusive`, while the test
  command itself can still exit 0. Read the report, not the exit code.
- `--target interpreted` runs a command's outcomes, transitions, `sets:`, events and invariants
  from the model. Views, bindings, time, redelivery and established entities come back
  `unsupported`.
- A mutation audit mutates the specification, not the implementation. Authored scenarios never
  kill a mutant, so a survivor is answered in the model or as a synthesis gap.
- The explorer models a subset of commands and input types; everything else is listed as excluded.
- A browser replay presents scenarios. It is not independent execution evidence.

## Delivery

- ESS is not a continuously running deployment control plane. Its explicit executor commands invoke
  local BuildKit, ORAS, and Helm clients; credentials and retry policy remain owned by the caller.
- Release evidence is checked for consistency only. Provenance is not verified SLSA, SBOM content is
  unverified, and signature verification is unsupported
  ([What release evidence establishes](../concepts/component-delivery.md#what-release-evidence-establishes)).
- `deployment reconcile` executes only under an authority an administrator provisions in a
  protected registry on the executing host. It supports one trusted execution host per cluster and
  ESS-generated charts only. The registry's completeness, the uniqueness of the host, and the
  absence of other writers are the administrator's assertions; ESS checks their concrete bindings
  and cannot discover them.
- A reconcile is a finite invocation, not a convergence promise. Once an external call may have
  started, a missing success record does not establish that nothing changed: the affected release's
  state is reported as **unknown**, later mutations stop, and no rollback is claimed or attempted.
  A supplied baseline document is admitted intent and never proof that it was applied.
- What a recovery observation covers is the declared content of the chart's own direct resources at
  the point in time it was read. It says nothing about Pod behaviour, readiness over time,
  controller-created descendants, Secret contents, or retained PersistentVolumeClaims, and a
  completed invocation is not a guarantee that anything stays as it was found.
- Kubernetes live access trusts the caller-selected cluster authority. Sanitization limits emitted
  data; it does not make an untrusted cluster safe to contact.

## Compatibility

- The project is pre-1.0. Public formats are still changed only through explicit versioned
  migrations, but Rust API compatibility is not yet promised across every minor release.
