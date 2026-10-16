#!/usr/bin/env bash
set -euo pipefail
: "${MYSQL_CNF:?Set MYSQL_CNF to a private MariaDB client defaults file}"
: "${RESTORE_DATABASE:?Set RESTORE_DATABASE to a separately created empty database ending in _restore}"
[[ "$RESTORE_DATABASE" =~ ^[a-zA-Z0-9_]+_restore$ ]] || { echo 'Use a separate database ending in _restore' >&2; exit 2; }
backup_file="${1:?Pass the backup .sql.gz path}"
count="$(mariadb --defaults-extra-file="$MYSQL_CNF" --batch --skip-column-names -e "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$RESTORE_DATABASE'")"
[[ "$count" == 0 ]] || { echo 'Restore target must be empty' >&2; exit 2; }
gzip -t "$backup_file"
gzip -dc "$backup_file" | mariadb --defaults-extra-file="$MYSQL_CNF" "$RESTORE_DATABASE"
