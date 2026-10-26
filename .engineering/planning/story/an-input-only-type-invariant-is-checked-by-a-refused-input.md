---
format: aep.planning-md/3
id: story:an-input-only-type-invariant-is-checked-by-a-refused-input
kind: story
status: draft
title: An invariant on a type only command input reaches is checked through a refused input
summary: ESS-SYNTH-013 refuses a value invariant on a newtype used only in a stateless command input and asks for a view the domain cannot have; correct refusal, misleading message.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2. A newtype `of: Integer` with value invariants `[value >= 1, value <= 65535]`, used only in the input of a stateless domain's one command (no entity, no view), passes `ess specify validate --strict-requires` and is refused by `ess verify conform synthesize`:

```text
refused[ESS-SYNTH-013]: refusal[ESS-SYNTH-013]: type <domain>.Port has no scenario
  no view publishes a field position that can answer what `<domain>.Port` declares of every value
  help: publish a field that holds a value of this type in some view, ...
```

Reproduced on 0.69.2 and on the 0.70.0 unit E tree: 89 scenarios, 1 refusal. The adopter works around it with named refusals of its own command.

Classification: a correct refusal (no view can observe the invariant) with a misleading message. The help asks for a view, which a stateless domain cannot have. An invariant on a type that only reaches command input is an input check, and synthesis could exercise it by sending a value outside the range and expecting the command's input refusal.

## Acceptance

- `an_input_only_type_invariant_names_the_input_route`: for a type reached only through command input, ESS-SYNTH-013 says so and names what would let a scenario check it, instead of asking for a view.
- `an_input_only_type_invariant_is_checked_by_a_refused_input`: where the command declares how it answers an invalid input, synthesis writes a scenario sending a boundary value outside each invariant (here 0 and 65536) and expecting that refusal, and a value inside (1, 65535) accepted; the refusal then goes away.
