---
format: aep.planning-md/3
id: review-result:adversary-5w-absent-pass-2
kind: review-result
status: active
title: Adversary pass 2, absent (the-5-waves)
relations:
- reviews: story:absent-command-input-outcome
revision: 1
---
Adversary pass 2 against story:absent-command-input-outcome (#170), aep:adversary, 2026-09-28, the-5-waves wave 3, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 9, red 2 (executed 1745→1754)
origin: introduced 1, pre-existing 1, undecided 0

New cases in `adversary_absent_pass2.rs` (ess-domain, ess-conformance). Red: replay admission does not count the new step as an invocation; missing(body.text) over a required struct field is admitted at ess/16. Green: De Morgan spellings, /25 admission, coverage /27, unknown_instance beside input_absent.

Ledger against pass 1: carried 0, new 2, resolved 3. Coordinator routing: both to the final correction round; the coordinator verifies the diff.

```findings
[{"file":"crates/verify/ess-conformance/src/replay.rs","line":449,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Replay admission does not count execute_command_without_input as an invocation."},
 {"file":"crates/specify/ess-domain/src/command/absent_input.rs","line":153,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"The never-holds refusal reads only one-segment paths, so missing(body.text) over a required struct field validates at ess/16 while synthesis cannot witness it."}]
```
