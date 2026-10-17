---
format: aep.planning-md/3
id: story:row-queries-quantify-over-a-list-of-references
kind: story
status: draft
title: A guard quantifies over the rows a List of references names
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#558
relations:
- serves: vision:O2
- decomposes: epic:generic-row-queries
- informed_by: story:accepted-input-arranges-referenced-list-rows
revision: 1
---
## Outcome

A guard quantifies over the rows a `List` of references names: `when_related` over a `List<Id>`
input (or stored field) refuses when any element names no row, with the first missing identity
readable in the payload, as the existential and universal quantifiers of the row-query feature
(`each`, `forall`) rather than as a new key.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 1, quantifiers
over a selection. Parent: `epic:generic-row-queries`.

## Fit review

Finding covered: https://github.com/beyond10x/ess/issues/558 (a downstream incident-ledger
specification, ess 0.59.0, `ess/24`).

1. **Need.** A command takes a list of referenced identities and must refuse when one of them names
   no row. The model declares three `references` relations of cardinality many whose identities are
   never checked. Refusal today: `ESS-COMMAND-002 "input.service_ids is List<catalog.ServiceId>, and
   when_related reads the one row an identity names"`
   (`crates/specify/ess-domain/src/command/related_guard.rs:1387`). Requester's syntax (theirs):
   `when_related: {via: input.service_ids, exists: false}`.
2. **Class.** Gap: a `references` relation of cardinality many is declared and nothing can check it.
3. **Already expressible?** No. A row-set guard counts rows matching a filter; it cannot say "every
   element of this list has a row".
4. **Fit.** The same need appears from the synthesis side in
   `story:accepted-input-arranges-referenced-list-rows` (a one-element list naming no row is sent
   on the accepted branch). One quantifier rule serves both: the guard states the check, synthesis
   arranges the rows. The existing `collection-quantifiers-witnessed-everywhere` story names the
   witness obligations for quantifiers. Targets: validation, interpreter, synthesis (a list with one
   missing element first, middle, last), generated Rust and Go, Entity Runtime, `ess verify diff`.
5. **Second adopter.** A booking that names several rooms; a role assignment naming several
   permissions.
6. **Cost.** A source format version; one diff classification; no new diagnostic code expected.
7. **Alternatives.** (a) Change nothing: the relation stays unchecked. (b) The requester's
   `via: <list>` on `when_related`: one more cell. (c) A quantifier over the selection the list names
   (chosen); the requester's spelling may remain as sugar if the binding design keeps it.

## Decisions

Accept, redesigned into the generic feature's quantifier.

## Acceptance

- `a_missing_list_element_takes_the_refusal`: a list whose second element names no row takes the
  refusal and its payload names that element.
- `every_element_present_takes_the_accepting_branch`: synthesis arranges every named row.
- `an_empty_list_takes_the_accepting_branch`: stated by the binding design, witnessed.
