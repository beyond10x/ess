---
format: aep.planning-md/2
id: story:outcome-shapes-beyond-ess-14
kind: story
status: draft
title: Create into a state, delete a subject, answer an unknown id, accept with no subject, seed the explorer
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 2
---
## Scope

- #145: an `unknown_instance:` answer that is not an external refusal (0.35.1 covers the external
  case).
- #150: `creates:` into a declared non-initial state.
- #151: an outcome that deletes its subject; synthesis asserts absence from the entity's views.
- #144: an accepted no-op on a command with no subject (`preserves:` covers the subject case).
- #152: an ambient precondition (declared seed state) the explorer and synthesis set up first.

## Acceptance

Each construct validates under a new source format, is refused below it, and has a synthesized
scenario the issue's repro passes against a correct implementation.
