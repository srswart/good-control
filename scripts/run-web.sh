#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Build Tailwind CSS first
echo "Building Tailwind CSS..."
cd assets/ui
npm run build:css
cd ../..

# Copy CSS to web app assets
echo "Copying CSS to web app..."
mkdir -p apps/web/assets
cp assets/ui/dist/tailwind.css apps/web/assets/tailwind.css

# Run web app
echo "Starting web app on http://localhost:8080"
echo "Press Ctrl+C to stop"
cd apps/web
dx serve --platform web --port 8080

