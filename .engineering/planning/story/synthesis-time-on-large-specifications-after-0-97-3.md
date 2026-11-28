---
format: aep.planning-md/3
id: story:synthesis-time-on-large-specifications-after-0-97-3
kind: story
status: draft
title: 'Synthesis time on large specifications: 0.97.3 took the largest known one from 2,159 s to 719 s, the way to 300 s is open'
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/provenance.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/disclosure.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/first_kept.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/made.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/model_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/ordered_head.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/run_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/seeds.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/shorthand.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/stored-rows-decided-once.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/stored_rows_decided_once_work.rs
revision: 26
---
## Outcome

`ess verify conform synthesize` on a large specification (tens of domains, about a hundred
commands, stored-field guards, actors with attributes) finishes in minutes on one machine. Its
cost grows with the distinct rows and inputs it has to decide, not with how often the same ones
are asked again. The suites it writes are byte for byte the suites written before. The largest
specification reported so far (797 scenarios) synthesizes in at most 300 s of user CPU; release
0.97.3 brought it to 719 s, and the rest is this story's remainder.

## Need

An adopter reported a downstream specification on 0.97.1: 20 domains, 29 entities, 110 commands,
41 views and 175 authored scenarios. Synthesis took 1,993 s in CI and 2,249 s locally (2,159
CPU-seconds, peak RSS 796 MB, one busy thread) for 797 test cases that then run in about 2 s.
A second reporter measured 744 s on 0.96.0 for a second downstream specification.

Reproduced on this host (instructions:u, user CPU; measured in the working session of the fix,
counters and profiles not kept in the repository):

- First specification. 0.97.1 runs over 300 s with all authored scenarios, with one, and with
  none. It is over 300 s with the earlier synthesis fix too, and without the served components.
  Authored scenarios do not drive the time.
- Second specification. 0.97.1 runs over 300 s (over 796e9 instructions). The earlier fix ends at
  667e9 (about 155 s CPU).
- Generic model: one entity whose N fields are each set by one of N commands, and each command
  refuses a row whose own field is set. Instructions: N=8 0.19e9, 16 0.83e9, 32 7.4e9, 48 33.6e9,
  so growth is about N^3.2 to N^3.7.

## Causes (measured)

Each cause carries what 0.97.3 did about it (details under "Delivered in 0.97.3").

1. **The same branch decision is made again for each search that advances a row.** The
   stored-row search (`search_rows`) advances every row by every move. For a command that reads
   stored fields, it decides for each input which branch the command selects on that row. Every
   moving branch and every goal asks this again. In the first caller reading of the first
   specification, 519,552 decisions were made for 54,991 distinct questions, and 32,673 profiles
   were computed for 4,998 distinct ones. Fixed by commit `fa431ec2a9`: second specification
   667e9 -> 469e9, same suite. Delivered.
2. **The whole stored row is bound again for each predicate evaluated.** `evaluate_row` clones
   the field list, rebuilds the values and binds every settled field of a row of up to 39 fields
   for every guard, leaf and hint. That is about 100-140 us per call, and the allocator takes
   37-44 % of cycles. On the generic model after cause 1, `profile` and `evaluate_row` are
   69-75 %. On the second specification `meet` -> `profile` is 47 %. Delivered: a row is bound
   once for all predicates decided over it.
3. **Caller readings synthesize the model again from nothing.** With actor attributes,
   synthesis reads the model as all-first and all-second, plus 2 focused runs for each command
   whose callers differ, plus 2 more per such command. Each reading rewrites the model at a new
   address, and the memories are keyed by address, so each reading repeats the searches of the
   ones before. On a generic model this costs 9x-17x. On the first specification the same
   external-witness searches (same keys, same 2,101 advance pairs) ran in both whole readings and
   in both focused runs of the command concerned. Partly delivered: the two whole readings are
   made once and keep what their searches found; the second caller reading still does not share
   the work of the first (see Remaining).
4. **The chain search costs about 28 ms per successor row.** In the first specification, a
   narrowed external branch (ess/29) with `later` searches a chain of rows. Each depth makes 24
   successor rows from 2 fresh ones, and only 2 of them are new. The search walks 18 depths to 38
   rows and ends without a witness, after 4-14 s of CPU per search. The cost of each successor is
   choosing its inputs and deciding the branch for each one: picked/selects 44 %, hinted 15 %,
   inputs_for 12 %, meet/profile 13 %. Reduced by the fixes of causes 1 and 2 and by the inputs
   toward a goal being made once; skipping already-seen successors was tried and dropped as
   inexact, so the search itself is unchanged.
