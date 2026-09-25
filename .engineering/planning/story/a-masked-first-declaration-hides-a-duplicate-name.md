---
format: aep.planning-md/2
id: story:a-masked-first-declaration-hides-a-duplicate-name
kind: story
status: implemented
title: A masked first declaration hides a duplicate name
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler/tests/design_page_inventory_covers_every_producing_source.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/design_page_matches_the_fixture.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/locator_citations.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/typed_diagnostics.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/masked_declaration_boundaries.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/masked_declaration_first_copy_attribution.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/masked_declaration_reference_cascade.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
revision: 9
---
# A masked first declaration hides a duplicate name

`Spec::insert` (`crates/specify/ess-domain/src/spec.rs:539`) refuses a command name declared twice.
`absorb` (`:697`) only reaches `insert` with declarations whose own `try_from` succeeded. So when
the **first** of two same-named declarations has any error of its own, it is never handed to
`insert` and the second silently takes the name.

The author is told about the first declaration's error and never about the two declarations.
Fixing the first error makes a second refusal appear that was true all along.

Measured by the wave-24 adversary at `adversary_locator_pass1.rs:349`, exit 101. The control in the
same case passes: two **valid** `shop.dup.Both` do produce `DuplicateDeclaration @ command
shop.dup.Both`. The masked case produces only `duplicate_declaration @
command.shop.dup.Both.outcomes.filed` and `conflicting_declaration @ command.shop.dup.Both.outcomes`
— no name-level duplicate.

## This is load-bearing on a shipped fixture

`crates/specify/ess-compiler/tests/fixtures/typed_diagnostics/repeated_names.yaml` gained a second
`shop.repeat.Solo` in wave 24. It is accepted **only** by this path, and
`typed_diagnostics.rs:179` pins the refusal list with `assert_eq!` on an exact five-element vector.
Closing this gap turns that shipped test red. Whoever takes this story changes both.

## Acceptance

Two declarations sharing a name are both reported as a duplicate, whether or not either of them has
an error of its own.

## Scope

Rewritten after the wave from the merged diff — `0321f0f2` and the follow-up `525a4a19` — rather
than from the pre-dispatch reading. The corrections are shown rather than deleted.

Held, and touched:

- `crates/specify/ess-domain/src/spec.rs` — `cited`
- `crates/specify/ess-compiler/tests/typed_diagnostics.rs` — `cited`

Wrong, and removed: `crates/specify/ess-compiler/tests/fixtures/typed_diagnostics/repeated_names.yaml`.
The story argued that closing the gap turns that shipped fixture red and that whoever took the story
would change both. The fixture was not touched. `typed_diagnostics.rs` absorbed the change on its
own, so the prediction named a surface the work never reached.

Not predicted, and touched:

- `crates/specify/ess-domain/tests/masked_declaration_boundaries.rs` — `cited`
- `crates/specify/ess-domain/tests/masked_declaration_first_copy_attribution.rs` — `cited`
- `crates/specify/ess-domain/tests/masked_declaration_reference_cascade.rs` — `cited`
- `crates/specify/ess-compiler/tests/design_page_inventory_covers_every_producing_source.rs` — `cited`
- `crates/specify/ess-compiler/tests/design_page_matches_the_fixture.rs` — `cited`
- `crates/specify/ess-compiler/tests/locator_citations.rs` — `cited`, from `525a4a19`: a case
  `#[ignore]`d against this story began to pass when the story landed, and was shipping skipped
  until the unit grepped the tree for its own id after finishing
- `docs/design/review-typed-diagnostics.md` — `cited`, and **shared with
  `story:component-declares-its-settings`**, which was the other unit in this wave. The two wrote the
  same design page in one wave without colliding because they were merged in sequence; a later wave
  reading this section will see the overlap it could not see before.
