---
format: aep.planning-md/3
id: story:validation-refuses-an-ordering-over-an-enum-attribute-read-at-every-predicate-site-instead-of-lowering-the-guard-to-false
kind: story
status: draft
title: Validation refuses an ordering over an enum attribute read at every predicate site instead of lowering the guard to false
tags:
- defect
- from-finding
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression/attributes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression/lexical.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/enum_attributes.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/enum_variant_refusals.rs
- confidence: inferred
  path: docs/design/language-grids.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 10
---
## Outcome

- `ess specify validate` refuses `<`, `<=`, `>` and `>=` where an operand is a read of an enum attribute whose type is an enum (`input.plan.level > low`), as `type_mismatch` naming the enum that declares the variants and the predicate as written. It does so at every site that lowers an attribute read: guards, `when_subject:`, `when_related:` `predicate:` and `where:`, a row-set selector, a view `filter:` and measure, an entity invariant and a type invariant.
- A model of this shape no longer compiles a guard, filter or invariant to `false`.
- The claim of `story:validation-refuses-an-ordering-comparison-over-an-enum`, "at every predicate site", holds for attribute reads too.

## Need

Found by the adversary review of the wave of 2026-11-22 (finding F2, `wave-1122/close/adv/REPORT.md`). The refusal that story added is in `expression.rs` `enum_ordering`, reached from `compare` for the operand types of a plain fact. An attribute read (`<fact>.<attribute>`, from `ess/23`) is rewritten to membership by `expression/attributes.rs` `lower` before `compare` runs, so the checker never sees the ordering. Reproduced on the released 0.97.4 and on the build of the integration branch (`ess-s3`, which reports 0.97.4). The models are generic and sit under `drafts2/f2/` (paths below relative to it); `run.txt` holds every command and output.

Model `m/attr-order.yaml`: enum `shop.order.Level` (`ordered: true`, `low, medium, high`), enum `shop.order.Plan` with an attribute `level` of type `Level` and a `String` attribute `label`, and the guard `when: input.plan.level > low`.

```text
0.97.4   ess specify validate --path m/plain-order.yaml    shop v1 — 1 file(s), valid
ess-s3   ess specify validate --path m/plain-order.yaml    m/plain-order.yaml was refused: - [type_mismatch] ...`level > low`: `shop.order.Level` is an enum, and an enum has no ordering ...
0.97.4   ess specify validate --path m/attr-order.yaml     shop v1 — 1 file(s), valid
ess-s3   ess specify validate --path m/attr-order.yaml     shop v1 — 1 file(s), valid
0.97.4   ess specify compile --path m/attr-order.yaml --format yaml    118:        predicate: false
ess-s3   ess specify compile --path m/attr-order.yaml --format yaml    118:        predicate: false
```

`m/plain-order.yaml` is the same model with a plain enum field instead of the attribute. Three things follow from the runs:

1. The plain-enum refusal is unreleased (`CHANGELOG.md` `[Unreleased]`, "**Breaking.** Validation refuses `<`, `<=`, `>` and `>=` where an operand is an enum ... at every format"); 0.97.4 validates both forms. The attribute form stays valid on the build that carries the refusal, and compiles to `false`.
2. Why `false`: `attributes.rs` `members` evaluates the comparison once per variant over a bare `FactStore`, which declares no byte order for text, so the evaluator answers `Unknown` for every variant (`ess-primitives` `predicate.rs`, test `ordering_text_needs_a_declared_scale`), and no variant is in the membership. The cause is read from the source and not run in a debugger (inferred); the outputs show the effect: `m2/v1..v4.yaml` (`> low`, `< high`, `>= medium`, `<= low`) each compile to `predicate: false` on 0.97.4, while `== medium` (`m2/v9.yaml`) lowers to `any_of: [Basic]`. The literal-on-the-left spelling was not run (`low < input.plan.level` reads `low` as an input in this grammar and is refused for that).
3. Every site that goes through `lower` is wrong the same way. `m/sites-literal.yaml` puts `plan.level >= medium` in an entity invariant, `plan.level > low` in a view filter and `input.plan.level > low` in a guard; both binaries validate it and compile `invariants: - false` (line 99), `predicate: false` (line 120) and `filter: false` (line 210). `lexical.rs` `resolutions` calls `lower` once per site, so one fix point covers all of them.

