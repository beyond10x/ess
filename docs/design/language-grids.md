# The language grids: what can be read where, which operation applies to which type

Status: tracked. The grids of [`generic-language-features.md`](generic-language-features.md) § "The
two grids", moved here so one page holds them (`story:the-language-policy-is-counted-and-cited`).
Each change that opens or refuses a row or a column of either grid updates this page in the same
change, and the fit-review skill (`.agents/skills/assessing-external-requests/SKILL.md`) triages
a request onto their cells. The closing rule stays with the record that decided it
([`generic-language-features.md` § "The two grids"](generic-language-features.md#the-two-grids)).

Paths are relative to the repository root; `dom` is `crates/specify/ess-domain/src`, `prim` is
`crates/specify/ess-primitives/src`, `conf` is `crates/verify/ess-conformance/src`, `synth` is
`crates/generate/ess-synth/src`, `comp` is `crates/specify/ess-compiler/src`. Each "today" cell
cites the tree the record cites (`0e862220b`) unless a later change updated it here.

### Grid R: what can be read where

Rows are positions, columns are roots. "yes" is admitted today, "no" refused, "part" admitted in a
narrower form, "n/a" has no meaning at that position. The last column cites where the row is
decided today.

| position | input | subject field / state | related row (by reference) | row set | caller | `now` | view param | today's row is decided at |
|---|---|---|---|---|---|---|---|---|
| `when:` | yes | no (separate key) | no (separate key) | no (separate key) | part: `==`/`!=` against a field (`ess/16`) | yes (`ess/16`) | n/a | `dom/command.rs:3263-3265`; `dom/command/caller_value.rs:359-363, 422-428`; `dom/expression.rs:487-505` |
| `when_subject:` | yes (`ess/15`) | yes; `state` from `ess/18` | no | no | I don't know | yes (`ess/22`) | n/a | `dom/command/subject_fact.rs:370-384, 403-405, 446-479` |
| `when_related:` predicate | yes | no | yes, its own row; `state` from `ess/20`; each row a `List` of references names, `exists: false` only (`ess/28`) | no: one kind of related row per command | I don't know | yes (`ess/22`) | n/a | `dom/command/related_guard.rs:1440-1487`; `dom/command/row_set.rs:295-297` |
| row-set `where:` / `forall:` | yes | yes (`subject.`) | candidate row only | the selection itself | I don't know | yes (`ess/22`) | n/a | `dom/command/row_set.rs:242-251, 269-284` |
| `instances:` / `affects:` `where:` | yes | `affects:` only | candidate row only | the selection itself | no | no | n/a | `dom/command/set_effects.rs:1738-1757, 1766-1800` |
| entity / type invariant | n/a | own fields and `state` | no | no | n/a | no | n/a | `dom/entity.rs:970-978, 1053-1057` |
| view `filter:` | n/a | source row fields and `state` | part: one hop through a reference of the source, `<reference>.<field>` (`ess/28`) | no | no | no | yes | `dom/view.rs:1154-1157`; `dom/expression.rs:2268-2287` |
| measure `where:` | n/a | source row fields | no | no | no | no | yes | `dom/view.rs:1194-1226` |
| measure input | n/a | part: one top-level field | no | no | no | no | no | `dom/view.rs:1316-1328`, `361-372` |
| view field | n/a | part: one source field, no computation | part: one hop, `field: {related: {via, field}}` (`ess/28`) | no | no | no | no | `dom/view.rs:1302`; `read-api-view-idioms.md:191-192` |
| `sets:` (subject) | yes | yes (`ess/14`) | yes (`ess/16`; two hops `ess/22`; `state` `ess/28`) | part: exactly one row (`ess/22`) | yes (`ess/16`) | no | n/a | `dom/command/value_expression.rs:339-462, 1018-1050`; `dom/command.rs:1162-1199` |
| `sets:` under `instances:` / `affects:` | yes | no | no | no | no | no | n/a | `dom/command/set_effects.rs:38-41` |
| event `payload:` | yes | yes | yes; `state` `ess/28` | part: exactly one row | yes | no | n/a | `dom/command/value_expression.rs:339-462` |
| error `payload:` | yes | part: only the row the refusal answers for (`ess/19`) | I don't know | I don't know | I don't know | no | n/a | `dom/command/value_expression.rs:151-188`; `dom/command.rs:4085-4108` |
| binding cause `where:` | n/a (root `event`) | n/a | n/a | n/a | n/a | no | n/a | `dom/binding/condition.rs:1-22, 279-305` |

From `ess/28` a value reads a related row's `state` (https://github.com/beyond10x/ess/issues/557)
and a view field or filter reads the row a reference of its source names, one hop
(https://github.com/beyond10x/ess/issues/564). No position reads `now` as a value
(`dom/command.rs:1070-1217` has no such source).

**Target.** Columns, closed whole:

- **input**: every command position (all rows above except views, invariants and bindings).
- **subject field and state**: every command position, including the error payload of any refusal
  answered after `Existence` (https://github.com/beyond10x/ess/issues/555), and `sets:` under
  `instances:`/`affects:` as the acting command's subject.
- **related row and row set**: every position where rows exist, which is every command position
  and every view position (G2), and invariants are excluded by name (an invariant holds of one row
  at all times; a cross-row invariant is a different construct, open question 9).
- **caller**: every command position. Views read no caller; read authority is the view grant
  (`dom/actor.rs:183-211`).
- **`now`**: every command position, every view position (a request clock), not invariants (an
  invariant that changes truth with time is not an invariant).
- **view param**: every view position.

Rows, closed whole: the **binding cause `where:`** takes the full expression language over its
`event` root (https://github.com/beyond10x/ess/issues/492, item 1); **measure input** and **view
field** take any expression over their row (G3).

### Grid O: which operation applies to which type

| operation | Integer | Decimal | Binary64 | Timestamp | Duration | String | enum | Boolean | identity | List / Map | today's row is decided at |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `==` `!=` | yes | yes | yes | yes | yes | yes | yes | yes | yes | no | `dom/expression.rs:2566-2571`; identity `dom/expression/lexical.rs:990-1050` |
| `<` `<=` `>` `>=` | yes | yes | yes | yes (instant, against Timestamp or literal) | no | yes (bytes) | I don't know | no | no | no | `dom/expression.rs:2585-2601, 2322-2337`; `prim/predicate.rs:1809-1928` |
| `in` / `not in` | yes | yes | yes | yes | yes | yes | yes | yes | I don't know | no | `dom/expression.rs:3122-3155` |
| `+ k` / `- k` constant offset, right side | yes | no | no | yes (`s`/`m`/`h`) | no | no | no | no | no | no | `dom/expression.rs:2544-2556, 2658-2691` |
| fact `+ - * /` fact | no | no | no | no | no | no | no | no | no | no | `dom/expression.rs:1464-1481, 2700-2734` |
| `Timestamp - Timestamp` | n/a | n/a | n/a | no | n/a | n/a | n/a | n/a | n/a | n/a | same |
| `starts_with` `ends_with` `contains` | n/a | n/a | n/a | n/a | n/a | yes (`ess/8`; param/input operand `ess/22`) | no | n/a | n/a | n/a | `dom/expression.rs:2782-2852, 2911-2995` |
| ignore-case equality, `in` | n/a | n/a | n/a | n/a | n/a | yes (`ess/15`) | no | n/a | n/a | n/a | `dom/expression.rs:3001-3060` |
| length | n/a | n/a | n/a | n/a | n/a | `.count` (`ess/11`), `utf8_bytes` (`ess/22`) | n/a | n/a | n/a | `.count` | `dom/expression.rs:897-905, 942-1005, 1036-1083` |
| `lower` `upper` `trim` | n/a | n/a | n/a | n/a | n/a | no | n/a | n/a | n/a | n/a | absent from `Predicate` (`prim/predicate.rs:1278-1367`) |
| `defined()` | Optional of any type | | | | | | | | | Optional aggregate from `ess/16` | `dom/expression.rs:3087-3121` |
| `coalesce(x, d)` | yes | yes | no | yes | yes | yes | no | yes | no | no | a value from `ess/27`; a command guard's operand, structured from `ess/27` and on the left of a compact comparison from `ess/29`, an absent `x` reading as `d` (beyond10x/ess#620); `prim/expr.rs` `Function::Coalesce`, `prim/predicate.rs` `Derived::value` |
| `forall` `exists` `distinct` | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | yes | `dom/expression.rs:2470-2471` |
| measure `sum` `avg` | yes | yes | no | no | no | no | no | no | no | no | `dom/view.rs:865-909` |
| measure `min` `max` | yes | yes | no | yes | no | yes | no | no | no | no | `dom/view.rs:865-909` |
| measure `count_distinct` | yes | yes | no | yes | no | yes | yes | yes | Uuid | no | `dom/view.rs:865-909` |

**Target.** Rows closed whole, by the operations table above: ordering applies to every ordered
type (Duration and an enum with a declared order join; https://github.com/beyond10x/ess/issues/566);
arithmetic applies to Integer and Decimal everywhere, not on the right of a comparison only; time
arithmetic applies to Timestamp and Duration; every text function applies to String and its
newtypes; `min`/`max` apply to every ordered type, `sum`/`avg` to Integer, Decimal and Duration.
Columns closed whole: Duration gains everything an ordered numeric type has except `*` between two
Durations; Boolean stays equality-only.
