---
format: aep.planning-md/1
id: task:deferred-protocol-ui-bindings
kind: task
status: draft
title: Deferred protocol and UI capabilities outside ESS evolution completion
relations:
- serves: vision:O2
revision: 1
---
## Deferred follow-up

Generic protobuf, typed UI and Flutter capabilities from the earlier ESS evolution proposal remain future work outside initiative:ess-evolution completion. The operator explicitly chose this separation on 2026-09-15 in plan ess-evolution-20260915 revision 1. This draft records preservation of the work, not approval to dispatch implementation now.

## Preserved requirements

Protocol imports produce reviewable candidates, source references, coverage accounting and unresolved questions. Existing protobuf stays authoritative until descriptor, wire and independent consumer compatibility pass. Preserve names, field numbers, reserved fields, presence, oneofs, enum/JSON behavior, streaming and HTTP annotations. Code generation uses explicit admitted bindings and existing language tooling.

Future UI composition retains existing themes, translations, widgets and SDK contracts, and handles loading, empty, failure, permission and reconnect states. Generate complete journeys incrementally; keep local navigation/drafts separate from authoritative service state. Define the domain and typed references before decomposing implementation. Independent tests derive expected behavior upstream of lowering, use versioned opt-in observations/emitters, and detect misbound actions and incorrect state updates.

## Scheduling and exclusions

This record has no decomposes or depends_on edge making it a prerequisite for the current ESS evolution initiative. It does not restore either excluded application or fake-backend adoption. Select actual adopters and acceptance targets in a future approved plan. Existing supported features and tests remain preserved now.
