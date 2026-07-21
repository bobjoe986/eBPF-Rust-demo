#!/usr/bin/env bash
set -euo pipefail

echo "sentinel-smoke-process"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
printf 'hello\n' > "$tmpdir/a.txt"
cat "$tmpdir/a.txt" >/dev/null
printf 'world\n' >> "$tmpdir/a.txt"
mv "$tmpdir/a.txt" "$tmpdir/b.txt"
rm "$tmpdir/b.txt"

echo "Smoke activity generated. Network smoke test will be added with Milestone 5."
