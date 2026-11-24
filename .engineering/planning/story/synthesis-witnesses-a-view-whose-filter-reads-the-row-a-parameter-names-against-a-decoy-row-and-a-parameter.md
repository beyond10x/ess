---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-view-whose-filter-reads-the-row-a-parameter-names-against-a-decoy-row-and-a-parameter
kind: story
status: draft
title: Synthesis witnesses a view whose filter reads the row a parameter names, against a decoy row and a parameter naming no row
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields
revision: 2
---
## Need

After `g08-view-param-row`, a view whose filter reads `param.<p>.<field>` validates and the
interpreter decides it, but `ess verify conform synthesize` has no arrangement for it: the `ess/28`
navigation scenarios arrange rows through a reference **of the source**
(`crates/verify/ess-conformance/src/synthesize/view_reference.rs`; the guide's description,
`website/docs/guides/specify/views.md:255-261`: two referenced rows, a source row pointed at each,
one pointed at nothing, one referenced row changed). A view nobody arranges is no claim the suite
checks. Report item 8, synthesis half.

## Outcome

Synthesis writes, per view and parameter reference, scenarios that arrange two rows of the named
entity (the named row and a decoy), source rows the named row's list names, rows only the decoy's
list names, and rows neither names; read with the parameter at the named row, the view must return
exactly the first set; read with a parameter naming no row, it must return none; after a command
changes the named row's list, a second read must follow it. A target that ignores the parameter,
reads the decoy, or caches the first answer fails.

## Acceptance

- First: `ess verify conform synthesize --path docs/design/view-parameter-rows.example` writes a
  `<view>/parameter-reference/<p>` scenario (name to be fixed with the design page) for
  `ProductsOfOrder`; before this story it refuses the view by name (the refusal
  `g08-view-param-row` leaves in place).
- Run against the interpreted target, the scenarios pass; three mutants fail them: the filter
  dropped (every product returned), the parameter read as the decoy's identity, the list read once
  and not again after the change. A test in `crates/verify/ess-conformance/tests/` runs them.
- A nested quantifier (a list of structs holding a list) is arranged: the named row's element names
  the expected source row only through the inner list, and the decoy's element names it through the
  outer element but not the inner list.
- Where no creating command can write the named entity's list from its input, synthesis refuses the
  view with a named `ESS-SYNTH-*` code and the field, as `filtered-related-reads.md` refuses an
  arrangement it cannot reach.
- The byte-pinned tables in `AGENTS.md` § Gate move only where the change claims them; the suite
  format moves only if the scenarios need a key the current suite format lacks, and the change says
  which (I don't know whether one is needed).

## Fit review

Tree `bcfc51555`, read-only. Covers report item 8 with `g08-view-param-row`.

1. **Need.** A checked claim for the construct of `g08-view-param-row`.
2. **Class.** *Gap* in the suite for a construct the language admits.
3. **Already expressible?** No arrangement exists for a parameter's row.
4. **Fit.** Reuses the decoy pattern of `view_reference.rs` and the row arrangements of
   `synthesize/related.rs`; no new vocabulary in the specification.
5. **Second adopter.** As `g08-view-param-row`.
6. **Cost.** Synthesis code and tests; a suite format move only if needed (above).
7. **Considered.** Leaving the view a refused synthesis forever (the construct would be
   unchecked); authored scenarios only (each adopter writes the decoys by hand). Chosen: synthesis.

## Decisions

- None beyond P1-D6 (`g08-view-param-row`; open, with the owner as AQ5).

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/view_reference.rs`
- inferred: `crates/verify/ess-conformance/src/synthesize/related.rs`,
  `crates/verify/ess-conformance/src/synthesize.rs`
- inferred: `crates/verify/ess-conformance/src/mutate.rs` (a parameter-reference mutant, if the
  existing filter mutants do not reach it)
- inferred: `crates/verify/ess-conformance/tests/view_parameter_rows.rs` (extended)
- Size: L (about 500 changed lines).

## Depends on

- `g08-view-param-row`.
