---
format: aep.planning-md/3
id: story:an-optional-member-sent-as-null-is-answered-as-the-openapi-answers-it
kind: story
status: draft
title: An Optional member sent as null is answered as the served OpenAPI answers it
summary: Rust server admits null for an Optional member the served schema refuses (W1 GEN1 review, pre-existing, backlog)
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:security-w1-gen1-pass-1
revision: 2
---
## Need

An `Optional` command member sent as `null` is taken as absent by the generated Rust server
(`crates/generate/ess-synth/src/rust/wire.rs:923`, pre-existing, same arm at b81deac78 `:883`) and answered
202, while the served OpenAPI request schema has no `null` branch and refuses the body. Found by
review-result:security-w1-gen1-pass-1 (red case
`an_optional_member_sent_as_null_is_answered_as_the_served_document_answers_it`, left out of the committed file).

## Acceptance

- For an `Optional` member sent as `null`, the generated Rust server and its served `openapi.json` give
  the same answer: either the schema admits `null` or the server refuses it at decode naming the path.
  The design records which.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.

## Repro

The W1 GEN1 security pass wrote `an_optional_member_sent_as_null_is_answered_as_the_served_document_answers_it`
in `crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs`; it was left out of the committed file because
the item is out of W1. It drives one generated Rust server with an `Optional` member sent as `null` and compares
the answer with the served `openapi.json`; it becomes this story's first failing test.
