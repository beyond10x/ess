---
format: aep.planning-md/3
id: review-result:ui-tui-app-generator-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:ui-tui-app-generator (wave ui-live-apps-w4)
relations:
- reviews: story:ui-tui-app-generator
revision: 1
---
needs-change

Adversary pass 2 on story:ui-tui-app-generator at 65d68ab31. Cases executed 877 to 882 (ess-cli), red 5, all introduced. Test file: crates/edge/ess-cli/tests/adversary_tui_app_pass2.rs.

- warning: --screen-once exits 0 when the home read is refused (403) or the server is down.
- note: a backtick in app escapes the doc-comment code span; the crate fails clippy -D warnings.
- note: with --out/src a regular file, Cargo.toml is written before the refusal.
- note: the guard writes through a symlinked --out/Cargo.toml.
- note: regenerating from the crate's own src/ui.yaml prepends a second mark line.

Held: escaping of U+0000, U+FEFF, combining marks, 4-byte emoji, U+2028/2029, bidi marks, \r, \t, U+007F, U+0085, U+10FFFF and a trailing backslash; the binding round-trips byte for byte; crate names from empty, '-', non-ASCII, std, core, test, self, fn, alloc, proc_macro, Clap, con all build; edited-but-marked files are replaced; mixed marks refused; extra files allowed; dangling symlink refused; fixed paths leave no stale files after a rename; %YAML/%TAG/--- documents still load; the generated crate is clippy -D warnings clean and builds in release; a slow server is bounded by 15 s.

```findings
[
 {"file":"crates/ui/ess-ui-tui/src/lib.rs","line":217,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"--screen-once exits 0 and prints a frame when the home read is refused (403) or the surface is down"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":174,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"a backtick in app escapes the doc-comment code span and fails clippy -D warnings"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":83,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"with --out/src a regular file, Cargo.toml is written before the refusal"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":66,"category":"boundary","severity":"note","verdict":"approve","origin":"introduced","message":"the guard reads and writes through a symlinked --out/Cargo.toml"},
 {"file":"crates/ui/ess-ui-tui/src/generate.rs","line":53,"category":"property","severity":"note","verdict":"approve","origin":"introduced","message":"regenerating from the crate's own src/ui.yaml prepends a second mark line each time"}
]
```
