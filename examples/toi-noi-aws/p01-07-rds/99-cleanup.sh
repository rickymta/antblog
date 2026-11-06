#!/usr/bin/env bash
set -euo pipefail
# Xoá toàn bộ tài nguyên của lab RDS: instance (kể cả bản restore), snapshot manual, subnet group,
# parameter group, security group và inline policy. KHÔNG hoàn tác được, không tạo final snapshot.
source "$(dirname "$0")/common.sh"
confirm_identity

case "$DB_ID" in toinoi-lab-*) ;; *) echo "Từ chối: DB_ID phải bắt đầu bằng toinoi-lab-" >&2; exit 1 ;; esac

delete_instance() {
  local id="$1"
  if ! aws rds describe-db-instances --db-instance-identifier "$id" >/dev/null 2>&1; then return 0; fi
  log "Xoá instance $id"
  aws rds modify-db-instance --db-instance-identifier "$id" --no-deletion-protection --apply-immediately >/dev/null || true
  aws rds delete-db-instance --db-instance-identifier "$id" --skip-final-snapshot --delete-automated-backups >/dev/null
  aws rds wait db-instance-deleted --db-instance-identifier "$id"
}
delete_instance "${DB_ID}-restored"
delete_instance "$DB_ID"

log "Xoá manual snapshot có tiền tố toinoi-lab-"
for s in $(aws rds describe-db-snapshots --snapshot-type manual \
  --query "DBSnapshots[?starts_with(DBSnapshotIdentifier, 'toinoi-lab-')].DBSnapshotIdentifier" --output text); do
  aws rds delete-db-snapshot --db-snapshot-identifier "$s" >/dev/null
  echo "đã xoá $s"
done

log "Xoá subnet group, parameter group, security group"
aws rds delete-db-subnet-group --db-subnet-group-name "$DB_SUBNET_GROUP" 2>/dev/null || echo "(subnet group không còn)"
aws rds delete-db-parameter-group --db-parameter-group-name "$DB_PARAM_GROUP" 2>/dev/null || echo "(parameter group không còn)"
if [[ -n "${VPC_ID:-}" ]]; then
  SG_DB="$(find_db_sg_owned)"
  if [[ -n "$SG_DB" ]]; then
    aws ec2 delete-security-group --group-id "$SG_DB"
  else
    echo "(sg-db thuộc lab bài 04 hoặc không còn: để cleanup.sh của bài 04 xoá)"
  fi
fi

if [[ -n "${EC2_ROLE_NAME:-}" ]]; then
  aws iam delete-role-policy --role-name "$EC2_ROLE_NAME" --policy-name "$ROLE_POLICY_NAME" 2>/dev/null || true
fi

log "Xoá log group RDS đẩy sang CloudWatch Logs (không tự xoá khi xoá instance)"
for id in "${DB_ID}-restored" "$DB_ID"; do
  aws logs delete-log-group --log-group-name "/aws/rds/instance/$id/postgresql" 2>/dev/null || true
done

log "Kiểm lại: không còn instance và snapshot toinoi-lab-*"
aws rds describe-db-instances --query "DBInstances[?starts_with(DBInstanceIdentifier, 'toinoi-lab-')].DBInstanceIdentifier"
aws rds describe-db-snapshots --snapshot-type manual --query "DBSnapshots[?starts_with(DBSnapshotIdentifier, 'toinoi-lab-')].DBSnapshotIdentifier"
