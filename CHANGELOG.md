# Changelog

## [0.2.0] — 2026-09-01

- Make `ess-conformance-report/1` the only ESS conformance handoff and remove the legacy
  workflow-shaped evidence, producer, provenance, and verifier API. AEP adaptation now lives only
  in AEP's optional adapter.
- Use the canonical `ess` command in newly generated provenance guidance and current engineering
  documentation. Persisted ESS and infrastructure IR envelopes remain version 1.

## [0.1.1] — 2026-09-01

- Publish the extracted ESS adopter documentation and browser lab at the standalone ESS Pages
  site, with commands, links, and regeneration guidance written for the canonical `ess` CLI.

## [0.1.0] — 2026-09-01

- Extract ESS, infrastructure modeling, schema contracts, generators, conformance, examples, and
  suites into a standalone repository with no AEP dependency.
- Add the canonical `ess` command and explicit import/project adapter contract.
- Add typed OpenAPI service-interface import, deterministic projection, coverage reporting, and
  semantic round-trip checks for the adapter's declared subset.
- Import the Kubernetes credential-edge scanner with pre-write Secret sanitization.
- Publish `ess-conformance-report/1` for optional workflow-side evidence adapters.
