---
format: aep.planning-md/3
id: review-result:zi-t2-tokens-adv-2
kind: review-result
status: active
title: Turn 2 adversary pass 2 on story:the-react-renderer-draws-the-declared-tokens
relations:
- reviews: story:the-react-renderer-draws-the-declared-tokens
revision: 1
---
unit: story:the-react-renderer-draws-the-declared-tokens, tree zi-t2-5 at HEAD 163d934990 (2820724849 plus the correction)
verdict: broken
new failing cases: 6 (adversary_t2_tokens_p2_css.rs::check_refuses_a_family_with_an_open_quote, ::generate_refuses_a_family_with_an_open_quote, ::check_refuses_a_family_with_an_open_paren, ::generate_refuses_a_family_with_an_open_paren, ::generate_refuses_a_color_with_an_open_paren, ::the_stylesheet_keeps_no_literal_the_scanner_misses)
mutants survived: none (none run)
cases: executed 26→33, red 6
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: none

1. **What I changed:** `git diff --stat` is empty. `git status --short` shows one untracked test file, `crates/ui/ess-ui-react/tests/adversary_t2_tokens_p2_css.rs`. I touched no other file.

2. **The new cases, run alone first.** Command: `build.sh adv2/run1.txt <tree> test -p ess-ui-react --locked --test adversary_t2_tokens_p2_css`. Result: `test result: FAILED. 1 passed; 6 failed`, `exit 101`, `slot B waited 0s ran 15s`.
   - **Blocker: an open quote or bracket gets past both layers.** The correction refuses `; { } < > \` and comments. It does not refuse an unclosed `"`, `'`, `(` or `[`. In CSS, an unclosed string runs to the end of its line, and an unclosed bracket runs to its matching `)` or `]`, or to the end of the file. Either way the `;` is skipped, so the following declarations become part of the token's value and the browser drops them with it.
     - `family: '"Segoe UI, system-ui'` gets no finding from `ess ui check`: the failure prints `[]`. `generate` writes `--ui-type-body-family: "Segoe UI, system-ui;`. Because `type` entries are filled from `body`, the caption and heading families get the same open quote. The browser then drops the three family declarations plus `--ui-type-caption-size` and `--ui-type-heading-size`.
     - `mono: {family: 'local(Menlo, monospace'}` also gets `[]` from the check. It swallows all 10 `--ui-tone-*` declarations and the closing `}` of `:root`.
     - Color `'rgb(0 0 0 / 0.3'` (missing `)`) is refused by the check's color grammar. `generate` does not run the check, and its own refusal lets it through, so it writes `--ui-color-backdrop: rgb(0 0 0 / 0.3;`.
     - The failures assert at `adversary_t2_tokens_p2_css.rs:103` and `:120` (check) and `:88` (generate).
     - What reaches it: `ess generate ui` calls `ess_ui_react::run`, which loads the document and renders it without running the check. A family with a missing quote is an ordinary typo.
     - Fix (not applied): in `token_value_ok` (`assets.rs:314`) and `css_unsafe` (`ess-ui-check style.rs:132`), refuse a value whose quotes or brackets are unbalanced. Optionally, the generator could also apply the color, length and weight grammars.
   - **Weak: literals the scanner still misses** (`the_stylesheet_keeps_no_literal_the_scanner_misses`, `:185`):
     - `.ui-chart-svg polyline { stroke-width: 0.6 }` and `stroke-dasharray: 2 1` are length literals.
     - `.ui-expand-toggle { line-height: 1 }` is a type literal.
     - `.ui-spinner { border-radius: 50% }`.
     - `transparent` appears in `.ui-emphasis-ghost` and the table-frame gradients. It is a named color, which the color grammar excludes.
     - This is in the acceptance ("no color, length or type literal"). The 50% radius and the gradient stops can be argued as structural, so treat it as a backlog item, not a blocker.
   - `a_quoted_family_stays_legal` passes. It checks that a balanced `"Segoe UI", system-ui` stays accepted by both layers. I left it in place so the fix cannot refuse too much.

3. **The suite, after the cases existed.** Command: `build.sh adv2/run3-suite.txt <tree> test -p ess-ui-react --locked --no-fail-fast` over `adversary_t2_tokens_css`, `confirm`, `field_tone`, `live_binding`, `tokens` and `adversary_t2_tokens_p2_css`.
   - Results: `3 passed; 1 ignored` / `1 passed; 6 failed` / `1` / `6` / `11` / `5` passed. `exit 101`, `slot B waited 0s ran 45s`.
   - "Before" (26) is the implementor's `k-react.txt` run of the same targets.
   - `run2-suite.txt` was the same run without `--no-fail-fast`. It stopped after the failing target, so its counts are incomplete.

4. **Judgement finding, warning: the new name rule is stricter than the schema.**
   - The check (`style.rs:121`) and the generator (`assets.rs:306`) now refuse any token name outside `^[a-z][a-z0-9_-]*$`.
   - The schema still types these keys as `name`, i.e. `^[A-Za-z0-9_.-]+$` (`schemas/ui/ess-ui.schema.yaml:224`). Neither the schema's Tokens doc nor `docs/design/ui-style-tokens.md` mentions the narrower rule.
   - So `space: {2xl: 2rem}` follows the schema, is a valid CSS custom property, and is now refused.
   - No fixture or example uses such a name. CONFIRMED / introduced.

5. **Attacked and held:**
   - The 0.92 built-in table and the partner-portal example still generate. The example's `themes:` are never written to `tokens.css`.
   - Type and tone names are a closed set, both in the check (`token_names`) and in the generator.
   - Tone color references are checked with the same name rule.
   - No group can write a name with another group's prefix: every group writes its own fixed prefix, so nothing can produce `--ui-size-*`.
   - `\`, unicode escapes and control characters are refused. Very long values only make long declarations.
   - A theme's overrides go through the same `value_problems`. They share the open-bracket gap but never reach `tokens.css`.

6. **Written outside the worktree:**
   - `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/the-react-renderer-draws-the-declared-tokens/adv2/run1.txt`
   - `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/the-react-renderer-draws-the-declared-tokens/adv2/run2-suite.txt`
   - `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/the-react-renderer-draws-the-declared-tokens/adv2/run3-suite.txt`

7. **Findings block:**
```findings
- file: crates/ui/ess-ui-react/src/assets.rs
  line: 314
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: token_value_ok and the check's css_unsafe accept an unclosed quote or bracket in a family (and generate accepts one in any value), so the declaration runs past its `;` and drops later tokens, up to every tone and the closing brace of :root
- file: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: styles.css still holds stroke-width 0.6, stroke-dasharray 2 1, line-height 1, border-radius 50% and the named color transparent, none of which the tokens scanner checks
- file: crates/ui/ess-ui-check/src/style.rs
  line: 121
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: token names are now refused outside ^[a-z][a-z0-9_-]*$ while the schema still types them as name (^[A-Za-z0-9_.-]+$) and no document states the narrower rule
```
