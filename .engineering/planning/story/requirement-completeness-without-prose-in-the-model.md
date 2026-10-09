---
format: aep.planning-md/3
id: story:requirement-completeness-without-prose-in-the-model
kind: story
status: draft
title: Requirement completeness is checkable without requirement prose in the model
refs:
- provider: github
  reference: beyond10x/ess#503
relations:
- serves: vision:O2
revision: 3
---
## Outcome

Requirement completeness (every MUST/SHOULD/MAY of a source standard is either covered by a
declaration or recorded as deliberately unmapped) is checkable without requirement text in the
specification. Requested in https://github.com/beyond10x/ess/issues/503 (part C).

## Fit review

- The requested `requirements:` block with `text:` contradicts `refs.rs:8-13` ("a construct names
  the record and the record keeps the prose"); `satisfies:` would be a second spelling of `refs:`;
  `unmapped:` overlaps `epic:typed-open-questions` (`open:` entries).
- Proposed design: under source format `ess/24`, `refs:` on the remaining constructs (types,
  entities, transitions, inputs, events, views); an unmapped disposition through the `open:`
  vocabulary; the register read as an external input (`--requirements <file>`) for a
  `completeness` section in validation. Not expressible as an idiom today.

## Decisions

- Deferred to the `ess/24` plan (0.59.0). Whether ESS holds requirement text at all is settled
  there; the default is no.

## Acceptance

- Written when the `ess/24` scope is settled.
