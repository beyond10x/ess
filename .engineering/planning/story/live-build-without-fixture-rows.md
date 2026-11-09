---
format: aep.planning-md/3
id: story:live-build-without-fixture-rows
kind: story
status: draft
title: A live-bound generated UI can be built without fixture rows
tags:
- ui
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/lib.rs
revision: 3
---

Every claim is marked verified (seen in a run or in the file named) or inferred.

## Observed

- `ess generate ui --target react --model <spec>` writes `src/fixtures.ts` with every fixture row of the document's `fixtures:` block, and the esbuild bundle carries them. Verified: `react-out/src/fixtures.ts:36-37` holds `fixture-marker-lamp-7731` and `fixture-marker-chair-7732`; `grep -o 'fixture-marker-[a-z]*-[0-9]*' react-out/www/assets/main.js` finds both (ess 0.90.0, `transcript.txt`).
- The same app is bound to the served surface: `src/main.tsx:10` calls `setDataAdapter(httpAdapter(binding))`, so no read is answered from those rows. Verified. The rows are dead weight in a live build and they are visible to anyone who opens the JS.
- `ess generate ui --help` lists seven options (`--target --path --out --model --http --refines --strict-requires`); none switches fixture data off. Verified.
- Only workaround found: remove the `fixtures:` block from the document before generating. Verified: `fixtures.ts` then holds no row; `ess ui check --model` answers two `fixture_per_view` warnings per read and exit 0. The same document can no longer serve the offline/fixture build, so a project keeping one document must write a second, stripped copy at build time.
- Re-run on 0.92.0: the same result, `transcript-092.txt` (both markers in `www/assets/main.js`). Verified. Source: `crates/ui/ess-ui-react/src/lib.rs:113` inserts `src/fixtures.ts` unconditionally, also when `binding` is `Some`.

## Expected

A flag or an input of `ess generate ui` that emits no fixture data (an empty `fixtures` table in `src/fixtures.ts`), so a live build ships none, while `ess ui check` keeps the document's fixtures valid and unchanged. Inferred as the smallest shape; a `--model` run that drops them by default is the alternative and is the maintainers' choice.

## Reproduction

1. `sh run.sh` in `.engineering/repro/live-build-without-fixture-rows/` (ess 0.90.0, node, npm with network). It writes `transcript.txt`.
2. Inputs: `spec/` (system `store`, domain `store.shelf`, entities Box and Item, view `store.shelf.Items`), `ui/shelf.ui.yaml`, `ui/fixtures/items.yaml` (two rows).
3. Read in `transcript.txt`: the option list, the `fixture-marker` hits in `src/fixtures.ts` and in `www/assets/main.js`, and the stripped-document workaround.

## Acceptance

`ess generate ui --target react --model spec` with the new flag over a document that has a `fixtures:` block writes a project whose built `www/assets/main.js` contains none of the fixture rows' values, and the same command without the flag still contains them.

## Scope

- cited: crates/ui/ess-ui-react/src/lib.rs
- inferred: crates/edge/ess-cli/src/main.rs

confidence: low
