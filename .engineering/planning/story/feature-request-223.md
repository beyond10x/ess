---
format: aep.planning-md/3
id: story:feature-request-223
kind: story
status: active
title: 'explorer strings reach .count boundaries and example: values'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#223
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T11:42:47Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T11:42:48Z", actor: "human:timo", revision: 5}
---
## Outcome

explorer strings reach .count boundaries and example: values (beyond10x/ess#223).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: story:feature-request-223 (beyond10x/ess#223)

Read tree `gaps-270` HEAD `e3bc9a2ff` (contains 0.48.0). Runs: `ess` 0.44.0.

1. **Need.** The explorer cannot reach a success branch that sits behind a minimum text length, so nothing that branch sets is ever explored. Its string pool is `""`, `"a"`, `"b"` plus guard text literals. A `.count` literal feeds only the integer pool, and authored `example:` values are ignored. Requester's syntax: none. The ask is lengths n-1, n and n+1 per `.count` literal, plus examples in the pool. Repro: `~/.cache/ess-gaps/fit2/repro-223`. `Register` refuses `label.count == 0` and `secret.count < 12`, and both inputs carry `example:`.
2. **Class: gap** in a verification tool. The pool is exactly what the design documents (`docs/design/mutation-audit-and-model-runner.md:420`, String row), so it is not a defect. The documented contract also "fails hard when a declared outcome is never reached" (`design:28`), which makes such a specification unexplorable.
3. **Already expressible?** No.
   - Pool: `exploreTexts = {"", "a", "b"}` (`crates/verify/ess-conformance/src/go/explore.go:149`; TS `src/ts/explore.ts:147`).
   - A `.count` literal enters only `integers` as n-1, n and n+1 (`explore.go:463-516`).
   - `example:` values are in the IR (`crates/specify/ess-compiler/src/ir.rs:1226-1230`; repro IR `examples: {label: "build-runner", secret: "correct-horse-battery"}`), but the explorer never reads them.
   - Synthesis, by contrast, uses both. The repro's suite sends `secret` as `correct-horse-battery`, `correct-horc` (12 characters) and `correct-hor` (11) (ess 0.44.0, `jq` over `repro-223/s.json`). The explorer is the sibling that lags.
4. **Fit.**
   - Reuse the synthesis rules rather than invent new ones: example first, then strings cut to the `.count` literals.
   - Text `.count` is already the rune count in the explorer's predicate (`src/go/predicate.go:832-848`), so drawn lengths must be counted in runes.
   - Better shape than the request: a pool per input field, taking the `.count` literals compared against that field, instead of one per command (`explorePools` is per command, `explore.go:456-516`). Otherwise every text input inherits every length.
   - Siblings: `example:` on Integer and enum inputs should feed their pools too. List `.count` is out of reach because lists are not drawn (`design:701`). Text types with invariants stay excluded (`design:701`).
   - Draw order is contract (`design:406-411`), and Go and TS must change together.
5. **Second adopter.** A sign-up form that refuses `password.count < 10` and `username.count == 0`. Exploration never creates an account, so no later login, rename or delete step is ever explored.
6. **Cost.**
   - No authored surface, format or diagnostic.
   - Generated explorer code in Go and TS changes.
   - Every seed's trace changes for commands with text inputs, which needs a release note: a failing seed recorded under an older release replays differently.
   - Tests pinning a pool, if any, are rewritten.
7. **Alternatives.**
   - (a) Change nothing; the adopter sets `AllowExcluded`: not possible, because the outcome is unreached rather than excluded.
   - (b) As requested, a per-command pool: works, but dilutes every text input.
   - (c) Chosen: per-field pools carrying the field's `example:`, plus lengths n-1, n and n+1 for each `.count` literal on that field, with the same rule applied to integer and enum examples.

## Decisions

- **accept, redesigned (proposed):**
  - Use a pool per input field instead of the request's per-command pool. Each text field gets its `example:` value and strings of n-1, n and n+1 runes for each `.count` literal compared against that field.
  - `example:` values also feed integer and enum pools.
  - Go and TS ship together, with a release note that seeds replay differently.
  - Overlaps: bundle with #221, which has the same explorer and the same draw contract. Related to #160 (closed, synthesis boundary witnesses) and #233 (open, byte length, a different construct).
  - Not a duplicate. Not fixed at HEAD `e3bc9a2ff` (`explore.go:149`).

- Coordinator (2026-10-01): adopted as proposed above. Scheduled together with #221.
