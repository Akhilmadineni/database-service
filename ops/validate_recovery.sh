#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL must be set}"

psql "${DATABASE_URL}" -c "select count(*) as capsule_count from capsule;"
psql "${DATABASE_URL}" -c "select count(*) as operation_count from operation;"
psql "${DATABASE_URL}" -c "select count(*) as audit_event_count from audit_event;"
