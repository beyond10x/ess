---
format: aep.planning-md/3
id: release-plan:ess-24-one-language
kind: release-plan
status: draft
title: 'Unscheduled: ess/24, one language after adoption'
relations:
- serves: vision:O2
- delivers: epic:language-consistency-after-adoption
- delivers: epic:typed-open-questions
- delivers: story:held-state-has-one-operand
- delivers: story:host-context-has-one-shape
- delivers: story:payload-fields-have-one-filling-rule
- delivers: story:related-guard-vocabulary-aligns
- delivers: story:reader-true-refused-for-closed-readers
- delivers: story:format-rule-for-relaxations
- delivers: story:ess-ui-type-grammar-aligns
- delivers: story:review-stale-lines-corrected
- delivers: story:specify-upgrade-command
- delivers: story:change-fragment-upgrade-obligation
- delivers: story:binding-conditions-compare-boolean-event-fields
- delivers: story:an-outcome-that-only-stores-is-observed-through-a-view
- delivers: story:stored-field-equals-returned-response-value
- delivers: story:response-and-struct-admit-undeclared-fields-when-declared-ignored
- delivers: story:requirement-completeness-without-prose-in-the-model
revision: 1
---
Carried from `release-plan:ess-057` on 2026-10-07, when that version number went to the release that actually shipped under it. Not scheduled to a version; its original intent follows.

## Intent

Release ESS 0.57.0 (target 2026-10-12) with source format `ess/24`: the constructs adopted from
downstream requests between 0.42.0 and 0.54.0 read as one language, open questions become typed
declarations, and `ess specify upgrade` moves a specification from `ess/23` to `ess/24` with a
checked delta, so adopters pay for the rename once.

## Scope

| group | artifacts |
|---|---|
| `epic:language-consistency-after-adoption` | `held-state-has-one-operand`, `host-context-has-one-shape`, `payload-fields-have-one-filling-rule`, `related-guard-vocabulary-aligns`, `reader-true-refused-for-closed-readers`, `format-rule-for-relaxations`, `ess-ui-type-grammar-aligns`, `review-stale-lines-corrected` |
| `epic:typed-open-questions` | decompose first; its plan merged into `integrate/ess-054` from pull request #417 |
| adopter path | `specify-upgrade-command`, `change-fragment-upgrade-obligation` |

`served-committed-command-answers-its-outcome` belongs to the same epic and ships earlier, in
`release-plan:ess-055`, because it is a defect in generated servers.

## Format versions

One source-format bump, `ess/24`, shared by every construct here; `ess/23` documents keep their
meaning and `ess specify upgrade` writes the `ess/24` spelling.

## Required completion

As `release-plan:ess-054`, plus an upgrade guide page and `ess specify upgrade` run over every
`ess/23` fixture in the repository.
