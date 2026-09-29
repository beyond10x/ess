---
format: aep.planning-md/3
id: story:error-payload-sources
kind: story
status: implemented
title: An error outcome declares the sources of its error's fields
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T21:25:27Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T21:25:27Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T22:38:56Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An outcome that reports an error can declare where each of the error's fields comes from, with the
same `payload:` sources an event takes (input, subject field, caller attribute, generated value,
literal), so the error is fully determined: the interpreter carries the fields, the synthesized
suite checks them, and a generated behaviour fills them.

## Acceptance

- `payload:` on an error outcome is admitted in a new specification format version; an older
  version refuses it by name; validation refuses a source whose type differs from the error field's
  and a payload naming a field the error does not declare, with codes and repairs.
- The conformance interpreter carries the declared fields on the reported error; synthesized
  scenarios assert them; an error field with no declared source is still carried as none, as today.
- The rust behaviour classifier treats an error whose every field has a declared source as
  determined, so its command is generated.
- A downstream specification (118 commands) that declares sources for the 17 error outcomes it
  reports as "no source" gets those commands generated.
- Specifications without error payloads compile to the same bytes.

## Origin

After generated behaviours, 17 of a downstream specification's 21 remaining command obligations
were kept only because their errors' fields had no declared source.
