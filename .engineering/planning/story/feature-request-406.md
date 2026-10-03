---
format: aep.planning-md/3
id: story:feature-request-406
kind: story
status: draft
title: Generated Go and Rust model libraries a producer can write against
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#406
- provider: github
  reference: beyond10x/ess#407
- provider: github
  reference: beyond10x/ess#408
- provider: github
  reference: beyond10x/ess#409
relations:
- serves: vision:O2
- decomposes: epic:message-contract-clients
revision: 1
---
## Outcome

A producer can write against generated Go and Rust model libraries directly: timestamps are native, anonymous shapes have readable names, a constant field carries its value, and the domain prefix can be dropped.

## Acceptance

- A model `Timestamp` realizes as Go `time.Time` and Rust `EssTimestamp(time::OffsetDateTime)`, round-trips an RFC 3339 string with offset and fraction byte for byte through serde/JSON, and refuses `"yesterday"` at decode; the Rust library builds offline (`crates/generate/schema-contract/tests/model_ergonomics.rs`).
- Model anonymous shapes are named by position (Go `ProbeMeterReadingSamples`), no `EssShape<hex>` remains in model output, and bundle output keeps its hash names.
- A newtype with `value == 2` yields Rust `VALUE` + `Default` and Go `<Name>Value` + `New<Name>()`.
- `--names short` declares `Reading`, `Sample`, `Version`; two components with one last segment are refused with `short_name_collision`; bundle input refuses the flag.
- The frozen old-output digest in `binary64_structural` is refrozen with the reason.

## Origin

beyond10x/ess#406, #407, #408, #409, filed by the adopter of the generated publishers (#395) before producers start writing against the API.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, 2026-10-03.

1. **Need.** Building one event payload with the generated Go library took `CreatedAt string`, `Version *X{Value: 2}`, `Usage EssShape344f09950b06c0e4` and a domain prefix on every name (#406–#409 reproductions).
2. **Class.** Gaps in the target mapping; no authored surface is asked for.
3. **Already expressible?** No: `types-only-realizations.md` leaves timestamps as strings and names anonymous shapes by digest.
4. **Fit.** All four are realizer choices for model input; bundle output is unchanged byte for byte. `time` is already in the workspace lock.
5. **Second adopter.** Any model library with a timestamp field or a list field.
6. **Cost.** Generated model APIs change (source-incompatible for code reading timestamp fields or naming hashed shapes); one new generated dependency in Rust; one CLI flag on two verbs; one frozen digest refrozen.
7. **Alternatives.** (a) Change nothing: every producer writes a conversion layer. (b) `chrono`: not in the workspace lock, so the offline gate could not build it. (c) Chosen as above.

## Decisions

- **accept (2026-10-03):** native timestamps via `time`, positional anonymous names with hash fallback, integer-const newtype constants, opt-in short names; all model-input only.
