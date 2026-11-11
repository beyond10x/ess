---
format: aep.planning-md/3
id: review-result:zi-t2-tokens-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:the-react-renderer-draws-the-declared-tokens
relations:
- reviews: story:the-react-renderer-draws-the-declared-tokens
revision: 1
---
unit: story:the-react-renderer-draws-the-declared-tokens
verdict: broken
new failing cases: 4 (adversary_t2_tokens_css.rs::a_type_family_cannot_write_a_second_declaration, ::a_token_name_cannot_close_the_root_block, ::every_property_styles_css_reads_is_declared_when_one_type_field_changes, ::the_stylesheet_keeps_no_type_or_viewport_literal)
mutants survived: none (no mutants run; see the gap noted at the end)

I ran `build.sh adv/run1.txt <tree> test -p ess-ui-react --locked --test adversary_t2_tokens_css` once. Result: `test result: FAILED. 0 passed; 4 failed`, `exit 101`, `slot A waited 332s ran 46s`.

1. **Blocker: a type family can override a fixed size.** `tokens.type.body.family: 'system-ui; --ui-size-nav-width: 0px'` passes `ess ui check` with no token findings. `tokens.css` then has `--ui-size-nav-width` twice, and the later copy wins. So the comment on `FIXED_SIZES` ("A document cannot change them") is false. `tokens_css` writes family values unescaped, and the sizes come before the type entries.
2. **Blocker: a color name can break out of `:root`.** A color named `'x: #fff; } body { display: none } :root { --ui-color-y'` also passes `ess ui check` with no token findings. It ends the `:root` block and adds a `body { display: none }` rule, which breaks the acceptance's "one `:root` block". Neither the loader nor the check limits token names (color, space, radius) or family to a safe set of characters. `ess generate ui` does not run the check, so values outside the grammar are written as they are too.
3. **Backlog: changing one type field drops others.** `tokens.type.heading: {family: 'Georgia, serif'}` passes the check, but `tokens.css` then has no `--ui-type-heading-size` or `--ui-type-heading-weight`. The `var()` reads of those in `h2`, `.ui-text-heading`, `.ui-sort` and `.ui-metric-value` then compute `unset`, so headings lose their 1.05rem size and 600 weight. This comes from the replace-whole rule plus a built-in `body` that has no size or weight. It goes against the spirit of acceptance (a).
4. **Weak: two literals remain.** `styles.css` still has `font-weight: normal` (line 34) and `min-height: 100vh` (line 12). The acceptance says "no type or length literal". The test's scanner misses them because it only looks for px, rem and em, and for values that a token holds.

What holds:
- `tests/fixtures/styles-0.92.css` matches the base's `styles.css.tmpl` byte for byte (checked with `cmp`).
- The `ui-emphasis-*` rename reaches every place the old button classes appeared at the base: `actions`, `confirm`, `filter_bar`, `form`, ess-ui-test `playwright.rs` and its tests, and the react tests. Docs and `emit.rs` had none.
- The 0.92 comparison only checks one way: old declarations must appear in the new stylesheet. Declarations the new stylesheet adds (`.ui-tone-neutral`, `.ui-tone-danger` color, `.ui-badge.ui-tone-*` background) and cascade order are never checked. A mutant that moves `FIXED_SIZES` after the type entries would also pass the suite.

The test file is left uncommitted in the tree because all four cases pin things the suite does not: `~/.local/state/worktree/trees/b10x/ess/zi-t2-5/crates/ui/ess-ui-react/tests/adversary_t2_tokens_css.rs`. Full output: `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/the-react-renderer-draws-the-declared-tokens/adv/run1.txt`
