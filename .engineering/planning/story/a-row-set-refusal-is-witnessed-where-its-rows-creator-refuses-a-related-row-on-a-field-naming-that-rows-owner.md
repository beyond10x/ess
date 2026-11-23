---
format: aep.planning-md/3
id: story:a-row-set-refusal-is-witnessed-where-its-rows-creator-refuses-a-related-row-on-a-field-naming-that-rows-owner
kind: story
status: draft
title: A row-set refusal is witnessed where its rows' creator refuses a related row on a field naming that row's owner
tags:
- defect
- from-finding
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_a_creators_owner_predicate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-an-owner-predicate.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 9
---
## Outcome

- Take a row-set refusal (`when_related: {entity: E, where: <field> == input.<x>, exists: true}`) whose rows' creator reads a related row through a `predicate:` that compares a field naming that row's owner with a plain input of the creator, and where that related row has an owner. The refusal's scenario is witnessed: rows are arranged under the creator's guards, and the `ess verify conform` scenario passes the interpreted target.
- It is no longer refused as `ESS-SYNTH-001` ("the command creating them refuses, on its stored fields, the related row the input the selector compares names").

## Need

Found by the adversary review of the wave of 2026-11-22 (finding F4, `wave-1122/close/adv/REPORT.md`, model `m/combo.yaml`), pre-existing: 0.97.4 and the integration-branch build (`ess-s3`, which reports 0.97.4) refuse it the same way, and their suites are identical on every model below. The report words it as "a row-set refusal over rows whose creator reads two related rows". The runs below show the count of related rows is not the trigger. Models and outputs under `drafts2/f4/` (paths relative to it); `run.txt` holds every command and output.

Smallest failing model, `m/c12.yaml`: `Region` owns `Room`, `Room` owns `Booking`. `BookRoom` (input `region_id`, `room_id`) answers `no-room` (`when_related: {via: input.room_id, exists: false}`), `elsewhere` (`when_related: {via: input.room_id, predicate: region_id != input.region_id}`) and `booked` (creates a `Booking` setting `room_id`). `CheckRoom` (input `room_id`) answers `no-room`, `busy` (`when_related: {entity: Booking, where: "room_id == input.room_id", exists: true}`) and `free`.

```text
$ ess verify conform synthesize --path m/c12.yaml --target ir --out out/c12.<bin>.suite.yaml
c12 0.97.4  synth rc=0 7 scenario(s) (0 authored), 1 refusal(s)
c12 ess-s3  synth rc=0 7 scenario(s) (0 authored), 1 refusal(s)
refused: refusal[ESS-SYNTH-001]: outcome lab.booking.CheckRoom/busy has no scenario `lab.booking.CheckRoom/outcome/busy`
  no witness: ... created by `lab.booking.BookRoom`, whose `elsewhere` (`when_related: {via: input.room_id, predicate: region_id != input.region_id}`) refuses the `lab.booking.Room` arranged for the rows to name: no arrangement tried, resting in `Open`, with an input steered toward passing the creator's guards passes them all, and the closest leave it refusing
```

The other 7 scenarios pass the interpreted target on both binaries. Boundary, each varying one thing (all on both binaries):

| model | what changes | refusals |
|---|---|---|
| `m/c12-noelsewhere.yaml` | `elsewhere` removed | 0: `CheckRoom/busy` is witnessed |
| `m/c7.yaml` | `Region` removed: the room carries a plain `region_id` that nothing owns | 0 |
| `m/c4.yaml` | `Region` kept and `BookRoom` also reads it (`no-region`), `elsewhere` removed | 0, with two related rows read by the creator |
| `m/c12.yaml` | owner kept, `elsewhere` kept | 1 (`ESS-SYNTH-001`) |

So the trigger is a predicate of the creator over the related row's owner field, compared with an input that is not itself named as a row. The reviewer's `IssueGrant` has exactly that (`elsewhere: site_id != input.site_id`, beside `no-site` and `no-desk`).

