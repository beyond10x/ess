---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-command-guard-that-compares-a-fact-with-an-expression-of-several-facts-on-both-sides-of
kind: story
status: active
title: Synthesis witnesses a command guard that compares a fact with an expression of several facts, on both sides of the comparison, so a target that drops a factor or a term fails
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-ess-39-a-command-guard-compares-a-fact-with-an-expression-of-several-facts-and-every-expression-reads-the
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:51:59Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-24T03:52:00Z", actor: "human:timo", revision: 6}
---
## Need

`sx12-guard-arithmetic` admits, at `ess/39`, a command guard whose operand is an expression of
several facts (`input.cells.count != lines.count * sites.count`) and decides it in the interpreted
target. A guard no scenario witnesses from both sides is a figure the suite does not check: a
target comparing with `lines.count + sites.count`, or with `lines.count` alone, must fail a
synthesized scenario.

Synthesis today grounds a derived operand that reads one fact: it arranges the stored value the
sent input computes (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:237-250`,
`ground_derived` `:1890-1951`), and the witness module treats `Derived` operands by their one
parent (`crates/verify/ess-conformance/src/witness.rs:3447-3470`, `:4739-4750`). The value
positions' arithmetic is computed, not searched (`story:expression-arithmetic-and-time`, Fit review
point 4: "bounded linear-arithmetic witness search"). A product of two list lengths is not linear in
the facts synthesis chooses.

Sixth list, item G12 (rank P2), the synthesis half.

## Outcome

- For each branch guarded by a comparison with a several-fact operand, synthesis chooses the facts
  the operand reads from a small bounded set (list lengths 0 to 4, integers from the literals the
  guard and its siblings name, plus and minus one), evaluates both sides with the shared evaluator,
  and writes one scenario where the comparison holds and one where it fails, both reaching the
  branch the precedence rule says. Lists are filled with distinct elements of their element type,
  so sibling `distinct` guards do not answer first.
- Where both sides cannot be reached inside the bounded set (for example `a * b == 7919`), the
  branch is refused by name, as `ESS-SYNTH-001` refuses a branch it cannot arrange, naming the
  operand and the bound searched; no scenario is invented.
- The mutation each scenario pair kills is the one the requester's figure needs: a target that
  drops a factor, swaps `*` for `+`, or compares with one list's length fails one scenario of the
  pair. The pair is chosen so that the alternatives disagree on it (for `a * b` with `a, b ≥ 2`, a
  value where `a * b ≠ a + b` and `a * b ≠ a`).
- The suite format does not move: the scenarios use inputs, rows and expectations of today's
  shapes. A model with no such operand keeps its suite bytes.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- First: a case in the existing target `crates/verify/ess-conformance/tests/expression_arithmetic_and_time.rs`
  (no new test binary) with the reproduction
  of `sx12-guard-arithmetic` (`demo.plan.Plan`, `Admit`, branches `cells-repeat`, `incomplete`,
  `admitted`). Before (with the domain unit landed): `ess verify conform synthesize` refuses
  `Admit/outcome/incomplete` or writes no scenario on its holding side. After: it writes a scenario
  reaching `incomplete` (for example 2 lines, 3 sites, 5 distinct cells) and one reaching
  `admitted` (6 distinct cells), and the interpreted target passes both.
- A hand-written target that compares `cells.count` with `lines.count + sites.count` fails one of
  the two scenarios, and one comparing with `lines.count` fails one: a test drives each through the
  interpreter's mutation hook or a small in-test target and names the failing scenario.
- `when: "total != input.price * input.qty"` gets one scenario per side, `total` set from the
  product and moved by one.
- An operand with no reachable side in the bound is refused by name with the operand and the
  bound; a test pins the text.
- `an_old_suite_keeps_its_bytes`: every committed suite and the byte pins named in `AGENTS.md` §
  Gate (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`) are unchanged.
- Run `ess-conformance` one `--test` at a time (its full build is the largest in the workspace).
  Each new test is timed and expected under 10 s; none may run over 60 s, each test step stays
  within 120 s, and each build-pool job within 10 minutes.

## Fit review

Verdict: **accept**, the synthesis half of the redesign in `sx12-guard-arithmetic` (sixth list,
item G12). Tree `bcfc51555`, read-only.

1. **Need.** The figure is checked only if a scenario separates the stated comparison from its
   likely mistakes.
2. **Class.** Gap that follows from the domain unit.
3. **Already expressible?** No.
4. **Fit.** A bounded search over small values, in the grounding step that already arranges derived
   operands; refusal by name where the bound is not enough, as every unarrangeable branch is.
5. **Second adopter.** Every guard of `sx12-guard-arithmetic`'s second-adopter list.
6. **Cost.** No suite format, no new refusal code (the existing "cannot arrange" refusal names the
   operand). Synthesis time: a bounded enumeration per branch; the test above is timed, and the
   change is timed on the repository's examples before and after (`AGENTS.md`, "Tests stay fast").
7. **Considered.** (a) A general non-linear solver: unbounded cost. (b) Refuse every several-fact
   operand in synthesis: the figure would stay unchecked, which defeats the item. Chosen: bounded
   enumeration with a named refusal.

## Decisions

- None of its own. Follows SX-D4 and SX-D5, both decided: A, 2026-11-24.

- FMT-B (decided: A, 2026-11-24): this story moves to `ess/38` with the rest of the pull request (D77: no `ess/39`); the Acceptance line on the format says so.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`ground_derived`
  `:1890-1951`, the derived-call grounding `:237-250`)
- cited: `crates/verify/ess-conformance/src/witness.rs` (`Derived` arms `:3447-3470`,
  `:4739-4750`, `:4929-4930`)
- cited: `crates/verify/ess-conformance/src/synthesize.rs` (where a branch is refused by name)
- inferred: `crates/verify/ess-conformance/src/synthesize/arithmetic.rs` (new: the bounded search)
- inferred: cases in `crates/verify/ess-conformance/tests/expression_arithmetic_and_time.rs`
- Size: L, about 450 changed lines.

## Depends on

- `sx12-guard-arithmetic`.
