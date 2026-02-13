#!/usr/bin/env sh
set -eu

if [ -z "${MIGRATIONS_URL:-}" ]; then
  echo "MIGRATIONS_URL is required" >&2
  exit 1
fi

if [ ! -d /migrations ]; then
  echo "/migrations directory not found" >&2
  exit 1
fi

found_any=0
for file in /migrations/*.sql; do
  if [ -f "$file" ]; then
    found_any=1
    echo "Applying $file"
    psql "$MIGRATIONS_URL" -v ON_ERROR_STOP=1 -f "$file"
  fi
done

if [ "$found_any" -eq 0 ]; then
  echo "No migration files found in /migrations"
fi

echo "Migration init complete"