Cause, read in `row_set.rs` `guarded_creator`, `rows_tried`, `left_refusing`, `steered_inputs` and `related_guard.rs` `with_row_where`; not run in a debugger (inferred). The refusal text is `ROWS_REFUSING_CREATOR`. The row-set path arranges the related row (the room) through its own creators, which also arrange its owner, so the room's `region_id` holds the minted identity of the region created for it. It then steers the creator's input by the room's stored fields. The input `region_id` is a plain input: steering gives it a candidate value, and nothing names it as the arranged owner, so `region_id != input.region_id` holds on every arrangement and `elsewhere` refuses, the closest arrangement included. The creator's own scenario, `BookRoom/outcome/booked`, is witnessed on the same model (`m/c12.yaml`, 7 scenarios, `booked` among them): the own-scenario path sends the owner as the captured instance (`subject_fact::naming_owners`, beyond10x/ess#271, "decided only on an input naming an arranged owner"). The row-set path has no such step.

This is a different gap from the existing draft `story:a-creator-guarded-by-a-related-row-and-a-row-set-drives-its-rows-for-another-commands-arrangement`. That one is a creator that itself reads a row set, decided in `related_guard.rs` `selects`. Here the creator reads no row set, and the code that fails is the row set's own search for rows (`row_set.rs`). Both touch `row_set.rs` and `related_guard.rs`: they run in series (D2).

Related to `story-2` of the same review (a minted identity that an input must equal), but there the arrangement cannot make the guard true; here it can, because the input is an ordinary input that a suite step may send as `{kind: instance}`.

## Fit review

- Origin: a pre-existing gap the review found, not an adopter request. It is the creator-side gap S2's fix (`ROWS_REFUSING_CREATOR`, beyond10x/ess#682) leaves: the related row arranged is steered, the creator's input is steered, but an input that must equal an arranged owner is not named as one.
- Covers F4. It does not cover the reviewer's wording ("two related rows"); `m/c4.yaml` is the evidence.
- Considered and rejected:
  - Adding F4 to the existing draft. The cause and the code differ (above), and that draft's fixture (`row-set-refusal-beside-a-moved-row.yaml`) does not hold an owner predicate in its creator.
  - Refusing it with a better message. The scenario is arrangeable.
  - Excluding the `elsewhere` branch from the creator's guards. A row the creator refuses does not exist.

## Design

- In the row set's search (`row_set.rs` `steered_inputs`, `guarded_creator`), where a creator's related predicate compares a field naming the related row's owner with a plain input, the input is sent as the captured instance of that owner, as `naming_owners` does for the own scenario, before the creator's guards are decided on the arrangement. The creator is then sent under the owner's identity, `elsewhere` does not hold, and the rows are created.
- Where no arrangement of that kind exists, the refusal stays `ESS-SYNTH-001`, with the sentence reworded to name the owner field: "the input `region_id` would have to name the owner of the arranged `Room`".
- No new code, no new construct, no format version (`AGENTS.md` § Determinism and formats: nothing persisted changes). Suites of models with this shape gain the scenario and lose a refusal; every other suite keeps its bytes (byte table changes only by the new fixture's lines, proved with `comm`).

## Acceptance

- `m/c12.yaml`, as a generic fixture in `crates/verify/ess-conformance/tests/fixtures/`: 8 scenarios, 0 refusals, `CheckRoom/outcome/busy` among them, every scenario passing `ess verify conform run --target interpreted`. Test file `a_row_set_branch_arranges_rows_past_a_creators_owner_predicate.rs`.
- `m/c12-noelsewhere.yaml`, `m/c7.yaml` and `m/c4.yaml`: suite bytes unchanged (they are controls).
- The reviewer's shape (`m/combo.yaml`, with the other row sets of that fixture kept) loses its `CheckDesk/busy` refusal; the three pin tables change only by the new fixture's lines. `stored_rows_decided_once_work` keeps its bounds.
- Tests are bounded: none over 60 s, the step within 120 s; the new test's time is reported if over 5 s.

## Decisions

- D1. How the input is named.
  - A (recommended): reuse `naming_owners`' step in the row-set search; the creator's input is the owner instance.
  - B: widen the candidate values `steered_inputs` tries until one equals the owner's identity. It cannot work for a minted identity and is rejected.
- D2. Order against `story:a-creator-guarded-by-a-related-row-and-a-row-set-drives-its-rows-for-another-commands-arrangement`.
  - A (recommended): after it. It rewrites the nearby `selects` and the text of the refusal; this story then reads the result.
  - B: before it. Both touch `row_set.rs`; they conflict on merge, not in design.
