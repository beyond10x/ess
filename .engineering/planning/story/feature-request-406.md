---
format: aep.planning-md/3
id: story:feature-request-406
kind: story
status: implemented
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
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/src/client.rs
- confidence: cited
  path: crates/edge/ess-cli/src/model_types.rs
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/go.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust_timestamp.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/ts.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/binary64_structural.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/model_ergonomics.rs
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T08:11:43Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-03T08:11:43Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:33Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":2,"review_outcome":1}}}
---
## Outcome

A producer can write against generated Go and Rust model libraries directly: timestamps are native, anonymous shapes have readable names, a constant field carries its value, and the domain prefix can be dropped.

## Acceptance

- A model `Timestamp` realizes as Go `time.Time` and Rust `EssTimestamp(time::OffsetDateTime)`, round-trips the timestamp value, numeric offset and fractional-second value through RFC 3339 serde/JSON; native serializers may canonicalize lexical spelling, including `.500` to `.5`, and refuses `"yesterday"` at decode; the Rust library builds offline (`crates/generate/schema-contract/tests/model_ergonomics.rs`).
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

## Source integration handoff

The operator explicitly handed off the bot-authored commit 7b3d70c7c527a1a2ba2540fcc01a48ef2b43db25 from the Claude transports session for integration and green-CI bot merge. It is locally merged into the existing integration branch, retaining its original commit. The scope paths are cited directly from this source diff. Independent test review remains pending; author-reported schema-contract, publisher, CLI, strict lint and projection checks are retained as author evidence, not substituted for the final integration gate. The requested completion is the exact main merge commit; no ESS tag or release is authorized at this boundary.

## Timestamp review disposition

Independent probes measured `.500+02:00` serializing as `.5+02:00` in both native targets. The author acceptance text overclaimed byte-for-byte preservation; the operator request is native timestamp types and RFC 3339 wire values. The contract now states native lexical canonicalization explicitly instead of adding raw-spelling storage or changing the native producer API. Retain the original red probes and verify the timestamp value and numeric offset against the clarified contract. This is a documentation correction, not a claim that the old byte-exact assertion passed.
