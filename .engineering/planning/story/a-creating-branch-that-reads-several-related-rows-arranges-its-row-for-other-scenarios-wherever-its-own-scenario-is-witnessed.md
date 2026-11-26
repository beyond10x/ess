---
format: aep.planning-md/3
id: story:a-creating-branch-that-reads-several-related-rows-arranges-its-row-for-other-scenarios-wherever-its-own-scenario-is-witnessed
kind: story
status: active
title: A creating branch that reads several related rows arranges its row for other scenarios wherever its own scenario is witnessed
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_creator_reading_several_rows_is_driven.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-owned-rows-is-driven.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-rows-is-driven.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T20:21:30Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-22T20:21:32Z", actor: "human:timo", revision: 11}
---
## Outcome

- A creating branch that reads two or more related rows, one of which is compared with the input,
  is driven as a setup step. Every other scenario that needs the row it creates (a move of that row,
  or the row a second command reads beside the row its scenario is arranged around) gets that
  row. Two shapes: a row beside has a predicate comparing a stored field with an input
  (`tag != input.tag`), and the link of the row the creation is filed under is compared with the
  input that names the other row (`site_id != input.site_id`, where `Site` owns `Desk` and `Desk`
  owns `Grant`).
- Where such a run still cannot be arranged, the refusal names the real cause: the row and the
  predicate that stopped the run. It does not say "which no input reaches" for a branch whose own
  scenario is in the same suite. It does not print "no candidate of the 1 tried" when that "1" is a
  constant and not a count.
- No format version and no `ValidationCode` change.

## Need

Reported by an adopter: a downstream suite has 119 synthesis refusals (29 ESS-SYNTH-003, 86
ESS-SYNTH-004, 4 ESS-SYNTH-008). The adopter read the cause as "synthesis cannot arrange a second
related row", because the refusal of the command that needs that row says "beside the row the
scenario is arranged around" and "1 candidate tried". Splitting that command's `any:` guard into
single predicates did not help.

What we reproduced on the released 0.97.3 and on the 0.97.4 build (both give the same output):

- The command with two related rows is not the cause. With the second row's creator simplified,
  the same command synthesizes every branch. `m-g2/model.yaml` gives
  `16 scenario(s) (0 authored), 0 refusal(s)`. The `any:` makes no difference: in
  `m-g5/model.yaml` it is a single predicate, and the output is the same as `m-g15`.
- The cause is the second row's creator, `IssueGrant`. It reads two related rows, `Site` through
  `input.site_id` and `Desk` through `input.desk_id`, and one branch compares a row with the input:
  - `m-g10/model.yaml`: the `Desk` row is the row beside, and its predicate reads the input
    (`wrong-tag: when_related: {via: input.desk_id, predicate: tag != input.tag}`);
  - `m-g15/model.yaml`: `Site` owns `Desk` and `Desk` owns `Grant`, and
    `elsewhere: when_related: {via: input.desk_id, predicate: site_id != input.site_id}` links the
    row the grant is filed under to the input that names the other row. This is the adopter's
    shape.

  In both models every `IssueGrant` branch has its own scenario (`IssueGrant/outcome/issued` is in
  `suite.json`), and every scenario that needs a `Grant` is refused (`m-g15/run.txt`, the same in
  `m-g10/run.txt` and in `m-g10/on-0973/run.txt` on 0.97.3):

  ```text
  refused: refusal[ESS-SYNTH-003]: outcome shop.desk.ReceiveTicket/received has no scenario `shop.desk.ReceiveTicket/outcome/received`
    no candidate of the 1 tried satisfies `a row of `shop.desk.Grant` for `input.grant_id` of `shop.desk.ReceiveTicket` on which no branch answering before `received` is selected, beside the row the scenario is arranged around`
  refused: refusal[ESS-SYNTH-004]: outcome shop.desk.RevokeGrant/revoked has no scenario `shop.desk.RevokeGrant/outcome/revoked`
    it needs an instance of `shop.desk.Grant` to move along `revoke`, and the route runs through `shop.desk.IssueGrant/issued`, which no input reaches
  10 scenario(s) (0 authored), 7 refusal(s), written to suite.json
  ```

  `ReceiveTicket`'s `no-clerk` and `ignored`, `grant-refused`, `RevokeGrant`'s transition and its
  `Revoked` state scenario go with them.
