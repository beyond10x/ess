---
format: aep.planning-md/3
id: review-result:adversary-e1-pass-2
kind: review-result
status: active
title: Adversary pass 2 on the Rust target's one-time response obligation
relations:
- reviews: story:the-rust-target-writes-a-one-time-response-outcome-as-an-obligation-that-carries-its-contract-instead-of-refusing-the-workspace
revision: 1
---
unit: E1 correction round 1 (pass 2)
verdict: NEEDS-CHANGE
cases: executed 87→93, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: scratch-adv2 (4 logs)
needs-coordinator: none

Cases in tests/feasibility.rs from line 2741: adv2_e1_web_bridge_over_an_escalation_on_a_command_that_cannot_refuse_compiles (red: E0107 struct takes 1 generic argument but 2 supplied, crates/credentials-web/src/lib.rs:191:72; E0277 System<Unrealized>: Bound not satisfied); 5 green under -D warnings (one crate, workspace with HTTP server, escalation no refusal selects, one called and one uncalled, uncalled keeps trait and stub).
Suite: feasibility 83 passed 1 failed; one_time_response 5; adversary pass 1 4 (exit 101).
Held: all escalate combinations in both layouts, served crate and entry, trait and stub kept, no committed tree moved, calls_escalation matches every call site, corrections 1 and 2 in place.

```findings
- file: crates/generate/ess-synth/src/web/bridge.rs
  line: 283
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the web bridge's Bound impl still adds the Obligations parameter for any universal escalate while System now takes it only where calls_escalation holds, so a web crate for an escalate on a command without refusals fails with E0107 (red in adv2_e1_web_bridge_over_an_escalation_on_a_command_that_cannot_refuse_compiles)
```
