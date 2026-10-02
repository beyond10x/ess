---
format: aep.planning-md/3
id: review-result:consumer-deletion-342-pass1-20261002
kind: review-result
status: active
title: Independent source review of guarded deletion post-state
relations:
- reviews: story:feature-request-342
revision: 1
---
unit: story:feature-request-342
verdict: nothing found in independent read-only review
cases: own executions 0; inspected implementor red 0/3, focused green 75/75
origin: current two-file diff against c28e3bdae
wrote-outside-worktree: no source/evidence writes; own review lease acquired and released
needs-coordinator: final grouped package verification remains deferred

The production change correctly makes deletion leave `Setup.after = None` at `subject_fact.rs:4073`, matching ordinary preparation at `synthesize.rs:3442`. Arrangement, identity, `before`, settled fields and input selection remain intact.

I traced the affected obligations:

- **Events survive:** `exercise_as` generates positive/negative event assertions independently of `run.after` (`synthesize.rs:2359–2442`).
- **Absence and retry survive:** `deletion_witness` uses the deletion effect, preserved instance and input; it requires absence, repeats the command for its missing outcome, checks its error and forbids unexpected events (`synthesize.rs:8577`).
- **Stored guards survive:** preparation still arranges and observes stored fields (`subject_fact.rs:3979`); boundary/overlap witnesses use pre-state and remain emitted (`:4427`). `around_row` already excluded deletion from moves/updates (`:4464`).
- **Only nonexistent-row checks disappear:** post-state observation (`synthesize.rs:2711`), regular row assertions (`:5652`) and deleted-subject invariant scenarios (`:10349`). Other outcomes retain their previous `after` value.
- **Replay remains safe:** domain validation permits only creating/moving replay origins (`crates/specify/ess-domain/src/command.rs:2121`).

The cart target independently stores revisions and removes rows. Its mutant changes only row removal, and the test requires failure of the exact absence check. The unguarded control and strict all-scenarios `Passed` checks provide meaningful discrimination (`tests/outcome_shapes.rs:99–257`).

Evidence inspected: `342-red.log` shows all three regressions failing; `342-focused.log` records 75 passed, zero failed/ignored; `342-clippy-final.log` completes successfully. Formatting success is implementor-reported with an empty retained log. I ran no tests or builds.

One nonblocking limitation: the structural event assertion accepts any `ExpectEvent`, including the setup event. Source inspection confirms the closing event remains generated; naming `CartClosed` would strengthen that assertion. No concrete behavioral counterexample found.

```findings
[]
```
