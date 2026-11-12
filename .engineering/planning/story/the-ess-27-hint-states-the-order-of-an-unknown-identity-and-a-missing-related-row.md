---
format: aep.planning-md/3
id: story:the-ess-27-hint-states-the-order-of-an-unknown-identity-and-a-missing-related-row
kind: story
status: draft
title: The ess/27 hint states the order of an unknown identity and a missing related row as the rule has it
summary: 'The ESS-COMMAND-004 hint below ess/27 says the unknown addressed identity answers after an exists: false over the input''s row; under ess/27 it answers before it (0.94.0)'
tags:
- backlog
- defect
- from-finding
- severity-low
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 1
---
## Defect

Below `ess/27` a command that pairs `when_related:` with an `unknown_instance:` branch is refused
(ESS-COMMAND-004, `conflicting_declaration`). Its hint states the `ess/27` order the wrong way
round:

```
help: declare `format: ess/27`, where the unknown addressed identity answers after an `exists: false` over the input's row and before the related row's predicates, or split the other guard into a command of its own
```

Under `ess/27` the unknown addressed identity answers before an `exists: false` over the input's
row: existence of the addressed record is step 2, every `exists: false` is step 4
(`website/docs/reference/predicates.md`, "From `ess/27` one precedence rule";
`docs/design/selection-plan.md`, "Under the one rule (`ess/27`)"). Measured on the 0.94.0 release
binary: for a command with an `exists: false` refusal over `input.parent_id`, a creating
`unknown_instance:` branch and an updating branch, synthesis sends a new key with a parent identity
no row carries and expects the creation, and the interpreted target passes that scenario.

An author who follows the hint moves to `ess/27` expecting the refusal first and gets the creation
first. Found 2026-11-11 while answering an adopter's question on this order.

Source: `crates/specify/ess-domain/src/command/related_guard.rs`, the `with_hint` at line 782 on
main `9f8141cb8b`.

Reproduction: any model of `format: ess/26` whose command declares
`when_related: {via: input.<field>, exists: false}` beside an `unknown_instance: true` branch;
`ess specify validate --path <model>` prints the hint.

## Acceptance

1. The hint says that under `ess/27` the unknown addressed identity answers before an
   `exists: false` over the input's row and before the related row's predicates.
2. A test pins the hint text against the order `precedence::ONE_RULE` gives, so the two cannot
   disagree again.
3. No other hint or reference sentence states the opposite order: the hints of the sibling
   refusals in `related_guard.rs` that name `ess/27` are read and corrected where they do.
