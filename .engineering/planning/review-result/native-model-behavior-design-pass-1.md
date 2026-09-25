---
format: aep.planning-md/2
id: review-result:native-model-behavior-design-pass-1
kind: review-result
status: active
title: Native model behavior technical design review, pass 1
relations:
- reviews: task:consumer-accounting-native-model-behavior
revision: 1
---
needs-revision

task:consumer-accounting-native-model-behavior — The executed-stage contract requires plan digest and receipt-key coverage but omits exact equality of its repeated selected_cases/cases/claims and each keyed receipt's case identity/source hashes with the plan/authority, so add those checks and mutation refusals before accounting/3 qualification — docs/design/consumer-model-behavior-native-cases.md:121

What I read: 3 AEP artifacts via `aep plan artifact show`; 1 binding design, 2 sibling decision/scope records, and the relevant load and consumer_coverage account/enforce/proposal/native/executor/consumer/preservation/mod sources via `rg` and `nl`.

What I could not establish: runtime behavior and test outcomes, because this design-only brief forbids builds and tests; this report qualifies no code or cells.

```findings
- file: docs/design/consumer-model-behavior-native-cases.md
  line: 121
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The executed-stage contract requires plan digest and receipt-key coverage but omits exact equality of its repeated selected_cases/cases/claims and each keyed receipt's case identity/source hashes with the plan/authority, so add those checks and mutation refusals before accounting/3 qualification
```
