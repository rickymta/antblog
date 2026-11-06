#!/usr/bin/env bash
set -euo pipefail
# Chuyển instance sang Multi-AZ rồi ép failover để thấy AZ đổi. Chi phí instance tăng gấp đôi trong lúc bật.
source "$(dirname "$0")/common.sh"
confirm_identity

show() {
  aws rds describe-db-instances --db-instance-identifier "$DB_ID" \
    --query 'DBInstances[0].{AZ:AvailabilityZone,Secondary:SecondaryAvailabilityZone,MultiAZ:MultiAZ,Status:DBInstanceStatus}' \
    --output table
}

log "Trước"; show
log "Bật Multi-AZ (apply ngay; mất vài phút, RDS tạo standby từ snapshot)"
aws rds modify-db-instance --db-instance-identifier "$DB_ID" --multi-az --apply-immediately >/dev/null
sleep 30
aws rds wait db-instance-available --db-instance-identifier "$DB_ID"
log "Sau khi bật"; show

log "Ép failover: kết nối hiện tại sẽ đứt, endpoint DNS trỏ sang standby"
aws rds reboot-db-instance --db-instance-identifier "$DB_ID" --force-failover >/dev/null
sleep 30
aws rds wait db-instance-available --db-instance-identifier "$DB_ID"
log "Sau failover (AZ chính đã đổi chỗ với standby)"; show

echo "Để quay về một AZ cho đỡ tốn: aws rds modify-db-instance --db-instance-identifier $DB_ID --no-multi-az --apply-immediately"
