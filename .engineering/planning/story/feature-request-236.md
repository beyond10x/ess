---
format: aep.planning-md/3
id: story:feature-request-236
kind: story
status: active
title: 'mutate --emit has no --component: a repository implementing one component cannot score mutants'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#236
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T11:42:49Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T11:42:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

mutate --emit has no --component: a repository implementing one component cannot score mutants (beyond10x/ess#236).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: story:feature-request-236 (beyond10x/ess#236)

Read tree `gaps-270` HEAD `e3bc9a2ff` (contains 0.48.0). Runs: `ess` 0.44.0.

1. **Need.** A repository that implements one component of a multi-component system cannot get a mutation score from `mutate --emit/--collect` that ends in a clean exit. Requester's syntax: `--component <name>` on `mutate --emit`, emitting component-scoped suites and only mutants whose site lies in an owned domain. Repro: `ess verify conform mutate --path repro-212/base --emit repro-236/emit --component x` → `error: unexpected argument '--component' found` (ess 0.44.0). HEAD has no such argument (`crates/edge/ess-cli/src/main.rs:690-713`).
2. **Class: gap**, narrower than filed.
   - The issue's stated blocker, "`--collect` refuses a baseline that is not fully green", is fixed: #210 closed, `2ce01b28b`, released in 0.41.0. The issue was filed 2026-09-29T03:40Z; 0.41.0 was tagged 04:35Z. A skipped or unsupported baseline scenario is now listed as not scored (`ess verify conform mutate --help`, 0.44.0).
   - What remains: a mutant whose changed scenarios all belong to another component is *inconclusive* ("inconclusive when a scenario it changed was not scored"), and any inconclusive mutant gives exit 3 (same help text). So a component repository cannot reach exit 0, and its score is padded with mutants it cannot answer.
3. **Already expressible?**
   - Partly. A runner can be pointed at whole-system suites and the report filtered by hand, since mutant ids `<class>/<site>` carry qualified names (`design:150`).
   - No exit status, `--class` selection or manifest field narrows the audit to a component.
   - `synthesize --component` exists, built on `synthesize_for` (`crates/verify/ess-conformance/src/synthesize.rs:1705`; `ess verify conform synthesize --help`), but `mutate` does not use it.
   - The design defers it: "`--component` for `mutate`" (`docs/design/mutation-audit-and-model-runner.md:700`).
4. **Fit.**
   - Reuse `synthesize --component`'s name and meaning: it keeps scenarios whose every command, event and view the component accepts, publishes or owns (`synthesize.rs:1688-1705`).
   - The requester's site filter, "owned domain", does not match that meaning. A component may accept commands of another domain, and a mutant there changes scenarios in its suite. The filter that fits is "the mutant changes at least one scenario in the component's suite", computed by the same changed-scenario diff the inconclusive verdict already uses.
   - `--collect` must know the component, so the manifest records it. That means `ess-mutation-manifest/4`, with `/3`–`/1` still read (`--help`).
   - `--target` (built-in whole-system reference targets) should refuse `--component` by name.
   - The deferral note's cost ("parent chains and scope re-derived per mutant", design `:700`) applies to coverage suites, not to ordinary `synthesize_for`.
5. **Second adopter.** A system with `billing` and `catalog` components, implemented in two repositories. The catalog repository wants its own mutation gate in CI.
6. **Cost.**
   - New CLI flag.
   - Manifest `/4`.
   - A report field naming the component, plus a count of mutants left out as another component's.
   - Tests over a two-component example.
   - No authored surface, no specification format.
7. **Alternatives.**
   - (a) Change nothing; filter the report downstream: refused, because exit 3 stays and the gate cannot be used.
   - (b) As proposed, an owned-domain filter: refused, because it misjudges accepted cross-domain commands.
   - (c) Chosen: `--component` on `--emit` and `--collect`, with suites from `synthesize_for` and mutants kept where they change a scenario in that suite. The rest are listed as out of scope, not scored.

## Decisions

- **accept, redesigned (proposed):**
  - `--component` on `--emit` and `--collect`, with the same meaning as `synthesize --component` (`synthesize_for`). The component is recorded in `ess-mutation-manifest/4`.
  - Keep mutants that change a scenario of the component's suite, rather than the request's owned-domain rule. List the others as out of scope.
  - `--target` refuses the flag by name.
  - Partly already fixed: the issue's stated blocker, `--collect` refusing a non-green baseline, closed with #210 (`2ce01b28b`, 0.41.0). What remains is exit 3 caused by other components' mutants being inconclusive.
  - Overlaps: #210 (closed), the design's Deferred row (`design:700`), and #212 (a shared mutate release could carry the manifest bump). Not a duplicate.

- Coordinator (2026-10-01): adopted as proposed above.
