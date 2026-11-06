---
format: aep.planning-md/3
id: story:move-buttons-refuse-past-either-end
kind: story
status: draft
title: Move buttons refuse a move past either end
summary: 'The React collection''s degraded move buttons (no_drag: move_buttons) still send a move past the first or last row (0.88.0 collection-controls correction, pre-existing)'
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

Found by the 0.88.0 collection-controls correction, pre-existing: in the generated React app a
collection with `reorder` and `no_drag: move_buttons` draws move buttons that still send a move past
either end (up on the first row, down on the last), while the drag handle and the keyboard move now
refuse it.

## Acceptance

- `move_buttons_refuse_past_either_end`: the up button of the first row and the down button of the
  last row are disabled or send nothing.
