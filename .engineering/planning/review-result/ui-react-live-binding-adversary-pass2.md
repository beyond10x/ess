---
format: aep.planning-md/3
id: review-result:ui-react-live-binding-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:ui-react-live-binding (wave ui-live-apps-w2)
relations:
- reviews: story:ui-react-live-binding
revision: 1
---
needs-change

Adversary pass 2 on story:ui-react-live-binding at 731a10c6923. Cases executed 80 to 87, red 7 (introduced 7, pre-existing 1). Test file: crates/ui/ess-ui-react/tests/adversary_live_binding_pass2.rs.

- blocker: names() is 131 lines; clippy::too_many_lines fails task clippy and PR #359's Checks.
- warning: three refused saves leave the second refused value in the draft.
- warning: the current-draft check compares by value (5, 50, 5 reverts over an accepted save).
- warning: refresh above 2^31-1 ms becomes a 1 ms busy loop.
- warning: a refresh written as an expr (0.5s) is silently defaulted to 5000 ms.
- warning, pre-existing: the confirm button has no pending guard.
- note: skip-while-in-flight has no deadline; a read that never answers stops polling.
- note: millis() overflows on a huge refresh (panic in debug).

Held: the form pending guard releases on every outcome; a failing read keeps polling; a bound project with header live indicators type-checks and references no runtime/live; 1s, 1000ms, 1m accepted, 999ms and 0s refused; region commands bind and refuse by node path; the end-to-end test goes red on a wrong read or command path and kills its server on panic.

```findings
[
{"file":"crates/ui/ess-ui-check/src/model.rs","line":491,"category":"judgement","severity":"blocker","verdict":"needs-revision","origin":"introduced","message":"Correction 1 grew names() to 131 lines; clippy::too_many_lines under -D warnings fails task clippy and PR #359's Checks job."},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl","line":140,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"Three refused saves-on-change answered in order leave the second refused value in the draft instead of the value the server holds."},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl","line":140,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"The current-draft check compares by value, so a late failure reverts over an equal later accepted change (5, 50, 5)."},
{"file":"crates/ui/ess-ui-react/src/emit.rs","line":1759,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A refresh above 2^31-1 ms passes the 1 s floor and becomes a 1 ms busy-loop poll."},
{"file":"crates/ui/ess-ui-react/src/emit.rs","line":1758,"category":"contract-drift","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A refresh the schema accepts as an expr is silently replaced by the 5000 ms default instead of being refused."},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/confirm.tsx.tmpl","line":89,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"pre-existing","message":"The confirm button has no pending guard; two clicks send the command twice."},
{"file":"crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl","line":512,"category":"concurrency","severity":"note","verdict":"approve","origin":"introduced","message":"Skip-while-in-flight has no deadline: a read that never answers stops every later poll tick."},
{"file":"crates/ui/ess-ui-react/src/fixtures.rs","line":63,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"millis() multiplies unchecked; a huge refresh panics the generator in debug and wraps in release."}
]
```
