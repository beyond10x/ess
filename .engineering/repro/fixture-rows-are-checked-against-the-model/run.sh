#!/bin/sh
# ess ui check --model reads no fixture row against the model's view.
# Usage: sh run.sh   (needs ess on PATH; reads nothing outside this directory)
set -u
cd "$(dirname "$0")" || exit 2

echo '$ ess --version'
ess --version

echo
echo '$ cat ui/fixtures-bad/items.yaml   (kind: Gadget is no variant of shop.channel.Kind, colour is no field of the view, label is a number, kind is absent)'
cat ui/fixtures-bad/items.yaml

echo
echo '$ ess ui check --path ui/bad-fixtures.ui.yaml --model spec'
ess ui check --path ui/bad-fixtures.ui.yaml --model spec
echo "exit $?"

echo
echo '$ ess ui check --path ui/bad-fixtures.ui.yaml --model spec --format json'
ess ui check --path ui/bad-fixtures.ui.yaml --model spec --format json

echo
echo '$ ess ui test --path ui/bad-fixtures.ui.yaml --model spec tests/bad.yaml   (the document'"'"'s fixtures, and a test'"'"'s own, hold the same rows)'
ess ui test --path ui/bad-fixtures.ui.yaml --model spec tests/bad.yaml
