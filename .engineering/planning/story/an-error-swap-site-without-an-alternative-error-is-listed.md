---
format: aep.planning-md/3
id: story:an-error-swap-site-without-an-alternative-error-is-listed
kind: story
status: draft
title: An error-swap site without an alternative error is listed
summary: A payload-less refusal in a single-error domain is skipped silently while its payload twin is listed (W2 MUT1 review finding, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:adversary-w2-mut1-pass-1
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

`ess verify conform mutate`'s `error-swap` lists a refusal that carries its error's payload and has
no other error that compiles as `no_compatible_error_alternative` (`error_payload_sites`,
`crates/verify/ess-conformance/src/mutate.rs:2149-2199`; wave W2,
story:error-swap-mutates-refusals-that-carry-a-payload). A refusal without a payload in a domain
that declares only one error has no other error to swap in either, yet it is skipped silently: no
mutant and no unmutated site. The `if let Some(other)` at `mutate.rs:2462-2474` has no `else`. Two
refusals with one cause are reported differently, and the site coverage count leaves the silent one
out. The W2 MUT1 adversary pass found it (review-result:adversary-w2-mut1-pass-1, F5); it was
already there before that wave.

## Problem

Reproduced on 0.92.0 with the `ONE_ERROR` model of
`crates/verify/ess-conformance/tests/adversary_w2_mut1_adversary.rs`: one error
`library.shelf.BookNotFound {book_id}`, and `LendBook` with

```yaml
- name: no-such-book
  unknown_instance: true
  error: library.shelf.BookNotFound
  payload:
    library.shelf.BookNotFound: {book_id: input.book_id}
- name: wrong-state
  wrong_state: true
  error: library.shelf.BookNotFound
```

`ess verify conform mutate --path spec.yaml --target interpreted --class error-swap` exits 3:

```
refusal[ESS-MUTATE-003]: `error-swap` find(s) no site it can mutate in this specification, so the audit would run nothing; 1 site(s) found and not mutated:
unmutated error-swap/library.shelf.LendBook/no-such-book (no_compatible_error_alternative): ...
```

`error-swap/library.shelf.LendBook/wrong-state` appears nowhere.

## Acceptance

- `error_swap_lists_both_refusals_of_a_single_error_domain` in
  `crates/verify/ess-conformance/tests/adversary_w2_mut1_adversary.rs` asserts
  `accounted(&selection, "error-swap/library.shelf.LendBook/wrong-state")` (today it asserts the
  opposite) and passes. Both refusals are listed under `unmutated_sites` with
  `no_compatible_error_alternative`, or with one new reason shared by both.
- The CLI run above lists 2 sites, and the summary's site count includes both.
- A model with a second error keeps its error-swap ids, verdicts and report bytes (the existing
  error-swap tests and the committed mutation-report fixtures are unchanged).

## Scope

- cited: crates/verify/ess-conformance/src/mutate.rs
- cited: crates/verify/ess-conformance/tests/adversary_w2_mut1_adversary.rs

confidence: high

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
