---
format: aep.planning-md/3
id: story:a-held-state-refusal-reads-back-where-the-view-cannot-observe
kind: story
status: draft
title: A held-state refusal reads the record back where the view cannot observe it
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U2's adversary pass (`review-result:adversary-d368-u2-pass-1`), pre-existing and out of that story's Acceptance, inferred from the code and not run: on the held-state half of an input refusal, where an identity view exists but `observe_selection_subject` fails for a held state (`existence.rs:1275`), neither the unchanged-row claim nor the no-view probe runs, so that state's send requires only the error and no event.

## Acceptance

- A test model reaches that case, or the code shows it cannot be reached and the branch is removed.
- Where reached, the held state's send reads the record back (through the view or the probe), so a target that moves the record on refusal fails.
