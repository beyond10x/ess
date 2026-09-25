---
format: aep.planning-md/2
id: story:adopter-reviewed-delta
kind: story
status: draft
title: An adopter's approval of a specification change is a committed delta
relations:
- serves: vision:O2
revision: 1
---
# An adopter's approval of a specification change is a committed delta

The adopter commits the JSON delta from `ess verify diff` beside the specification; `ess specify validate` refuses when the pinned previous revision no longer produces that delta, so an upgrade nobody approved cannot pass silently. Filed by ESS-EVOLUTION.md revision 3.
