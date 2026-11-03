---
format: aep.planning-md/3
id: story:ess-cli-flag-and-repeated-sources
kind: story
status: draft
title: ess-cli takes a flag over a required Boolean and repeated over a List
summary: ess-cli/1 and /2 accept flag over a required Boolean and repeated over List<T>; gap 3 is declined with a refusal that names it (docs/plan/wave-ess-issues-1.md:158)
refs:
- provider: github
  reference: beyond10x/ess#694
relations:
- decomposes: epic:adopter-gaps-generated-code
- serves: vision:O2
revision: 2
---
## Need

beyond10x/ess#694, kept by the 2026-11-02 reconcile (store hygiene H0 of wave W1, section 2f). The
generated CLI projections `ess-cli/1` and `ess-cli/2` should accept `flag` over a required Boolean and
`repeated` over `List<T>`; gap 3 of the issue is declined with a refusal that names it
(`docs/plan/wave-ess-issues-1.md:158`).

## Acceptance

- An `ess-cli` source of `flag` over a required Boolean input, and of `repeated` over a `List<T>`
  input, validates and generates a CLI that reads them.
- The declined gap 3 is refused by validation with a code that names it.

## Schedule

Roadmap W6 (0.89.0), unit CLI1.