- Where it is cut, at `e29183b7b9` (wave tree):
  - The command's own scenario is arranged by `related_guard::arranged` (`related_guard.rs`
    3529-3588). It admits rows beside without an input (`beside(.., (BESIDE, false))`), chooses
    the input, and arranges each row beside again for that input (`steered_beside`, 3651-3698).
  - A setup run of the same branch goes through `drive` → `drive_several` (2796-2906). That path
    arranges each row beside strictly, for every input (`beside_row(.., (at, true))`, 2824-2830),
    and gives up at 2842-2846 when a predicate over that row reads the input. It re-arranges a
    row only for an input the caller already chose (`resteer_beside`, 2853-2861), and a setup run
    has none. It also shares no owner between the row it is arranged around and the row beside.
  - `created_owned_made` (`synthesize.rs` 8758-8774) and `invoke` (8969-8983) turn any error of
    that run into `Unreachable::Unwitnessable`, which prints "which no input reaches".
  - `beside_refused` (`related_guard.rs` 1477-1492) always reports `tried: 1`.
- Documented limit: `docs/design/cross-record-and-stored-field-guards.md` 746-750, § "Several
  related rows": "A driver running such a command arranges the other rows first, each where no
  branch answering before the driven one can hold whatever the input; a row no such arrangement
  reaches — a predicate reading the input — refuses the run naming `arrange_related_row`". The run
  is refused as the page says. The text the user sees is a defect: it names neither
  `arrange_related_row` nor the predicate, and it claims a branch with its own scenario is
  unreachable.
