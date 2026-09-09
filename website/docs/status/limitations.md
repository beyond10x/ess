---
title: Limitations and trust assumptions
description: What ESS deliberately does not infer, apply, or attest.
---

# Limitations and trust assumptions

- ESS is not a continuously running deployment control plane. Its explicit executor commands invoke
  local BuildKit, ORAS, and Helm clients; credentials and retry policy remain owned by the caller.
- A reconcile is a finite invocation, not a convergence promise. Once an external call may have
  started, a missing success record does not establish that nothing changed: the affected release's
  state is reported as **unknown**, later mutations stop, and no rollback is claimed or attempted.
  A supplied baseline document is admitted intent and never proof that it was applied.
- What a recovery observation covers is the declared content of the chart's own direct resources at
  the point in time it was read. It says nothing about Pod behaviour, readiness over time,
  controller-created descendants, Secret contents, or retained PersistentVolumeClaims, and a
  completed invocation is not a guarantee that anything stays as it was found.
- Import coverage is adapter-specific. Unsupported input is reported rather than placed in an
  arbitrary property bag.
- Projection is not universal reversibility. Supported IR can round-trip semantically; concrete
  source formatting may normalize.
- Generated structural code contains obligations where behavior still needs an implementation.
- The built-in conformance targets demonstrate the contract. A production adapter must establish
  its own independent execution and evidence boundary.
- Kubernetes live access trusts the caller-selected cluster authority. Sanitization limits emitted
  data; it does not make an untrusted cluster safe to contact.
- `EssIr` and `InfraIr` are intentionally separate until a concrete comparison requires otherwise.
- The project is pre-1.0. Public formats are still changed only through explicit versioned
  migrations, but Rust API compatibility is not yet promised across every minor release.
