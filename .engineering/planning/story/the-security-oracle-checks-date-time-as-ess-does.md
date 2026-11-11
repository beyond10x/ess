---
format: aep.planning-md/3
id: story:the-security-oracle-checks-date-time-as-ess-does
kind: story
status: draft
title: The gen1 security oracle checks date-time as ESS does
summary: document_admits in adversary_w1_gen1_security still uses jsonschema's own date-time checker (turn-2 security review, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-datetime-sec-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`document_admits` in `crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs` (line 509)
decides whether the served OpenAPI document admits a request body with
`jsonschema::draft202012::options()`, whose `date-time` checker takes `+`, `-` and `.` at some
digit positions. Since `story:the-date-time-agreement-oracle-refuses-non-digits`, ESS checks
RFC 3339 `date-time` itself through `schema_contract::uniqueness::options()`, and the agreement
test and the gen1 adversary oracle build from it. The security oracle still does not, so a future
request body holding such a non-digit would make it disagree with the server for no reason in the
code under test. Found by the turn-2 security review (`review-result:zi-t2-datetime-sec-1`);
the same code is at the turn's base `7623ee5ec1`, so it is pre-existing.

## Acceptance

- `document_admits` builds its validator from `schema_contract::uniqueness::options()`.
- A case posts a body whose `date-time` member is `202+-10-29T12:00:00Z`; the oracle says the
  document refuses it, as the server does.

## Scope

- `crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs` (`fn document_admits`)
