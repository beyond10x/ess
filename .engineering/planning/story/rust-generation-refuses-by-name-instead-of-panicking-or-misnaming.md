---
format: aep.planning-md/3
id: story:rust-generation-refuses-by-name-instead-of-panicking-or-misnaming
kind: story
status: draft
title: Rust generation refuses by name instead of panicking or allocating a bad identifier
summary: A synthesis panic, `self` becoming `Self`, and colliding view method names (workbench report items 28, 29, 32)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

Three specifications that `ess specify validate` accepts stop code generation. Reproduced on 0.92.0
with the report's minimal examples, `ess generate synthesize --target rust --layout crate --path spec
--out rust-out` (and `--target go`):

- **Item 28, a panic in both targets.** A command whose error outcomes are guarded by
  `when_related` (`exists: false`, or a `predicate` over the related row) and whose payload reads
  the subject, `parent_id: {subject: parent_id}`, while another outcome `moves:` the subject:

  ```
  thread 'main' panicked at crates/generate/ess-synth/src/rust/behaviour.rs:2850:28:
  the plan admits `{subject:}` only where a row is held
  (exit 101)
  ```

  Go panics the same way at `crates/generate/ess-synth/src/go/behaviour.rs:3661:28` (exit 101).
- **Item 29, an outcome named `self`.** Rust refuses the workspace, exit 1:
  `gap.demo.FinishItem.self: outcome variant allocates invalid Rust identifier \`Self\` in
  \`outcomes:gap.demo.FinishItem\``. Go generates all 19 capabilities.
- **Item 32, two views of one last name in two domains** (`gap.one.Things`, `gap.two.Things`, one
  component). Rust refuses, exit 1: `` `things` is allocated 2 times in `port methods:gap-service`
  (view method, view method); give one of them `naming: { code: … }` ``. Go generates all 18
  capabilities. Commands of one name across domains got collision-free codec names in
  beyond10x/ess#415 (closed); view port methods did not.

Items 28, 29 and 32 of the 2026-11-09 workbench report.

## Fit review

1. **Need.** A valid specification either generates or is refused by name with a remedy; it never
   panics, and an authored name never needs renaming because of one target's keyword list or
   because two domains share a last segment. Requester's expectation (theirs): the panic becomes a
   refusal or code; `self` is escaped or refused at validation; distinct view method names without
   `naming:`.
2. **Class.** Item 28: defect (a panic on an accepted specification; the plan's own assertion is
   wrong for this shape). Items 29 and 32: defect against the target-parity rule (Go accepts both;
   Rust refuses a name the specification may legally hold), and #415 already decided the command
   case.
3. **Already expressible?** Item 32 has a workaround, `naming: { code: … }`, and the refusal says
   so; it is an idiom for a collision the generator could avoid itself, as it does for commands.
   Item 29: renaming the outcome changes the outcome's wire and suite name. Item 28: none.
4. **Fit.** One rule per allocator: Rust identifiers are allocated collision-free and keyword-safe
   for every role (outcome variants, view methods, commands), as Go already does. A keyword
   outcome name is mangled (`r#self` is not a legal raw identifier), not refused at validation,
   because validation must not carry one target's keyword list.
5. **Second adopter.** An outcome named `type` or `match`; two bounded contexts each exposing a
   `Summary` view served by one component.
6. **Cost.** No source or suite format change. Generated Rust identifiers change only for
   specifications that are refused today; every model that generates keeps its bytes.
7. **Alternatives.** (a) Change nothing: 29 and 32 keep a named refusal, 28 keeps a panic.
   (b) Refuse `self` at validation for every target: puts a Rust keyword list in the language.
   (c) Fix the panic, mangle keywords, and qualify colliding view methods as #415 qualified
   commands (chosen).

## Decisions

Accept, redesigned: item 29 is escaped in the Rust allocator rather than refused at validation;
item 32 follows #415's qualification for view methods; item 28 is fixed in both Rust and Go.

## Acceptance

- `a_subject_read_in_a_related_guarded_error_payload_generates_or_is_refused_by_name`: example 28's
  model, `ess generate synthesize --target rust` and `--target go`, exits 0 or 1 with a named
  refusal, never 101; when it generates, the conformance suite the example synthesizes passes
  against the generated server.
- `an_outcome_named_by_a_rust_keyword_generates`: example 29 (`- name: self`) and an outcome named
  `type` synthesize for Rust with exit 0; the outcome's wire name stays `self`.
- `two_views_of_one_last_name_in_two_domains_get_distinct_methods`: example 32 synthesizes for
  Rust with exit 0 and no `naming:`; a model with no collision keeps its bytes.

## Scope

- cited: crates/generate/ess-synth/src/rust/behaviour.rs
- cited: crates/generate/ess-synth/src/go/behaviour.rs
- cited: crates/generate/ess-synth/src/rust/feasibility.rs
- inferred: crates/generate/ess-synth/src/rust/name.rs
- inferred: crates/generate/ess-synth/src/plan.rs
- inferred: CHANGELOG.md

confidence: medium

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
