---
format: aep.planning-md/3
id: approval-record:ess-0900-wave
kind: approval-record
status: approved
title: 'ESS 0.90.0 runs wave-2026-11-07-ess: issue 761, host-slot stories 1-2 and two runtime fixes in three units (five-story cap)'
summary: Coordinator approval of 2026-11-06 under the operator grant, with units and build terms
tags:
- non-interactive
relations:
- decides: story:a-list-identity-guard-beside-an-updates-subject-is-witnessed
- decides: story:a-ui-document-declares-a-host-slot
- decides: story:a-host-slot-names-the-generator-item-it-stands-in-for
- decides: story:host-covered-rows-are-their-own-count
- decides: story:a-host-slot-reads-and-commands-only-what-it-declares
- decides: story:a-null-tag-draws-no-chip
- decides: story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id
- decides: story:a-column-may-show-a-plain-string-row
- decides: story:a-client-paged-collection-offers-sizes-and-a-range
- decides: story:a-guard-over-several-list-literals-is-witnessed
revision: 10
transitions:
- {from: "draft", to: "approved", at: "2026-11-06T22:18:50Z", actor: "human:timo", revision: 2}
---
## Decision

On 2026-11-06, the coordinator approved ESS 0.90.0 under the operator's grant (class C: a wave,
bot commits, one bot pull request and its merge on green, a release through ess's own process).
In its words:

> ESS 0.90.0 is approved: record the approval in your store.
> - Rank 1: issue 761.
> - Then the host-slot stories, starting with the section-1 story. Accept them on Timo's "A"
>   (approval-record:adr-ui-host-slots-option-a).
> - Then the gap items B2-19, B4-7, B4-40, B3-9, B4-19, B2-3 and the two runtime fixes, as
>   surfaces allow.
> - Cap: 4 units per wave. store/ess-0.89.0-close rides this wave's PR.
> - Follow the same delivery rule and build rules as 0.89.0.

## Units (wave-2026-11-07-ess)

| unit | stories | surface |
|---|---|---|
| U1 | story:a-list-identity-guard-beside-an-updates-subject-is-witnessed (issue 761) | `ess-synth` scenario planner, `ess-conformance` tests |
| U2 | host-slot stories 1-4, in order: a-ui-document-declares-a-host-slot, a-host-slot-names-the-generator-item-it-stands-in-for, host-covered-rows-are-their-own-count, a-host-slot-reads-and-commands-only-what-it-declares | `ess-ui` model, schema, `ess-ui-check`, reference page |
| U3 | story:a-null-tag-draws-no-chip, story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id | React runtime templates (`fields`, `data`, `core`), terminal tags display |
| U4 | story:a-column-may-show-a-plain-string-row, story:a-client-paged-collection-offers-sizes-and-a-range | `ess-ui` Field and Pager, `ess-ui-check`, React `collection` and `display`, terminal |

U2 and U4 both touch `crates/ui/ess-ui/src/model.rs`, `ess-ui-check` and the `ess-ui/1`
reference page, in different structs and sections. U3 and U4 both touch `core.tsx.tmpl`: U3 the
section read, U4 `display`. Conflicts are resolved at merge into the integration branch.

## Gap items

- B2-19 (G-14, a choice placeholder) is already shipped: `Choice.placeholder`. No unit.
- B3-9 (G-20, a client substring filter over loaded rows) is answered by 0.89.0's `contains` row
  filter. No unit.
- B2-3 (A06, tags over objects and a dotted path into a list) was fixed in 0.87.0 (`tagText`,
  `getPath` over lists). The remaining tags defect is the null chip, which is U3.
- B4-40 (G-51) and B4-7 (G-27) are U4, with the lead's grammar decisions recorded in their
  stories.
- B4-19 (G-37: a confirm naming its row, and a confirm on form submit) needs a grammar design and
  moves to 0.91.0.
- Host-slot stories 5 and 6 (the React registry and the terminal region) depend on story 1 and go
  to 0.91.0 as its head.

## Terms

- Each unit is limited to 30 minutes of cargo, with a hard stop at 60.
- U4 is the first out if a unit reaches its stop limit.
- There is an adversary pass per unit before its merge.
- One local gate, one bot pull request (squash), merge on green, tag after main's CI is green,
  release, verify-release.
- Disk rule of 2026-11-06 17:54Z: `target/` is kept between the steps of a running wave or gate,
  and a tree is cleaned when it merges or when a disk notice names it.
- `build-slot.sh` caps concurrent cargo at 3. Under host I/O pressure, cargo runs one at a time
  under `nice -n 10 ionice -c3` with `RUST_TEST_THREADS=4`.

## Amendment: five-story cap

The ess `AGENTS.md` caps a wave at five stories, and this record first listed nine. The wave is cut
to five stories in three units:

- U1: issue 761.
- U2: host-slot stories 1 and 2.
- U3: the two runtime fixes.

These four stories are accepted and active, but are not delivered by this wave. They lead 0.91.0,
tagged queue-0.91.0:

- story:host-covered-rows-are-their-own-count
- story:a-host-slot-reads-and-commands-only-what-it-declares
- story:a-column-may-show-a-plain-string-row
- story:a-client-paged-collection-offers-sizes-and-a-range

