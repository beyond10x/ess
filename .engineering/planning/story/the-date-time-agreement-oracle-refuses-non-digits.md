---
format: aep.planning-md/3
id: story:the-date-time-agreement-oracle-refuses-non-digits
kind: story
status: draft
title: The date-time agreement oracle refuses non-digits at digit positions
summary: jsonschema 0.52.1 takes + - . for digits in date-time (W1 GEN1 review, pre-existing, backlog)
tags:
- backlog
- defect
- review-finding
- severity-high
- zero-open-issues
relations:
- informed_by: review-result:security-w1-gen1-pass-1
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/Cargo.toml
- confidence: cited
  path: crates/generate/ess-synth/tests/json_body_closure.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/date_time.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/lib.rs
- confidence: cited
  path: crates/generate/schema-contract/src/uniqueness.rs
- confidence: cited
  path: crates/generate/schema-contract/src/validate.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/date_time_digits.rs
revision: 9
---
## Need

`jsonschema` 0.52.1 (the workspace's pinned version) takes `+`, `-` and `.` for digits in
`format: date-time`. Its `parse_four_digits` and `parse_two_digits`
(`src/keywords/format.rs` of the crate) test all bytes at once with a borrowing subtraction, so a
byte just below `0` in the top position, or one whose borrow is absorbed by its neighbour, passes as
a digit. Found by review-result:security-w1-gen1-pass-1 (153 of 60,053 seeded strings, e.g.
`202+-10-29T12:00:00Z`).

Two places take that validator's verdict on `date-time`:

- The shipped `ess generate schema validate` (`crates/generate/schema-contract/src/validate.rs`,
  `fn validator`, `.should_validate_formats(true)`) admits such a timestamp. Reproduction on
  0.92.0, a schema with `"$id": "urn:example:stamp"` and a property
  `at: {type: string, format: date-time}`:

  ```
  $ ess generate schema validate --schemas schemas instances/plus-in-year.json    # "at":"202+-10-29T12:00:00Z"
  1 schema(s), 1 instance(s): valid          (exit 0)
  $ ess generate schema validate --schemas schemas instances/minus-in-hour.json   # "at":"2026-10-29T1-:00:00Z"
  1 schema(s), 1 instance(s): valid          (exit 0)
  $ ess generate schema validate --schemas schemas instances/letter-in-day.json   # "at":"2026-10-2xT12:00:00Z"
  [invalid_instance] instances/letter-in-day.json/at: "2026-10-2xT12:00:00Z" is not a "date-time"   (exit 1)
  ```

- The agreement test
  `crates/generate/ess-synth/tests/json_body_closure.rs::the_server_refuses_exactly_what_its_served_openapi_refuses`
  uses the same validator as its oracle, so a future body could pass the agreement check while
  RFC 3339, and the generated servers' own `timestamp_at`, refuse it.

## Acceptance

- `cargo test -p schema-contract --locked --test date_time_digits`, case
  `schema_validate_refuses_a_non_digit_in_a_date_time`: on the reproduction's schema,
  `ess generate schema validate` exits 1 with `invalid_instance` naming member `at` for each of
  `202+-10-29T12:00:00Z`, `2026-10-29T1-:00:00Z` and `2026-1.-29T12:00:00Z`, and exits 0 for
  `2026-10-29T12:00:00Z`.
- `cargo test -p ess-synth --locked --test json_body_closure`, new case
  `the_date_time_oracle_refuses_a_non_digit_at_every_digit_position`: the oracle, now built from
  `schema_contract::uniqueness::options()`, refuses `2026-10-29T12:00:00Z` with each of `+`, `-`,
  `.` and `x` put at each of its 14 digit positions, and admits the unchanged string.

## Decisions

Keep `jsonschema` 0.52.1 and add an RFC 3339 `date-time` format checker of ESS's own, registered
with `with_format("date-time", …)` in `schema_contract::uniqueness::options()` (the crate's one
constructor, which its own doc comment says every validator and test oracle starts from) and used
by the agreement test through a `schema-contract` dev-dependency of `ess-synth`: our own checker is
pinned by our own test, whereas an upgrade depends on an upstream release whose digit check this
repository cannot hold. It keeps every currently valid timestamp valid.

## Scope

- cited: crates/generate/schema-contract/src/validate.rs
- cited: crates/generate/schema-contract/src/uniqueness.rs (`options`, line 18)
- inferred: crates/generate/schema-contract/src/date_time.rs (new)
- inferred: crates/generate/schema-contract/src/lib.rs
- inferred: crates/generate/schema-contract/tests/date_time_digits.rs (new)
- cited: crates/generate/ess-synth/tests/json_body_closure.rs (the oracle, line 370)
- cited: crates/generate/ess-synth/Cargo.toml (dev-dependencies)

confidence: medium

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no format row, no byte-pin row.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is
released. The shipped-command reach above is new since the draft and may move it forward.
