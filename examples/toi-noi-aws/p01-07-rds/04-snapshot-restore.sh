#!/usr/bin/env bash
set -euo pipefail
# Tạo manual snapshot, xem thời điểm PITR sớm nhất/muộn nhất, và (tuỳ chọn) khôi phục ra instance MỚI.
# Restore luôn tạo instance mới với endpoint mới: nó không ghi đè instance cũ. Instance mới cũng tính tiền.
source "$(dirname "$0")/common.sh"
confirm_identity

SNAP="${DB_ID}-manual-$(date -u +%Y%m%d%H%M)"
log "Tạo manual snapshot $SNAP"
aws rds create-db-snapshot --db-instance-identifier "$DB_ID" --db-snapshot-identifier "$SNAP" \
  --tags "${COMMON_TAGS[@]}" >/dev/null
aws rds wait db-snapshot-available --db-snapshot-identifier "$SNAP"

log "Cửa sổ point-in-time recovery của instance"
aws rds describe-db-instances --db-instance-identifier "$DB_ID" \
  --query 'DBInstances[0].{Retention:BackupRetentionPeriod,LatestRestorableTime:LatestRestorableTime}' --output table

if [[ "${RESTORE:-0}" == "1" ]]; then
  TARGET="${DB_ID}-restored"
  log "Khôi phục tới thời điểm gần nhất ra instance $TARGET"
  aws rds restore-db-instance-to-point-in-time \
    --source-db-instance-identifier "$DB_ID" --target-db-instance-identifier "$TARGET" \
    --use-latest-restorable-time --no-publicly-accessible \
    --db-subnet-group-name "$DB_SUBNET_GROUP" \
    --vpc-security-group-ids "$(find_db_sg)" \
    --tags "${COMMON_TAGS[@]}" >/dev/null
  aws rds wait db-instance-available --db-instance-identifier "$TARGET"
  echo "Đã có $TARGET. Nhớ xoá nó (99-cleanup.sh làm việc này)."
else
  echo "Đặt RESTORE=1 để thử khôi phục ra instance mới."
fi