Host-slot stories 5 and 6 and gap B4-19 follow them.

## Amendment: row filter out, object tag drafted

On 2026-11-07, the coordinator decided under the operator's grant:

- story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id leaves this wave. It was
  not reproduced on 0.89.0; the repro attempts are in a patch kept by the lead. The reporter is
  asked for its exact document and data.
- The object-tag gap the U3 implementor found becomes a draft:
  story:the-terminal-draws-an-object-tag-by-its-label.
- U3 goes to the adversary on story:a-null-tag-draws-no-chip only.

The wave now delivers four stories: issue 761, host-slot stories 1 and 2, and the null tag chip.

## Amendment: adversary decisions

The coordinator decided the following under the operator's grant on 2026-11-07.

**U1 (issue 761):**

- The adversary's blocker is fixed in the unit: the creating step must send a list that satisfies
  its guard when that branch is the only route.
- The forall guard and the several-literal guard are pre-existing, and are drafts:
  story:a-forall-list-guard-is-witnessed and story:a-guard-over-several-list-literals-is-witnessed.

**U3 (null tag chip):**

- All four adversary findings are fixed in the unit before it merges:
  - the PLAIN_PROJECT re-pin it introduced;
  - a picked suggestion saving the text "null";
  - the input showing an empty item for a held null;
  - a held null hiding a suggestion named "null".
- Finding 2 undoes the story's outcome after a save, so the story is not done without it.

**U2 (host slots 1-2):**

- All five adversary findings are U2's own and are fixed in the unit before it merges:
  - the host name is held to the node-name pattern;
  - `reason` is a prose key;
  - a page slot drops its kind's header refs;
  - the reason is trimmed in the report;
  - a leading byte-order mark in the hosts file is stripped.
- A reused template holding a slot stays refused as "declared twice" in 0.90.0, the fail-closed
  reading of the ADR's section 1. The draft story:a-host-slot-in-a-reused-template-is-one-registration
  takes it to 0.91.0.

## Amendment: U1 merged, U4 several-literal guard

On 2026-11-07, the coordinator decided under the operator's grant:

- U1 merges into the integration branch (d91b3a039c2). Its fix round (0d69cf8f32a) also fixed the
  vacuous forall guard. story:a-forall-list-guard-is-witnessed is therefore absorbed into the issue
  761 story and archived, and its adversary case passes.
- The several-literal guard would ship as an added `#[ignore]` in the 0.89.0..0.90.0 diff, and
  verify-release fails on that. story:a-guard-over-several-list-literals-is-witnessed is taken
  into this wave as U4, which removes the ignore. U4 works in tree ess-w090-u1, on branch
  unit/w090-u4-literals.
- If U4 fails its gate twice on one cause, 0.90.0 ships with the ignore and a recorded verify
  FAIL, and the coordinator is told.

The wave now delivers five stories in four units:

- issue 761;
- host-slot stories 1 and 2;
- the null tag chip;
- the several-literal guard.

## Amendment: U4 trimmed cases

The U4 adversary left three cases red:

- two lists;
- a leaf beside the literals that its base value refutes;
- `not exists` never tested.

On 2026-11-07, the coordinator accepted moving them to
story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation, on one condition: they
must predate U4. The lead runs the three cases on base d91b3a039c2, without U4.

- If all three fail the same way there, they predate U4. The draft is ranked first for 0.91.0, and
  U4 merges.
- If any passes on base, U4 broke it, and U4 fixes it before merging.

The third case means a target that ignores the negated child of a list guard passes the suite. The
0.90.0 CHANGELOG names it under Known limitations.

## U4 base run

The lead ran the three trimmed cases on base d91b3a039c2 without U4, one cargo, niced. All three fail
there the same way:

- **Two lists:** ESS-SYNTH-003 on the guarded branch, and four ESS-SYNTH-022 on `updated`.
- **Leaf beside the literals:** ESS-SYNTH-003 and ESS-SYNTH-022.
- **`not exists`:** the mutant that drops the negated child passes every scenario (10 run).

They predate U4. The draft is tagged queue-0.91.0 and rank-1, and U4 merges.

## 0.91.0 ranking

The coordinator's 0.91.0 ranking of 2026-11-07 is recorded as `rank-N` tags beside `queue-0.91.0`:

1. story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation
2. story:the-react-project-registers-each-declared-host-slot
3. story:the-terminal-draws-a-host-slot-as-a-labelled-region
4. story:generated-rust-types-pass-rustfmt-and-clippy (spindle wave 12)
5. story:an-enum-value-starting-with-a-digit-keeps-its-name (spindle wave 12)
6. story:an-optional-list-view-parameter-is-read-as-published (spindle wave 12)
7. story:validate-bundle-enforces-a-string-equality-invariant (spindle wave 12)
8. story:host-covered-rows-are-their-own-count
9. story:a-host-slot-reads-and-commands-only-what-it-declares
10. story:a-host-slot-in-a-reused-template-is-one-registration
11. story:a-column-may-show-a-plain-string-row
12. story:a-client-paged-collection-offers-sizes-and-a-range

The two generator stories are ranked 2 and 3 because the 0.90.0 generators refuse a host slot with
"host slots are generated from 0.91.0". Under the four-unit cap, a lower rank would break that
promise.
