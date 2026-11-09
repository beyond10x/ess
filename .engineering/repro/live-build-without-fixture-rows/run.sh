#!/bin/sh
# Repro 1: a build bound to a live backend still ships the fixture rows.
# Usage: sh run.sh   (needs ess on PATH, node and npm with network for esbuild/react)
set -u
here=$(cd "$(dirname "$0")" && pwd)
cd "$here"
rm -rf react-out nofx-out nofx.ui.yaml

echo '$ ess --version'
ess --version

echo
echo '$ ess generate ui --help | grep -E "^      --"   (every option of the command)'
ess generate ui --help | grep -E '^      --'
echo
echo '$ ess generate ui --help | grep -ciE "no-fixture|without-fixture|fixtures-off|omit-fixture|strip-fixture"'
ess generate ui --help | grep -ciE 'no-fixture|without-fixture|fixtures-off|omit-fixture|strip-fixture'

echo
echo '$ ess ui check --path ui/shelf.ui.yaml --model spec'
ess ui check --path ui/shelf.ui.yaml --model spec
echo
echo '$ ess generate ui --target react --path ui/shelf.ui.yaml --out react-out --model spec   (live build: bound to the served surface)'
ess generate ui --target react --path ui/shelf.ui.yaml --out react-out --model spec

echo
echo '$ grep -n fixture-marker react-out/src/fixtures.ts'
grep -n 'fixture-marker' react-out/src/fixtures.ts
echo
echo '$ grep -n "httpAdapter\|setDataAdapter" react-out/src/main.tsx   (the app reads over HTTP, not from fixtures)'
grep -n 'httpAdapter\|setDataAdapter' react-out/src/main.tsx

echo
echo '$ (cd react-out && npm install && npm run build)'
(cd react-out && npm install --no-audit --no-fund >npm-install.log 2>&1 && npm run build 2>&1 | tail -4)

echo
echo '$ grep -o "fixture-marker-[a-z]*-[0-9]*" react-out/www/assets/main.js | sort -u'
grep -o 'fixture-marker-[a-z]*-[0-9]*' react-out/www/assets/main.js | sort -u
echo
echo '$ grep -c "fixture-marker" react-out/www/assets/main.js   (lines of the minified bundle that hold a fixture row)'
grep -c 'fixture-marker' react-out/www/assets/main.js

echo
echo '--- Only workaround found: delete the fixtures block from the document for the live build'
echo '$ ess ui check --path nofx.ui.yaml --model spec   (the document without its fixtures block)'
grep -v -E '^fixtures:|^  dir: fixtures|^  views:|^    store.shelf.Items: items.yaml' ui/shelf.ui.yaml >nofx.ui.yaml
ess ui check --path nofx.ui.yaml --model spec
ess generate ui --target react --path nofx.ui.yaml --out nofx-out --model spec
echo '$ grep -c fixture-marker nofx-out/src/fixtures.ts'
grep -c 'fixture-marker' nofx-out/src/fixtures.ts
rm -rf nofx-out nofx.ui.yaml
