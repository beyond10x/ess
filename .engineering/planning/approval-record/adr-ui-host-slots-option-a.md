---
format: aep.planning-md/3
id: approval-record:adr-ui-host-slots-option-a
kind: approval-record
status: approved
title: 'The operator chose option A: host slots in the ess-ui/1 document'
summary: Operator answer "A" of 2026-11-06 ~21:0xZ, relayed verbatim by the coordinator
tags:
- non-interactive
relations:
- decides: architecture-decision-record:a-ui-document-declares-the-host-components-it-does-not-generate
- decides: story:a-ui-document-declares-a-host-slot
- decides: story:a-host-slot-names-the-generator-item-it-stands-in-for
- decides: story:host-covered-rows-are-their-own-count
- decides: story:a-host-slot-reads-and-commands-only-what-it-declares
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-06T21:10:21Z", actor: "human:timo", revision: 2}
---
## Decision

On 2026-11-06 at about 21:0xZ, the operator answered the options table of
architecture-decision-record:a-ui-document-declares-the-host-components-it-does-not-generate
with:

> A

The coordinator relayed the answer verbatim in its message of the same evening. Option A is
**host slots in the document**: one construct in `ess-ui/1` with check rules, a host-covered count
beside covered and uncovered, a registration seam in the React project, and a placeholder region in
the terminal renderer. Options B (a side file) and C (change nothing) are not taken.

## Effect on the plan

- The record moves from proposed to accepted.
- design:ui-host-slots holds the design. Six draft stories follow from the record's sections 1 to 4
  and its Effect section.
- The stories are queued for 0.90.0, ranked after the issue 761 story
  (story:a-list-identity-guard-beside-an-updates-subject-is-witnessed), which leads that wave.
