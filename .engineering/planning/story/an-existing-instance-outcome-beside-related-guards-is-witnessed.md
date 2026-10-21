---
format: aep.planning-md/3
id: story:an-existing-instance-outcome-beside-related-guards-is-witnessed
kind: story
status: active
title: An existing-instance outcome beside related guards is witnessed
summary: 'Synthesis reaches existing_instance: true behind related catalogue guards, or names the blocking declaration'
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:50:14Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-21T22:50:14Z", actor: "human:timo", revision: 4}
---
## Story

As a specification author, an `existing_instance: true` outcome on a create command whose success branch sits
behind guards on a related catalogue row is witnessed by synthesis, or refused by a diagnostic that names the
declaration that would let synthesis reach it, so that `validate` and `compile` passing is not followed by a
refusal that points at no fix.

## Fit review

1. **Need.** A create command `Select` makes an instance `Selection` (resting in `Selected`) from input fields
   that must match a related catalogue row (two guards: the row exists for the input's key; the row's revision
   equals the input's revision). A further outcome `selection-exists` (`existing_instance: true`) refuses an id
   already held. `ess specify validate --strict-requires` and `ess specify compile` exit 0; `ess conformance
   synthesize` refuses with ESS-SYNTH-004: "the route runs through `Select/selected`, which no input reaches".
   Removing both catalogue guards gives 0 refusals; removing either one alone still gives 1 (requester's
   variant table, reported by session `b10x-chef-orchestrator` on 2026-10-22 from a downstream adopter at
   ESS 0.67.0). The requester proposes no syntax.
2. **Class: defect.** The `selected` branch is reachable (the unchanged specification synthesizes it with 0
   refusals), so ESS-SYNTH-004's "which no input reaches" is false for the prerequisite route. Inferred by the
   requester from source, not traced: the existence route builds a fresh catalogue row that is not steered to
   the input it chose.
3. **Expressible today?** Only by declaring both catalogue outcomes `external:`, which drops their
   conformance. No idiom keeps both guards checked. I don't know of another.
4. **Format consequence:** none expected; synthesis output for specifications that synthesize today must not
   change (byte-stable).

## Acceptance

- `existing_instance_beside_related_guards_is_witnessed`: a minimal ESS-owned specification with the shape
  above synthesizes a scenario `Select/outcome/selection-exists` with 0 refusals, and it passes against
  `interpreted`.
- `existing_instance_beside_one_related_guard_is_witnessed`: the same with only the key guard, and with only
  the revision guard: 0 refusals each.
- `prerequisite_route_refusal_names_the_blocking_declaration`: where synthesis still cannot build the
  prerequisite instance, the refusal names the guard or outcome that blocks the route, not "no input reaches"
  for a branch other scenarios reach.
- `synthesis_bytes_unchanged_without_existing_instance`: the existing synthesis byte-stability tests stay
  green.

## Scope

`crates/verify/ess-conformance/src/synthesize/**` (existence prerequisite construction, related-row
arrangement), `crates/verify/ess-conformance/tests/**`; `crates/generate/ess-synth/**` only if the arrangement
lives there.

## Decisions

- Taken by the coordinator: either outcome the requester named is acceptable; reaching the guard is preferred,
  a named refusal is the fallback when the arrangement is not decidable.