Two neighbours, found while reading `attributes.rs`:

- The comparison of an attribute with another fact (`input.floor >= input.plan.level`, `m/sites.yaml`, `compare_facts` and `expand`). 0.97.4 validates it and expands it to `floor >= medium` and `floor >= high` per variant, ordered by bytes. `ess-s3` refuses it, but only because the expansion reaches the checker: it reports two errors, `` `floor >= medium` `` and `` `floor >= high` ``, predicates the author never wrote, one per variant.
- A `String`-typed attribute: `m/attr-string-order.yaml`, `when: input.plan.label > "a"`, validates on both binaries and compiles to `predicate: false`, where a plain `String` field (`m2/plain-text.yaml`, `input.note > "a"`) compiles to `note > a` and is ordered by bytes at run time. The lowering and the run time disagree. This one is not an enum ordering; it is the same function giving the same silent `false`.

Regression asset: red test `an_ordering_over_an_enum_typed_attribute_is_refused_naming_the_enum` in `crates/specify/ess-domain/tests/enum_variant_refusals.rs`, branch `review/w1122-adv`, commit `a223907510` (`git -C <repo> show a223907510`). It is red on the wave head: the model validates. It asserts a `TypeMismatch` whose message contains `` `demo.desk.Level` ``. It covers one site and one operator; the acceptance below widens it.

## Fit review

- Origin: a pre-existing defect the review found beside the wave's own unit, not an adopter request. It is the unfinished half of the active story `story:validation-refuses-an-ordering-comparison-over-an-enum`, whose outcome says "at every predicate site".
- Covered findings: F2 (the literal form, every site), the fact-comparison form (message and per-variant duplicates), and the `String` attribute form (decision D2).
- One story for every site, because the sites share one function. A refusal added at `compare` cannot work: by then the read is gone. A refusal per site is the shape that missed this one. The fix is in `attributes.rs`, before the rewrite, where `read` already knows the attribute's declared type (`Read::attributes.words` is present for an enum-typed attribute).
- Considered and rejected:
  - Teaching `members` to order by declared variant order. That is the language feature the wave's refusal says is "not in the language yet"; it would give the attribute a meaning the plain field lacks.
  - Lowering to the byte order of the names, as `compare_facts` does today. It is the wrong answer the refusal exists to prevent.
  - Leaving `false` and warning. A guard that is `false` makes a branch unreachable without a trace.

## Design

