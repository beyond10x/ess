---
format: aep.planning-md/3
id: story:feature-request-297
kind: story
status: draft
title: Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#297
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

Resolve beyond10x/ess#297: Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected.

## Origin

beyond10x/ess#297, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 8d, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Confirmed missing observation primitive. docs/design/concurrent-history-conformance.md decision6 explicitly excludes restart because no specification, realization or ConformanceTarget seam declares it. Current target.rs exposes begin/end scenario but no process restart; scenario/history vocabulary contains no restart action. Treating EndScenario/BeginScenario as restart would change isolation and erase the very durable state the requested counterexample needs.

The request needs a reviewed capability and history contract before implementation: distinguish restarting a process while preserving durable state from resetting a scenario; establish process incarnation and completion/failure evidence; decide which creating identities must remain unique against retained rows and how reads witness persistence; define unarrangeable identities/views and unavailable target capability; preserve model state across the restart step in explorers. All runtimes must execute the admitted primitive consistently, with unknown/older formats refusing it. A counter-reset mutant plus healthy durable issuer must distinguish this from an ordinary two-create test, including timeout/failed restart and no-capability controls. Do not allocate a new suite/history major until that contract is closed and coordinated with389.

Issue389 explicitly reports restart/concurrency as intrinsic unsupported coverage today; that disclosure is not delivery of297. This remains planned capability work with a binding-design prerequisite. Source audit only; own new test/build executions0.
