---
format: aep.planning-md/3
id: story:the-help-under-a-synthesis-refusal-names-how-an-authored-scenario-answers-it
kind: story
status: draft
title: The help under a synthesis refusal names how an authored scenario answers it
tags:
- from-finding
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/own_scenario_nesting_bounded_work.rs
- confidence: cited
  path: website/docs/guides/verify/author-scenarios.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 5
---
## Outcome

- The `help:` line under `refusal[ESS-SYNTH-003]` that `ess verify conform synthesize` prints says that a scenario a person wrote answers the refusal with `covers: [<the id the refusal prints>]` in an `ess-scenario/7` document, and names the page that shows it, `guides/verify/cover-a-refusal`.
- The guide `website/docs/guides/verify/author-scenarios.md` links that page.
- No behaviour changes: the refusal, its code, its position in the output, the suite bytes and the exit code stay.

## Need

A reader who meets `refusal[ESS-SYNTH-003]: outcome <id> has no scenario …` is told by the line under it to "write the branch's condition over values a candidate can carry, or supply a fixture for it". Neither repair is the one the cause may leave: when the specification is right and the input cannot be generated, an authored scenario that names the id in `covers:` answers the refusal, and the help does not say so.

Read in the tree on 2026-11-23:

- The refusal's help is `RefusalCause::hint` in `crates/verify/ess-conformance/src/synthesize.rs` (about line 1658, the arm of `GuardUnsatisfiable` with no more specific reason). Its text holds no `covers`.
- The line counts generated scenarios only. An authored scenario answers the refusal only by `covers: [<id>]` in a document of `type: ess-scenario/7`; after that the suite written at `--suite-format 5` records `covered_by` and the summary's "covered by authored" count rises. The refusal line itself stays in the output (`website/docs/guides/verify/cover-a-refusal.md`, lines 24 to 27).
- `website/docs/guides/verify/author-scenarios.md` has no link to `cover-a-refusal.md`; a reader who writes authored scenarios from that guide does not learn the mark exists.
- The help text is pinned by `crates/verify/ess-conformance/tests/own_scenario_nesting_bounded_work.rs` (lines about 220 to 235) and is the second column of the `ESS-SYNTH-003` row of `website/docs/reference/diagnostics.md`.

## Design

- Extend the help of the `GuardUnsatisfiable` arm that has no more specific reason with one clause, in the register of the arms around it: where the specification is right and no candidate satisfies the condition, an authored scenario answers the refusal with `covers: [<the id the refusal prints>]` (`ess-scenario/7`; see `guides/verify/cover-a-refusal`). The arms that name a specific repair (a race answered first, a branch declared below an accepting one) are left as they are.
- Add one sentence and a link to `author-scenarios.md`, beside the section on expected branches, pointing at `cover-a-refusal.md`.
- Re-pin the help text where tests and the diagnostics row hold it; the row is regenerated from the catalogue, never edited by hand.
- Nothing else moves: no new code, no changed output order, no changed bytes of any suite.

## Acceptance

- `ess verify conform synthesize` on a specification with one unsatisfiable guard prints a `help:` line that contains `covers:` and `cover-a-refusal`; the line before it and the code are unchanged; the suite bytes and the exit code are those of the release before (compared with `cmp`).
- The test target that pins the help text names the new text, and fails against the old text.
- `website/docs/guides/verify/author-scenarios.md` links `cover-a-refusal.md`, and the docs check passes with the link resolved.
- `task test-xtask` passes (the diagnostics table and its pins).
- Each added or changed test runs under 5 s.

## Decisions

- D1. Where the pointer goes.
  - A (recommended): the help of the generic `ESS-SYNTH-003` arm only.
  - B: the help of every cause an authored scenario can answer. Wider; each arm's pins move.
