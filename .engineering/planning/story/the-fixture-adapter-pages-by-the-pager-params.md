---
format: aep.planning-md/3
id: story:the-fixture-adapter-pages-by-the-pager-params
kind: story
status: draft
title: The fixture adapter pages a read by the params its pager names
summary: The fixture adapter slices server-paged reads by the param names page and size, not by the pager (UI-adopter wave 1 U3 review note, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u3-adversary-pass-1
revision: 1
---
## Need

The generated fixture adapter pages a server-paged read by the param names `page` and `size`
(`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:141-142`). A collection whose `pager`
names other params (for example `p` and `n`) shows "Page 2 of 3" against the fixture adapter but
lists every row on every page, so a fixture preview does not show what a server would return. Found
by the UI-adopter wave 1 adversary pass on the page-size unit
(review-result:w086-u3-adversary-pass-1, note); pre-existing at 83391ebaeb.

## Acceptance

- The fixture adapter slices a server-paged read by the params the read's `pager` names (and by the
  `page` param of a read without a `pager`), never by a param's name.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
