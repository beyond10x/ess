---
format: aep.planning-md/2
id: review-result:adversary-wave24-unit3-pass-2
kind: review-result
status: active
title: Adversary pass 2 against enum variant in an entity invariant
relations:
- reviews: story:enum-variant-in-an-entity-invariant
revision: 1
---
# Adversary pass 2 against `story:enum-variant-in-an-entity-invariant`

Worktree `wt-ebe43ade0b00`, uncommitted over base `bd722fa964bd225b9755f272e22b45fab334449f`.
Verdict **NEEDS-CHANGE**. Cases 2899 → 2905, 3 red. Budget exhausted.

**No shipped case moved.** `-p ess-compiler` is 122 passed / 2 ignored before and after; all three
reds are the adversary's own new files. Three further cases it wrote were green on their first run
and are reported as attacks that failed, not findings.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| A1 | `docs/design/review-typed-diagnostics.md:334` | NEEDS-CHANGE / introduced / **blocker** | `adversary_pass2_design_page.rs:104`, exit 101. `("<document>", None)` against the page's `("repeated_names.yaml", 35:5)` | the unit's 18-line fixture edit added a second `shop.repeat.Solo`, so both its refusals are unlocated. The page is not in the diff. `grep -rn review-typed-diagnostics crates/edge/ess-xtask/src Taskfile.yml` → nothing: no gate compares the page to the code |
| A2 | `docs/design/review-typed-diagnostics.md:321` | NEEDS-CHANGE / introduced / warning | `adversary_pass2_design_page.rs:129`, exit 101. `("repeated_names.yaml", 12:5)` against the page's `("<document>", None)` | the same section says every refusal the fixture produces is unlocated. Three of the five are now located, because `whole_name` narrows `name: shop.repeat.File` away from `FileTwo` and `Filed` |
| A3 | `crates/specify/ess-compiler/src/resolve.rs:557` | NEEDS-CHANGE / introduced / warning | `adversary_pass2_locator.rs:157`, exit 101. `command.shop.probe.Doit.outcomes.filed`, declared and refused in `a.yaml`, cited at `b.yaml:13:13` — the payload line of `shop.probe.Other`, not refused at all. The sibling one segment up is cited correctly at `a.yaml:12` | `whole_name_matters`' new doc justifies the exemption with "something else is not unique, so no line is reported". That is a non-sequitur. It held for pass 1's fixture only because that fixture wrote a second field `refiled` |
| A4 | `crates/specify/ess-compiler/src/resolve.rs:889` | CONFIRMED / **pre-existing** / warning | same case, same output | `needles_for`'s doc: "Guessing wrongly is safe … a bad guess produces no line rather than the wrong one." False for every `command.*.outcomes.<name>` whose `<name>:` occurs exactly once elsewhere. The adversary checked the base: `scan` there is `text.match_indices(needle)` with no filter and `whole_name_matters("filed:")` is `false`, so base and current take the identical path. Not reachable in `examples/billing` — all 8 outcome names checked, 0 raw `<name>:` matches |
| A5 | `crates/specify/ess-compiler/src/resolve.rs:3926` | CONFIRMED / introduced / warning | read, plus `adversary_pass2_locator.rs:157` as the state it does not construct | the class check's probe document is `"{needle} value\nq{needle} value\n"` — always two matches. For the key-needle branch it asserts `located.is_none()` and concludes the guess is safe. It never builds the one-match document, the only state in which the exemption is unsafe. Its corpus `needle_shapes()` at `:3841` is a literal 10-entry array chained with `ConstructKind::ALL` — machine-enumerated over kinds, hand-maintained over shapes |
| A6 | `crates/specify/ess-domain/src/expression.rs:704` | CONFIRMED / introduced / note | read, not run | in `quantified`, `declaring_variants` is computed into a `ValueType` whose only consumer reads `typed.declared`. Written and never read on that path |
| A7 | `crates/specify/ess-domain/tests/expression.rs:472` | CONFIRMED / introduced / note | read, not run | `carries_a_literal` at `:451` is exhaustive over `Predicate` and its doc says a new form "stops this file compiling rather than arriving quietly". True of the helper. `enum_literal_cases` is a hand-written 12-entry list nothing forces a new form into |

## Attacked and could not break

- **`whole_name_matters` being one character** separates the two kinds for every reachable needle.
  `split_path` splits on `.`, ` `, `[`, `]`, so no token contains a space and a key needle can never
  begin `"name: "`; no `format!` building a `location` in `ess-domain/src` contains a `:`, so no
  declaration needle can end in one. A needle of `":"` or `""` is unreachable — `needles_from_tokens`
  emits `"{last}:"` only for a non-empty `last` whose first character is lowercase.
