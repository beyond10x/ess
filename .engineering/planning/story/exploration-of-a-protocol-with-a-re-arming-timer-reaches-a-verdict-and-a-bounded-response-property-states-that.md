---
format: aep.planning-md/3
id: story:exploration-of-a-protocol-with-a-re-arming-timer-reaches-a-verdict-and-a-bounded-response-property-states-that
kind: story
status: draft
title: Exploration of a protocol with a re-arming timer reaches a verdict, and a bounded-response property states that one event follows another within a time
summary: A protocol with a re-arming timer explores to inconclusive because configurations hold absolute time, and no property states that one event follows another within a time (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

A protocol has a step that repeats for as long as a condition holds: a heartbeat every 5 s while a
stream is open, a retry with backoff and no attempt limit, a reconciliation pass every 20 s with no
end. Two facts must be checkable: that exploration covers the repeating model and reaches a verdict,
and that one event is answered by another within a time (a bounded response).

Reproduction, validated and explored with the installed `ess` (a heartbeat server: `open` arms
`beat` at 5 000 ms, the `beat` transition emits `heartbeat` and re-arms `beat` at 5 000 ms, `close`
cancels it; bounds `max_steps: 40, max_states: 512, max_time_ms: 30000`):

```console
$ ess specify protocol validate --path heartbeat.yaml     # valid ess-protospec/1, no cap on repeats
$ ess verify protocol explore --path heartbeat.yaml
{ "time_policy": "inputs_at_initial_time_and_timer_deadlines", "verdict": "inconclusive",
  "reason": "logical time bound exhausted", "explored_states": 27, ... }
```

So a periodic or unbounded timer is already expressible. A self-re-arming timer and a `Backoff` that
sits at its ceiling are valid (`Duration::Fixed | Backoff`, `crates/specify/ess-domain/src/protocol.rs:228`).
What is missing is a verdict:

- A configuration holds the absolute clock and every timer's absolute deadline (`state.now_ms`,
  `t.deadline_ms`, `crates/verify/ess-conformance/src/protocol/engine.rs:222-240`), and exploration
  closes a branch only when `seen.contains(&step.configuration)` (`monitor.rs:230`). A cycle of the
  model never returns to a configuration it has seen, so the run ends on `max_time_ms` or
  `max_states`, and "exhaustion never constitutes a passing result" (`protocol.rs:275`).
- The property vocabulary has no timed property: `NeverState`, `EventCount`, `FlushBeforeClose`,
  `StateImplication` (`protocol.rs:287`, "unbounded liveness is deliberately unsupported").
- The reporter's workaround, a cap of 2 or 3 repeats stated in the model, turns a periodic fact into a
  finite one and checks a different model.

Reporter's proposal (theirs): a declared periodic or unbounded timer, and a bounded response
`within: <ms>` from one event to another.

## Outcome

Exploration of a model whose timers re-arm reaches `complete` when every reachable configuration, with
time taken relative to now, has been seen, or `inconclusive` only when a bound is truly exhausted. A
new property `bounded_response` states that after one event or state the named other event or state
follows within a number of milliseconds, and it is checked over every explored path, including around
the cycle. Unbounded liveness stays unsupported: the property always names its bound.

## Acceptance

- `ess verify protocol explore` on the heartbeat reproduction above returns `verdict: complete` within
  its declared `max_states`; before, `inconclusive`. A model whose configurations really exceed
  `max_states` is still `inconclusive` (a test pins it), and an exhausted bound is never a pass.
- The committed example protocols keep their digests and their explore verdicts.
- A property `{kind: bounded_response, name, participant, after: <event or state>, then: <event or
  state>, within_ms: N}` validates; each of these is refused with a string code and its path: a name that
  is neither an event nor a state of the participant, `within_ms` of 0, and a `then` the participant can
  never reach.
- The heartbeat model with `within_ms: 5000` from `open` to the first `heartbeat` passes; with
  `within_ms: 4999` it fails and the verdict carries the counterexample trace. A retry model whose
  backoff ceiling is above the bound fails the same way.
- A recorded target run is checked against the property by the monitor (`check`), and a late reply fails.
- Each test runs within 60 s and each test step within 120 s; exploration in tests stays inside the
  examples' declared bounds.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`Property` 287), `crates/specify/ess-compiler/src/protocol.rs`
  (property compile), `crates/verify/ess-conformance/src/protocol/engine.rs` (`Configuration`),
  `monitor.rs` (`check`, `explore` 163-248), `tests.rs`, `docs/design/protocol-specification.md`.
- Size: M, about 500 changed lines. Edits the same three crates as the other open `ess-protospec/1` stories, so it
  follows them one at a time.

## Fit review

1. **Need.** A model of an endless periodic behaviour must be explorable and a timed response
   checkable. Reproduction above; the reporter's syntax is labelled theirs.
2. **Class: gap** (verdict and timed property); the timer itself is expressible.
3. **Already expressible?** The timer: yes (probe above). The verdict: no. Capping the repeats is no
   answer under the fix-by-idiom test: it checks a model the reader does not see.
4. **Fit.** Protocol kind only. Both changes keep the checker finite (relative-time configurations,
   an explicit `within_ms`), so the document's rule that exhaustion is no pass stays.
5. **Second adopter.** A lease renewed every 10 s while held, with "a release follows an expiry
   within 15 s".
6. **Cost.** One property kind, about three codes, a changed configuration identity. Format per PR-D3
   (`ess-protospec/1` additive, default A).
7. **Alternatives.** Change nothing (cap the repeats); unbounded liveness (rejected at `protocol.rs:287`);
   a `periodic:` timer key (no gain, the re-arming transition already says it).

## Decisions

AC9-D3 (ARCHITECTURE only under B): the bounded-response property. A: add it to `ess-protospec/1`, additive, as PR-D3 default A. B: a new `ess-protospec/2`. C: decline; the cap idiom stays. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as redesigned (no periodic key; closure and one property). If the implementation finds it cannot be additive in `ess-protospec/1`, the story stops and the decision returns as an architecture decision.
