---
format: aep.planning-md/3
id: story:a-null-member-matches-an-absent-optional-in-a-nested-assertion
kind: story
status: active
title: An Optional struct member asserted null matches it being absent at any depth
summary: 'A contains assertion writing value: null on an Optional member of a list element fails on the interpreter, which leaves the member absent; the absent-equals-null rule applies only to top-level dotted paths.'
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T23:36:33Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-22T23:36:34Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
---
## Need

Reported by an adopter (tracker row 48, 2026-10-23, 0.67.0 and 0.68.0). An authored scenario asserts
a view row whose `quantities` is a list of structs, one element written
`{kind: cached_input_tokens, source: unknown, value: null}`, where `value` is an `Optional` member
the command left unset. On the interpreted target the scenario fails as ESS-CF-VIEW: expected
`{"kind":"cached_input_tokens","source":"unknown","value":null}`, observed
`{"kind":"cached_input_tokens","source":"unknown"}` (measured with `--format json`). Without
`value: null` in the assertion the suite passes, 470 of 470.

## Where it lives

- `crates/verify/ess-conformance/src/runner.rs:3395-3404` (`matches`): an absent key equals `null`
  only for a top-level dotted path (the suite/26 rule); a nested struct is compared as a whole
  `Node`, so a missing Optional member never equals an explicit `null`.
- `runner.rs:3417-3430` (`mismatch`) carries the same rule for the failure detail.
- The interpreter fills a top-level unset Optional with `null` (`interpret/views.rs:470-474`) but
  copies nested struct values as stored (`interpret/execute.rs:2775`).
- The reason is present in `ess verify conform run --format json` (the check's `diagnostic`);
  `--report-out` writes the count summary only (`counts.rs:225`, `258-273`) by design.

## Acceptance

- In a `contains` or `equals` assertion, an absent Optional struct member and an explicit `null`
  are one value at any depth (list elements, nested structs), in `matches` and in `mismatch`: an
  asserted `null` matches an observed element that omits the member or carries `null`, and an
  asserted element that omits the member matches an observed `null` (the adopter's testkit target
  emits `null`, the interpreter omits it, so neither spelling passes on both today). Tests in
  `tests/authored.rs` assert all four combinations on the adopter's shape.
- A member written with a value still fails against an absent member, and `null` still fails
  against a present non-null value; tests assert both.
- The generated Go and TypeScript runners give the same verdict for that shape, or the story names
  the runner that differs and a follow-up story is filed.
- No format change.