5. **The external-witness search itself has no repeat.** Within one reading, 25 calls (7 with
   `later`) had 25 distinct keys. The plain search after a `later` search reads the same depths
   from memory (0.03-0.2 s). The searches end before the 64-row budget, so stopping at the budget
   saves nothing. inputs_for's witness questions are already memoised by their exact guards, and
   they miss because each row carries new values. A cache in that search changes nothing. Still
   true on 0.97.3.

## Delivered in 0.97.3

Tag `0.97.3`, commit `cd9698fd65`. It carries the content of 0.97.2, which was tagged and never
released (its gate failed, see `story:a-release-preparation-runs-every-table-that-pins-the-trees-models`).
The `0.97.2` section of `CHANGELOG.md` at that tag lists about 40 fixes from 17 parallel tracks
plus 6 costs found by profile, one line each; they cover the stored-row search, the binding of a
row to facts, the caller readings, creating-branch rows and value-invariant runs, related-row
arrangement, per-model analyses kept once, `ess specify validate` and the loaders, and the
commands around synthesis (reading and serializing each document once).

Measured, user CPU and instructions:u:

| specification | 0.97.1 | first fix | 0.97.3 |
|---|---|---|---|
| largest reported (797 scenarios) | 2,159 s (adopter's figure) | 1,134.4 s | 719.2 s, 1,780.8e9 |
| second downstream specification | 470.7 s, 1,822.9e9 | | 24.6 s, 102.0e9 |
| third downstream specification | not finished within 1,800 s | | 23.7 s, 101.5e9 (released binary) |

The changelog gives 24.1 s for the third, from an earlier build; the figure above is the released
binary's. Suite bytes of the second are unchanged. Of the largest, 8 scenarios (read-grant
scenarios) lose 24 copied `query_view` and `expect_view` steps; the other 789 keep their bytes.

Two breaking output changes shipped with the release:

- `ess verify conform synthesize --component`, `ess specify validate --component` and
  `ess verify conform mutate --emit --component` synthesize and report only the operations that
  component realises (see `story:a-run-with-component-says-what-it-covers-in-text-and-in-json`).
- A read-grant scenario no longer copies the reads of other views from the scenario it borrows its
  arrangement from.

Each fix has a work-bound test in `ess-conformance` (13 `*_work.rs` targets at the tag; their
placement is `story:work-bound-tests-live-in-one-binary-and-behind-one-counter-module`).

## Remaining

- The largest reported specification takes 719 s against the 300 s wanted. A profile of the
  release code on it is being made; the next story is cut from that profile, not from this list.
- The second caller reading does not share the work of the first.
- Skipping already-seen successors in the chain search was dropped as inexact.
- Contract hashing still copies a JSON map (`construct` bodies in `crates/generate/ess-gen/src/provenance.rs`).
- `row_projections` is still built per call at 5 sites of `synthesize.rs`.
- `Specification::validate_after` runs twice (`crates/specify/ess-domain/src/spec.rs`).

## Acceptance

- Each fix comes with a work-bound test in `ess-conformance` on a generic model with invented
  names. The test bounds a work count (not a clock) that the code before the fix exceeds, and it
  runs under 60 s. Met for every fix of 0.97.3; it holds for each fix still to come.
- Suites are byte-identical (`cmp`) before and after on `examples/billing` (formats 4 and 5), the
  earlier fix's reproductions, the second downstream specification, and generic models
  N=4..64. The byte pins `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes` and `external_beside_held_guard` pass. Met for 0.97.3
  apart from the read-grant change above.
- The first downstream specification synthesizes with the adopter's command and finishes. Met:
  719.2 s on 0.97.3, measured to the end. Open: at most 300 s, measured once to the end the same
  way (instructions:u, user CPU, suite hash), after the Remaining items chosen from its profile.

## Out of scope

- Threads in synthesis: reported, not built.
- Changing any synthesized byte, suite format or refusal beyond the two breaking changes of 0.97.3.
- A coarser key for any memory. A key must contain everything the computation reads.

## A slowdown from 0.99.8 on an adopter's specification, 2026-11-27

An adopter reported that `ess verify conform synthesize` on one specification became much slower with 0.99.8. It is a
defect of the released tool, not of the specification: the extra scenarios are wanted, the cost per scenario is not.
Measured by running the released binaries one after another on one copy of the specification (the same input, pinned
to the oldest release for the older binaries), and by building the commits between 0.99.7 and 0.99.8:

| release or commit | wall time | scenarios | steps | peak memory |
|---|---|---|---|---|
| 0.99.2 to 0.99.7 | 18 s to 27 s | 320 | 2,736 | 1.1 GB to 1.6 GB |
| 0.99.8 | 158 s to 245 s | 565 | 21,683 | 17.2 GB |
| `77f1f6f9ba`, parent of the first slow commit (load on the host 41) | 39.3 s | 320 | 2,736 | 1.09 GB |
| `10c8356d23`, the first slow commit (pull request 835) | 244.5 s | 559 | 21,683 | 17.2 GB |

