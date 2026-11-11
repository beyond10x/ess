---
format: aep.planning-md/3
id: story:page-numbered-paging-declares-a-default-and-a-maximum-size
kind: story
status: draft
title: Page-numbered paging declares a default and a maximum size
summary: 'paging: with page: has no default_size or max_size key on 0.93.0 (adopter report)'
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- depends_on: story:a-view-answer-takes-its-protocols-list-shape
revision: 1
---
## Need

A downstream adopter's page-numbered list reads apply a default page size when the caller sends
none and reduce a size above a maximum to that maximum. `paging:` with `page:` has no key for
either.

Reproduced on 0.93.0 (`ess/37`; `view-paging.yaml` with one key added to `paging:`):

```text
paging: {page: page, size: size, total: true, default_size: 10}
  - system.yaml: unknown field `default_size`, expected one of `page`, `size`, `first_page`, `total`
paging: {page: page, size: size, total: true, max_size: 100}
  - system.yaml: unknown field `max_size`, expected one of `page`, `size`, `first_page`, `total`
```

`story:a-view-answer-takes-its-protocols-list-shape` adds `default_size:` for continuation paging
(`cursor:`). This story is the page-numbered form and the maximum.

## Acceptance

- `paging: {page: …, size: …, default_size: N}` validates where the size parameter is `Optional`:
  a read sending no size answers pages of `N` rows.
- `max_size: M` validates: a read asking for more than `M` rows answers `M`, and the answer's
  paging facts state the size applied. `default_size` above `max_size`, and either below 1, are
  refused by name.
- Synthesis writes a scenario for the absent size and one for a size above the maximum; a target
  that answers every row, or refuses the large size, fails.
- OpenAPI states the default and the maximum on the size parameter.
- A view without the keys keeps its IR bytes and its suite bytes.
