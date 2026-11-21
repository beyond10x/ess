---
format: aep.planning-md/3
id: story:every-format-version-hint-names-the-source-that-declares-the-system
kind: story
status: draft
title: Every format-version hint names the source that declares the system
summary: 82 hints that ask for a higher specification format name no source; one function builds the clause, every hint goes through it, and a test fails on a hint without it
tags:
- defect
relations:
- decomposes: epic:adopter-gaps-domain-language
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/validate_format_hint.rs
- confidence: inferred
  path: crates/specify/ess-domain/src
- confidence: inferred
  path: crates/specify/ess-domain/tests
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: docs/design/ordered-related-head.md
- confidence: inferred
  path: docs/design/published-value-operand.md
- confidence: inferred
  path: docs/design/related-rows-quantifier.md
- confidence: inferred
  path: docs/design/set-effect-identity-filters.md
revision: 2
---
## Outcome

Every validation hint that asks the writer to raise the specification format says where the
`format:` key goes: on the source that declares the system. One function in `ess-domain` builds
that instruction and every such hint goes through it, so no hint can name a version without the
place, and a Rust test fails when one does. The refusal a writer meets after putting `format:` on
the wrong source names the source that declares the system.

## Need

A downstream specification's writer, a small model running unattended on ESS 0.96.0, was told by
`ess specify validate` to "declare `format: ess/18`". It wrote `format: ess/18` into the domain
file the diagnostic pointed at, left the system file at `ess/1`, and was refused four times until
its budget was spent. Of 12 validate refusals in nine runs, 5 were this format-version class. The
reporter noted that other hints already add "on the source that declares the system".

The reporter's proposal, theirs and not the design: one place builds the location clause, all
hints go through it, and a test fails if a format-version hint is emitted without the clause.

## Fit review

1. **The need, apart from the proposed syntax.** The format is a property of the system, read
   from the one source that carries `system:` (`crates/specify/ess-domain/src/spec.rs:1277`,
   `format: file.format.unwrap_or(FormatVersion::V1)` inside `absorb_header`), but most hints that
   ask for a higher format do not say so, and the diagnostic's location is the construct's file,
   not the system's. Minimal reproduction, run with the released `ess 0.96.0` on 2026-11-21:
   `system.yaml` with `format: ess/1`, `system: demo`, `domains: [demo.shop]`, and
   `domains/shop.yaml` declaring a struct with `undeclared_fields: ignored` (an `ess/24` key).
   - `ess specify validate --path .` prints
     `help: declare \`format: ess/24\`, or drop \`undeclared_fields:\`` and
     `--> domains/shop.yaml:5:5`.
   - With `format: ess/24` added to `domains/shop.yaml`, it prints two refusals: the same
     `unsupported_format_version` again, and `[missing_declaration] domains/shop.yaml.system: this
     source sets system-level fields but does not declare \`system:\`` (ESS-SPEC-005) with the hint
     "the source that carries the format, the version and the domain list is the one that names
     the system". Neither names `system.yaml`.
   - With `format: ess/24` in `system.yaml` instead: `demo v1 — 2 file(s), valid`.

2. **Class: defect.** No authored surface changes. ESS gives one instruction two ways: 22 hints
   say "write `format: ess/N` on the source that declares the system", for example
   `crates/specify/ess-domain/src/command/absent_input.rs:266`, `entity.rs:1108` and the templated
   `expression.rs:2659`, and 82 hints in `ess-domain` ask for a format and name no source
   (counted on `58242e0bc7`, release 0.97.0; generated sources and `tests/` excluded):
   - 79 spell "declare `format: …`": 74 on one line, for example
     `primitive_admission.rs:360` and `binding/context.rs:118` and `:133`, including the two
     templated `ess/{}` hints at `selection.rs:306` and `binding/condition.rs:399`; 3 split across
     a line break (`command.rs:4609`, `:6231`, `command/value_expression.rs:1004`); one more split
     before the backtick (`command.rs:6307`); and one template, `format: {needs}`, at
     `primitive_admission.rs:670`.
   - 1 spells "write `format: ess/30`" with no source: `expression.rs:2740`.
   - 2 say "from `format: ess/N`": `command/set_effects.rs:3064` and `command.rs:7784`.

   One more is outside `ess-domain`: the authored-scenario help at
   `crates/verify/ess-conformance/src/authored.rs:2403` ("from `format: ess/30`").
   About 28 further hints name a version in prose only, such as "(ess/28)" at `view/from.rs:318`
   or "(specification format ess/36)" at `view.rs:1708`. This count comes from a multiline
   search and is approximate; the implementation lists them exactly.

   No helper exists. Each site writes its own literal; the closest thing is
   `undeclared_fields::below_format` (`undeclared_fields.rs:307-321`), which builds one refusal
   with its own literal hint. The two `format!` sites (`selection.rs:304-307`,
   `binding/condition.rs:398`) interpolate only the version.

