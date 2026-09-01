---
title: Limitations and trust assumptions
description: What ESS deliberately does not infer, apply, or attest.
---

# Limitations and trust assumptions

- ESS describes and checks systems; it is not a deployment engine or control plane.
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
