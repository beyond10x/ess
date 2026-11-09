---
format: aep.planning-md/3
id: story:a-text-operator-reads-a-binder-or-a-field-as-its-operand
kind: story
status: draft
title: 'A text operator reads a binder or a field as its operand ({fact: t}), and a bare word spelling a binder is refused'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#759
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression/lexical.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: inferred
  path: crates/specify/ess-primitives/tests/text_operand_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 4
---
## Need

A row matches when each token of a list parameter is a substring of one of its fields: a text
operator (`contains`, `starts_with`, `ends_with`, and the ignore-case operators) must test a field
against the element a quantifier binds. Its operand can only be a literal, `{param: <name>}` or
`{input: <name>}` (`crates/specify/ess-primitives/src/predicate.rs:3706-3735`, from `ess/22`). A bare
word is "the text it spells", never a path (`website/docs/reference/predicates.md`, after the
`starts_with` examples). A comparison's right side already reads a binder in scope and takes the
explicit operand `{fact: <name>}` (`predicates.md:262-282`).

The trap: the specification the requester believed expressible validates and means something else.
Minimal reproduction (0.92.0, `ess/36`): view `example.calls.CallSearch`, parameter
`q: List<String>`, filter `not: {exists: {in: param.q, as: t, that: {not: {any: [{from_number:
{contains: t}}, {to_number: {contains: t}}]}}}}` (`repro/run.txt`):

```text
$ ess specify validate --path contains
example v1 — 1 file(s), valid
$ ess verify conform synthesize --path contains --out contains-suite.json
  `example.calls.CallSearch` filters on `not (exists t in param.q: (not ((from_number contains "t" or to_number contains "t"))))`, ...
```

`t` is the one-letter text `"t"`. `{contains: {param: t}}` is `undeclared_reference` (no parameter
`t`). The compact `from_number contains t` does not parse.

## Acceptance

- At the new source format, `{contains: {fact: t}}`, `{starts_with: {fact: t}}`, `{ends_with: {fact:
  t}}` and `{equals_ignore_case: {fact: t}}` (and the operators of
  `story:text-shape-operators-ignore-case`, in whichever of the two stories lands second) take
  `{fact: <name>}` naming a binder in scope, a binder's
  member (`t.label`), or a field the place reads, typed String or a newtype of it; the reproduction
  with `{fact: t}` validates, and synthesis renders it `from_number contains t`.
- At the new format, a bare operand of a text operator that spells a binder in scope is refused as
  `type_mismatch`, naming `{fact: t}` as the repair, as a dotted word naming a parameter in `in` is
  refused (`predicates.md:615-621`). Below the new format such an operand keeps its meaning and
  `ess specify validate` prints a warning naming the repair (exit status unchanged).
- A `{fact: …}` operand that is not text is `type_mismatch`; below the new format it is
  `unsupported_format_version` naming it.
- Synthesis witnesses the reproduction view: a row whose `from_number` holds each token of `q`
  (selected), a row missing one token in both fields (not selected), and a decoy whose fields hold
  the letter `t` but not the tokens. `ess verify conform run --target interpreted` passes, and a
  target reading the operand as the literal fails.
- The interpreted target and the Go and TypeScript predicate runtimes evaluate the operand
  (`crates/verify/ess-conformance/src/go/predicate.go`, `crates/verify/ess-conformance/src/ts/predicate.ts`).
  Generated Rust and Go view queries keep such a view an obligation named by its filter, as for
  quantifiers today.

## Scope

- cited: crates/specify/ess-primitives/src/predicate.rs
- inferred: crates/specify/ess-domain/src/expression.rs
- inferred: crates/specify/ess-domain/src/expression/lexical.rs
- cited: crates/verify/ess-conformance/src/witness.rs
- cited: crates/verify/ess-conformance/src/go/predicate.go
- cited: crates/verify/ess-conformance/src/ts/predicate.ts
- inferred: crates/specify/ess-primitives/tests/text_operand_fact.rs
- cited: website/docs/reference/predicates.md
- cited: docs/design/language-grids.md
confidence: medium

## Fit review

1. **Need.** A text test against a value bound by a quantifier over a list parameter. Requester's
   text (theirs): "Every token of a list is expressible too, with a `List<String>` parameter and a
   double negation; this validates". It validates and means the literal `"t"` (`run.txt`).
2. **Class.** Gap, with a trap. The operand form has no binder (`predicate.rs:3706-3735`). The
   silent reading follows the documented rule ("The operand is the text it spells"), so it is not a
   defect of the evaluator. It is a trap the comparison operators already close: a binder named like
   a field is refused there (`predicates.md:270-273`).
3. **Already expressible?** No. `{param: t}` names no parameter, and the compact form does not
   parse (`run.txt`, `mapping-bound`, `compact-bound`).
4. **Fit.** It reuses the comparison's explicit operand `{fact: …}` (`predicates.md:276-282`) for
   the text operators, so one operand spelling reads a fact in every operator. That opens the
   operand column of grid O's text rows (`docs/design/language-grids.md:73-74`, "param/input operand
   `ess/22`") for every fact the position reads, not for binders alone. It composes with
   quantifiers (`exists`, `forall`), `when_related:` predicates and row-set `where:` wherever those
   read binders.
5. **Second adopter.** A permission check `exists: {in: input.scopes, as: s, that: {granted_scopes_text:
   {contains: {fact: s}}}}`; a tag search `forall: {in: param.prefixes, as: p, that: {sku:
   {starts_with: {fact: p}}}}`.
6. **Cost.** The next open source format. One operand form on more operators, no new key. One new
   refusal at the new format, which breaks documents that relied on the literal reading, so it is
   only a warning below the format. No suite format: filters are decided at synthesis.
7. **Alternatives.** (a) Change nothing: the trap stays. (b) Read a bare binder name as the binder:
   refused, it silently changes the meaning of documents that validate today. (c) `{fact: …}` plus
   a refusal of the ambiguous bare word: chosen.

## Decisions

accept, redesigned. Not in the request: the request assumed this worked. The finding is that it
does not, and the repair is the existing `{fact: …}` operand.

## Source

github:beyond10x/ess#759
