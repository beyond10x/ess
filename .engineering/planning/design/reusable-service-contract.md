---
format: aep.planning-md/1
id: design:reusable-service-contract
kind: design
status: draft
title: Preserve complete compiler semantics in a selected ServiceIr
relations:
- designs: story:reusable-service-contract
revision: 3
---
## Design

docs/design/ess-evolution/service-contract.md defines the pure borrowed ServiceIr extraction
contract for story:reusable-service-contract. It preserves existing complete compiler values,
separates selected ownership/ports from definition context, validates the full synthesis plan,
retains ordered capabilities and explicit obligations/refusals, and introduces no persisted format.
Typed selection/index model: docs/design/models/service-contract/, validated by installed ESS
(service_contract v1 — 2 file(s), valid; exit 0). This is a proposed owner-local design, not an
implementation or acceptance claim. Local managed source and independent examination are tracked
under ess-evolution/waves/0009-service-convergence/.

## First concrete interface examination and correction

Original review-result:service-contract-design-pass-1 was recorded unchanged (report SHA
6a54e5f8c29b3b32b81e136ac258f7bd2807a3ca7fff0b41d087992003e493c4), with two introduced
design blockers. Root verified each against current compiler/synth source, then corrected the
owner-local design to SHA14ce1677ae0f201d0751e335e69eee91edbe4f8145080d9e4e03987991932159.

- Fixed incoming ownership closure: relations_carried_by exposes owners declared on foreign source
  entities whose owns-carrier is the selected child. Follow these incoming sources as well as
  outgoing targets to a finite fixpoint; do not promote them to local owned_entities. Required
  fixture asserts the foreign owner capability and carried relation with unchanged local surface.
- Fixed transport binding selection: source component publications and command emissions are
  independent. Select reacting bindings from exact component.publishes, retaining operation
  emissions only as event/outcome context when unpublished. Bindings invoking accepted operations
  remain independently selected. Require both asymmetric publish/emit fixtures and exact roster.

These are bounded corrections to approved M5 extraction. No ER/runtime semantics, persisted format
or wider completion requirement was added. The first reviewer closed its assignment and released
its own lease. The separately assigned second and final design examination now checks the corrected
contract, preserving this two-pass limit; source implementation is not yet claimed.

## Accepted concrete extraction interface

Final design SHA14ce1677ae0f201d0751e335e69eee91edbe4f8145080d9e4e03987991932159 is accepted
under the operator's approved ESS evolution step6 scope. review-result:service-contract-design-pass-2
holds original report53c7dfd744306af38e119d847c052ec6863939936368808ebd28dab2d8ee654d,
approve/findings[]: both first-pass blockers resolved, no new findings. Both design assignments
closed and their own leases were released; no third design round. This accepts a design, not code.

The pure source assignment owns only crates/specify/ess-service-contract/, workspace member and
dependency entries in Cargo.toml, and necessary Cargo.lock resolution. Root retains design/model,
CHANGELOG and sole canonical planning-store writes. A separate managed source tree avoids concurrent
Cargo/lock writes with the existing preservation tree; final integration is serialized and full
combined-source consumer coverage remains mandatory. No copied planning journal is merged.

Implementation contract: local-evidence:ess-evolution/waves/0009-service-convergence/service-contract-implementation-brief.md.
Test-first pure extraction, exact design fixture assertions, crate tests/Clippy/fmt/Rust1.85 and
two extraction mutation failures/restoration precede fixed author handoff and closure. Root owns
source examinations, full task check/site-build and integration. No ER/SDK/Eventlog work or later
service acceptance is delegated to this assignment. Additional work needs a separately justified
assignment after closure, never an extension to keep a worker busy.
