#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL must be set}"
: "${BACKUP_DIR:=./backups}"

mkdir -p "${BACKUP_DIR}"
STAMP="$(date -u +'%Y%m%dT%H%M%SZ')"
OUTFILE="${BACKUP_DIR}/metadata-${STAMP}.dump"

pg_dump --format=custom --file "${OUTFILE}" "${DATABASE_URL}"
echo "backup written to ${OUTFILE}"
