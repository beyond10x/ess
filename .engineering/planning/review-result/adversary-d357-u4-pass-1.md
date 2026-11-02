---
format: aep.planning-md/3
id: review-result:adversary-d357-u4-pass-1
kind: review-result
status: active
title: 'D357 U4 adversary pass 1: identities a binding scenario compares'
relations:
- reviews: story:binding-site-identity-comparisons
revision: 1
---
unit: D357 U4 (story:binding-site-identity-comparisons), b1a810d3d on unit/ess-d357-u4, reviewed against 700b4faa6
verdict: NEEDS-CHANGE (2 warnings); fixed at 50110c61e
cases: executed 272→274, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory u4/adv-scratch (cargo log only)
needs-coordinator: none

**Cases.** `crates/verify/ess-conformance/tests/adversary_d357_u4_pass1.rs`: `CloseTicket` acts on an existing ticket, names an agent in its input and emits `TicketClosed`; the binding `closed-engages` engages the agent. One case carries `ticket_id: {subject: ticket_id}` in the event with the destination arranged before the trigger; the other has the ticket store `agent_id` and the event carry it, with that stored agent as the destination. In both, a target publishing a wrong id passed flow, mapping and delivery, while the reference interpreter passes every binding scenario.

**Findings.**
- F1 (warning, introduced), `crates/verify/ess-conformance/src/synthesize.rs:9250`: when a binding's setup differs from the trigger's own run, `expect_trigger_event` dropped the subject instance and the settled identities, so a `{subject: …}` or stored identity in the trigger event went uncompared. Fixed at 50110c61e: kept when the setup starts with the trigger's own, and a stored identity is compared with the value the arrangement wrote.
- F2 (warning, introduced), `crates/verify/ess-conformance/tests/adversary_273_pass1.rs:291`: the rewritten pin swept only input-copied identities over single-file fixtures. Fixed at 50110c61e: input-, subject- and stored-sourced identities over every fixture including system directories (519 identities).

**Residue.** The wider sweep finds 7 input identities uncompared in binding refusal scenarios produced by the refusal-policy witness site (`refusal_policy.rs` near :225), which still emits a shape-only event step. Outside this story's scope (decided before the pass); pinned by name in `refusal_witness_residue()` and left to a later story.

**Attacked, not broken.** The re-pinned bytes rows moved only by the expect step; a bounded retry swapped to another publisher passes a matching branch and run; a trigger event with two identity fields compares the input-copied one; never-invoked later triggers read nothing stale; no committed suite moved; no correct target is wrongly failed.

**Fix-round run (50110c61e).** ess-conformance lib 172 and 13 targets (274 passed in all, adversary_d357_u4_pass1 2); bytes tables unchanged; clippy -D warnings clean; crate fmt check clean.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":9250,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"where a binding's destination is arranged before the trigger, expect_trigger_event dropped the subject instance and settled identities, so a subject or stored identity in the trigger event was uncompared in flow, mapping and delivery; fixed at 50110c61e"},
{"file":"crates/verify/ess-conformance/tests/adversary_273_pass1.rs","line":291,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the pin swept only InputField sources over single-file fixtures, so it stayed at zero while subject- and stored-sourced identities and system-directory models were unchecked; widened at 50110c61e"}
]
```
