---
format: aep.planning-md/3
id: story:a-seed-says-it-does-not-serve-a-set-move-filter
kind: story
status: implemented
title: A seed says it does not serve a set-move filter
summary: A selected seed row of a set-move filtered entity is named in the refusal, which says seeds do not serve a set move filter; the CLI help and design page state the two obligations a seed serves
tags:
- ess-0.75.0
relations:
- informed_by: story:a-synthesis-seed-is-accepted-for-a-caller-model
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T08:42:08Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "proposed", to: "active", at: "2026-10-28T08:42:09Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "active", to: "implemented", at: "2026-10-28T10:58:28Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Problem

An adopter reported this against 0.74.0 (gap R). The reduced reproduction is 49 lines with no caller attributes.

Model:
- `Cap` has the lifecycle Held ⇄ Parked.
- `Park` and `Resume` are set moves that select `instances: {where: owner == input.owner}`.
- Only `Park` leads into `Parked`.

Without a seed, `Resume/outcome/resumed` is refused as ESS-SYNTH-001: a state only a set move reaches is not arranged, so there are no three rows the filter selects. The run reports 2 selected and 1 refused.

With `--synthesis-seed` naming one `Cap` row in `Parked`, the result is the same: 2 selected, 1 refused, the same text. The suite's `provenance.synthesis_seeds` records 1 selection and 0 applications. The adopter's specification behaves the same with 1, 2 or 3 seed rows.

Observed on the 0.74.0 head 846a40ddf.

Classification: a limit stated in the design, but neither the CLI help nor the refusal states it.
- `docs/design/synthesis-seeds.md` § Arrangement and obligation tries a seed only at two obligations:
  - the primary witness of a branch that reads the stored subject row;
  - one side of a stored counter's limit.
- A set move's filter needs several selected rows. § Not in this design excludes arbitrary multi-row snapshot seeding.
- The `--synthesis-seed` help says a row is established "for a generated obligation no bounded arrangement reaches". This refusal reads as that case.
- The refusal admits the seed as a selection and never says the seed does not serve this obligation.

## Acceptance

- When a seed is selected but no obligation it could serve exists, synthesis says so:
  - The refusal of a set-move filter (`instances:` or `affects:`) names each selected seed row of the filtered entity.
  - The refusal says seeds do not serve a set move's filter.
  - The refusal's help points to an authored scenario (`ess-scenario/1`).
- The `--synthesis-seed` help and the synthesis-seeds design page name the two obligations a seed serves, and say a set move's filter is not one of them.
- On the reduced reproduction, the suite bytes stay the same apart from the refusal text. The provenance keeps recording the selection and an empty application list.
- Seeding the rows of a set-move filter is a design change. It would need a format decision and its own design section, and is not part of this story.
