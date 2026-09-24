---
format: aep.planning-md/1
id: task:runtime-adopter-gaps-9-13
kind: task
status: active
title: Deliver five additional measured runtime gaps before the backend retrofit
tags:
- priority-high
relations:
- decomposes: initiative:ess-evolution
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
- serves: vision:O2
revision: 3
---
## Outcome and standing authorization

Handle the five additional runtime-measured asks numbered 9 through 13. The user explicitly required new arrivals to be high priority and implemented with sub-agents in the same final ESS PR. These asks were missed when the earlier eight-gap snapshot was imported; their omission was not a lack of adopter evidence. Source intake is the latest primary task, preserved byte-for-byte in private evidence.

Gap 13 must be usable before the first area of the 36-operation agent API retrofit lands; no calendar date is stated. Both response-backed payload sources and refusal for incomplete payloads of genuinely emitting outcomes are required. Unproduced events with explicitly unknown causation remain distinct.

## The gaps, appended after the first conformance run

Four more, all found by **executing** the suite rather than by reading the model. That is the point worth
noting about them: none was visible to any gate, any reviewer or any of the six adversary passes that went
over this specification. They are appended on 2026-09-11, after the consumer's first conformance run against
its real server (64 of 340 passed, 275 skipped, 1 failed).

### 9. An error cannot declare its wire code

`RawErrorSpec` admits `fields`, `name` and `summary`. Every other externally visible declaration — command,
view, event — carries `naming.wire`. An error does not, and an error crosses the wire as a code string.

**Cost measured.** A conformance target must map the code a system returns back to the outcome that declares
it, and cannot. Snake-casing the ESS name reaches 24 of the consumer's 27 codes; the target carries a
hand-written exception table for `no_call` (two errors share it, told apart by message text) and resolves
`bad_request` by reading which branch is guarded on input. Without that guess, **117 `expect_error` steps are
unattributable**.

Second-order, and the reason this matters beyond tooling: the consumer types **eleven distinct refusals**
that all travel as one `bad_request`. The specification says they are different and the wire says they are
not, and nothing could compare the two, because one side of the comparison has no spelling in the model.

### 10. The string form of a predicate silently mis-parses a disjunction

`when: to == "" or text == ""` compiles to **one** comparison of `to` against the literal string
`" or text == "`. The empty-string quotes close and reopen; the string form has no infix `or`; and ess
**accepts** the result rather than refusing it.

**Cost measured.** The branch became unreachable by any honest input, `synthesize` built a candidate sending
`" or text == "` as the recipient, the server correctly accepted it, and the scenario reported the server as
wrong. It survived authoring, `validate`, `compile`, six adversary passes and a human review. It was found by
one conformance run.

The structured form (`any: [to == "", text == ""]`) is correct and compiles as a real disjunction, so this is
a diagnostic gap rather than a missing construct: a predicate whose remainder after a quoted literal contains
a bare `or`, `and` or `not` is far more likely a mis-written disjunction than a literal, and refusing it costs
nothing. A silently valid wrong parse is the worst outcome available here.

### 11. `ess specify compile` drops the view declarations

A domain's `views` in the compiled IR is a **list of names**. No `naming.wire`, no `source`, no `consistency`,
no fields — and there is no top-level `views` key either. Commands carry all of theirs.

**Cost measured.** A consumer cannot learn from the IR what answers a view, so the conformance target could
not tell an RPC-backed read from a field of the pushed snapshot. `task spec:conform` now derives that one map
out of the domain YAML with a Python one-liner, beside the IR the same task writes — a second reader of the
specification, which is exactly what compiling to an IR exists to prevent.

### 12. The conformance report counts a skip as a failure

`ess-conformance-report/1` carries `scenarios_total` and `scenarios_failed`, and no skipped count. A run of
340 with 275 skips and **one** genuine failure reports:

```
scenarios_failed: 276
failed_scenarios: 276 entries, 275 of them prefixed "skipped ", 1 prefixed "failed "
```

**Cost measured.** The only way to recover the real number is to parse a prefix out of a list of strings. Any
tool reading the numeric field — including `aep plan artifact evidence --from <report>`, which exists to take
the count from the report rather than from a person — records 276 failures where there is one. `status:
failed` is right (a skip makes a run inconclusive at best), but the count that sits beside it is not a count
of failures.

A `scenarios_skipped` field, and `scenarios_failed` meaning what it says, would settle it.

## Gap 13, from running the backend leg

### 13. An outcome's payload cannot read the command's response

A payload field takes `input.<field>` or a literal. There is no way to say that an event carries what the
call it describes **answered**.

**Cost measured**, 2026-09-11, on the consumer's first run of the backend leg. Its
`consumer.backend` domain models the 21 REST calls the server makes, and each one's success event
carries the response body the backend really sends — `WrapUpCancelled{item: WrapUp, message: String}` for
`DELETE /api/v2/agent/wrap-up`, which the controller answers with `{item, message}`
(`WrapupController.groovy:26-29`). The outcome that emits it can fill `message`, a literal, and **cannot
fill `item` at all**: the command has no input to derive it from, and the response is not addressable.

