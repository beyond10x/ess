---
format: aep.planning-md/3
id: story:ui-spec-style-tokens
kind: story
status: draft
title: An ess-ui document declares design tokens, themes and user preferences
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:ui-spec-schema
scope:
- confidence: cited
  path: crates/ui/ess-ui
revision: 2
---
## Outcome

An `ess-ui/1` document declares design tokens (colour, spacing, type, radius, elevation) with theme
variants, composites and regions refer to tokens instead of raw values, and the document declares
which options the end user may set as preferences (theme, density, language) — renderer-neutral, so
a TUI maps what it can and degrades the rest.

## Acceptance

- A `tokens:` block: named token groups (`colour`, `space`, `type`, `radius`, `elevation`) with
  typed values, and `themes:` whose variants override token values (at least `light` and `dark`).
- Style references: a composite, primitive, widget or region names tokens (`tone: {token: colour.danger}`,
  `padding: {token: space.md}`) and never a raw value unless a `raw:` escape is declared; a check
  refuses a reference to a missing token and a theme that leaves a token undefined.
- A `preferences:` declaration lists the end-user options (theme, density, language, and any other
  enumerated option) with their values, default and state placement (`store:`), replacing the ad-hoc
  `theme` preference state.
- Renderer profile capabilities `no_colour` and `fixed_density`, with `degrades` entries (a TUI maps
  colour tokens to its palette or to emphasis, and ignores density).
- The schema carries doc data for every new construct; the partner-portal example uses tokens,
  two themes and preferences.

## Origin

A voice-driven UI editor plans Components, Styles and Docs workspaces over one document (R8).
