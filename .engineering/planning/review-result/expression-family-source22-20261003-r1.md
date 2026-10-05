---
format: aep.planning-md/3
id: review-result:expression-family-source22-20261003-r1
kind: review-result
status: active
title: Expression-family design independent review pass 1
relations:
- reviews: component-design:expression-family-source22
revision: 1
---
needs-revision

# Family F expression design review — round 1 (publication-safe)

Publication-safe normalization of the independent review whose full report SHA256 is
`97ed0d63f27ecc9716ebd069f4bdf37466edcfe64325dd09ed7cd2fb278f00c5`. The reviewed proposal
SHA256 is `8cf552090ae25760546ab9660b82466e20e1edae7ed799113239bd6d4c7cc05d`; source was inspected at
integration commit `72167e08fb2f9c9703645dd0f5664c20a080ccb9`. The pass was static and read-only: no compiler,
test, formatter, generator, AEP mutation or repository write was performed.

Five blockers require revision:

1. The proposed lexical `UnquotedText` lives in the same serializable/evaluable predicate tree that
   compiled IR stores. The design relies on every caller invoking a transforming checker, but the
   current IR directly serializes `Predicate`. A lexical AST must be non-serializable and distinct
   from the resolved predicate, or all persisted fields must require a validated resolved-predicate
   type and reject lexical operands at serialization/digest time.
2. `.utf8_bytes` is represented only by a plain path plus transient checker metadata. A persisted
   suite cannot distinguish the derived String selector from a declared nested field actually named
   `utf8_bytes`, so suite40 selection and suite39 refusal cannot both preserve old suites. Persist a
   tagged resolved selector and test direct-field, derived-field and direct-binding precedence.
3. Integer base and magnitude each fitting `i64` does not prevent their sum or difference leaving
   the Integer domain. Define whether an out-of-domain intermediate is Unknown or is compared in a
   wider exact carrier, bind the same Rust/Go behavior, and test both extrema against wrap,
   saturation, rounding and accidental-Unknown faults.
4. A3 admits time in stored and row-set predicates while retaining native interpreter/history
   refusal. The approved row-set contract requires the separate time slice to execute before final
   acceptance. Bind one occurrence-scoped instant per command for native/history evaluation and
   prove subject, identity-related, `where`, `forall`, snapshot and setup-time controls without using
   host time.
5. `distinct` states exact Decimal/Integer/Timestamp equality and finite-domain witness behavior but
   its acceptance uses no values that distinguish those semantics. Add equivalent differently
   spelled instants and decimals, Integers across the binary64 precision boundary, Boolean
   exhaustion, singleton-enum no-witness and two-variant enum witnesses, with paired mutants in each
   required execution lane.

The resolution order for A1, old-source conversion, A4 Optional/fallback semantics, typed text
operands, UTF-8 byte definition and the 40/41 allocation are otherwise coherent. The approved
row-set Unknown, correlation, snapshot and precedence semantics remain unchanged. This review makes
no implementation or execution claim.

```findings
[
  {
    "file": "design-proposal.md",
    "line": 45,
    "category": "architecture",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "UnquotedText is proposed as a serializable/evaluable variant of the same public Predicate tree stored directly in EssIr, so the resolved-only persistence rule depends on every caller remembering a transforming checker; use a non-serializable lexical AST or a mandatory validated resolved-predicate boundary and prove direct assembly, IR digest and suite writing reject lexical operands"
  },
  {
    "file": "design-proposal.md",
    "line": 321,
    "category": "compatibility",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": ".utf8_bytes remains only a plain FactPath plus transient Access metadata, so persisted suites and old-reader admission cannot distinguish the new derived selector from an existing declared member named utf8_bytes; persist a tagged resolved selector identity and add direct-field, derived-field and suite39/40 controls"
  },
  {
    "file": "design-proposal.md",
    "line": 143,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "individually i64-bounded Integer bases and magnitudes can still overflow the Integer domain, contradicting the claim that exact arithmetic cannot overflow and leaving Rust/Go results undefined; bind out-of-domain intermediate semantics and test max/min add/subtract against wrap, saturation, rounding and accidental-Unknown mutants"
  },
  {
    "file": "design-proposal.md",
    "line": 198,
    "category": "scope",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "A3 admits current time in stored and row-set predicates but retains native interpreter/history refusal, contradicting the approved filtered-related contract that the separate time slice must execute before final acceptance; bind one occurrence-scoped command instant for native/history evaluation and decisive healthy/faulty cases without host-time fallback"
  },
  {
    "file": "design-proposal.md",
    "line": 302,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "distinct specifies exact numeric and instant equality plus finite-domain no-witness behavior, but its acceptance lacks lexically unequal equal timestamps/decimals, binary64-boundary integers, Boolean exhaustion and singleton/two-variant enum controls; add those witnesses and paired mutants in every required execution lane"
  }
]
```
