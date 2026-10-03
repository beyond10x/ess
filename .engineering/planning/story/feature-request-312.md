---
format: aep.planning-md/3
id: story:feature-request-312
kind: story
status: active
title: Suites state their empty-target assumption and act across callers
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#312
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: docs/design/scenario-initial-state-and-cross-caller-witnesses.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T19:07:02Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-10-03T19:07:02Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

Suites either hold on a shared target or say they need an empty one, and some scenario acts on a row as a different caller than arranged it.

## Origin

beyond10x/ess#312, found in the #287 adversary pass; pre-existing on 0.49.0.

## Fit review

The accepted canonical intake record at consumer-backlog commit `acb88e97d3c99587c3b0a501314ce35d73fa4f3b` and its retained continuation classifies this as a defect in suite claims and cross-caller evidence. The suite must declare an empty logical modeled namespace before each scenario and must arrange and act on the same row as different declared callers where the source permits it. No new source keyword or implicit physical database deletion is introduced. Actor/caller authority, source refusal precedence and existing composition restrictions remain intact. The binding implementation contract is `docs/design/scenario-initial-state-and-cross-caller-witnesses.md`.

The coordinator accepted initial-state provenance in the unreleased suite34/35 pair; formats1-33 retain unspecified initial-state semantics and their old bytes. CountReport/2 remains closed. Fresh ordinary/coverage synthesis selects34/35. Historical format-specific tests must explicitly construct legitimate legacy documents without new provenance when testing legacy admission, while current-emission tests assert34/35 and typed Empty. A report/1 writer cannot accept a newly synthesized suite34; modern tests must use exact-suite-bound report/2 without rewriting genuine Failed/Skipped/Unsupported categories.

The current bundle imports the production source, but final combined verification remains incomplete. This continuation reconciles the stale draft mirror with the accepted canonical active story; it does not close312 or turn prior partial evidence into current success.

## Bounded compatibility correction — 2026-10-03

Bounded #312 report compatibility slice, candidate c5682d541bb95bf050b14c748a9acaf56e74ca32, base f5857f8571d4dddbf86210a03af55a3e9b7733db. Root authored three test files after exact pre282 baseline reproduced ten report/1 suite-major precondition failures. No production changes. All 17 original tests retained; two new controls added. Independent review1 assigned to existing Sol reviewer_292; no compiler grant yet, no acceptance or integration claim.

Baseline six binaries: 20 passed/18 failed, exit101. Scoped three binaries: 1pass/4fail,2pass/2fail,4pass/4fail. Corrected:5/5,4/4,10/10, zero ignored, exit0. Actual mixed legacy suite4/report1 and current report2 collector results equal the allcurrent result; deliberate collector stand-ins remain fabricated test input, not target pass claims. Wrong suite digest, count mismatch and contradictory status are refused. Existing manifest1/2/3 and unchanged unsupported-scenario verdict controls preserved. Scoped strict lint and owning fmt/check exit0. Retained exact log and patch hashes in review brief.

Other baseline failures reproduced before282: three absent controls, one aggregate-delta old-format control, and four Go parity freshformat assertions. Go parity did not yet execute Go. Full31target/166failure inventory remains open, none waived. Required final checks still pending.

#282 author correction patch50226754db2dee0d1beec948f24a227b2a0b454dd20c9d3e69771cc9f01de69f delivered source-only before assigned model hit usage limit. Root took verification custody, acquired separate own lease without clearing author's, and began whole interpreted_command_execution tests. Initial attempt correctly refused disk start guard; after own terminal ess-synth cache dry-run/review/clean675files838MiB, actual start free13,608,767,488. No other repository cache touched. Corrected source remains uncommitted/unintegrated until actual results and final review2.

## Remaining mutation report fixture migration

Batch A compatibility continuation, based on fe251b2a4. Report migrationc5682d541 is independently approved and integratedcb6940709; current-suite candidate82803ca32 is author-verified21/21 with independent Go4 still pending resource headroom. Story312 remains open.

Dispatch the existing scope_nested_increment worker for the five test files mutation_external, mutation_external_adversary, mutation_gained_refusals, mutation_skipped_baseline and mutation_unkillable. Exact diagnosisSHAf603e230ee962723a97cc9d8c03d38b7804f3774a952430febb4a3a3f6e0a4e9 establishes40 report/1 precondition failures among50tests before collector assertions. Only test-fixture migration is authorized: actual runs retain their exact report2 categories, coherent explicit stand-ins are re-admitted, intentionally invalid membership reaches its intended refusal, and existing manifest and genuine legacy reader controls remain decisive. No production change, skipped acceptance or issue closure.

Required acceptance is all50 original tests plus the integrated genuine legacy mixed-reader control, scoped strict lint, owning formatting, and independent whole-unit review on the frozen candidate before serial integration. Root owns planning and integration; source-only preparation begins while the sole ESS compiler lane is reserved for the preceding Go review and fresh12GiB start guard. The bounded brief is retained outside the planning store; this record selects the already accepted story and its existing cited test-directory scope.
