#!/bin/sh
# A type name that resolves nowhere is refused for a page parameter and accepted everywhere else.
# Usage: sh run.sh   (needs ess on PATH; reads nothing outside this directory)
set -u
cd "$(dirname "$0")" || exit 2

echo '$ ess --version'
ess --version

echo
echo '$ ess ui check --path ui/page-params.ui.yaml --model spec   (page params naming NoSuchId; ItemId resolves in the model)'
ess ui check --path ui/page-params.ui.yaml --model spec
echo "exit $?"

echo
echo '$ ess ui check --path ui/widget-params.ui.yaml --model spec   (widget params: {optional: NoSuchId} and NoSuchType)'
ess ui check --path ui/widget-params.ui.yaml --model spec
echo "exit $?"

echo
echo '$ ess ui check --path ui/state-types.ui.yaml --model spec   (page state types and a document type naming NoSuchId, NoSuchType)'
ess ui check --path ui/state-types.ui.yaml --model spec
echo "exit $?"

echo
echo '$ ess ui check --path ui/unused-widget.ui.yaml --model spec   (a widget no page uses names a view and a column the model lacks)'
ess ui check --path ui/unused-widget.ui.yaml --model spec
echo "exit $?"
