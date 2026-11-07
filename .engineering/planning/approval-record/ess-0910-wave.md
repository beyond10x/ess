---
format: aep.planning-md/3
id: approval-record:ess-0910-wave
kind: approval-record
status: approved
title: 'ESS 0.91.0 runs wave-2026-11-08-ess: host-slot checks, two UI gaps, aggregate measures, the Go decoder and two synthesis fixes in six units'
summary: Coordinator dispatch of 2026-11-07 under the operator grant; non-interactive
tags:
- non-interactive
relations:
- decides: story:a-host-slot-reads-and-commands-only-what-it-declares
- decides: story:host-covered-rows-are-their-own-count
- decides: story:a-client-paged-collection-offers-sizes-and-a-range
- decides: story:a-column-may-show-a-plain-string-row
- decides: story:aggregate-measures-over-expressions
- decides: story:generated-decoders-enforce-declared-value-constraints
- decides: story:a-system-issued-awaited-step-honours-its-outcomes-input-guard
- decides: story:a-union-variant-scenario-reads-each-row-under-its-own-identity
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-07T18:24:42Z", actor: "human:timo", revision: 2}
---
## Decision

On 2026-11-07 the coordinator dispatched this work under the operator's grant: deliver the
repository's active stories in waves (`aep plan artifact waves`), with one full gate, one pull
request per wave, a merge on green and a release within four hours of the merge. No operator is
present at the stage-1 stop. This record stands in for it, and the wave runs.

Skill: aep:implementing 0.26.0, wave mode. Check path: `aep plan artifact waves --kind story
--status active` (computed), after typed scope entries were written for eight unassessed stories.

## Wave plan (wave-2026-11-08-ess, ESS 0.91.0): wave 1 of 3

| unit | stories | objective | surface |
|---|---|---|---|
| U1 | story:a-host-slot-reads-and-commands-only-what-it-declares, then story:host-covered-rows-are-their-own-count | host slots checked and counted (ADR sections 2 and 3) | `ess-ui-check` (`model.rs`, `lib.rs`, `hosts.rs`), its tests |
| U2 | story:a-client-paged-collection-offers-sizes-and-a-range, story:a-column-may-show-a-plain-string-row | two UI gaps reported against 0.88.0 | `ess-ui` `Pager`/`Field`, `ess-ui-check` `rules.rs`/`names.rs`, React `collection`/`core`, terminal `app.rs`/`view.rs`/`expr.rs`, `ess-ui-test` pager parsing, the UI schema |
| U3 | story:aggregate-measures-over-expressions | G3 measures over expressions, ordered enums | `ess-domain` `view.rs`/`types.rs`, `ess-compiler`, `ess-conformance` aggregate, `ess-synth` view queries, `ess-diff`, the next source format |
| U4 | story:generated-decoders-enforce-declared-value-constraints, part A only: the generated Go server decoder | the Go server refuses a value outside a declared constraint | `ess-synth/src/go/` |
| U5 | story:a-system-issued-awaited-step-honours-its-outcomes-input-guard | awaited system-issued synthesis | `ess-conformance` `scenario.rs`, `synthesize/system_issued.rs` |
| U6 | story:a-union-variant-scenario-reads-each-row-under-its-own-identity | union-variant identity reads | `ess-conformance` `runner.rs`, `synthesize.rs`, Go and TS runtimes |

Waves 2 and 3 of this plan: story:aggregate-group-keys-are-expressions,
story:navigation-reaches-any-depth-in-view-filters-and-guards and
story:primitive-canonical-serialization (wave 2); story:derived-measures-and-interval-union and
decoders part B (wave 3).

## The verb's lines for this wave

- Collisions it printed, between stories of this wave:
  - a-client-paged-collection-offers-sizes-and-a-range x a-column-may-show-a-plain-string-row (5 paths);
  - a-host-slot-reads-and-commands-only-what-it-declares x host-covered-rows-are-their-own-count (2 paths).
- Each colliding pair is one unit, worked by one implementor in order.
- Its remaining collisions with this wave's stories are on `CHANGELOG.md` only (U3 x U4, U1 x U3,
  U1 x U4). The coordinator owns `CHANGELOG.md`: implementors write their entry to their scratch
  directory and never edit the file.
- Placement: the verb placed aggregate-group-keys-are-expressions in wave 1 and
  aggregate-measures-over-expressions in wave 2. They collide on six paths. The plan runs measures
  first, the scoper's order: both rewrite the aggregate types and measures is the smaller change.
- Unassessed: story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id.

## Left out

- story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id: not reproduced on
  0.89.0. The reporter was asked for their document on 2026-11-07. Not implementable until it
  arrives.
- story:navigation-reaches-any-depth-in-view-filters-and-guards: collides with U3 on
  `ess-domain/src/view.rs`. Wave 2.
- story:primitive-canonical-serialization: collides with U6 on
  `ess-conformance/src/go/runtime.go`. Wave 2.
- Decoders part B (the Go and TypeScript types targets, Rust prefix and invariants): the
  TypeScript types target emits declarations only. Whether it gains a runtime decoder or refuses
  constrained types is a design decision, recorded as a blocker on the story. The story stays
  active after U4.

## Pre-flight

- Disk: 164 GB free of 848 GB at the start.
- Load: 9.3 on 20 cores.
- `controllers.max_subagents` is 6, so the wave is 6 units. Build slots: 3, shared through the
  build wrapper.
- One `ess-conformance` test build is the largest in the workspace (AGENTS.md); U3, U5 and U6 run
  it one `--test` at a time.

## Terms

- Each unit works in its own managed tree, builds into that tree's `target/`, and gates
  package-scoped: the AGENTS.md package-gate subset for the crates it touches.
- An adversary pass runs for each unit before its merge. Introduced findings go back to the unit;
  pre-existing ones become drafts.
- One full gate runs on the integration head, then one bot pull request (squash) and a merge on
  green, then the tag, the release and the release verification.

## Commits this record authorises

- one or more bot commits per unit on its unit branch;
- one `--no-ff` merge per unit into `integration/wave-2026-11-08-ess`;
- the coordinator's store commits (this record, scopes, review records, evidence, moves), the
  CHANGELOG and version commit, and the release preparation;
- the squash merge of the wave's pull request into `main`, and the `0.91.0` tag.

Nothing else.
