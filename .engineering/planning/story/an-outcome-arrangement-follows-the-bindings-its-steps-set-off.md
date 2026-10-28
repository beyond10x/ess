---
format: aep.planning-md/3
id: story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off
kind: story
status: draft
title: An outcome arrangement follows the bindings its steps set off
summary: A related-guard arrangement that sets off a binding rewriting the arranged row no longer yields a scenario the interpreter fails
tags:
- ess-0.74.0
relations:
- informed_by: review-result:ess-0730-wave-adversary-pass-1
revision: 1
---
## Problem

Found by the 0.73.0 adversary pass (finding F2). It already fails on the released 0.72.0.

A command `Give` refuses `not-flagged` through `when_related: {via: input.grant_id, predicate: flag == false}`.
To reach that branch, synthesis arranges a grant with `flag == false` through `Lower`. `Lower`
publishes `Lowered`, and the binding `raise-on-lowered` answers it by setting the flag back to
true. At the time `Give` is sent the grant is flagged again, so the interpreter answers `given`
and the synthesized outcome scenario `Give/outcome/not-flagged` fails. On 0.72.0 the model
synthesizes 16 scenarios, of which 15 pass and 1 fails.

Cause, read from the code: the related-guard arrangement (`synthesize/related_guard.rs`, about
:3136 and :4118) treats the row its steps leave as final. It does not follow a binding those
steps set off. `when_subject` searches likely share the gap (inferred).

## Acceptance

- `adversary_0730_pass1::f2_a_binding_that_rewrites_the_arranged_row_is_followed` asserts no
  failing scenario. Its pinned known failure is removed in the same change.
- An outcome scenario whose arrangement sets off a binding that rewrites the arranged row either
  follows that binding (and arranges a row that still refuses) or is refused by name. It never
  fails against the interpreter of the same model.
