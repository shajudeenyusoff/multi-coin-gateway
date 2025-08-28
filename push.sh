#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
git add -A
git commit -m "${1:-chore: sync server changes}"
git push origin main
echo "✅ Pushed to GitHub."
