---
format: aep.planning-md/3
id: review-result:consumer-one-time-browser-pass1
kind: review-result
status: active
title: Independent browser policy projection review, first pass
relations:
- reviews: story:feature-request-389
revision: 1
---
needs-revision

Independent source-only review, own test/build executions0. Reviewed carrier ess-backlog-next-20261002 frozen patch one-time-browser-review.patch SHA256 c677e49246615c903df17cf96af024780700d2af27846027db188520009cf791. New one_time_browser.rs initially matched d25903b8e728cbda806ddff56451e49782d90d3a58745bb50ffb61fe5a169ab8; the final inspected source matches7bdfed794316155f041bcd26b92af4074b9edd1299b22be359a81b4964145dd2 after its local site variable was renamed output_directory. Producer reports actual Firefox tests red3/3 then green3/3; those are not my executions.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/web.rs",
    "line": 158,
    "category": "contract-drift",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Ordinary model outcome projection still omits one_time_response. The new player.js:86 display reads only scenario.one_time_response.origins. A marked model paired with an ordinary empty or limited suite lacking marked-origin trace metadata therefore loses the source policy entirely. This is also reachable with the stage-one synthesizer before the source-derived trace installer exists. Preserve marked policy on model outcomes and display source obligations independently of selected scenario trace presence, or refuse explicitly. Add an empty/limited-selection control. Source-only finding; no own execution."
  },
  {
    "file": "crates/verify/ess-conformance/src/web_replay.rs",
    "line": 136,
    "category": "boundary",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The public AdmittedReplay::new constructor still accepts a marked EssIr with its matching coverage input and builds replay/1 through web::model at line160. Only web::emit_input checks the new policy, so a direct caller bypasses the named refusal and receives the reduced policy-free model. Centralize the model-aware refusal in the public replay constructor (or a helper invoked there) and add a direct-constructor regression. Keep legacy replay bytes and schema unchanged. Source-only finding; no own execution."
  }
]
```

The marked command catalog path correctly overrides dispatchable to false, preserves component and outcome/field provenance, and makes no enforcement claim. The ordinary annotated-suite DOM test correctly checks Unexecuted declaration status and no fabricated state/events. Those positive checks do not cover the two alternate public inputs above. Root accepted both findings and owns the corrections and deciding regressions; this report does not approve the uncorrected snapshot.
