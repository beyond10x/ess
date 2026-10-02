---
format: aep.planning-md/3
id: review-result:ui-tui-app-generator-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:ui-tui-app-generator (wave ui-live-apps-w4)
relations:
- reviews: story:ui-tui-app-generator
revision: 1
---
needs-change

Adversary pass 1 on story:ui-tui-app-generator at 31794caab. Cases executed 21 to 24, red 3, all introduced. Test file: crates/edge/ess-cli/tests/adversary_tui_app_pass1.rs.

- blocker: app names build/deps/examples/incremental go into [[bin]] name; cargo refuses the manifest.
- blocker: app is interpolated unescaped into main.rs doc comments; a line break yields a crate rustc cannot compile.
- warning: generation into a directory holding another crate replaces its Cargo.toml and src/main.rs.
- note: an ESS built between releases writes the previous release's tag, which lacks run_embedded/Embedded/ScreenSize.
- note: --screen-once accepts up to 65535x65535 with no area bound; -h omits ESS_UI_AUTHORIZATION; +12x+3 accepted.

Held: the generated crate resolves online by git tag without [patch]; a package named clap depending on clap builds; the built binary makes every base-url refusal ess ui run --tui makes before printing; --screen-once rejects 0x0 and 12x and prints 1x1 without touching the terminal; raw-string embedding covers any text; output is deterministic; --out as a file is refused; --model missing writes nothing; react unchanged.

```findings
[
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":70,"category":"boundary","severity":"blocker","verdict":"needs-revision","origin":"introduced","message":"crate_name passes app names build/deps/examples/incremental into [[bin]] name, which Cargo refuses to parse"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":108,"category":"boundary","severity":"blocker","verdict":"needs-revision","origin":"introduced","message":"app is interpolated unescaped into main.rs doc comments, so a line break in app yields a crate rustc cannot compile"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":51,"category":"judgement","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"generation into a directory holding another crate silently replaces its Cargo.toml and src/main.rs"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":26,"category":"contract-drift","severity":"note","verdict":"approve","origin":"introduced","message":"between releases the generated tag names the previous release, which lacks run_embedded/Embedded/ScreenSize, so the crate fails to compile"},
 {"file":"crates/ui/ess-ui-tui/src/lib.rs","line":165,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"ScreenSize accepts up to 65535x65535 with no area bound, so a large --screen-once allocates gigabytes in TestBackend"}
]
```
