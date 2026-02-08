#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

MODE="${1:-headless}"

echo "Running UI tests in $MODE mode..."

cd tests/ui

case "$MODE" in
  headed)
    npm run test:headed
    ;;
  debug)
    npm run test:debug
    ;;
  ui)
    npm run test:ui
    ;;
  *)
    npm test
    ;;
esac

