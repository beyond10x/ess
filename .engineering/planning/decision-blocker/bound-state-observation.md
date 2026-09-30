---
format: aep.planning-md/3
id: decision-blocker:bound-state-observation
kind: decision-blocker
status: open
title: Is a binding's state change observed immediately or eventually?
relations:
- blocks: story:feature-request-266
withholds: test_result
revision: 1
---
## Question

When a binding's bound command moves state, does a synthesized scenario observe the bound state immediately after the triggering command answers (the 0.47 generated servers deliver before answering), or only eventually? And does the reported behaviour reproduce on a minimal ESS specification?

## Why it blocks

`story:feature-request-266` cannot state its expectations without this; the interpreter runs no bindings (`crates/verify/ess-conformance/src/interpret.rs:247-255`).
