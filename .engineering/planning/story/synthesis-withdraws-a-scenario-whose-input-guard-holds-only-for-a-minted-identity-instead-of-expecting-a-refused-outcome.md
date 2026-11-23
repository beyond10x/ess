---
format: aep.planning-md/3
id: story:synthesis-withdraws-a-scenario-whose-input-guard-holds-only-for-a-minted-identity-instead-of-expecting-a-refused-outcome
kind: story
status: draft
title: Synthesis withdraws a scenario whose input guard holds only for an identity the target mints, instead of expecting an outcome the interpreter refuses
tags:
- defect
- from-finding
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller/arranged.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_guard_over_a_minted_identity_is_not_assumed_true.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_273_pass1.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_caller_bound_pass2.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-guard-over-a-minted-identity.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 12
---
## Outcome

- A synthesized suite holds no scenario that expects an outcome the interpreted target cannot give. Where an input guard holds only if an input equals a value fixed before the run (a caller attribute, or a constant), and that input names a row the run creates under an identity the target mints, synthesis withdraws the scenario that needs it, as `ESS-SYNTH-003` naming the guard and the input. It does not emit it.
- `ess verify conform run --target interpreted` passes every scenario of every suite synthesized from the models in the acceptance table. A correct implementation no longer fails its own suite on this shape.

## Need

Found by the adversary review of the wave of 2026-11-22 (finding F3, `wave-1122/close/adv/REPORT.md`, model `m/caller.yaml`). The most severe of the three: the suite asserts a wrong outcome, so a correct implementation fails it, and a fix to the implementation would make it wrong. Pre-existing: the suites of 0.97.4 and of the integration-branch build (`ess-s3`, which reports 0.97.4) are byte for byte identical on every model below. Models and outputs are under `drafts2/f3/` (paths relative to it); `run.txt` holds every command and output. The reviewer's model is rewritten under generic names.

Smallest model that fails, `m/r5.yaml` (about 40 lines of commands): an actor `Staff` with an attribute `region_id`; `OpenRegion` creates a `Region` under a generated identity; `BookRoom` takes `region_id` and answers

```yaml
- name: foreign
  when: region_id != caller.region_id
  error: lab.booking.OtherRegion
- name: no-region
  when_related: {via: input.region_id, exists: false}
  error: lab.booking.RegionNotFound
- name: booked
  emits: [lab.booking.RoomBooked]
```

```text
$ ess verify conform synthesize --path m/r5.yaml --target ir --out out/r5.<bin>.suite.yaml
r5 0.97.4  synth rc=0 3 scenario(s) (0 authored), 1 refusal(s)        (refused: BookRoom/no-region, ESS-SYNTH-003)
r5 ess-s3  synth rc=0 3 scenario(s) (0 authored), 1 refusal(s)
$ ess verify conform run --target interpreted --path m/r5.yaml --suite out/r5.<bin>.suite.yaml --report-format 2
r5 0.97.4  run rc=1   3 scenarios: 2 passed, 1 failed, 0 error, 0 unsupported
r5 ess-s3  run rc=1   3 scenarios: 2 passed, 1 failed, 0 error, 0 unsupported
  failed lab.booking.BookRoom/outcome/booked: expected "outcome = booked"; observed "outcome = foreign", "error = lab.booking.OtherRegion"
```

The reviewer's shape, `m/full.yaml` (`BookRoom` with `foreign`, `no-region`, `no-room`, `elsewhere`, `booked`, a room creator), fails on the same two binaries in `BookRoom/outcome/no-room` ("expected outcome = no-room, observed outcome = foreign"). The suite sends the same request twice in that scenario, once expecting `foreign` and then `no-room`, under the same caller (steps of `no-room` read from `out/full.new.suite.yaml`: both sends carry caller `...886338c3278a` and `region_id: region-360361`). A request cannot answer differently twice.

Boundary, each varying one thing:

