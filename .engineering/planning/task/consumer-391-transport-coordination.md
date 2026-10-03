---
format: aep.planning-md/3
id: task:consumer-391-transport-coordination
kind: task
status: draft
title: Reconcile channel address template ownership and delivery evidence
refs:
- provider: github
  reference: beyond10x/ess#391
relations:
- decomposes: task:consumer-backlog-20261002
- informed_by: epic:message-contract-clients
revision: 1
---
## Outcome

Track the unclosed channel-address-template request beyond10x/ess#391 under the user's full consumer-backlog objective, with the existing transport implementation owner and an explicit acceptance/evidence disposition rather than losing it outside a batch's scope.

## Current evidence

Fresh read-only GitHub inventory on 2026-10-03 shows #391 open. The request is to bind producer address tokens to payload/context fields and constrain each token to the channel's token grammar; existing consumer delivery context and literal naming.wire do not express that producer operation. This is the request's need, not an accepted new syntax or implemented contract.

epic:message-contract-clients explicitly excludes channel address templates (#391) from its transport/publisher batch. PR402's merge and closure of #390/#392/#395 therefore do not prove #391 done. A full canonical artifact-list/reference join found no exact github:beyond10x/ess#391 reference before this record. That is a tracking gap, not proof that the separate transport session has no private plan.

## Ownership and disposition

All ess-transports implementation remains with the user's designated third Claude session. This coordinator owns backlog reconciliation only for this item; no transport source changes, new implementation story/design, PR or duplicate delivery are authorized here. The full-backlog instruction keeps #391 outstanding even though the earlier release batch excluded it. It is not permanently deferred by this record.

The next owner action is to supply or link the transport lane's accepted fit/design and planning artifact for this exact request, then implement/validate it in that lane's coordinated delivery. If a plan already exists, relate this record to it rather than create a second implementation. No intake receipt, accepted syntax, gate success or released support is currently established here. Cross-session messenger capability is unavailable in this session, so this local record is not presented as a delivered request or owner acknowledgement.

## Acceptance

The transport owner and shared integrator provide an explicit evidence-backed disposition for #391 linked to its accepted plan and source/runtime tests, with release evidence if implemented, while the canonical backlog retains the issue until that disposition is verified.

## Retained snapshots

Private canonical target/backlog-input/open-issues-20261003-reconciled.json and aep-inventory-20261003-reconciled.json capture the issue and prior missing reference; open-issue-aep-map-20261003.json captures the corrected CLI-string-reference join. No GitHub mutation occurred.
