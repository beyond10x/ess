---
format: aep.planning-md/3
id: review-result:zi-t2-interp-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence
relations:
- reviews: story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence
revision: 1
---
Queued cargo run 1 has now finished. The test file had 11 cases: 8 passed and 3 failed, exit 101 (`adv/run1-adv.txt`, slot A, waited 2657 s, ran 105 s). Each failure is the one the earlier report predicted. This is that file's first run, so these failures were captured before any suite run.

```
unit: story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence (tree zi-t2-1, HEAD 1aa33e964e)
verdict: weak
new failing cases: 3 (adversary_t2_interp_presence_pass1.rs::adv_struct_member_null_when_absent_step_spells_null, ::adv_binding_mapping_of_an_omitted_when_absent_field_passes, ::adv_binding_mapping_of_an_absent_optional_field_passes_without_presence)
mutants survived: none found
```

- **What passed (8):**
  - The CLI reproduction run inside a test (1 scenario).
  - Optional-input variants under `omitted_when_absent` and no presence.
  - The row-read reproduction under `null_when_absent`, `omitted_when_absent` and no presence, run against the interpreted target. No test did this before.
  - A null input under all three presences.
  - The struct-member case at suite level.
  - A binding mapping a `null_when_absent` field.
- **Struct member, `:251`:** `left: None, right: Some(Null)` with `{"kind": Text("k")}`. When an event payload builds a struct field by field, a member declaring `null_when_absent` is left out instead of written as `null`. The suite does not catch it, and the story limits the rule to top-level fields. This is a backlog item, and the base already behaves this way.
- **Binding mapping, `:380` (no presence) and `:386` (`omitted_when_absent`):** the scenario reports ``"`example.token.TokenContributed` carried no field `note`"``. The cause is `runner.rs:3435`, where the runner insists the mapped field is in the event. This is a backlog item, not caused by this unit: the unit's new line only fires for `null_when_absent`.
- **Acceptance:** nothing I ran contradicts it. It does not block the unit. The "8 scenarios" count still needs the triage spec.
- **Suite run:** I did not run the existing suite afterwards. All 5 cargo runs are used up, and my file only adds tests.

`git --no-pager diff --stat` is still empty. The only change is the untracked test file. The findings block is unchanged from the earlier report, except the struct-member row, which this run measured:

```findings
- file: crates/verify/ess-conformance/tests/absent_payload_read.rs
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no case ran the interpreted target on the row-read reproduction under the three presences; the adversary file's adv_row_read_* cases now do and pass
- file: crates/verify/ess-conformance/src/runner.rs
  line: 3435
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: a binding mapping an Optional event field the trigger omits (no presence or omitted_when_absent) errors with "carried no field" instead of passing
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: a payload struct literal's member declaring null_when_absent is omitted rather than null in the interpreted event (measured at step level); the suite does not observe it and the story scopes the rule to top-level fields
```
