---
format: aep.planning-md/3
id: story:field-sensitivity-class
kind: story
status: draft
title: A field can carry a sensitivity class from a declared vocabulary
relations:
- serves: vision:O2
- informed_by: story:a-secret-value-appears-in-no-output-of-the-process
revision: 3
---
## Outcome

A field can carry a sensitivity class, so a specification says which data is personal or secret
and every consumer (projections, the compiled IR, downstream stores) reads it from one place.

## Acceptance

- A system declares its class vocabulary once in a `sensitivity:` block (initial classes used by a
  downstream consumer: `pii.phone`, `pii.email`, `pii.name`, `pii.address`, `credential`); a field
  names one with `sensitivity: <class>`; an undeclared class is refused at validation with a
  diagnostic code and a repair.
- The class is carried in the compiled IR and in the machine-readable documentation export
  (`--kind docs-ir`), the formats a downstream generator reads.
- A new specification format version records the key; older versions refuse it by name; the
  schema, formats.md and spec-versions.md are updated; specifications without the key keep their
  compiled bytes.

## Decisions (2026-09-29)

- ESS does not generate the EKR ontology document. EKR owns that generator and reads ESS's compiled
  IR or docs-ir export, so ESS does not track EKR's format versions.

Append a section, text only (no acceptance line changes):

## Related (2026-11-24)

A class that the vocabulary marks secret is the one `story:a-secret-value-appears-in-no-output-of-the-process`
reads, to check that no event, answer or output line of a process under test carries a value of it,
for a field's value and for one a process reads from its environment. Whether a class is secret and
what that forbids is SV-D6 of `decision-blocker:secret-values-and-their-output-lines`, with the
owner; this story declares and carries the class and checks nothing about outputs.

## Fit review (2026-11-25)

A further finding of 2026-11-25, from a consumer: the marker is needed on a command's inputs as well as on an entity's fields. An
input carries a password, a client secret or a token as often as a field does, and the executor that runs the command has to keep it
out of logs, events and read views. A marker on fields alone leaves the input of the command that sets the field unmarked, so the
value is protected at rest and open in flight.

- **Class.** The same feature, a wider place for the key: `sensitivity: <class>` is accepted on a command input as well as on a field,
  against the same `sensitivity:` vocabulary, refused for an undeclared class with the same diagnostic, carried in the compiled IR and
  in `--kind docs-ir` for inputs as for fields.
- **What the story does not claim.** It declares and carries the class; what an executor does with it (redaction in logs, events and
  read views) is the consumer's, and what a conformance run counts as a line that carries a secret value is
  `story:a-secret-value-appears-in-no-output-of-the-process` with its decisions SV-D1 to SV-D6.
- **Format.** The same format step as the field key: one new `ess/N` records both places, so the step is not made twice.
