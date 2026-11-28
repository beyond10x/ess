---
format: aep.planning-md/3
id: review-result:adversary-e3-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:rust-binding-wraps-a-plain-copy-into-an-optional-member
relations:
- reviews: story:rust-binding-wraps-a-plain-copy-into-an-optional-member
revision: 1
---
unit: E3, story:rust-binding-wraps-a-plain-copy-into-an-optional-member, working tree ess-wr-e3 on base e29c029fab (uncommitted)
verdict: red, from a defect that was already in the base; nothing the unit claims broke
cases: executed 76→81, red 1
origin: introduced 0, pre-existing 1, undecided 0
wrote-outside-worktree: 6 paths in `scratch-adv/`
needs-coordinator: yes. One acceptance line can't hold as written, and the red defect needs a backlog story

Cases added in crates/generate/ess-synth/tests/feasibility.rs: adv_e3_every_plain_copy_wrapped_into_an_optional_input_compiles (green), ..._compiles_for_the_web (green), adv_e3_a_selecting_binding_wraps_a_plain_copy_and_compiles (green), adv_e3_an_accessor_and_a_wrapped_copy_into_optional_inputs_compile (green), adv_e3_control_a_selection_over_an_enum_member_compiles (red, feasibility.rs:2308: E0422 cannot find struct, variant or union type 'EnumVariant', 14 errors in selection-system/src/lib.rs).

Suite: binding_optional_wrap_go 9 passed; feasibility 71 passed, 1 failed (exit 101); with --skip adv_e3_ 9 + 67 passed.

Attacked without a break: 11 plain source kinds wrap once and compile (Rust and web); Optional into the same Optional not wrapped; wrap beside a condition-proved member; wrap inside a selecting binding; accessor next to a wrap; nested Optional<Optional<T>> still refused; Go bytes unchanged; no re-stamp needed; a different newtype stops at validation or becomes an obligation with a conversion.

```findings
- file: crates/generate/ess-synth/src/rust/selection.rs
  line: 91
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the Rust selection predicate over an enum member emits a Debug print of the domain EnumVariant value, so a binding the plan reports generated does not compile (red in feasibility.rs:2308, adv_e3_control_a_selection_over_an_enum_member_compiles)
- file: crates/generate/ess-synth/tests/binding_optional_wrap_go.rs
  line: 527
  category: acceptance
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the story's acceptance demands a BindingAssignment refusal for a newtype into an Optional of another newtype, which validation's type_mismatch makes unreachable, and the test asserts type_mismatch instead
- file: CHANGELOG.md
  line: 37
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the web target also stops refusing a plain copy into an Optional (recorded before as a web binding-assignment refusal) and the entry names only the Rust target
```
