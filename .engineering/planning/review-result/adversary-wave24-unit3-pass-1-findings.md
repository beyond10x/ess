---
format: aep.planning-md/1
id: review-result:adversary-wave24-unit3-pass-1-findings
kind: review-result
status: active
title: Findings of wave 24 unit 3 adversary pass 1, in enumerable form
relations:
- reviews: story:enum-variant-in-an-entity-invariant
- supersedes: review-result:adversary-wave24-unit3-pass-1
revision: 1
---
# Findings of adversary pass 1, in enumerable form

`review-result:adversary-wave24-unit3-pass-1` is the record of the pass. It states its findings as
prose and a table, so nothing can enumerate them, and `aep plan artifact findings` cannot build a
ledger against a later pass. Review-results are immutable, which is right — a record editable after
the fact is not evidence — so this supersedes it rather than amending it.

Nothing here is new. Every row is the pass-1 report's own wording, and the measurements are in the
record this supplements.

```findings
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 529
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: "whole_name makes the speculative trailing-key needle unique, so a refusal about a command in a.yaml is now cited at an unrelated payload line in b.yaml, which is the confidently wrong line the Locator header says is worse than none."
- file: crates/specify/ess-domain/src/expression.rs
  line: 658
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "the refusal names typed.declared, so a field whose declared type is a newtype over the enum is refused naming only the wrapper, which fails acceptance clause 2 and is the case the unit's own new test weakened its assertion to accommodate."
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 427
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "ValidationError records no source document, so acceptance clause 1's file it was read from is a name search that degrades to the whole specification whenever the name is not unique, including when one entity is declared in two files."
- file: crates/specify/ess-domain/src/spec.rs
  line: 697
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "a command name declared twice is not reported when the first declaration's own conversion failed, and the unit's repeated_names.yaml edit now depends on that gap to keep an exact assert_eq list at five entries."
- file: crates/specify/ess-compiler/tests/typed_diagnostics.rs
  line: 140
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the comment's claim that every entity was in this shape states a naming convention as a universal, since an entity's identity type reference is free and need not be named after the entity."
```
