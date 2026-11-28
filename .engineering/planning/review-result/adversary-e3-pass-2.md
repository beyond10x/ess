---
format: aep.planning-md/3
id: review-result:adversary-e3-pass-2
kind: review-result
status: active
title: Adversary pass 2 on story:rust-binding-wraps-a-plain-copy-into-an-optional-member
relations:
- reviews: story:rust-binding-wraps-a-plain-copy-into-an-optional-member
revision: 1
---
unit: E3, story:rust-binding-wraps-a-plain-copy-into-an-optional-member, working tree ess-wr-e3 on base e29c029fab (uncommitted, after correction 1)
verdict: red. The correction now compiles, but it compares the wrong spelling, so the selection silently picks the wrong item
cases: executed 81→85, red 2
origin: introduced 1, pre-existing 1, undecided 0
wrote-outside-worktree: `scratch-adv2/`, 4 logs and `probe/` (2 models and 3 generated trees)
needs-coordinator: yes. The Rust fix goes back to the implementor; the Go defect needs a backlog story

Rust (introduced): selection predicate compares the variant name; since correction 1, src/rust/selection.rs:93 writes variant.wire(), so {name: Internal, wire: internal} gives "internal" != "Internal". Test adv2_e3_a_selection_over_a_wire_named_enum_variant_selects_it (feasibility.rs:2488) fails at choose.rs:12, left Some("fallback"), right Some("internal-leg"). Fix: variant.name(); correct CHANGELOG.md:38.
Go (pre-existing): go/selection.rs:125 same defect; adv2_e3_the_go_selection_over_a_wire_named_variant_answers_the_compared_spelling (feasibility.rs:2500) fails.
Suite: binding_optional_wrap_go 9 passed; feasibility 74 passed, 2 failed (exit 101).
Not broken: escaped wire spellings, enum via nested struct / newtype / Optional struct with == and !=, selected enum into Optional input; other {:?} prints in src/rust/ print strings.

```findings
- file: crates/generate/ess-synth/src/rust/selection.rs
  line: 93
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the selection enum arm answers the wire spelling while the predicate literal is the variant name, so a variant with a declared wire spelling is never selected (red at feasibility.rs:2488, choose.rs:12 left Some("fallback")); write variant.name() and correct CHANGELOG.md:38
- file: crates/generate/ess-synth/src/go/selection.rs
  line: 125
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the Go selection arm answers the wire spelling and selectionCompare compares it with the variant name, so a wire-named variant is never selected (red at feasibility.rs:2500)
```
