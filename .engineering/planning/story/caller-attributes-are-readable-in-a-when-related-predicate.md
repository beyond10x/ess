---
format: aep.planning-md/3
id: story:caller-attributes-are-readable-in-a-when-related-predicate
kind: story
status: draft
title: Caller attributes are readable in a when related predicate
summary: 'Adopter report G1: A when_related predicate cannot read a caller attribute (ess#636).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#636
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

A `when_related` predicate reads the related row's stored fields, `input.<field>` and `state`, but not `caller.<attribute>`, although `when:`, `when_subject` and `sets:` read it (#168). A refusal for a related row another caller's account owns cannot be stated (ESS-COMMAND-003, `caller` is not a declared observable root). Adopter report G1, ess 0.67.0 and 0.68.0.

## Acceptance

- `a_when_related_predicate_compares_a_stored_field_with_a_caller_attribute`: the adopter shape (`predicate: account_id != caller.account_id` under `via: input.parent_id`) validates at the new format and is refused below it by name.
- `a_caller_attribute_the_actor_does_not_declare_is_refused_in_when_related`: an undeclared attribute keeps ESS-COMMAND-003 with a hint naming the actor's attributes.
- `the_other_account_refusal_is_witnessed_with_a_second_caller`: synthesis arranges the related row under one caller and sends the command as a caller whose attribute differs, expecting the refusal; the same caller reaches the accepting branch.
- `the_interpreter_answers_the_caller_scoped_related_refusal`: the interpreted target passes both scenarios; a target that ignores the caller fails the refusal scenario.
