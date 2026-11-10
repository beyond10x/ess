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
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_w2_mut1_adversary.rs
revision: 5
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

- `cargo test -p ess-conformance --locked --test adversary_w2_mut1_adversary
  error_swap_lists_both_refusals_of_a_single_error_domain` passes with the test asserting
  `accounted(&selection, "error-swap/library.shelf.LendBook/wrong-state")` (today it asserts the
  opposite) and asserting that `selection.unmutated` holds both
  `error-swap/library.shelf.LendBook/no-such-book` and `.../wrong-state` with reason
  `no_compatible_error_alternative`.
- The CLI run above still exits 3 with ESS-MUTATE-003, its message reads `2 site(s) found and not
  mutated`, and it lists one `unmutated` line for each of the two refusals.
- `cargo test -p ess-conformance --locked` passes with every other error-swap test unchanged, and
  `git diff` shows no change to the committed `mutation-report-*.json` fixtures under
  `crates/verify/ess-conformance/tests/fixtures/`.

## Decisions

- The payload-less refusal is listed with the existing `no_compatible_error_alternative` reason,
  not a new one: its cause is the same (no other error of the domain to take), and reusing the
  reason adds no new spelling to the mutation report.

## Scope

- cited: crates/verify/ess-conformance/src/mutate.rs
- cited: crates/verify/ess-conformance/tests/adversary_w2_mut1_adversary.rs

confidence: high

Shared surfaces: moves none.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
