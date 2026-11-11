#!/bin/sh
# Fixture mode: a confirmed command answers done and the row stays in the filtered view it left.
# Usage: sh run.sh [ess] [playwright-core dir] [chrome executable]
#   ess defaults to `ess` on PATH. The browser part needs node, npm with network, and
#   playwright-core; without a playwright-core dir it is skipped. Chrome defaults to
#   playwright-core's own.
set -u
here=$(cd "$(dirname "$0")" && pwd)
cd "$here"
ESS=${1:-ess}
PW_DIR=${2:-}
CHROME=${3:-}
rm -rf react-out

echo '$ ess --version'
"$ESS" --version
echo
echo '$ ess validate --path spec'
"$ESS" validate --path spec
echo
echo '$ ess ui check --path ui/desk.ui.yaml --model spec'
"$ESS" ui check --path ui/desk.ui.yaml --model spec
echo
echo '$ ess generate ui --target react --path ui/desk.ui.yaml --out react-out   (no --model: fixture mode)'
"$ESS" generate ui --target react --path ui/desk.ui.yaml --out react-out

echo
echo '--- generated code (file:line)'
echo '$ grep -n "httpAdapter\|setDataAdapter" react-out/src/main.tsx | wc -l   (nothing swaps the fixture adapter out)'
grep -n 'httpAdapter\|setDataAdapter' react-out/src/main.tsx | wc -l
echo '$ grep -n "send(command, input) {" -A2 react-out/src/runtime/data.ts   (the fixture adapter accepts every command and changes nothing)'
grep -n 'send(command, input) {' -A2 react-out/src/runtime/data.ts
echo '$ grep -n "let rows = view.rows" react-out/src/runtime/data.ts   (a read serves the generated fixture rows as written)'
grep -n 'let rows = view.rows' react-out/src/runtime/data.ts
echo '$ grep -n "await dataAdapter().send" -A4 react-out/src/runtime/actions.tsx   (read again, then toast done)'
grep -n 'await dataAdapter().send' -A4 react-out/src/runtime/actions.tsx
echo '$ grep -c "ApproveRequest\|Approved" react-out/src/fixtures.ts   (the fixtures carry no command effect)'
grep -c 'ApproveRequest\|Approved' react-out/src/fixtures.ts

echo
echo '--- terminal renderer, headless (ess-ui-test/1)'
echo '$ ess ui test --path ui/desk.ui.yaml ui/approve.test.yaml'
"$ESS" ui test --path ui/desk.ui.yaml ui/approve.test.yaml
echo "exit $?"
echo '$ ess ui test --path ui/desk.ui.yaml --model spec ui/approve.test.yaml   (model given, reads still from fixtures)'
"$ESS" ui test --path ui/desk.ui.yaml --model spec ui/approve.test.yaml
echo "exit $?"

if [ -z "$PW_DIR" ]; then
  echo
  echo '--- browser part skipped: no playwright-core dir given'
  exit 0
fi
echo
echo '$ (cd react-out && npm install && npm run build)'
(cd react-out && npm install --no-audit --no-fund >npm-install.log 2>&1 && npm run build 2>&1 | tail -4)
echo
echo '--- headless chrome: approve the first pending request through its confirm'
echo '$ node check.mjs react-out/www $PW_DIR $CHROME'
node check.mjs "$here/react-out/www" "$PW_DIR" $CHROME 2>&1
