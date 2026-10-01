---
format: aep.planning-md/3
id: review-result:adversary-gaps-265-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-265
relations:
- reviews: story:feature-request-265
revision: 1
---
unit: story:feature-request-265, tree gaps-265
verdict: NEEDS-CHANGE, red 4

- blocker: a command granted to no actor was sent as no actor, which served contracts refuse with 403, so a served model could not pass its own suite.
- warning: denied scenarios read only the refused command's direct events, so a target that ran and published before refusing passed.
- warning: each granted command was sent only by the lowest-named actor holding it.
- warning: denied scenarios were synthesized for in-process components.
- warning: checks_grants keyed on declared actors, so unserved billing's OpenAPI declared a 403.
- notes: Go admit named an undeclared actor where Rust answers null; the contract did not say views are open; gatepass committed-suite targets did the grant check in test code.

Correction 1: GrantedToNoActor note and withheld scenarios; log check via an unpublished list in Rust, Go and TS; admitted-actor rotation; denied scenarios only on served components; grants_checked_on per component; Admit answers null; contract text; gatepass drivers call the generated admit.
Tests: tests/adversary_265_pass1.rs (4).
