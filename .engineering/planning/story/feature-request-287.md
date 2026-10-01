---
format: aep.planning-md/3
id: story:feature-request-287
kind: story
status: active
title: A singleton entity can be declared and synthesized
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#287
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T16:38:14Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-01T16:38:15Z", actor: "human:timo", revision: 6}
---
## Outcome

A specification declares an entity with exactly one row, and synthesis witnesses it.

## Acceptance

- A one-value identity synthesizes with no ESS-SYNTH-001.
- A second install is refused, and the refusal is witnessed.

## Origin

beyond10x/ess#287, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

1. Need: a system has exactly one row of an entity (one pause switch), so a guard on that row holds for every caller. Requester's forms: a one-value identity that synthesizes, or an explicit singleton declaration.
2. Class: gap. A one-variant enum identity validates but synthesis refuses 5 branches with ESS-SYNTH-001, "too few values to name an identity" (reported on 0.48.0; not re-run here).
3. Partial idiom: a `String` identity that the creating command refuses for every value but one, plus `existing_instance:`. It synthesizes with 0 refusals. Its caller-swapped rerun failed only because of #275; #275 as merged now keeps the first run with an `UnswappedCallers` note when no fresh in-guard value is left (`synthesize/caller.rs`, `tests/adversary_275_pass2.rs`). This is inferred, not run on this shape.
4. Fit: an explicit `singleton:` key would be a second way to say "one identity value". The one-variant enum already says it in the type system and needs no new surface, so synthesis should arrange the one row and stop asking for a second value.
5. Second adopter: a system-wide settings row, or one tenant configuration per deployment.
6. Cost: synthesis only; no format bump, no new keyword. Refusals that need a second row (the swapped run, `existing_instance` second create) must use the one row, or note why not.
7. Considered: (a) change nothing, and document the String idiom (works after #275, but the guard is noise); (b) a `singleton:` key (new surface, duplicates the enum); (c) a one-variant enum synthesizes (chosen: no surface).

## Decisions

- **accept, redesigned:** no `singleton:` key. An identity whose type has one value (a one-variant enum) synthesizes: existence scenarios arrange the one row, a second create is witnessed as `existing_instance`, and the caller-swapped run keeps the first run with a note. Synthesis only, no format change. Check the String-guard idiom on 0.49.0 (#275) first, and reply to the requester with it as the interim answer.

- Coordinator (2026-10-01): decided as above.

## Evidence

- Evidence 2026-10-01 (requester, ess 0.49.0): the String-guard idiom is **not** enough. A String identity accepting only one value, plus `existing_instance`, still refuses 7 branches with ESS-SYNTH-001; a one-value enum refuses 5. The coordinator's earlier inference that #275 covers it was wrong. The decided fix stands, raised to priority 1.