- Neighbour, already planned and not taken over here:
  - `story:a-creator-under-a-row-set-and-when-is-arranged`, and the follow-up named in the
    coordinator section of `story:a-row-set-refusal-is-witnessed-beside-the-related-row-its-creator-needs`
    (`related_guard::selects` treats a row-set branch as always selected). Same symptom (the
    branch's own scenario is witnessed but "no input reaches" it as a setup step), different cause.
- Neighbour, also planned:
  - `story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed`: the creating
    branch's own scenario fails when the created row is not owned by the linked row
    (`m-g14/model.yaml`; its repro `m-ownerlink/model.yaml` still refuses `Bind/bound` on the
    0.97.4 build).
  - The same failure also happens with a `references` link whose creator refuses a missing row,
    with no `owns` (`m-g3/model.yaml`, one related row; `m-g13/model.yaml`): see D3.

## Fit review

Decision: **accept, redesigned**. The adopter asked for "a story that arranges a second related
row". Synthesis already arranges a second related row (`m-g2`). What it cannot do is drive that
row's creator when the creator itself reads several rows and one predicate reads the input.

1. **Need.** Every scenario that needs a row created by a branch reading several related rows,
   one of them compared with the input, gets that row. Brand-free reproductions: `m-g10`, `m-g15`
   (above). The requester's own framing was "a second related row beside the row the scenario is
   arranged around". That is evidence of the symptom, not the design.
2. **Class.** It is a defect in two parts. The setup path refuses what the branch's own scenario
   arranges: `drive_several` against `arranged`, both in `related_guard.rs`. And the refusal
   contradicts the design page: it should name `arrange_related_row`, and instead it says "no input
   reaches" a witnessed branch. The limit itself is documented (page 746-750), so lifting it is a
   gap closed in synthesis, with no authored surface.
3. **Already expressible?** The model validates. Two rewrites synthesize today, and neither one
   answers the need (skill question 3):
   - Declare the row whose predicate reads the input first, so that it becomes the row the run is
     arranged around. `m-g11/model.yaml` gives `17 scenario(s) (0 authored), 0 refusal(s)`. This
     works only when one row reads the input and no `owns` decides the focus first
     (`related_guard.rs` `focus` 1065-1083, `owns_through` 125-147). Moving declarations also
     changes the order in which missing rows answer.
   - Make the comparison an `external:` refusal. `m-g16/model.yaml` gives
     `17 scenario(s) (0 authored), 0 refusal(s)`. This moves a checked rule out of the
     specification.
4. **Fit.** No new vocabulary. The setup path takes the arrangement the command's own scenario
   already uses: `beside` non-strict, the input chosen with the rows beside, those rows arranged
   again for it (`steered_beside`), and the rows of one command sharing their owner (as
   `story:related-rows-of-one-command-share-their-owner` does for a command's own scenario). The
   siblings are the other routes through a creating branch: a move of the created row
   (ESS-SYNTH-004), the row beside another command (ESS-SYNTH-003), and the row a cascade or an
   aggregate creates. All of them go through `created_owned_made` or `invoke`, so all of them gain
   it. Targets: synthesis only. The interpreter, generated code, `ess verify diff` and the formats
   are untouched.
5. **Second adopter.** A ticket that needs a grant issued to a desk of the site the caller names.
   A shipment that needs a slot reserved on a dock of the warehouse the caller names. An order
   line that needs a price filed under the catalogue the caller names.
6. **Cost.** Synthesis only. No format and no diagnostic code. Suites of models with this shape gain
   scenarios. Rows of `synthesis-suite-bytes-table.tsv` move only for those models, plus every
   model whose refusal text for a setup run changes.
7. **Alternatives.** (a) Change nothing: adopters write `external:` or reorder declarations, and
   lose a checked rule or change the order of answers. (b) Lift `strict` alone in `drive_several`:
   that fixes `m-g10` but not `m-g15`, whose rows must share an owner. (c) Reuse the own-scenario
   arrangement (chosen, D1).

## Design

- In `drive_several`, where a row beside cannot be arranged for every input, or where the
  caller chose no input: arrange as `arranged` does, at the driver's distinction. Admit the rows
  beside without an input, choose the input with them (`Around`), arrange each row whose guards
  read the input again for that input (`row_for`), and file the row the run is arranged around and
  the rows beside under one owner where an `owns` link is compared with the input. The run is then
  sent with the chosen input.
- `created_owned_made` and `invoke` keep the cause of a failed setup run. Where the branch has a
  witness of its own, the refusal is `Unreachable::StoredReference`-like: it names the branch, the
  row and the predicate (`arrange_related_row`) instead of `Unwitnessable`. `beside_refused`
  reports the rows it searched, or no count at all.
- The § "Several related rows" paragraph of `docs/design/cross-record-and-stored-field-guards.md`
  (746-750) is changed to say what a driver now arranges, and what it still refuses.
- Format: none. Meaning, identity, references, canonicalization and names are unchanged
  (`AGENTS.md` § Determinism and formats). Only synthesized suites change.

## Acceptance

- New test `crates/verify/ess-conformance/tests/a_creator_reading_several_rows_is_driven.rs` with
  fixtures shaped as `m-g10` and `m-g15`:
  `ess verify conform synthesize` prints `0 refusal(s)` for each. The suite holds
  `RevokeGrant/outcome/revoked`, `ReceiveTicket/outcome/received`, `ReceiveTicket/outcome/no-clerk`
  and `Grant/transition/revoke/...`. `ess verify conform run --target interpreted` passes every
  scenario. Before the change each fixture prints the two refusals quoted in Need.
- Same test: in the `m-g15` fixture's `RevokeGrant/outcome/revoked` scenario, the `IssueGrant`
  step sends a `site_id` equal to the site the desk it names is filed under.
- Same test: a fixture whose row beside can be arranged for no input (a contradictory predicate
  pair over it) is refused. The refusal names the creating branch, the row and
  `arrange_related_row`, and contains neither `which no input reaches` nor `of the 1 tried`.
- `cargo test -p ess-conformance --locked --test related_guard_multiple_vias` and
  `--test adversary_283_pass1` keep their verdicts.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes. Every row it
  re-pins is a model with a creating branch reading several related rows, or one whose setup-run
  refusal text changed, and the cause is named beside the pin.
- `task test-xtask` passes (`CHANGELOG.md` entry).

## Decisions

- D1. How a setup run of such a branch is arranged: (A) `drive_several` falls back to the
  own-scenario arrangement (`arranged` with `steered_beside`, plus owner sharing) at the driver's
  distinction; (B) the branch's own witness is remembered and replayed, re-addressed, as the setup
  run; (C) only `strict` is lifted. Recommended: A. B needs a re-addressing step that does not exist
  yet. C leaves the adopter's shape (`m-g15`) refused.
- D2. Scope of the truthful refusal: (A) every setup run whose branch has its own scenario keeps the
  driver's cause in place of `Unwitnessable`, whatever the cause (row set, several rows, other);
  (B) only setup runs of branches reading several rows. Recommended: A. The row-set stories then
  print their real cause until they land, and no other story owns `created_owned_made`'s mapping.
- D3. The neighbour where the creating branch's own scenario also fails, for a `references` link
  whose creator refuses a missing row (`m-g3`, `m-g13`): (A) extend
  `story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed` to that link; (B) a story
  of its own; (C) fold it in here. Recommended: A, since it is that story's own-scenario search
  (`subject_fact::naming_owners`) and not this story's setup path.
- D4. Order in the wave: this story edits `related_guard.rs`, which
  `story:a-creating-branch-sends-every-row-it-creates-under-its-own-identity` (in this wave) edits
  in `drive`. Recommended: start after that unit merges, beside the row-set follow-up, which edits
  `selects` in the same file.

## Downstream report 2026-11-26

A downstream application reported on 0.99.2 (gap 5 of a report of ten) that conformance synthesis
refuses 55 scenarios because it cannot set up rows whose creating command needs another row first.
The report gives the count, not a reduction, codes or guard shapes. 0.99.3 and 0.99.4 changed this
area (the setup of a creating command that reads several related rows is built once per set of
values; scenarios whose setup the model itself refuses are no longer written, `CHANGELOG.md`
0.99.3 and 0.99.4), so the count on `main` may differ. The first step is a reduction of the
reported refusals, synthesized on `main` and sorted by code and cause; a cause this story's outcome
does not cover gets a story of its own.
