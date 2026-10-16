#!/usr/bin/env bash
set -euo pipefail
: "${MYSQL_CNF:?Set MYSQL_CNF to a private MariaDB client defaults file}"
: "${DATABASE_NAME:?Set DATABASE_NAME}"
[[ "$DATABASE_NAME" =~ ^[a-zA-Z0-9_]+$ ]] || exit 2
output_file="${1:?Pass the output .sql.gz path}"
umask 077
scratch_file="$(mktemp "${output_file}.tmp.XXXXXX")"
trap 'rm -f "$scratch_file"' EXIT
mariadb-dump --defaults-extra-file="$MYSQL_CNF" --single-transaction --routines --triggers "$DATABASE_NAME" | gzip > "$scratch_file"
gzip -t "$scratch_file"
mv "$scratch_file" "$output_file"
