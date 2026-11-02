---
format: aep.planning-md/3
id: review-result:security-w1-gen1-pass-1
kind: review-result
status: active
title: 'W1 GEN1 security pass 1: generated server refuses what its OpenAPI refuses'
summary: 'APPROVE: nothing inside the acceptance; three pre-existing items to the backlog'
relations:
- reviews: story:the-generated-rust-server-refuses-what-its-openapi-refuses
- reviews: story:a-json-command-body-refuses-an-unknown-member
revision: 1
---
unit: W1 GEN1 (story:the-generated-rust-server-refuses-what-its-openapi-refuses, story:a-json-command-body-refuses-an-unknown-member), reviewed in tree ess-w1-gen1 at 1af2b1a6c
verdict: APPROVE (nothing inside the acceptance; one red case, out of scope)
cases: executed 183→188, red 1
origin: introduced 0 / pre-existing 4 / undecided 0

**Findings and what became of them (coordinator).**
- S1 (note, pre-existing): an `Optional` member sent as `null` is admitted while the served schema refuses it
  (`src/rust/wire.rs:923`); not among the acceptance's body classes. Backlog draft
  story:an-optional-member-sent-as-null-is-answered-as-the-openapi-answers-it.
- S2 (note, pre-existing): the agreement test's oracle, jsonschema 0.52.1, takes `+ - .` for digits.
  Backlog draft story:the-date-time-agreement-oracle-refuses-non-digits.
- S3 (note, pre-existing): Go without a binding reads no member names; same as the adversary's F2, fixed in W1.
- S4 (note, pre-existing): the Go server admits any Timestamp text (`src/go/http.rs:1168`). Backlog draft
  story:the-generated-go-server-refuses-a-malformed-timestamp.

**Attacked, not broken.** Every closed position refuses an undeclared member naming its path; the decoder and
`format: date-time` agree over 60,053 seeded strings (oracle defect aside); hostile sizes are linear; a member
named twice is read and validated at its first.
