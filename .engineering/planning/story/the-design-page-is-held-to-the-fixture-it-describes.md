---
format: aep.planning-md/1
id: story:the-design-page-is-held-to-the-fixture-it-describes
kind: story
status: draft
title: The design page is held to the fixture it describes
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/tests/layout.rs
- confidence: inferred
  path: crates/specify/ess-compiler/tests/typed_diagnostics.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
revision: 4
---
# The design page is held to the fixture it describes

`docs/design/review-typed-diagnostics.md` has been wrong about
`crates/specify/ess-compiler/tests/fixtures/typed_diagnostics/repeated_names.yaml` twice, in
opposite directions, and both times an adversary found it rather than a gate.

| Round | What the page said | What was true |
|---|---|---|
| first version | the fixture asserts `located: None` | the test pinned all three `shop.repeat.File` refusals to line 12 (pass 1, F3/F4) |
| wave 24 | `File` unlocated; `Solo` "cited at `repeated_names.yaml:35:5` — pinned exactly" | each half's opposite: `File` cited at `:12`, `Solo` `<document>`/`None` (pass 2, A1/A2) |

In the same edit, **every** `resolve.rs:<line>` citation in the page was found stale, having drifted
with a 170-line diff: `:452`→`:475`, `:565`→`:754`, `:591`→`:780`, `:637`→`:830`, `:696`→`:889`.
The page also still called `Locator::span` "a substring scan" after it stopped being one.

`grep -rn review-typed-diagnostics crates/edge/ess-xtask/src Taskfile.yml` returns nothing. Nothing
compares this page to the code, so a third inversion would also reach `main`.

## The shape of the answer is not obvious

Two kinds of claim are stale here and they need different treatment:

- **`file:line` citations** drift on any edit above them. `crates/edge/ess-xtask/tests/layout.rs`
  already asserts that every literal `crates/…` path in the repository resolves; it does not check
  the line. Extending it to check that a cited line holds the symbol the sentence names is one
  candidate, and it is a bigger idea than this page.
- **Behavioural claims** — "every refusal it produces is unlocated" — can only be checked by running
  the fixture. The design page's own assertions would have to become a test, or the test's own
  output would have to become the page.

Whoever takes this says which, and says what it costs to be wrong in the other direction: a gate
that pins prose makes every wording change a red build.

## Acceptance

A statement in `docs/design/review-typed-diagnostics.md` that names a line or a citation is checked
by something that runs, and that check goes red when the fixture changes and the page does not.

## Scope

- `docs/design/review-typed-diagnostics.md` — `cited`
- `crates/edge/ess-xtask/tests/layout.rs` — `inferred`, if the citation half is taken there
- `crates/specify/ess-compiler/tests/typed_diagnostics.rs` — `inferred`, if the behavioural half is
  taken as a test
