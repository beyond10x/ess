---
format: aep.planning-md/3
id: story:a-synthesized-scenario-whose-setup-its-own-model-refuses-is-written-with-the-rows-its-guards-relate-or-refused-by-synthesis
kind: story
status: draft
title: A synthesized scenario whose setup its own model refuses is written with the rows its guards relate, or refused by synthesis
summary: '25 of 201 scenarios of a downstream model fail interpreted on 0.99.0 to 0.99.2: a link compared with an input (older than 0.97.4) and a re-pointed creation (regression fd8a86bcf3, 0.98.0); two red tests, fixes planned'
tags:
- conformance
- defect
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-guard-compares-a-link-field-with-an-input
- informed_by: story:a-setup-runs-refusal-names-the-real-cause-where-its-branch-has-a-scenario-of-its-own
revision: 3
---
## Summary

A synthesized conformance suite of a downstream application's model fails 25 of its 201 scenarios
when it runs against the declared interpreter of that same model (`ess verify conform run --target
interpreted`); the identical numbers hold on 0.99.0, 0.99.1 and 0.99.2. The suite of 0.97.4 had 124
scenarios and all passed. All 25 failing scenarios are among the 77 that 0.97.4 did not write: 17
were first written, and failing, by 0.98.0, the other 8 by 0.99.0. A scenario the interpreter of
its own model refuses is a synthesis defect: the setup step sends an input the model's own guards
refuse, or sends rows apart from the link the subject holds, so the model answers a refusal that
the scenario did not expect.

## Evidence

Three groups, by the reporter's counts and by the diagnosis (each verdict is by reading and by the
red test named; reductions are generic models, committed as tests):

1. **A link compared with an input, beside refusals that read the row the input names.** 13
   scenarios of two reply commands, answered with a "not a reply of that kind" refusal. A command
   has a `when_subject` guard that compares the subject's stored link with an input
   (`stored ref != input ref`) beside `when_related` refusals that read the row that input names.
   The later branches are sent a related row arranged apart from the subject's link, and a setup
   that ignores the held-state guards, so the model answers the held-state refusal where the
   scenario expects a later outcome; the interpreter follows the declared precedence. Older than
   0.97.4: the reduction fails on every release from 0.97.4 to 0.99.2, and the larger coverage of
   0.98.0 and 0.99.0 exposes it. No regression commit.
   Red test `a_link_guard_beside_a_missing_row_refusal_on_the_same_input_passes_its_own_model` in
   `crates/verify/ess-conformance/tests/link_field_guard.rs`, commit `32f75b7249` (0.07 s for the
   binary): the interpreter refuses two scenarios of the reduction with an "other-account" refusal.
2. **An "elsewhere" refusal.** 8 scenarios of three commands. Two causes under one refusal name:
   6 scenarios of one command are group 1's defect (a stored environment compared with an input
   beside related refusals on that input); the 2 others fail in a setup grant whose component lives
   in another environment than the one sent, the symptom of group 3 (the mechanism is inferred by
   pattern, not traced).
3. **A setup creation refused by the guards of its own creating act.** 4 scenarios of a revoke
   command have the grant command of their setup step answered with a "namespace not allowed"
   refusal. The revoke's scenarios point the created row's stored link at an arranged row by
   rewriting the creating act's input (`relink`, the `Link::Row` arm of
   `crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs`), and that act was
   arranged as its own scenario (`as_own_scenario`) with rows its guards relate to the old link, so
   the rewritten send is refused. A regression of `fd8a86bcf3` (released in 0.98.0): at its parent
   `df7eb37a65` the revoke outcome is refused by synthesis and 13 scenarios pass; at `fd8a86bcf3`
   it is written, 17 scenarios, 4 fail.
   Red test `a_setup_run_arranged_as_its_own_scenario_keeps_the_rows_its_guards_relate` in
   `crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs`, commit `a8d35b75de` (binary
   3.33 s, 19 tests): the interpreter refuses four scenarios of the reduction.

Both red tests are on the branch `fix/n88-n89-2026-11-25` (pushed at `a8d35b75de`), without a fix.

## Fix plan (by group, not built)

- Group 1, `related_guard.rs` (`drive` and `drive_several_strict` for a command with a subject):
  where a `when_subject` guard answered before the branch compares a stored link of the subject
  with the input that a `when_related` read uses as its via, bind that via to the row the subject's
  link names (the pins and bound values) instead of arranging a row beside it; where the branch
  needs another row there, refuse the branch with a note (as the "no account" outcome below an
  "other account" row needs).
- Group 3, `related_guard/stored.rs` (`relink`, the `Link::Row` arm): refuse the scenario as
  unpointed where a `when_related` guard that the creating act answers before its writer relates
  the row the rewritten input names to another input (a predicate over another row reading that
  input, or over that row reading another input), so the scenario is refused as on 0.97.4 instead
  of sent a creation its own guards refuse. Later: re-arrange the creator with the link bound.
- Group 2 follows from the two above; its two scenarios of the group 3 symptom are checked after
  group 3, since their mechanism was not traced.

## Acceptance

- The two red tests are green with the fixes, and the interpreted run of the reporter's model class
  passes every scenario that is written: a scenario the interpreter of its model refuses is
  refused by synthesis with a note instead.
- Suites move. A fix that refuses a branch or binds a different row changes the written suites;
  every moved committed suite and byte pin is re-pinned in the change with its cause beside it, and
  the release's `CHANGELOG.md` names the moved suites.
- The fix leaves group 3's 0.97.4 behaviour (the revoke outcome refused by synthesis) where the
  creating act's guards refuse the rewritten send, and writes the scenario again where the creator
  is re-arranged with the link bound.

## Fit review

1. **Class.** A defect of synthesis (a suite that fails against its own model). Group 3 is a
   regression of `fd8a86bcf3`; group 1 is older than 0.97.4.
2. **Release.** A fix moves suites, so it is a release of its own after 0.99.3 (the patch release
   of `story:a-setup-run-that-falls-back-to-its-creating-commands-own-scenario-builds-that-scenario-once-per-distinct-arrangement-not-once-per-candidate-row`),
   not part of it. The changelog names the moved suites.
3. **Borders.** `story:a-guard-compares-a-link-field-with-an-input` (a link compared with an input
   is refused, ESS-SYNTH-003; implemented) and
   `story:a-setup-runs-refusal-names-the-real-cause-where-its-branch-has-a-scenario-of-its-own`
   (a draft about the refusal text).
4. **Safety net.** A check after synthesis that runs each written scenario against the model is a
   story of its own:
   `story:a-synthesis-runs-each-written-scenario-against-the-declared-interpreter-of-its-model-and-turns-a-failing-one-into-a-refusal-with-a-note`.

## Decisions

- None open. The release route of the fixes (after 0.99.3, a release of its own) follows from the
  moved suites.

## Delivery

Branch `fix/n88-a-synthesized-setup-is-accepted-by-its-own-model`, head
`090b691e2dc686620b0940243b297885a6d3c67b`, pushed: the group 3 fix `39f1b33bc3`, a third red test
`4eb8a5a6a3` (`a_link_guard_on_the_input_naming_a_row_beside_passes_its_own_model`), and the group
1 fix `090b691e2d`. On the reported model 0 of 178 scenarios fail (25 of 201 on 0.99.2). 23
scenarios are no longer written; each of them failed before, and none that passed was lost. No
committed byte table moved. The gate ran every step with exit 0.

## Left open

The 23 scenarios are refused by synthesis and not arranged; arranging them is not built.
