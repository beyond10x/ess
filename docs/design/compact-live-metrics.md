# Live event captures and exact integer observations

This extends compact authoring with `ess-scenario/4`, ordinary suite/12 and
inventory suite/13. Earlier source and suite formats retain their bytes and
meaning. New steps in an older envelope are refused before target invocation.
The source manifest and report envelopes keep their existing meaning.

An `observe` action may name an actual occurrence with `anchor`, capture declared
payload fields, and assert integer bounds on that occurrence:

```yaml
- do: observe
  service: monitor
  event: monitor.metrics.Status
  matches: {status: live}
  anchor: first-live
  integers:
    catchup_ms: {min: 0, max: 10000}
```

The matcher selects the first live status. Bounds assert on that exact occurrence;
a later fast status cannot hide a slow first one. Integer assertions require an
actual present value even when its declaration is optional.

Capture syntax is `capture: {baseline: {path: waiting}}`. An optional field also
requires `require_present: true`; missing or null is never a captured zero.
The compiler retains the original event owner, declared shape, payload path and
actual occurrence identity. Captures have unique names and cannot be referenced
before binding. Captured values may feed later event matches and requirements.
Recipe arguments and command inputs retain their existing response-capture
vocabulary; this extension does not add event-derived fixture inputs.

An exact offset is `{$offset: {capture: baseline, plus: 3}}` in a matcher or
requirement. It lowers to one closed native integer operation over an earlier
typed capture, followed by the ordinary payload comparison. The evaluator binds
the derived value; the adapter never calculates a verdict or expected count.
Both operands and the result must fit signed 64-bit integers. Missing,
fractional and out-of-range values, or checked-add overflow, fail explicitly.
Rust and generated Go preserve exact integers, including values above 2^53.

A `quiet` action carries `service`, `event`, `matches`, `anchor` and positive
`quiet_for_ms`, plus optional captures and bounds. On activation, the native
evaluator requests a fresh complete observation batch and records its watermark
as the earliest start of the quiet interval. It selects the latest actual scoped
snapshot, waits a full interval after both that snapshot and the fresh fence,
and resets the candidate when a newer scoped snapshot arrives. A quiet interval
completed before setup cannot be reused. Without any scoped snapshot, a complete
watermark, or a healthy source there is no baseline. The scenario deadline bounds
continuous churn.

The persisted additions are `capture`, `offset`, `integer_bounds` and `quiet`
checks. Existing stability checks consume the derived offset binding and still
inspect every scoped frame. Claimed fields cannot filter a stability scope.
All new capture and anchor references are admitted before setup, and both
evaluators enforce the same type, presence, identity and interval rules.

Required counterexamples include a slow first status followed by a fast one;
missing/null/fractional/out-of-range integers; forward, duplicate or mistyped
captures; an offset overflow; a nonzero baseline; a late quiet-period reset;
reuse of a pre-setup quiet interval; transient count violations; and incomplete,
cancelled or malformed observation lifetimes. Run identical suites and batches
through Rust and generated Go, and preserve legacy-format canonical fixtures.

A `do: stable` action uses the same `since`, `stable_for_ms`, `matches` and
`required` fields as a final stability observation. It completes that native
assertion before later commands (for example, before hanging up queued calls).
This source /4 form adds no new persisted assertion semantics.