3. **Can it already be expressed?** The writer can already do the right thing: put `format:` on
   the source with `system:`. The tutorial says a single file carries "the header"
   (`website/docs/guides/specify/layout-and-validation.md:21`). The hint is the only instruction
   a writer acting on the refusal reads, and for 82 refusals it is incomplete. No idiom answers a
   diagnostic that tells half of its repair.

4. **Does the design fit what is already there?** The language grids are not touched: no cell of
   grid R, grid O or the effect × target grid moves, because nothing authored changes.
   - *Vocabulary.* The clause is the existing wording, "write `format: ess/N` on the source that
     declares the system", so the 22 hints that have it keep their bytes and the other 82 adopt
     it. "Declare" is not kept as a second verb for the same act.
   - *Where `format:` is read and what a wrong-source `format:` does.* `RawSpecFile.format`
     (`spec.rs:46-48`) is parsed on every file. Only the header of the file with `system:`
     becomes the system's format (`spec.rs:1277`). A file without `system:` that sets
     `format:` (or `version:`, `domains:` or `preconditions:`) is refused as
     `missing_declaration` at `<source>.system` (`spec.rs:1253-1268`; reproduced in item 1). So
     the refusal of a wrong-source `format:` exists and is **not** added by this story. Its hint
     names no file, though, and its message ("does not declare `system:`") points a literal
     reader towards adding `system:` there, which earns a second refusal ("the system is declared
     in … and in …", `system.rs:1378-1383`). That hint is part of this story: it is the
     diagnostic the writer meets right after following a format hint to the wrong source.
   - *Where the source can be named.* The validators that build the 82 hints receive the
     system's `FormatVersion` but not the header's `Source`. `SpecHeader` has no source
     (`system.rs:703`). So the clause is static text. Only the wrong-source refusal is raised
     where the sources are known (`Specification::absorb_header`, `spec.rs:1246`, and header
     resolution in `system.rs:1350`), so that hint names the header's path when exactly one
     source declares the system.
   - *Siblings.* `ess-conformance`'s authored-scenario help (`authored.rs:2403`) asks for the
     same key and uses the same function. Format refusals for other document formats are not
     specification-format hints and stay as they are: `ess-diff`'s delta and acknowledgement
     formats (`crates/verify/ess-diff/src/raw.rs:85`, `compatibility.rs:1562`), `ess-history/N`
     (`ess-conformance/src/history.rs:659`), `ess-inputs` (`ess-cli/src/input_discovery.rs:306`)
     and `ess-ui` metric formats (`ui/ess-ui/src/path.rs:549`).
   - *Targets.* Rust, Go, TypeScript, web, the interpreter, `ess verify diff`, the Entity Runtime,
     synthesis and authoring are untouched. Only hint text moves. No IR, suite, schema or
     generated byte depends on a hint.

5. **A second, unrelated adopter.** Any specification split into a system file and domain files
   (the layout of `examples/billing`, `system.yaml` plus `domains/*.yaml`) that uses a construct
   above its declared format. For example, a team adding `paging:` (`ess/16`) to a view in
   `domains/catalog.yaml` reads `--> domains/catalog.yaml` and a hint that names only the
   version. This is not local policy: the writer is a person or a model, and the instruction is
   incomplete either way.

6. **Cost.**
   - No format version, no new key, no new `ValidationCode`, no new diagnostic class or family.
     The `validation_codes!` digest in
     `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` does not move.
   - The typed-diagnostics inventory (`docs/design/review-typed-diagnostics.md`, held by
     `crates/specify/ess-compiler/tests/typed_diagnostics.rs`) counts `ValidationError::new`
     sites per file. It does not move if the function builds only the hint `String` and every
     site keeps its constructor. This is a constraint on the implementation, not a re-pin.
   - Pins that compare the hint text exactly and must be re-pinned, naming this story as the
     cause:
     - `crates/specify/ess-domain/tests/view_paging/absent_answer.rs:107-110`
     - `crates/specify/ess-domain/tests/published_value_operand.rs:355`
     - `crates/specify/ess-domain/tests/adversary_m2_pass1.rs:46-47`
     - `crates/specify/ess-domain/tests/adversary_validation_selection_plan_pass1.rs:218-222`
       (`R2_REFUSAL`)
   - Substring pins that stay green: `crates/specify/ess-domain/tests/row_sets.rs:684`,
     `crates/edge/ess-cli/tests/validate_undeclared_fields/empty_struct.rs:94`,
     `crates/verify/ess-conformance/tests/author_generated_value.rs:832` and
     `crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs:454`.
   - Design pages that quote the old hint and are read by no test:
     `docs/design/set-effect-identity-filters.md:163`, `docs/design/related-rows-quantifier.md:215`,
     `docs/design/ordered-related-head.md:219` and `docs/design/published-value-operand.md:148`.
   - No page under `website/` quotes a format hint. `website/docs/reference/diagnostics.md` is
     rendered from the class and family catalogues, which do not change.
   - Longer hints: a composite hint ("…, or drop `absent:`") grows by the clause.

7. **What else was considered.**
   - *Change nothing.* The writer's loop stays possible, and it cost one writer its whole budget.
     Refused.
   - *Fix the reported hint alone* (`ess/18` at `primitive_admission.rs:360`). The other 81 stay
     incomplete, and the next construct copies the old literal. Refused by the rule that
     downstream findings are designed into generic features (`AGENTS.md`, "Where work is
     tracked").
   - *Name the system file's path in every hint.* The 82 sites do not hold the header's `Source`
     (item 4). Threading it into every validator is the per-site change this story avoids, and
     the static clause, together with a wrong-source refusal that names the path, ends the loop
     in at most one more round.
   - *Let `format:` on a domain source raise the system's format.* This changes what a document
     means and needs a format decision. Refused: the format is one fact of the system, read in
     one place.
   - *The requester's shape.* Taken: one builder, every hint through it, and a failing test.
     Extended by two things the request did not name: the version-in-prose hints, and the hint
     of the wrong-source refusal.

## Decisions

- **Adopt as a generic change.** This is the skill's *accept, redesigned*: the requester's shape is
  kept and extended (question 7). One change covers every hint that asks for a specification
  format, not one fix per hint. Decided by the planning session on 2026-11-21, on the downstream
  session's plan decision. No release is ordered by this decision.
- **The clause is the existing wording**, "write `format: ess/N` on the source that declares the
  system", so the 22 hints that already carry it keep their bytes.
- **The wrong-source refusal is not new.** `missing_declaration` at `<source>.system`
  (`spec.rs:1253-1268`) stays, with its code and location. Only its hint changes: it names the
  source that declares the system and says to move `format:` there.

## Acceptance

- One function in `ess-domain` takes a `FormatVersion` and returns
  "write `format: ess/N` on the source that declares the system". Every specification-format
  hint in `crates/specify/ess-domain/src` builds its instruction through it, and so does the
  help at `crates/verify/ess-conformance/src/authored.rs:2403`: the 22 that already name the source, the 79 "declare" sites,
  `expression.rs:2740`, the two "from `format: ess/N`" hints, and each hint that names a version
  in prose as one to reach.
- A Rust test fails, naming the file and line, when a non-test string literal under
  `crates/specify/ess-domain/src` or `crates/verify/ess-conformance/src` spells `` `format: ess/``
  or `` `format: {`` outside that function. Literals continued with `\` are joined before the
  check. Restoring `"declare `format: ess/18`"` at `primitive_admission.rs:360` turns it red.
- A test over emitted diagnostics fails when a hint naming `ess/<N>` lacks the clause. It covers
  at least the reported `ess/18` refusal, one hint of each of the three old spellings
  ("declare", "write" without a source, "from") and one version-in-prose hint.
- An `ess-cli` test rebuilds the reproduction of fit-review item 1 as two sources: a system at
  `ess/1` and a domain source using an `ess/24` key.
  - `ess specify validate` prints a `help:` line holding
    "on the source that declares the system", and its JSON output carries the same hint.
  - With `format: ess/24` moved to the domain source, the `missing_declaration` refusal's hint
    names `system.yaml`.
  - With `format: ess/24` on `system.yaml`, the specification is valid.
- No `ValidationCode`, diagnostic class, format version or `ValidationError::new` count changes.
  `cargo test -p ess-compiler --test typed_diagnostics` and `task test-xtask` pass unchanged.
- The re-pinned tests of fit-review item 6 say beside the pin that this story moved them.
  `CHANGELOG.md` has an entry under the next version.

## Out of scope

- A `format:` on a source without `system:` still decides the operand grammar that one file is
  parsed with (`spec.rs:206-216`, `written_format`), although the file is refused. This is left
  as it is; it is a parse-time reading, not a hint.
- Rendering a second location (`--> system.yaml`) under a format refusal in the CLI's text
  output.
- Format hints of document formats other than the specification's (`ess-inputs/N`,
  `ess-history/N`, delta and acknowledgement formats, `ess-ui` metrics).

## Overlap

`story:the-ess-27-hint-states-the-order-of-an-unknown-identity-and-a-missing-related-row`
(`active`) rewrites the ESS-COMMAND-004 hint "declare `format: ess/27`, where …" in
`crates/specify/ess-domain/src/command/`. Both stories edit that literal. Land that story first, or
in the same wave with this one rebased onto it.

## Scope

- inferred: `crates/specify/ess-domain/src/` (the function, about 130 hint sites in about 40
  files, `spec.rs` `absorb_header` and the header resolution in `system.rs`)
- inferred: `crates/specify/ess-domain/tests/` (the four re-pinned tests and the source and
  emitted-diagnostic tests)
- inferred: `crates/verify/ess-conformance/src/authored.rs`
- inferred: `crates/edge/ess-cli/tests/validate_format_hint.rs` (new)
- inferred: the four `docs/design/` pages of fit-review item 6, and `CHANGELOG.md`