`ess specify validate` does not refuse the gap, so synthesis builds a scenario asserting a *shape* over
`item.callType`, `item.dateCreated`, `item.dateEnd` and `item.remaining`, and nothing can ever produce one.
**15 scenarios fail on exactly this**, and the specification is not wrong — the backend does send `item`.

It is a class, not an instance: **21 of the domain's events declare at least one field no emitting outcome
fills** — `item`, `items`, `success`, `message`, `call` — across every call that answers with a body.

Two things would settle it, and they are separable:

1. **A source for it**, so a payload can say the field comes from the response — the shape the model already
   has for `input.`, one step along.
2. **A refusal**, so an event field no emitting outcome fills is reported rather than silently synthesized
   into an unsatisfiable shape. That half is worth having even without the first: today the only signal is
   a conformance run failing on an assertion nobody wrote.

Any system whose events carry what a call returned has this, and a system that models an upstream leg at
all has it for every call in the leg.

**Reproduced independently, and the counts agree to the field name.** The specs-authority session
recomputed it from the compiled IR without taking the number: 21 events, with `item` 15, `success` 5,
`message` 4, `items` 1, `call` 1. Recomputed here the same way: identical, name for name.

**Two shapes a naive count would merge, and only one of them is this gap.** In the same domain, 11 events
are emitted by nothing at all — their producers are platform-event subscribers outside the retrofitted
subset, and the model marks the causation UNMAPPED on purpose. That is a documented absence. The 21 are
different: something *does* emit them, and the emitting outcome simply has no expression that fills a field
carrying the response. A refusal built for this gap must separate the two, or it will fire on the honest
UNMAPPED and be switched off.

**This stops being one consumer's problem shortly.** `epic:agent-api-retrofit` in `specs/services/backend`
moves the 36 `/api/v2/agent/*` operations to the project that owns them, one domain area at a time. That
project models exactly one command today — `backend.push.PushToAgents`, whose single outcome emits an event
mapped entirely from input — so it has never met this wall. Every write among the 36 answers with `{item}`,
`{success}` or `{message}`: the same five names measured above. **The first area that lands there hits this
on day one**, and its owner faces the same two options with no third: scenarios that cannot pass, or an
event that omits what the wire sends.

So the case is not "a consumer's synthesis is red". It is that the construct is missing for anyone typing a
REST write at all, and the project that owns these operations is about to be the one typing them.

**One thing that looks like cover and is not.** `consumer/scripts/check_push_types.py` reports
`11 event(s), 307 field(s) checked` and exits 0 throughout. It walks the `data` subtree of the 11 *push*
events, which have no emitting outcome by design, so it says nothing whatever about the 21. Neither side
should read its green line as evidence here.


## Acceptance

Each of gaps 9–13 has a scoped high-priority story, an executable reproduction from the measured case, and either a delivered correction or verified existing capability plus an actual adoption path. Missing adopter proof is not an excuse to skip an ask. Do not silently change legacy report/1 semantics, manufacture input fields for command responses, drop real views, or guess ambiguous wire errors.

Validate the adopter specification against the candidate source3/authoring2/report2 formats before claiming the original bindings fit. Keep the original primary and its staged work untouched. Public ESS source and new planning entries use generic examples and exclude private consumer identifiers.

## Source and publication

The original intake snapshot is local-evidence:priority-wave/runtime-gaps/original-thirteen-gaps.md; its SHA256 is 4dadcc0871e00fc98d33c5ea08fbeb46d29026b2f8bbf77c3ddab798e478e61d. All planning mutations use AEP. The existing Gates cumulative scan-size refusal on another journal version remains a publication constraint to resolve without weakening checks.

## Actual adopter constructs

Isolated adopter source3 and43 authored2 scenarios validate. A new real CallRecord/CallHistory setup scenario also admits with one backend-owned history row,0refusals. Attempted actual lead push mapping resolves event.data.body but refuses host_context.agent_id with unobservable_fact: host mappings require a periodic cause. This is an actual missing event-context contract, not absent adopter evidence. No fabricated event fields were introduced. The accessor story remains active with that delivery obligation visible.

Read-only source-grounding additionally found ClearWrapUp item is nullable at the real backend; caller declared it required. Migration work corrects that model fact and supplies typed response sources for all26missingfields across21events; no generated placeholder is being used to hide response provenance.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 e5457c10ab0614bc8b0f2d66bb65bfb670893136cb5e3f11970104878bce2313, retained as local-evidence:runtime-gaps/publication-replay/snapshots/e5457c10ab0614bc8b0f2d66bb65bfb670893136cb5e3f11970104878bce2313.md. Source creation recorded at 2026-09-11T04:53:15Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