- In `atom_read`, a `Compare` of a read with a literal (either side) whose `op` needs an ordering, over an attribute whose type is an enum, returns `Some(Err(message))` before `members`. The message is the one `enum_ordering` writes, so the two sites read alike: the predicate as written (`input.plan.level > low`), the enum that declares the variants (`shop.order.Level`), the hint `in: [...]` or `==`/`!=`. One refusal per comparison. It uses the existing `Bool` ordering refusal as its model in the same function.
- In `compare_facts`, the same check before `expand`: where either side reads an enum-typed attribute and `op` needs an ordering, refuse once, naming the predicate as written. `expand` then never builds the per-variant comparisons the checker refuses a second time.
- `Truthy`, `Defined`, `AnyOf` and `NoneOf` over such a read stay as they are (`==`, `!=` and `in` keep validating).
- No new `ValidationCode`: the code is `type_mismatch`, as the plain refusal. Adding no code keeps `macro-guards.json` and `review-typed-diagnostics.md` unchanged; if the message moves a per-file count there, `task test-xtask` says so (`AGENTS.md`, "A new `ValidationCode` or diagnostic moves pins").
- Format consequence (`AGENTS.md` § Determinism and formats): a model that validates today stops validating, at every format from `ess/23`, as the plain-enum refusal does ("at every format"). No new `ess/N`. The rule asks for a version when meaning, identity, references, canonicalization, names or the persisted envelope change. The refused model's present meaning is a guard that is `false` for every input, which is the defect; no accepted model changes its bytes, and no persisted `ess-ir` of a valid model changes. Same decision as the plain refusal, shipped in the same release, so there is one census and one Breaking entry. Recommended; D1 asks.
- What this means for adopters: a model that orders an enum attribute stops validating with the new binary. Such a model never took the branch, so refusing it finds a dead branch, not a regression of behaviour, but the build breaks.
  - Census, before the release, for each downstream specification: `ess specify validate --path <dir>` with the new binary (the exit status and the `type_mismatch` lines naming an enum), and on the released binary `ess specify compile --path <dir> --format yaml | grep -n 'predicate: false\|filter: false\|^ *- false$'`, which lists the sites that lowered to `false` today (a `false` the author wrote also shows; read each).
  - The `CHANGELOG.md` entry joins the existing `[Unreleased]` "**Breaking.**" line for enum orderings (one line, "or reads an enum attribute"), not a second one. `release-notes/<version>.md` is rewritten once.
- Docs: `website/docs/reference/predicates.md` and `docs/design/language-grids.md` name the refusal for plain enums; add the attribute read.

## Acceptance

- `m/attr-order.yaml` fails `ess specify validate` with exactly one `type_mismatch` naming `shop.order.Level` and `input.plan.level > low`, on the new binary. Checked by the red test above, kept as is.
- Each of `>`, `>=`, `<`, `<=`, also under `not:`, refuses once. Test in `enum_variant_refusals.rs`. (The literal-on-the-left spelling is checked against how the grammar reads a bare word on the left; if it cannot be written, the test says so.)
- One test per lowering site (`lexical.rs` `resolutions` arms): input guard, `when_subject:`, `when_related:` `predicate:`, row-set `where:`, view `filter:`, view measure, entity invariant, type invariant. Each refuses; `==`, `!=` and `in` over the same read still validate and lower as before (`enum_attributes.rs` keeps passing, unchanged).
- `m/sites.yaml`'s `below-fact` refuses once, naming `input.floor >= input.plan.level`, not `floor >= medium`.
- No compiled model validates with `predicate: false`, `filter: false` or `invariants: [false]` where the source wrote an ordering over an attribute: a test that compiles `m/sites-literal.yaml` expects the refusal and asserts no IR.
- `task test-xtask` and `cargo test -p ess-domain --test enum_attributes --test enum_variant_refusals` pass; `ess-cli` gated by the package-gate table for `ess-domain` changes. The new tests run in under 5 s each (time them before adding, `AGENTS.md` test budget).
- Census command named in the release notes draft and run on the downstream specifications before the release.

## Decisions

- D1. Format version.
  - A (recommended): no new `ess/N`; refused at every format from `ess/23`, in the same release as the plain refusal, one Breaking line.
  - B: gate the attribute refusal at the next `ess/N`. It leaves the dead guard valid for existing documents; the plain-enum refusal already ships at every format, so the two would disagree.
- D2. A `String` attribute ordered against a literal (`label > "a"`), which lowers to `false` while a plain `String` field is ordered by bytes.
  - A (recommended): refuse it in the same function, naming the attribute and "a text attribute has no ordering in a predicate yet; compare with `==`, `!=` or `in`". Smallest, safe, symmetric with the enum refusal.
  - B: lower it with byte order (the lowering's store declares it, as the runners do), so it agrees with a plain `String`. It changes compiled output for every existing model that has one, and needs a format decision.
  - C: leave it for its own story.
- D3. Order against the wave: fold into the wave's Unreleased entry before the release (recommended), or ship after it as a patch with its own Breaking line.
