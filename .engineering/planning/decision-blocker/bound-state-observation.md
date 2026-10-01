---
format: aep.planning-md/3
id: decision-blocker:bound-state-observation
kind: decision-blocker
status: cleared
title: Is a binding's state change observed immediately or eventually?
relations:
- blocks: story:feature-request-266
withholds: test_result
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-09-30T13:34:38Z", actor: "human:timo", revision: 3}
---
## Question

When a binding's bound command moves state, does a synthesized scenario observe the bound state immediately after the triggering command answers (the 0.47 generated servers deliver before answering), or only eventually? And does the reported behaviour reproduce on a minimal ESS specification?

## Why it blocks

`story:feature-request-266` cannot state its expectations without this; the interpreter runs no bindings (`crates/verify/ess-conformance/src/interpret.rs:247-255`).

Answered 2026-09-30. Reproduced by the coordinator on ess 0.48.0 with the downstream's minimal specification (attached to beyond10x/ess#266): `repro.job.Start/outcome/started` sends `Start` right after `Create`, although binding `created-starts` (`Created -> Start`) has already moved the job. Observation semantics: bindings deliver eventually, as ESS documents (`crates/verify/ess-conformance/src/synthesize/delivery_context.rs:13`).
