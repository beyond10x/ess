---
format: aep.planning-md/3
id: decision-blocker:document-form-rules-are-declared-in-the-specification-or-in-a-sibling-document
kind: decision-blocker
status: open
title: The owner picks where document-form rules b to e are declared (DF2-D1)
summary: Architecture decision for the story's design page; recommendation A, no default
relations:
- blocks: story:a-specification-declares-how-a-document-is-read-into-its-entities-and-the-generated-decoder-applies-it
revision: 1
---
# The owner picks where document-form rules b to e are declared (DF2-D1)

The design page `docs/design/document-form-rules.md` (branch `design/document-form-rules`, commit `087d2b67b03be0e6d28af6fa4f453dd2b9d65403`) does not confirm TD-D11 (`TD-D11: NOT CONFIRMED`). Architecture: the owner's; no default. Nothing of the blocked story is built until this is picked.

## Options

| option | what it does |
|---|---|
| A | A sibling document `ess-document-form/1` pinned to the specification, read by the document decoder only. No `ess/N`; follows `ess-http/1` and `ess-transport/1`. The page recommends it. |
| B | Keys on the type, field and entity declarations in the next free `ess/N` (TD-D11 as picked), each read by the document decoder only. Moves `RawSpecFile`, the schema, `macro-guards.json`, an `ess-diff` kind per rule and every projector's answer. |
| C | An `ess-normalization/N` extension. Its paths reach fields and list items, not map entries, and it has no cross-file set. |

## What waits

`story:a-specification-declares-how-a-document-is-read-into-its-entities-and-the-generated-decoder-applies-it`, and DF2-D2 to DF2-D6 behind it.
