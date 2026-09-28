---
format: aep.planning-md/3
id: story:a-wrong-trailing-key-guess-is-reported-as-a-line
kind: story
status: active
title: A wrong trailing-key guess is reported as a line
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/locator_citations.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/trailing_key_guess_adversary.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/trailing_key_guess_adversary_pass2.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/trailing_key_guess_citations.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/trailing_key_guess_named_lists.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T10:14:57Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-09-28T10:14:58Z", actor: "human:timo", revision: 7}
---
# A wrong trailing-key guess is reported as a line

`crates/specify/ess-compiler/src/resolve.rs:889`, `needles_for`, builds a speculative first needle
from a refusal path's last segment: `"{last}:"`. Its doc says the guess is safe:

> Guessing wrongly is safe: `Locator` only reports a line for a needle that occurs exactly once, so
> a bad guess produces no line rather than the wrong one.

A bad guess that occurs exactly once produces the **wrong** line.

Measured by the wave-24 unit-3 pass-2 adversary at `adversary_pass2_locator.rs:157`, exit 101: the
refusal `command.shop.probe.Doit.outcomes.filed`, whose command is declared *and refused* in
`a.yaml`, is cited at `b.yaml:13:13` — the payload line of `shop.probe.Other`, a different command,
in a different file, that is not refused at all. Its sibling refusal one path segment up is cited
correctly at `a.yaml:12`.

## Why every outcome refusal is in this shape

An outcome is written `- name: filed`. It is never written `filed:`. So for every
`command.*.outcomes.<name>` refusal the first needle tried **can never match its own target**; it can
only ever match something else, and it is reported whenever that something else occurs exactly once
anywhere in the specification — an event field of the same name in one payload block is enough.

`Locator`'s own header says a confidently wrong line is worse than no line.

## Not a wave-24 regression

The adversary checked the base: `git show bd722fa9:…/resolve.rs`'s `scan` is
`text.match_indices(needle)` with no filter, and wave 24's `whole_name_matters("filed:")` is `false`,
so base and current take the identical path for this needle. Wave 24 made the claim *newly visible*
by writing a second doc comment asserting it; it did not introduce the behaviour.

Not reachable in `examples/billing` — all 8 outcome names checked, 0 raw `<name>:` matches.

## Acceptance

A refusal is never cited in a file that holds no refusal. A needle that cannot match its own target
is not tried, or its match is not reported as that refusal's line.

## Scope

Re-derived 2026-09-28 by `story-scoper`; corrected at close of wave correctness-1 from the implementor's reports (merge `b0c89d384`).

- **Landed in:** `crates/specify/ess-compiler/src/resolve.rs` (`Locator::span`/`encloses`, `NAMED_LISTS`, `MAPS_UNDER`, `STRUCTURAL`) — cited
- **Tests:** `trailing_key_guess_citations.rs` (1 → 3), `trailing_key_guess_adversary.rs` (5), `trailing_key_guess_adversary_pass2.rs` (3), `trailing_key_guess_named_lists.rs` (2); `locator_citations.rs` doc comments — cited
- **Docs:** `docs/design/review-typed-diagnostics.md` — cited
- **Correction:** the fix went in `Locator::span`, not `needles_from_tokens` alone; the unit test at `resolve.rs:~4880` changed and the scope had not named it
