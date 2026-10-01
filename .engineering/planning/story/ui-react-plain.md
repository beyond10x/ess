---
format: aep.planning-md/3
id: story:ui-react-plain
kind: story
status: active
title: 'ess generate ui --target react emits plain React: react and react-dom only, generated routing, one esbuild step'
refs:
- provider: github
  reference: beyond10x/ess#315
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: crates/edge/ess-cli/src/ui.rs
- confidence: cited
  path: crates/ui/ess-ui-react
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/ess-ui-test.md
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:49Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:49Z", actor: "human:timo", revision: 4}
---
## Outcome

`ess generate ui --target react` emits a plain React project: `react` and `react-dom` are its only runtime dependencies, page routing is generated into it (no router library), and it builds with one esbuild step (no Vite, no plugin).

## Fit review

Read at `origin/main` (`f86180f30`, 0.49.0). The issue has no comments.

1. **Need, apart from syntax.** The issue asks for an outcome: a generated React app whose runtime dependencies are only `react` and `react-dom`, with no router library, no build plugin, and one build step. Today `package.json.tmpl:13-24` declares `react-router` and `@vitejs/plugin-react`, and `vite.config.ts.tmpl:2,5` loads the plugin. The app shell imports `BrowserRouter`, `Routes`, `Route` and `Navigate` (`emit.rs:2317-2320`), shells import `Outlet` (`emit.rs:2247`), and five runtime templates import from `react-router` (`core.tsx.tmpl:12`, `navigation.tsx.tmpl:4`, `primitives/link.tsx.tmpl:3`, `shell.tsx.tmpl:4`, `state/url.ts.tmpl:1`). Minimal reproduction: a one-shell, two-page document shaped like the `VAULT` fixture (`ess-ui-react/tests/adversary_pass1.rs:86-111`); generating it lists `react-router` and the plugin (read from the code, not run).
2. **Class: gap** in generated output; no authored surface; `ess-ui/1` untouched.
3. **Already expressible? No.** `ReactArgs` has only `--path` and `--out` (`lib.rs:131-139`); editing generated output is forbidden by `README.md.tmpl:3-4`.
4. **Fit.** The output uses ten `react-router` symbols (`types/react-router.d.ts.tmpl:5-28`). `routes.ts` already emits `hrefFor` and `pageAt` (`emit.rs:2441-2459`), but `pageAt` ignores aliases and does not rank, while `App.tsx` routes aliases (`emit.rs:2336-2341`): the plain output removes that inconsistency. TUI, `ess ui test`, ess-ui-docs, ess-ui-check unchanged. The Playwright emitter needs no change: it uses `route_pattern` and `page.goto(path)` (`playwright.rs:278-282`), which needs a server answering every path with `index.html`; esbuild's `--serve-fallback` does that. Synth targets, verify diff, entity runtime not involved.
5. **Second adopter.** A team serving an internal dashboard as static files whose dependency review admits `react` and `react-dom` at runtime plus one pinned build binary; today it gets about 13 more direct packages (vite, rollup, postcss, babel via the plugin, react-router's cookie libraries).
6. **Cost.** No format bump, keyword, schema change or CLI flag. One new refusal: two pages routing the same path. Breaking for existing generated projects (regenerate into a clean directory: `generate` never deletes files, `lib.rs:107-118`). Vite HMR lost; the esbuild dev server rebuilds on request. One more pinned tool in CI (`ci.yml:305`). No known consumers.
7. **Designs.** D0 change nothing: rejected (edit "do not edit" output). D1 an option flag: rejected (two router stacks kept equivalent forever, doubled tests, a new flag, for a generator two days old with no consumer). D2 Vite without the plugin plus a generated router: rejected, fallback if esbuild serving proves inadequate. D4 tsc emit plus an import map: rejected (React 19 on npm is CommonJS only). D5 hash routing: rejected (every URL and `page.goto` path changes). **D3, chosen: replace with a generated history-API router plus the esbuild CLI.**

## Decisions

**Accept as proposed; replace, not option.**

- `package.json`: `dependencies` react and react-dom only; `devDependencies` `@types/react`, `@types/react-dom`, `typescript`, `esbuild ^0.28.0`. Scripts keep their names: `dev` = `esbuild src/main.tsx --bundle --jsx=automatic --sourcemap --outdir=www/assets --servedir=www --serve-fallback=www/index.html --serve=5173`; `build` = `tsc --noEmit && esbuild src/main.tsx --bundle --jsx=automatic --minify --outdir=www/assets`; `preview` = `esbuild --servedir=www --serve-fallback=www/index.html --serve=4173`; `typecheck`, `typecheck:offline` unchanged.
- esbuild strips types and compiles TSX (`--jsx=automatic`); CSS imported from `main.tsx` becomes `www/assets/main.css`; `tsc --noEmit` stays the checker.
- Removed: `vite.config.ts`, `types/react-router.d.ts`, the `react-router` line in `tsconfig.offline.json.tmpl:8`. `index.html` becomes `www/index.html`, loading `/assets/main.js` and `/assets/main.css` by absolute path.
- New runtime module `runtime/router.tsx`: `Router` (`window.location`, `popstate`, `pushState`/`replaceState`), `useLocation`, `useNavigate`, `useSearchParams` with react-router's signature so `url.ts` is unchanged, `Link` (an `<a href>` intercepting unmodified primary clicks), `Redirect` (replace; only when the target differs), `Outlet` and page params from contexts. The five runtime templates change only their import line.
- `routes.ts`: `matchPage(pathname) → {page, params} | undefined` over pages and aliases, params URI-decoded, case-insensitive with optional trailing slash, table pre-sorted so a static segment outranks a param; two patterns of identical shape are a `GenerateError` naming both pages; `pageAt` = `matchPage(...)?.page`; `route_pattern` unchanged.
- `App.tsx`: `Router > AppRoot > Pages`; unmatched paths redirect home; every page of one shell renders under that shell's element so the shell stays mounted; a page with no shell renders directly.
- No change to synthesis, the TUI, the `ess ui test` runner or the Playwright emitter's code; the docs gain a line that `page.goto` needs `npm run dev`/`preview` as `baseURL`.

## Acceptance

- `ess-ui-react/tests/partner_portal.rs::the_project_depends_on_react_and_react_dom_only`
- `partner_portal.rs::the_generated_project_type_checks_offline` (updated file list)
- `ess-ui-react/tests/bundle.rs::the_build_script_bundles_with_esbuild_against_stub_react`
- `bundle.rs::the_dev_server_answers_a_page_path_with_index_html`
- `ess-ui-react/tests/router.rs::a_page_path_matches_its_page_and_decodes_its_params`
- `router.rs::a_static_segment_outranks_a_param_segment`
- `router.rs::an_alias_path_reaches_its_page`
- `router.rs::two_pages_routing_the_same_path_are_refused`
- `router.rs::an_unknown_path_redirects_home_with_replace`
- `router.rs::a_link_click_pushes_history_and_a_modified_click_does_not`
- `router.rs::popstate_rerenders_the_matched_page`
- `router.rs::url_state_replaces_history_and_keeps_other_query_keys`
- `partner_portal.rs::every_page_of_a_shell_renders_under_one_shell_element`
- Guards kept green: `generation_is_deterministic` (`partner_portal.rs:159-174`), `ess-cli/tests/ui_commands.rs::generate_ui_react_writes_the_project_the_crate_renders`.

The `router.rs` tests compile with `tsc` to CommonJS and run under `node` with `window`/`history` stubs (pattern `adversary_pass2.rs:5-7,197-260`).

## Scope

- `crates/ui/ess-ui-react/src/emit.rs` (cited: `app()` `:2314-2395`, `Outlet` import `:2247`, `routes()` `:2397-2461`, doc `:146`), `src/assets.rs` (cited: `RUNTIME` `:23-64`, `project()` `:130-186`), `src/lib.rs:1`.
- Templates (cited): `project/package.json`, `index.html`, `tsconfig.offline.json`, `README.md`; delete `project/vite.config.ts`, `types/react-router.d.ts`; import lines in five runtime templates; new `runtime/router.tsx.tmpl` (inferred).
- Tests: `partner_portal.rs`, `adversary_pass2.rs` (router stub removed); new `tests/router.rs`, `tests/bundle.rs`.
- `crates/edge/ess-cli/src/ui.rs:60` doc; `website/docs/reference/cli.md:371-372`; `website/docs/reference/ess-ui-test.md:304-307`; `.github/workflows/ci.yml:305` (add `esbuild@0.28.2`); `CHANGELOG.md` `[Unreleased]` "Changed".
- Must not change: `schemas/ui/ess-ui.schema.yaml`, the `ess_ui` crate, `route_pattern` output, `data-ui-path`, `httpAdapter` paths (the binding story changes those), script names, `ReactArgs`, the TUI, determinism.

## Sequencing

No collision with #287, #310, #306 or #272; `origin/release/0.50.0` touches nothing here. Shared files: `CHANGELOG.md` (merge-time), `ci.yml` (only the TypeScript install step). Breaking change to generated output: ships in the minor after 0.50.0. Runs in parallel with the open units.
