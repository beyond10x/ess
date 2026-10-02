---
format: aep.planning-md/3
id: review-result:consumer-interpreted-facts
kind: review-result
status: active
title: Independent source review of supplied interpreter facts
relations:
- reviews: story:interpreted-scenario-supplied-facts
revision: 1
---
approve

Coordinator independent source review of interpret/facts.rs and interpreted_scenario_facts.rs in native-interpreter-frozen.patch ac0800023e84f2be71177d1c86a1c57cc8578cd5714047d57ff37c39f727ead4. These two files were authored by scope_aggregate, so that worker's review of the other ten native files does not establish independence for them. Own executions for this review: zero; final combined carrier execution is pending.

The delivery path checks active correlation, external event authority, complete declared payload and the union of matching context fields before retaining or dispatching an occurrence. Retained delivery is separate from publication history. Redelivery uses the latest supplied payload and context, and lifecycle reset clears both. Relative marks record the logical instant and actual publication cursor; holds advance to the maximum requested time without repeatedly adding a duration. Counts filter actual later publications by event and correlation. Unknown events, arithmetic exhaustion and missing host facts refuse explicitly. No absolute timestamp or undeclared stored value is synthesized.

The five actual-target tests include authority/type/correlation failures before effects, latest-context retention, payload/context decoys, separate authority channels, publication cursor boundaries and reset. Their expectations are literal values and observed calls, not only absence of errors. Initial three-test red and final five-test green evidence are separately identified by the author; the extra guard and decoy controls are not represented as having run in the original baseline. Absolute-time guards and periodic host effects remain bounded missing-fact refusals; this source review does not prove full story completion or authorize weakening the trust matrix.

```findings
[]
```
