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
revision: 5
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
