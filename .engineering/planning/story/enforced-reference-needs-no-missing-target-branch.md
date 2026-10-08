---
format: aep.planning-md/3
id: story:enforced-reference-needs-no-missing-target-branch
kind: story
status: draft
title: A reference whose target cannot be missing needs no missing-target branch
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#509
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Validation does not demand an `exists: false` branch for a related row the model makes
unreachable: a reference whose target is created before the referencing row and never deleted.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/509 (ess 0.56.0): with the `no-server` branch of the
https://github.com/beyond10x/ess/issues/505 model deleted, validation reports ESS-COMMAND-005
`non_exhaustive_branches`, though the creating command guards the target's existence and no
command deletes it.

## Acceptance

- The specification can state that a relation's integrity is enforced (or derive it from the
  creating guard and the absence of a deleting command); a command reading it then needs no
  missing-target branch.
- A command or binding that can delete the target makes the declaration refused, naming that
  command.
- Conformance checks the enforced integrity: a target that deletes the target row fails.
