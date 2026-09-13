---
format: aep.planning-md/1
id: review-result:adversary-wave24-unit3-pass-1
kind: review-result
status: active
title: Adversary pass 1 against enum variant in an entity invariant
relations:
- reviews: story:enum-variant-in-an-entity-invariant
revision: 1
---
# Adversary pass 1 against `story:enum-variant-in-an-entity-invariant`

Worktree `wt-ebe43ade0b00`, branch `impl/enum-variant-in-an-entity-invariant`, uncommitted over base
`bd722fa964bd225b9755f272e22b45fab334449f`. Verdict **NEEDS-CHANGE**. Cases 565 → 570, 4 red.
The adversary wrote two untracked test files and no implementation file.

## Findings

| # | Where | Defect | Measured | Verdict | Origin |
|---|---|---|---|---|---|
| F1 | `crates/specify/ess-compiler/src/resolve.rs:529` (`whole_name`) | `whole_name` narrows a needle's match count monotonically. Applied to `needles_from_tokens`' **first** needle — the trailing-key guess `"{last}:"` — it turns a guess that was safely ambiguous into a unique hit, and a unique hit is reported as a line. An outcome is never written `filed:`; it is written `- name: filed`. So the needle cannot match its target and can now only match something else. `needles_for`'s own doc: "Guessing wrongly is safe: `Locator` only reports a line for a needle that occurs exactly once." | `adversary_locator_pass1.rs:164`, exit 101. Refusal `command.shop.probe.Doit.outcomes.filed` cited at **b.yaml:13:13** — the payload-target line of `shop.probe.Other`, a different command, in a different file, not refused at all. Sibling refusal `command.shop.probe.Doit.outcomes` cited correctly at a.yaml:14. | CONFIRMED | introduced |
| F2 | `crates/specify/ess-domain/src/expression.rs:658` | The refusal names `typed.declared`. Where the compared field's declared type is a newtype over the enum, that is the **wrapper**. The refusal lists the enum's variants and never names the enum. Acceptance clause 2 — "names the enum, not only the field, so the fix is one lookup away" — becomes two lookups. | `adversary_enum_variant_pass1.rs:106`, exit 101. Message: `` `wrapped_channel == Fax` compares `wrapped_channel` to `Fax`, which `sample.WrappedChannel` (Text) does not declare as an enum variant; values: `Email`, `Post`, `Portal` ``. Control in the same case (field declared as the enum) names `sample.Channel` and passes. | NEEDS-CHANGE | pre-existing |
| F3 | `crates/specify/ess-compiler/src/resolve.rs:427` | `ValidationError` (`ess-primitives/src/error.rs:663`) records no source document. Acceptance clause 1's "the file it was read from" is recovered by searching for the declaration's name, and drops to `<document>` whenever that search is not unique. | `adversary_locator_pass1.rs:252`, exit 101. `entity shop.probe.Order.invariants[0]` cited `("<document>", None)`, as are two sibling refusals. Reached by one entity declared in two files — a state ESS refuses with `DuplicateDeclaration`, so one it is built to reach. `whole_name` cannot help: both occurrences are whole names. | CONFIRMED | pre-existing |
| F4 | `crates/specify/ess-domain/src/spec.rs:697` (`absorb`) / `:539` (`insert`) | A command name declared twice is refused by `insert` — unless the **first** declaration's own `try_from` failed, in which case it never reaches `insert` and the second silently takes the name. | `adversary_locator_pass1.rs:349`, exit 101. Control passes: two valid `shop.dup.Both` produce `DuplicateDeclaration @ command shop.dup.Both`. Masked case produces only the two outcome-level refusals. Narrows the unit's passing observation: a duplicate of two **valid** declarations **is** refused, so "a duplicate command declaration is silently accepted" is too broad. | CONFIRMED | pre-existing |
| F5 | `crates/specify/ess-compiler/tests/typed_diagnostics.rs:140` | Comment claims "every entity was in this shape". An entity's `identity.type` is a free type reference; `ess-domain/src/entity.rs` imposes no naming relation. `<Entity>Id` is a convention stated as a universal. | Read, not run. The unit's own `billing.rs` guard `assert!(!colliding.is_empty())` is the right guard; the comment overstates what it proves. | CONFIRMED | introduced |

## Attacked and could not break

- `whole_name` at first and last byte of a file: both sides `is_some_and` over `None`/empty. Read, not run.
- UTF-8 boundaries: `match_indices` yields char boundaries; `name.rs:34,42` restricts segments to `[A-Za-z][A-Za-z0-9_-]*`, so a declared name cannot be extended by a non-ASCII letter. Unreachable.
- `spelt_with`'s alphabet against the name grammar: exactly the name alphabet joined by `.`.
- A `name: X` needle pointing at the wrong declaration: neither neighbour of `  - name: X⏎` is a name character. For `name:`-shaped needles the change can only narrow correctly.
- `Locator::scan` cross-file accounting and the `located` memo.
- `type:`/`of:` needles: ESS writes `Optional<T>`, not `T?` (`types.rs:176`).

## Honesty note from the adversary, recorded rather than dropped

An earlier revision of the duplicate-command case asserted only `code == DuplicateDeclaration` and
went **green for the wrong reason** — that code is also used for a repeated outcome name, which the
fixture carries. The assertion was tightened to `location == "command shop.dup.Both"`. The loose
version's green is reported here rather than hidden.

The three shipped tests whose expected values the unit changed were each checked and are honest.