The first slow commit is `10c8356d23`. The 262 scenarios it adds carry 17,629 steps (average 67, longest 281 steps; the
scenarios before it average 8.6), so the cost per scenario went from about 57 ms to about 377 ms and memory grew by
sixteen times. The new scenarios are wanted; the way they are decided is what costs.

A profile of that commit (sampled with last-branch records) shows where the time goes:

- About 21 % in `core::fmt::write`, building Debug-string memo keys (`subject_fact::reached_key`, the keys of
  `created_memo`, `held_key`: actor maps, arrangements and step vectors are printed to make a key).
- 11.7 % inclusive (6.2 % self) in `EssIr::drivers`, which rebuilds the map from entity to its driving commands on
  every call.
- About 9 % in clones of step lists and arrangements.
- 49 % of the samples are in the C library (`memcpy`, `malloc`, `free`).

Reading, inferred from the profile and not yet tried: a key built by printing a long step list costs time and memory in
proportion to the length of the list, so a scenario of 281 steps asks it often; the map of drivers is a function of the
model alone and wants building once; a step list wants sharing, not cloning, where a search extends it.

### Acceptance

- On a generic model with invented names whose scenarios carry long step lists, a work-bound test in an existing
  `*_work.rs` target of `ess-conformance` counts key bytes printed, builds of the driver map and step-list clones, and
  the counts exceed the bound on 0.99.8 and meet it on the fix. No test runs over 60 s. A key still holds everything the
  computation reads (the rule under Out of scope).
- The suite written for the reported specification by the fix is byte-identical to the one 0.99.8 writes (565
  scenarios), compared with `cmp`; the byte pins of `AGENTS.md` pass.
- Synthesis of the reported specification, measured to the end the same way (user CPU, peak memory, suite hash), costs
  at most twice 0.99.7's cost per scenario (57 ms, so 565 scenarios in about 65 s) and stays under 4 GB. The figures
  are proposed; the fix's own measurement may move them with its reason.
- The same measurement is run before and after on `examples/billing` and on the largest reported specification of the
  story above; no figure rises.

## Status

The hotfix is delivered as a tag, not yet as a release. 0.99.9 tagged 2026-11-27 18:00 CET on `88d3d695df` (`hotfix/0.99.9`); release run [46038774492](https://github.com/beyond10x/ess/actions/runs/46038774492) ("Release preparation") queued, and release 0.99.9 not found, at 18:10 CET. "Shared source gates" on the tag passed (run [46038774677](https://github.com/beyond10x/ess/actions/runs/46038774677)). A release that carries the fix names this story in its pull-request line. Not "published" until the release exists with its four archives and `SHA256SUMS`.

[0.99.9](https://github.com/beyond10x/ess/releases/tag/0.99.9) published 2026-11-27 21:28 CET (bot; four archives and `SHA256SUMS`). An adopter measured 223 s on a host at load 20-22 against 343 s on 0.99.8, same suite bytes.

## A second adopter after 0.99.9, 2026-11-28

A second adopter measured a slowdown from 0.99.2 to 0.99.9 on 2026-11-28, on a specification of
reusable building blocks (32 files: a core and four packs), with
`ess verify conform synthesize --compact --strict-requires --jobs 4`, one run each on a shared
host under load, no other change between the runs:

| release | wall time | peak resident memory | scenarios | refusals |
|---|---|---|---|---|
| 0.99.2 | 70 s | not measured | 913 | 293 |
| 0.99.9 | 319 s | 9.4 GB | 920 | 287 |

That is 77 ms per scenario on 0.99.2 and 347 ms on 0.99.9 (4.5 times), for 7 more scenarios. A
repeat on 0.99.9 on a copy of the specification with one invariant added took 337 s. The figures
are one sample each on a loaded host, as the earlier ones of this story, and the suite bytes of the
two releases were not compared.

They are over the bounds of the 0.99.8 section's acceptance above: at most twice 0.99.7's 57 ms per
scenario, and under 4 GB. 0.99.9 carries the fix for that section, so on this specification the
slowdown the fix targeted is still there after it. The first adopter's own run on 0.99.9 (223 s
against 343 s on 0.99.8, same suite bytes) also stays over the 300 s wanted.

No neutral reproduction yet: the specification is private. The acceptance of the 0.99.8 section (a
work-bound test on a generic model with long step lists, and a before and after on
`examples/billing`) applies to this adopter too, and the adopter's own run is the check that the
figure comes down. A bisect between 0.99.2 and 0.99.9 with the timing as the test is the first
step, as `AGENTS.md` asks of a step that got slower.
