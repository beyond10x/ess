---
format: aep.planning-md/3
id: story:a-periodic-binding-may-run-on-a-wall-clock-schedule
kind: story
status: draft
title: A periodic binding may run on a wall-clock schedule
summary: 'The periodic cause admits only a whole-second every: anchored at host activation, so cron:, PT1H and a wall-clock anchor are refused (adopter item 17, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (recheck batch item 17, the second team's L10, relayed by the orchestrator; adopter names
kept out). A job runs at a wall-clock time (on the hour, nightly at 02:00), not every N seconds after the host started. The
periodic binding cause (format ess/3) admits one profile: a whole-second `every:` anchored at host activation. A wall-clock
schedule cannot be declared, and `every:` refuses ISO-8601 durations other than `PT<seconds>S`.

Needed: a periodic cause anchored on the wall clock (a calendar or cron-like schedule with a declared time zone), with the
conformance observation that a controlled target clock crossing the boundary invokes once; or a recorded decision that
wall-clock scheduling stays a deployment concern (the 0.62.0 workload `schedule:` is deployment only), with the idiom named.
Secondary: `every:` could admit `PT1H` and `PT5M` beside `PT3600S`.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: a `Sweep(worker)` command emitting `Swept`, component `sweeper`,
and a binding `when: {periodic: {SCHEDULE, anchor: host_activation, first: after_period, cadence: fixed_rate, overlap:
serial_per_instance, missed: one_pending_drop_excess, lifetime: host_instance, cancellation: stop_acknowledged, host: {...}}}`
invoking `Sweep`. `ess specify validate --path spec.yaml`, identical output on 0.68.0 and 0.67.0:

| SCHEDULE / anchor | result |
|---|---|
| `every: PT3600S` (control) | valid; interpreted conformance 5 scenarios, 1 passed, 4 unsupported ("target has no bound typed periodic host fixture"), both versions, byte-identical suites |
| `every: PT1H` | `invalid elapsed time reference "PT1H": is not written `PT…S`; a length of time is written `PT<seconds>S`, in whole seconds` |
| `cron: "0 * * * *"` | `unknown field `cron`, expected one of `every`, `anchor`, `first`, `cadence`, `overlap`, `missed`, `lifetime`, `cancellation`, `host`` |
| `every: PT3600S`, `anchor: wall_clock` | `unknown variant `wall_clock`, expected `host_activation`` |

Same refusal for `cron:` with `format: ess/30` on 0.68.0.

## Where it lives

- `crates/specify/ess-domain/src/binding/periodic.rs:250` (`PeriodicCause`, `deny_unknown_fields`, "the only initially
  admitted periodic profile"); `Anchor` admits `HostActivation` only (`periodic.rs:117`).
- `crates/specify/ess-primitives/src/time.rs:711` (`parse_elapsed_seconds`): `PT<digits>S` only.
- `story:periodic-binding-trigger-contract` (implemented) lists cron and calendars as not implied.

## Regression status

Not a 0.68.0 regression: identical refusals on 0.67.0 and 0.68.0. Gap. Does not block 0.68.0.
