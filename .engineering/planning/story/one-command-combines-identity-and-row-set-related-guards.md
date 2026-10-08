---
format: aep.planning-md/3
id: story:one-command-combines-identity-and-row-set-related-guards
kind: story
status: draft
title: One command may guard on an identity-addressed related row and on a row set, and a related guard may sit beside when_subject
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#473
relations:
- serves: vision:O2
revision: 1
---
## Outcome

https://github.com/beyond10x/ess/issues/473: three everyday rules become expressible. Today an
identity-addressed `when_related` and a row-set `when_related` in one command are refused
(`ESS-COMMAND-009`), and a `when_related` beside `when_subject` is refused (`ESS-COMMAND-004`).

## Constraints

- Validation lives in `crates/specify/ess-domain/src/command/`. This story waits until another
  session's selection-plan wave 3 has merged; it does not edit `crates/specify/ess-domain/src/command.rs`,
  `crates/specify/ess-compiler/src/ir.rs`, `src/command/subject_state.rs` or
  `src/command/related_guard.rs` before then.
- Spec first: the admitted combinations are stated in the specification language's own reference and
  validated with the newest `ess` before validation or synthesis changes.
- A fit review per `.agents/skills/assessing-external-requests/SKILL.md` is owed before acceptance
  is written; the issue's reproduction (`shop.orders.AddLine`) is reproduced on the newest release
  first.
