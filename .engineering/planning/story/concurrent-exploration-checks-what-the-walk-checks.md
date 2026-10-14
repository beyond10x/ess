---
format: aep.planning-md/3
id: story:concurrent-exploration-checks-what-the-walk-checks
kind: story
status: active
title: Concurrent exploration checks what the sequential walk checks
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T14:33:27Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-14T14:33:28Z", actor: "human:timo", revision: 3}
---
## Context

The Go conformance package of `ess verify conform synthesize --target go` holds a target to less
under concurrent exploration than under the sequential walk (beyond10x/ess#576, found on 0.59.0
with a model of 13 commands and no entities):

1. `ess-history/1` operations record neither the command's input nor the events it published, so
   on a model without entities `ess verify conform check-history` accepts any declared outcome. A
   wrong answer that is still a declared outcome passes `ExploreConcurrent`.
2. Only the sequential walk calls `ConfigureExternalOutcome`, so concurrent exploration never
   arranges an `external:` cause, even with `Inject`.
3. Inside a multi-seed run, once earlier seeds have excluded commands, `ExploreOptions.Seed`
   replays a different sequence than that seed drew in the run, so a failing seed reported by a
   run does not reproduce alone.

## Outcome

Concurrent exploration checks what the sequential walk checks. A history operation carries its
input and its published events, and `check-history` compares them with the model. A concurrent run
arranges external causes as the sequential walk does. A seed replays alone the sequence it drew
inside a run.

## Acceptance

- A history operation records the command's input and the events it published. `check-history`
  refuses a history whose outcome the model does not admit for that input, or whose published
  events differ from the outcome's `emits:`, on a model without entities. Test: a module in
  `crates/edge/ess-cli/tests/explore_concurrent.rs` (an existing binary) with a planted target that
  answers a declared but wrong outcome. It fails before the change and passes after.
- `ExploreConcurrent` arranges an `external:` cause through `ConfigureExternalOutcome` whenever the
  sequential walk would. Test: a planted backend-dependent defect that fires in at least one of 200
  concurrent histories, in `explore_concurrent.rs`.
- A seed reported failing by a multi-seed run replays the same command sequence when run alone with
  `ExploreOptions.Seed`. Test: in `explore_concurrent.rs`, the sequence drawn for seed N inside a run
  equals the sequence of a run with only seed N, where an earlier seed excluded a command.
- If the history format gains fields, that is decided as a format change under the repository's
  format rules. The old-reader compatibility test comes first.
- No new `tests/*.rs` file. Every test goes into an existing binary of the crate.
