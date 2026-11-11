#!/bin/sh
# Repro 2: a view answering 501 (or 4xx/5xx) renders as 0 in a header metric.
# Usage: sh run.sh   (needs ess, node, npm with network; a headless chrome via CHROME=<path>,
# and playwright-core installed under PW_DIR, defaults below)
set -u
here=$(cd "$(dirname "$0")" && pwd)
cd "$here"
PW_DIR=${PW_DIR:-$here/../tools/node_modules/playwright-core}
CHROME=${CHROME:-$(ls -d "$HOME"/.cache/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell | sort | tail -1)}
rm -rf react-out

echo '$ ess --version'
ess --version
echo
echo '$ ess generate ui --target react --path ui/shelf.ui.yaml --out react-out --model spec'
ess generate ui --target react --path ui/shelf.ui.yaml --out react-out --model spec

echo
echo '--- generated code (file:line)'
echo '$ grep -n "response.status < 200" react-out/src/runtime/data.ts   (the read helper throws a DataError on a non-2xx answer)'
grep -n 'response.status < 200' react-out/src/runtime/data.ts
echo '$ sed -n 510,514p react-out/src/runtime/data.ts   (useRead turns it into status failed with rows [])'
sed -n 510,514p react-out/src/runtime/data.ts
echo '$ sed -n 21p;26,27p;38p react-out/src/runtime/composites/metric.tsx   (MetricView never reads data.status)'
sed -n '21p;26,27p;38p' react-out/src/runtime/composites/metric.tsx
echo '$ sed -n 50,52p;56,58p react-out/src/runtime/composites/metric.tsx   (count of no rows is 0; sum of no rows is 0)'
sed -n '50,52p;56,58p' react-out/src/runtime/composites/metric.tsx
echo '$ grep -n "status" react-out/src/runtime/composites/metric.tsx | wc -l'
grep -n 'status' react-out/src/runtime/composites/metric.tsx | wc -l
echo '$ sed -n 431,435p react-out/src/runtime/core.tsx   (by contrast a section frame has a failed state)'
sed -n 431,435p react-out/src/runtime/core.tsx

echo
echo '$ (cd react-out && npm install && npm run build)'
(cd react-out && npm install --no-audit --no-fund >npm-install.log 2>&1 && npm run build 2>&1 | tail -4)

echo
echo "--- headless chrome against a static server that answers the view route with a chosen status"
echo "\$ node check.mjs react-out/www \$PW_DIR \$CHROME"
node check.mjs "$here/react-out/www" "$PW_DIR" "$CHROME" 2>&1
