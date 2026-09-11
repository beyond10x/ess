---
format: aep.planning-md/1
id: story:closed-enum-outcome-coverage
kind: story
status: implemented
title: Align closed-enum outcome coverage with reachable synthesis witnesses
tags:
- priority-high
relations:
- decomposes: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: inferred
  path: docs/design/closed-enum-outcome-coverage.md
revision: 6
---
## Outcome and priority

Immediate correctness priority for gap 4. A supported complete set of closed-enum equality branches validates and receives actual synthesis witnesses without a fictional default or extra producer variant.

## Established mismatch

ess-domain/src/command.rs:1457 counts unconditional outcomes and refuses when there are none; it does not inspect enum domains or guard truth tables. ess-conformance/src/synthesize.rs:1789 seeks a default input refuting all guards, while witness.rs:329 enumerates only real declared variants. Therefore the narrow fully guarded six-value case has a genuine mismatch. Current predicate evaluation is three-valued (input.rs:208): Unknown and general candidate exhaustion are not proof of completeness or impossibility.

## Acceptance

This story is complete when the two measured six-value command shapes move from the contradictory coverage/reachability refusals to validated commands with executable witnesses for every real branch, while an omitted value still produces a concrete coverage failure and all proof-boundary checks below pass.

Required verification:

- Reproduce the two consumer report-command shapes with exactly their six declared values. Complete reachable equality guards need no unconditional branch; a missing variant reports a concrete uncovered value. Overlapping/duplicate guards do not silently prove exactly-one selection.
- Define and review a bounded finite-domain proof fragment using existing named-enum/wrapper/literal type authority. Open/unsupported domains retain conservative requirements; Optional absence/null, additional-input conjunctions and Unknown remain explicitly accounted for.
- Validator acceptance and synthesis share consistent coverage/witness semantics for that fragment. Synthesize a witness for every reachable declared branch, with no artificial-default refusal or invented seventh variant.
- Retained genuinely unreachable branches still receive honest accounting; no blanket unreachable exemption, suppressed refusal or weakened assertion.
- Preserve commands with real defaults and current external/wrong-state behavior. The separate question of held-subject-state behavior is not settled by making these input branches exhaustive.

## Scheduling

No functional dependency on accessor syntax or authored setup. Its domain/synthesis edits collide with active accessor and later subject-state work. It may run alongside compact writer work only after scope preflight confirms the exact listed paths remain disjoint.

## Historical source

Archived argument: story:exhaustiveness-and-reachability-disagree. Preserved original snapshot and the corrections above define the new unit.


## Provenance and delivery

Decomposes task:ess-gaps-measured-in-a-consumer-specification under initiative:ess-evolution. Prioritized by the operator on 2026-09-11. Source-only scoping at ESS dcdc3343 and observed consumer commit 2497faf27959b59b6eb0700829bb321f851adf1c is retained at local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/state-views.md. Original archived argument bytes/hashes remain under gap-scoping/source. Counts and historical test results are inherited evidence, not newly executed checks. Archived source entries stay terminal; this distinct unit records the maintainer's assessed implementation contract. Overlapping accessor source edits must be integrated or explicitly isolated before dispatch; that collision does not require completion of its downstream adoption acceptance. No full local/ownership gate or unchanged test reruns. Consumer-only PR pipeline failures are accepted; core feature evidence remains required.

## Scope

- Cited: crates/specify/ess-domain/src/command.rs.
- Cited: crates/verify/ess-conformance/src/synthesize.rs.
- Cited: crates/verify/ess-conformance/src/witness.rs.
- Cited: crates/verify/ess-conformance/tests/synthesis.rs.
- Inferred: crates/specify/ess-domain/src/expression.rs.
- Inferred: docs/design/closed-enum-outcome-coverage.md.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 e65aeb9309ebc1d693da97dc26dd2a8d3ffa6cd929f8bdb81353b029dba51f57, retained as local-evidence:runtime-gaps/publication-replay/snapshots/e65aeb9309ebc1d693da97dc26dd2a8d3ffa6cd929f8bdb81353b029dba51f57.md. Source creation recorded at 2026-09-11T00:26:23Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
