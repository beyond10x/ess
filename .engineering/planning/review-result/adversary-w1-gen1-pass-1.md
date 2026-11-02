---
format: aep.planning-md/3
id: review-result:adversary-w1-gen1-pass-1
kind: review-result
status: active
title: 'W1 GEN1 adversary pass 1: generated server refuses what its OpenAPI refuses'
summary: 'NEEDS-CHANGE: Go server without a binding admits an undeclared member (inside story 2); comments and CHANGELOG corrected; two items to the backlog'
relations:
- reviews: story:the-generated-rust-server-refuses-what-its-openapi-refuses
- reviews: story:a-json-command-body-refuses-an-unknown-member
revision: 1
---
unit: W1 GEN1 (story:the-generated-rust-server-refuses-what-its-openapi-refuses, story:a-json-command-body-refuses-an-unknown-member), 56b6441ef and 1af2b1a6c merged as 12ca705bc, reviewed in tree ess-w1-gen1 at 1af2b1a6c
verdict: NEEDS-CHANGE
cases: executed 183→187, red 3
origin: introduced 4 / pre-existing 2 / undecided 0

**Findings and what became of them (coordinator).**
- F1 (warning, introduced): `src/rust/json.rs:885` admits a leap second and fractions past 9 digits, which the
  tree's instant vectors and the generated instant grammar refuse; the served OpenAPI admits both, so the
  acceptance (decoder agrees with OpenAPI) holds. The false doc comment at `json.rs:836` goes back to the
  implementor; the grammar choice is backlog draft story:an-admitted-timestamp-is-an-instant-the-server-can-order.
- F2 (blocker, pre-existing code, inside the acceptance): the Go server without a binding admits an undeclared
  JSON member (`src/go/http.rs:1863`). Ruling: derived routes read the default request, whose `unknown` is
  `refuse`, so story 2 covers them; back to the implementor.
- F3 (warning, introduced): CHANGELOG Go line overstated; corrected with F2.
- F4 (note, introduced): `src/go/http/bound.rs:295` comment claims the Rust wording; corrected.
- F5 (note, pre-existing): Duration read as any text; backlog draft story:a-duration-member-is-refused-as-its-format-refuses.
- F6 (note, introduced): `decode_system_event` refuses older log envelopes; CHANGELOG states it, beside the
  adopter-replay open question.