| model | what changes | result on both binaries |
|---|---|---|
| `m/r3.yaml` | `no-region` removed: no related row is read through `region_id` | 5 scenarios, 5 pass |
| `m/r6.yaml` | `OpenRegion` takes `region_id` from its input instead of generating it | 3 scenarios, 3 pass: the arrangement creates the region under the caller's value, `BookRoom/outcome/booked` sends it |
| `m/r8.yaml` | no actor, no caller: `when: region_id != "00000000-0000-4000-8000-0000000000aa"` | `booked` fails the same way (observed `foreign`). The defect is not about callers |
| `m/r9.yaml` | `foreign` declared after `no-region` | refused at validation (`conflicting_declaration`: an input-guarded refusal answers at step 1). "Declared first" is not a variable |

The failing scenario of `r5` (steps read from `out/r5.new.suite.yaml`, listed at the end of `f3/run.txt`): `OpenRegion` ×3 (captures `region-138600..2`), then `BookRoom` with caller `...886338c3278a` and `region_id: region-138601`, the captured instance, expecting `booked`. The interpreter mints the region's identity; it is not `...886338c3278a`, so `region_id != caller.region_id` holds and `foreign` answers.

Which step leaves the caller attribute unsatisfied: the arrangement of the row `no-region` needs present. The `BookRoom` send must name an existing `Region`, so `region_id` is sent as the captured instance, whose identity the target mints. The caller attribute is a literal chosen before the run (`caller.rs`: two callers, `first` and `second`, each one value per attribute, "far from every witness an input is given"). The guard needs `region_id == caller.region_id`; the instance cannot equal a literal fixed earlier. Why synthesis does not see it (read, not run in a debugger, inferred): `related_guard.rs` `with_row_where` decides the branch on an input in which `on_row` has put the row's identity into the `via` field "unchanged where the target mints that identity" (doc comment of `on_row`). With the field left at the candidate the guard search steered to (a value equal to the caller's literal), `foreign` is false and `booked` is selected; at emission `supply` replaces the field with the instance. The decision is made on a literal the send does not carry. The neighbour `no-region` is refused correctly because its own search (`unclaimed`) never finds an input outside `foreign` and reaches the "every such input taken first by `foreign`" refusal.

Self-check, answered. Synthesis runs no scenario against the interpreter: `Interpreted` does not occur in `src/synthesize*` apart from a comment. The suite is checked against the interpreter only in tests. Three kinds: one fixture per fix (`a_caller_attribute_keeps_the_binding_refusal_scenario.rs`, `adversary_caller_bound_pass2.rs`, with the same oracle sentence, "every scenario synthesis writes passes against the interpreted target of the same model"), the committed example systems and fixture files through one differential sweep, `adversary_273_pass1.rs` `sweep_identity_comparisons_never_fail_the_reference_interpreter` (it asserts only that a scenario failing with the identity comparisons also fails without them, never that it passes), and the wave's own fixtures. No test asserts "the interpreter passes every scenario synthesized from every model the repository carries". This shape (an input guard against a caller value, on an input that also names a row the run creates) is in none of them. `adversary_caller_bound_pass2.rs` is the nearest: it fixes a swapped run that keeps the guard when the step also sends a captured instance (`caller.rs` `keeps_literal_guards`), but that code decides only the swapped run, not the main run's choice of input.

## Fit review

- Origin: a defect the review found, not an adopter request.
- Covers F3. It is one defect with two spellings (a caller attribute, a constant). Not a caller feature.
- Considered and rejected:
  - Choosing the caller's attribute equal to the instance. The suite's `caller:` is a map of literals (`scenario.rs` `caller: BTreeMap<String, Node>`); naming an instance there is a suite-format change, kept as option B of D1.
  - Making `foreign` skip the arranged row's identity. The guard is the model's; a scenario cannot be arranged around it.
  - Pinning only the reviewer's model. The generic rule is the cause.

## Design

