---
format: aep.planning-md/3
id: review-result:adversary-e1-pass-1
kind: review-result
status: active
title: Adversary pass 1 on the Rust target's one-time response obligation
relations:
- reviews: story:the-rust-target-writes-a-one-time-response-outcome-as-an-obligation-that-carries-its-contract-instead-of-refusing-the-workspace
revision: 1
---
unit: E1 one-time response obligation, working tree ess-wr-e1 at base cf4b3b19b plus uncommitted changes
verdict: CONFIRMED (one contract-drift warning, two notes; the code itself held)
cases: executed 10→14, red 1
origin: introduced 2 / pre-existing 0 / undecided 1
wrote-outside-worktree: 1 path (scratch-adv log directory)
needs-coordinator: yes. The design-page sentence needs a rewording, and finding 3 needs a run at the base commit to settle its origin.

Cases (tests/adversary_one_time_obligation_pass1.rs): the_design_pages_every_other_capability_is_generated_holds_for_siblings (red: left ["command_behavior" credentials.api.Issue, "command_behavior" credentials.api.Rotate], right [Issue]); every_marked_outcome_of_one_command_names_its_own_fields (green); a_marked_command_behind_a_per_refusal_binding_synthesizes_for_rust_only (green); go_web_and_clap_refuse_with_the_unchanged_detail (green).
Suite: adversary 3 passed 1 failed; one_time_response 5 passed; refusal_policy 5 passed (exit 101).
Not broken: Rust workspace and single-crate output for marked models type-check (except finding 3); output differs from unmarked only in provenance digests and contract docs; replays: from a marked origin refused by the compiler; no reader parses the contract string, no persisted key changed; no "; " added; every named mutant killed.

```findings
- file: docs/design/one-time-response-values.md
  line: 232
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the page says every other capability of a marked model is generated, while the unit's own SIBLINGS model owes Rotate's behaviour (red in the_design_pages_every_other_capability_is_generated_holds_for_siblings)
- file: crates/generate/ess-synth/tests/one_time_response.rs
  line: 27
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: named's JSON phrase check is now satisfied by the plan clause embedded in every refusal document, so a changed Go/web/CLI refusal text stays green; compare cause.detail() exactly instead
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 1078
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: the generated System's obligations field is never read for a component with an owed binding escalation, so the generated workspace fails cargo check under -D warnings, with or without one_time_response
```
