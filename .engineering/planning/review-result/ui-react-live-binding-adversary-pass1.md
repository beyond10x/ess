---
format: aep.planning-md/3
id: review-result:ui-react-live-binding-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:ui-react-live-binding (wave ui-live-apps-w2)
relations:
- reviews: story:ui-react-live-binding
revision: 1
---
needs-change

Adversary pass 1 on story:ui-react-live-binding at 74deccc48dc. Cases executed 64 to 80, red 9 (introduced 7, undecided 2). Test file: crates/ui/ess-ui-react/tests/adversary_live_binding_pass1.rs.

- blocker: a bound page header's live channels still run fixture scripts (emit.rs:2067).
- warning: names() skips shell region props, so an account menu command the model lacks binds (ess-ui-check model.rs:505).
- warning: a refused account menu command is an unhandled rejection (shell.tsx.tmpl:155).
- warning: usePoll refetches while a read is in flight; a slow server leaves the section loading (data.ts.tmpl:507).
- warning: refresh: 0s generates a busy-loop poll (emit.rs:1755).
- warning: a late refusal reverts over a later accepted change (form.tsx.tmpl:121).
- warning: double submit sends the command twice (form.tsx.tmpl:75).
- warning: the bound README advises a cross-origin proxy, which the epic excludes (README.md.tmpl:38).
- note: the TS classifier and classify disagree on a lone surrogate and 200-deep JSON; no server emits either.

Held: byte identity without --model; no marker leaks; determinism; poll cadence and unmount; no_live refuse; request method, path, wire names, Authorization, base URL; transport classes.

```findings
[
{"file":"crates/ui/ess-ui-react/src/emit.rs","line":2067,"category":"acceptance","severity":"blocker","verdict":"needs-revision","origin":"introduced","message":"A bound page header's live channels still open useChannels, so the header and channel.* values show scripted fixture events next to the real server's data."},
{"file":"crates/ui/ess-ui-check/src/model.rs","line":505,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"names() skips shell region props, so an account menu does: naming a command the model lacks binds."},
{"file":"crates/ui/ess-ui-react/templates/runtime/shell.tsx.tmpl","line":155,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"undecided","message":"A refused account menu command becomes an unhandled rejection: nothing shown, nothing notified."},
{"file":"crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl","line":507,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"usePoll refetches on every tick regardless of the read in flight."},
{"file":"crates/ui/ess-ui-react/src/emit.rs","line":1755,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"refresh: 0s generates usePoll(__read, 0), a busy-loop poll of the server."},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl","line":121,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A late refusal of an earlier save-on-change reverts the draft over a later accepted change."},
{"file":"crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl","line":75,"category":"concurrency","severity":"warning","verdict":"needs-revision","origin":"undecided","message":"Submitting twice before the answer sends the command twice; there is no pending guard."},
{"file":"crates/ui/ess-ui-react/templates/project/README.md.tmpl","line":38,"category":"contract-drift","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"The bound README advises allowing cross-origin requests in a proxy, which the epic excludes."},
{"file":"crates/ui/ess-ui-react/templates/runtime/answer.ts.tmpl","line":33,"category":"property","severity":"note","verdict":"approve","origin":"introduced","message":"JSON.parse accepts a lone-surrogate escape and 200-deep nesting that serde_json rejects; no served server emits either."}
]
```
