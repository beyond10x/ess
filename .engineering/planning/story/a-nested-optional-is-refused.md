---
format: aep.planning-md/3
id: story:a-nested-optional-is-refused
kind: story
status: draft
title: A nested Optional is refused or its three readings are told apart
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U1's review passes, pre-existing and out of that story's Acceptance: the validator admits `Optional<Optional<T>>` (`types.rs:163`), whose `null` cannot be told from absent on the wire.

## Acceptance

- `ess specify validate` refuses a type `Optional<Optional<T>>` with a named diagnostic citing the file and the declaration, or the specification documents what each of absent, `null` and a present `null` means and synthesis tells them apart.
- Committed models that use the type, if any, are named.
