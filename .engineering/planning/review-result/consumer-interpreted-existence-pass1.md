---
format: aep.planning-md/3
id: review-result:consumer-interpreted-existence-pass1
kind: review-result
status: active
title: Supplied identity and existence first independent review
relations:
- reviews: task:consumer-backlog-20261002
revision: 1
---
needs-revision

```json
[
  {
    "file": "crates/verify/ess-conformance/src/interpret/execute/existence.rs",
    "line": 67,
    "category": "incorrect-outcome-selection",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "independent-source-review",
    "message": "The preselection loop reads every creation's identity/entity. A row belonging only to an unselected accepting branch therefore forces existing_instance on a request that would create a distinct entity or use a different identity field. Use the uniquely shared address when one exists; otherwise establish the selected creation address without treating unrelated storage as that command's duplicate."
  }
]
```

Reviewed root production diff against fc676ff13, new existence helper and current tests before final freeze. Reviewer executions: 0; root's builds remained untouched.

Concrete source/control: the admitted TWO_ENTITIES source in crates/generate/ess-synth/tests/adversary_upsert_by_existence_pass1.rs:231 has a BookSlot command whose `as_item == true` branch creates Item and whose default creates Slot, both reading input.slot_id. From an empty store, send `(slot_id="shared", label="item", as_item=true)`, then `(slot_id="shared", label="slot", as_item=false)`. The first creates Item; the second should create Slot, an independent entity namespace. The current loop finds the Item and returns already-booked without reaching the Slot branch. Repeating the second call should then refuse and preserve Slot. Exercise reverse order too. For different input fields on one entity, give each branch a separate identity input and populate only the unselected branch's address; the selected fresh address must remain creatable.

Existing generated implementations deliberately keep mixed-entity/field creations as an obligation in ess-synth/src/determined.rs:991–1040, rather than treating any matching storage as one address. The generated lookup contract at rust/behaviour.rs:1004 describes the identity the command's creation would take. No test execution is claimed here; the wrong branch follows directly from the current loop and this existing source fixture.

Uniform creation address behavior otherwise follows the stated precedence: related commands test duplicate before the related row, then input refusal; other commands test input refusal before duplicate. Optional omission/null with generated fallback skips lookup and uses the bounded generator, whereas a supplied identity remains exact. Given/Recorded still control only generated slots; explicitly sourced identities use their declared source, and ordinary emitted identity values reuse the created identity.

Mixed related-dependent creation addresses need an explicit decision: related_guard.rs allows creation subjects without imposing one shared entity/address, while its stated precedence checks the own identity before reading the related row. Selecting the address from that row first would invert the precedence. The existing unsupported generator disposition is evidence of that boundary, not authorization to redefine it. Keep this case explicit rather than guessing a global any-row rule or silently changing precedence.

Input validation is intentionally partial before precedence-driven refusals in existing helpers. The new early duplicate route validates the address value but does not validate unrelated required inputs; I have not classified that as a new violation without a binding contract requiring full request validation before declared refusals.