- **`needle_shapes()` omitting a shape.** It is a literal table (A5), but no omitted shape was found
  for which `needles_of_site != needles_for(render)`. The one candidate — a `ConstructKind::as_str()`
  containing a separator, which `tokens_of_site` does not `split_path` — is covered by the
  `ConstructKind::ALL` chain.
- **`declaring_variants` from `terminal`.** Newtype, newtype-over-newtype, `Optional<enum>`,
  newtype-over-non-enum, list element and struct member all produce the right parenthetical; the
  direct case prints `` `sample.Channel` (Text) `` with no empty or duplicated clause. Three green
  cases now pin it. Before them, `let reached = String::new();` was a surviving mutant — the
  `reached` branch had no shipped assertion at all.
- **The three `contains` calls.** No reachable wrong message satisfies all three. Only
  `` contains("`sample.State`") `` is load-bearing — `` `Ready` `` is satisfied by the values list
  and `Fax` by the echoed literal — and it does kill the F2 mutant, because reverting `declaring` to
  `typed.declared` drops the enum's name from the newtype message entirely.
- **The two `#[ignore]`s.** Both stories exist. Run with `--ignored`, each fails at exactly the
  assertion its story describes; neither hides a third failure.
- **The F5 comment counts.** Counted independently with `declared_names`' own rule: `examples/billing`
  declares 31 names, 2 colliding — `billing.invoice.Account` (`domains/invoice.yaml:69`) and
  `billing.invoice.Invoice` (`:97`). `nested.yaml` declares 4, 1 colliding, located at `11:5`. Both
  comments exactly right.
- **The fixture edit weakening the two design-page guards.** Both still green, neither relaxed; the
  fixture carries both halves now. The unit disclosed inside its `#[ignore]`d F4 case that the new
  `Solo` duplicate is assembled only through the masking defect — stated, sourced and routed.
- **Narrowing losing a correct line.** `whole_name` can only remove matches, so it could turn 1 → 0
  for a declaration needle. No YAML declaration line was found whose `name: <X>` match has a name
  character adjacent: flow style, trailing comment, CRLF and EOF all give a non-name neighbour.

## What the coordinator did with A1 and A2

They are the coordinator's: the story's `scope:` does not list the design page, the wave protocol
reserves shared files to the coordinator, and a handback naming that page was already pending.

The section was rewritten to state both halves as measured — `shop.repeat.File` declared once at
`:12` and cited there under whole-name matching; `shop.repeat.Solo` declared twice, at `:35` and
`:56`, so both its refusals are `<document>`/`None`.

Something no finding named was found in the same edit: **every** `resolve.rs:<line>` citation in the
page had drifted with the unit's 170-line diff.

| cited | actual symbol at that line now |
|---|---|
| `:452` → `:475` | `Locator::span` |
| `:565` → `:754` | `family_of` |
| `:591` → `:780` | `class_of` |
| `:637` → `:830` | `STRUCTURAL` |
| `:696` → `:889` | `needles_for` |

The page also still called `Locator::span` "a substring scan", which it is not any more except for
the one exempt needle. Corrected, with the exemption named and `resolve.rs:557` cited for it.

```findings
- file: docs/design/review-typed-diagnostics.md
  line: 334
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the page pins shop.repeat.Solo's two refusals cited at repeated_names.yaml 35:5 pinned exactly, and the fixture edit made both of them unlocated, with the page left unedited and no gate comparing the two."
- file: docs/design/review-typed-diagnostics.md
  line: 321
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the same section says every refusal repeated_names.yaml produces is unlocated and that Locator reports located None instead of picking the first, and three of the five are now located at 12:5."
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 557
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the new whole_name_matters doc justifies exempting the trailing-key guess with something else is not unique so no line is reported, which is not a property of anything - a wrong guess that is unique is reported, at a line in a file that holds no refusal."
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 889
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "needles_for's guessing wrongly is safe claim is false for every command outcomes refusal whose trailing key occurs exactly once elsewhere, and the base commit's unfiltered scan behaves identically."
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 3926
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the class check that certifies the key-needle exemption only ever builds a two-match document, so it never tests the one-match state in which the exemption is unsafe, and its corpus needle_shapes is a hand-written table of shapes chained with a machine enumeration of kinds."
- file: crates/specify/ess-domain/src/expression.rs
  line: 704
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "quantified fills declaring_variants into a ValueType whose only consumer reads typed.declared, so the field is written and never read on that path."
- file: crates/specify/ess-domain/tests/expression.rs
  line: 472
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "carries_a_literal is exhaustive and will break compilation when a Predicate form is added, but enum_literal_cases is a hand-written list that nothing forces the new form into, so the coverage the doc comment claims is not the coverage the compiler enforces."
```
