---
format: aep.planning-md/3
id: story:unicode-normalization-and-case-folding-functions
kind: story
status: draft
title: Predicates compare text through nfc, nfkc and casefold, and mutate wraps comparisons in them
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A predicate can compare text after Unicode normalization or case folding: `nfc(<text>)`,
`nfkc(<text>)` and `casefold(<text>)` are pure value functions usable on either side of a
comparison. `ess verify conform mutate` gains operators that wrap a string comparison's operands in
normalization or case folding, so a suite shows whether an implementation compares code points
where its protocol requires exact equality, or folds where it requires folding.

## Evidence

A consumer writing OpenID Connect contract specifications on ess 0.56.0: OIDC Discovery §5 and
Core §14 require issuer and string comparisons by code-point equality, with no normalization or
case folding. Today a server that normalizes or folds passes every synthesized scenario, because
no scenario sends two strings that differ only in normalization form or case, and `mutate` has no
operator that would show the gap (compare
https://github.com/beyond10x/ess/issues/515, mutate operators for subject and related guards).

## Acceptance

- `nfc`, `nfkc` and `casefold` evaluate per the Unicode version the release pins (named in the
  reference page), checked by known-answer vectors; the interpreted target and the native, Go and
  TypeScript runners agree.
- Synthesis witnesses an exact-equality guard with a pair of strings that are equal after NFC or
  case folding and unequal as code points, so an implementation that normalizes fails a scenario.
- `mutate` generates normalization and case-folding wrap mutants for string comparisons and scores
  them as other comparison mutants are; an implementation comparing code points kills them.
- The format consequence is decided explicitly; below the new `ess/N` the functions are refused
  naming the version.

## Scope (inferred)

`crates/specify/ess-primitives`, `ess-domain`, `ess-compiler`, `crates/verify/ess-conformance`
(interpreter, synthesis, `mutate.rs`, Go and TypeScript runtimes),
`website/docs/reference/predicates.md`, the mutation-audit guide.
