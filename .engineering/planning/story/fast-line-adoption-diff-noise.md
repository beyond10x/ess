---
format: aep.planning-md/3
id: story:fast-line-adoption-diff-noise
kind: story
status: draft
title: Adopting ESS on an existing service produces a large unexplained diff
tags:
- fast-line
revision: 1
---
## Outcome

Adopting ESS on an existing service produces only the files a reviewer needs, and the documentation says which generated files are committed and in which repository.

## Origin

Fast-line intake, 2026-10-04. A downstream engineer reported on 2026-10-02 that trying ESS and AEP on an existing service for a small task produced a large diff of JSON, Markdown and YAML files, and asked whether to commit them and whether they belong in the service repository or a separate specification repository. After a plugin upgrade the diff was still large. It is not yet established whether the files come from ESS or from AEP planning.

## Open question

- Which tool wrote which files: reproduce on a small existing service with the current plugins and list every path written, by tool.
