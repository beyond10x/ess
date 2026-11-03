---
format: aep.planning-md/3
id: story:a-union-variant-scenario-reads-each-row-under-its-own-identity
kind: story
status: active
title: A union-variant scenario reads each row under its own identity
summary: An observed identity resolves to the first event, not the latest, so the second variant row is read under the first id; reporter repro union-identity.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:56Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:56Z", actor: "human:timo", revision: 8}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

A scenario that covers each union variant in turn creates two rows and reads the second under the first event's observed identity: `{"kind":"observed"}` resolves to the first event, not the latest. Reporter repro `union-identity` (1 scenario, fails): the view check expects `box_id = …0001, kind = Plain` where the target correctly holds the second row under its own id.

Reproduced from the reporter's synthesized suite and run report; not re-run against a target here.

## Acceptance

- `a_second_variant_row_is_read_under_its_own_identity`: the neutral repro as a fixture; each variant's view expectation names the identity of the event that created that row, and the reference target passes.
- An observed-identity reference that is ambiguous between two events is resolved to the latest or refused by name; it never silently binds to the first.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (view-row synthesis, and the three runners' resolution of `{"kind":"observed"}`) — cited.
- **Files (resolve):** `src/runner.rs:3426-3432` uses `.find` over `seen` (first occurrence); `runner.rs:3475-3487` does the same for `ObservedSelection` and `ObservedAccessor`; `src/go/runtime.go:3766-3779` and `src/ts/runtime.ts:5941-5956` each say "the first occurrence in it" — cited.
- **Files (emit):** `src/synthesize.rs:13726-13738` (`identity_of` emits `ScenarioValue::observed` whenever no instance is bound); `synthesize.rs:4917-4931` (the capture gate `if related || captured || rows`, so a plain outcome witness captures nothing) — cited.
- **Symbols:** `ScenarioValue::Observed`, `identity_of`, `minted_row` (`synthesize.rs:11923`), `capture_instance` (`runner.rs:2446`, reads `run.last_command`, unambiguous) — cited.
- **Also likely:** `src/synthesize/identity.rs:49-53` (token text `observed:<event>.<field>`; two refs to one `event.field` alias one token); `src/synthesize/caller.rs:1831-1890` (`recaptured`) — inferred.
- **Format:** `NEWEST_ADMITTED_SUITE_MAJOR = 63` (`src/go/mod.rs:429`); a synthesis-only fix (capture) stays inside `/63`; changing the runners' meaning of `observed` for existing suites needs a new suite major — inferred.
- **Confidence:** medium — the defect sites match the story's repro; the reporter's suite is not in the tree and nothing was run.
- **Would collide with:** any unit touching view-row synthesis in `synthesize.rs` (`identity_of`, `minted_row`, the capture gate), and any unit changing `observed` resolution in `runner.rs`, `go/runtime.go` or `ts/runtime.ts` — inferred.
- **Safety fact:** the capture gate is the only place a plain outcome witness binds an instance (`synthesize.rs:4917-4931`) — cited, unproven at runtime.
- **Size:** medium, about 150-200 lines; large if the runners' resolution changes for `/63` suites — inferred.
- **ess-conformance targets:** `captured_identities`, `minted_value_sources`, `view_filter_identity`, `union_unit_variants_go`, plus a new test file — inferred.

## Schedule (D401, 2026-11-02)

Left wave W1 (orchestrator D401). Bullet 2 needs the runners' meaning of `observed` to change, so a
new suite major; the story joins the W2 suite-major unit beside `story:a-view-read-checks-every-rows-shape`
(#680), `story:expect-error-checks-generated-field-presence` and
`story:view-and-error-assertions-carry-their-value-shape`. The SYN1 patch for bullet 1 is parked in
the coordinator scratch and rides with that unit.
