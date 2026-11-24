---
format: aep.planning-md/3
id: story:the-process-layout-s-listen-inbox-and-peer-take-a-unix-domain-socket-address-unix-path
kind: story
status: draft
title: The process layout's `--listen`, `--inbox` and `--peer` take a Unix domain socket address `unix:<path>`
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#792
relations:
- serves: vision:O2
- depends_on: story:each-component-runs-in-a-process-of-its-own
revision: 1
---
## Outcome

Item 3 of beyond10x/ess#792 (local inter-process communication) as an address form. In the process
layout of `story:each-component-runs-in-a-process-of-its-own`, the options `--listen`, `--inbox` and
`--peer` of a generated component server take a Unix domain socket address `unix:<path>` where they
take a TCP address today.

## Decisions

- D16-1 (decided: C, 2026-11-24): #792 closes in the one pull request. Step 16 carries increment 1
  of the branch design; this story builds item 3 of the request, local IPC, as the address form
  `unix:<path>`. Everything else of increments 2 to 4 is not asked for by the issue and is not in
  this pull request.
- S14 (open: security, with the owner): who may reach a Unix domain socket address of `--inbox`,
  and with what file permissions, is the open point D16-3 of step 16 (a carried invocation runs as
  the system acting, with no grant check). This story is not accepted and not built until S14 is
  answered.

## Open

Acceptance and scope are written when S14 is answered.