- Fix direction (D1 A). Where `on_row` leaves a `via` field to a target-minted identity and an input guard reads that field, the guard is not decided on the candidate's literal: it is undecided for that send. A branch selected only through that decision is unselected, and synthesis reports it as `ESS-SYNTH-003` with the cause already used beside it, in the same words as `no-region` ("the input names a row created under an identity the target mints, and `<guard>` needs it equal to a value fixed before the run"). `selects`, `chooses` and `with_row_where` are the places; `caller/arranged.rs` is the second place if the swapped run decides the same way. No new code, no new construct.
- Format consequence (`AGENTS.md` § Determinism and formats): none for the model, no `ess/N`. The suite loses scenarios for models with this shape and gains one `ESS-SYNTH-003` each; every other suite keeps its bytes (the pin table `synthesis-suite-bytes-table.tsv` changes only by the new fixtures' lines, proved with `comm`). The suite version does not change: a scenario that could never pass is removed, no new construct is written. Adopters see a refusal and a lower scenario count for such a model; the census is `ess verify conform synthesize --path <dir> --out <file>` with both binaries and `ess verify diff`-style comparison of the suite lists. A suite pinned with `ess verify conform select` or a known-failure file naming a withdrawn scenario needs the same cleanup as for any removed scenario (inferred; not run).
- The generic self-check (D2). A test that synthesizes every model the repository carries (the `every_model()` set of `adversary_273_pass1.rs`, plus the new fixtures) and requires every scenario to pass the interpreted target, with a short named list of known failures that each name their story. This is the property that would have caught F3, and the first run may show more of them: the story's first step is to run it once and count, read-only, before the fix, and to report the count.
- Cost (`AGENTS.md` test budget): `conform.sh` ran synthesize plus two interpreted runs on a model of about 100 lines in under 0.7 s on both binaries (4 invocations). The sweep reuses the set `adversary_273_pass1` already builds; it must fit one test under 60 s and the step under 120 s, so it builds each IR once and reads a committed table of suite digests where it can. Time it before adding it.

## Acceptance

- `m/r5.yaml`, as the generic fixture `crates/verify/ess-conformance/tests/fixtures/a-guard-over-a-minted-identity.yaml`: synthesis refuses `BookRoom/booked` as `ESS-SYNTH-003` naming `region_id != caller.region_id` and `region_id`; `ess verify conform run --target interpreted` passes every remaining scenario. Test `a_guard_over_a_minted_identity_is_not_assumed_true.rs`.
- `m/r8.yaml` (a constant, no caller): the same.
- `m/full.yaml`: `BookRoom/outcome/no-room` is withdrawn (it is refused beside `no-region`, `elsewhere` and `booked`) and the interpreter passes the 4 that remain.
- Controls keep their suite bytes: `m/r3.yaml` (5 scenarios), `m/r6.yaml` (3 scenarios, `booked` passes), and the S1, S2 and S3 fixtures of the wave. Checked by the byte table.
- The sweep of D2 is green, or red only for a named known failure each with a story id.
- Every `ess verify conform run --target interpreted` in the story's tests is bounded: no test over 60 s, the step within 120 s, reported with the ten slowest.

## Decisions

- D1. How the shape is handled.
  - A (recommended): withdraw the scenario with `ESS-SYNTH-003`, as above. Correctness first, smallest change, matches what its neighbour `no-region` already does.
  - B: arrange it. Allow a caller attribute to name a captured instance (`caller: {region_id: {kind: instance, ...}}`), so the guard can be false. This witnesses `booked` and `no-room`, but it is a suite-format change (a new `ess-conformance/N` and runner support in Rust, Go and TypeScript). A follow-up story after A.
- D2. The self-check.
  - A (recommended): the sweep in the test lane, with a named known-failure list.
  - B: a check inside `synthesize` that runs every scenario against `Interpreted` and withdraws a failing one. Strong, but it adds a run to every synthesis and hides a synthesis defect instead of naming it.
  - C: nothing beyond the fixtures.
- D3. Release path. A (recommended): a wave story; the defect is not adopter-reported and the suite it removes is wrong today. B: a hotfix on request, by the Hotfix section of `AGENTS.md`; it changes suites of existing models, so the notes say so.
