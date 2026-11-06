#!/usr/bin/env bash
set -euo pipefail
# Tạo RDS PostgreSQL một AZ trong private subnet. KHÔNG bật Multi-AZ ở đây (xem 03-multi-az-failover.sh).
# Instance bắt đầu tính tiền theo giờ ngay khi tạo, kèm dung lượng đĩa và backup vượt phần miễn phí.
source "$(dirname "$0")/common.sh"
: "${VPC_ID:?Không thấy VPC toinoi-lab-vpc: chạy lab bài 04 hoặc điền VPC_ID trong env.sh}"
: "${DB_SUBNET_IDS:?Thiếu hai subnet private-data}" "${SG_APP_ID:?Thiếu security group của tầng app}"
confirm_identity

log "DB subnet group (hai AZ)"
# shellcheck disable=SC2086
aws rds create-db-subnet-group \
  --db-subnet-group-name "$DB_SUBNET_GROUP" \
  --db-subnet-group-description "ToiNoi lab: private subnet cho RDS trips" \
  --subnet-ids $DB_SUBNET_IDS \
  --tags "${COMMON_TAGS[@]}" >/dev/null

log "Security group của DB: chỉ nhận 5432 từ SG app"
SG_DB="$(find_db_sg)"
if [[ -n "$SG_DB" ]]; then
  echo "Dùng lại sg-db $DB_SG_NAME của lab bài 04 (đã có luật 5432 từ sg-app)."
else
  SG_DB="$(aws ec2 create-security-group \
    --group-name "$DB_SG_NAME" --description "ToiNoi lab: RDS trips, chi nhan tu sg-app" \
    --vpc-id "$VPC_ID" \
    --tag-specifications "ResourceType=security-group,Tags=[{Key=Project,Value=ToiNoi},{Key=Env,Value=lab},{Key=Service,Value=trips},{Key=Owner,Value=platform}]" \
    --query GroupId --output text)"
  aws ec2 authorize-security-group-ingress --group-id "$SG_DB" \
    --protocol tcp --port 5432 --source-group "$SG_APP_ID" >/dev/null
fi
echo "SG DB: $SG_DB"

if [[ -z "${PG_FAMILY:-}" ]]; then
  if [[ -n "${PG_ENGINE_VERSION:-}" ]]; then VER_ARGS=(--engine-version "$PG_ENGINE_VERSION"); else VER_ARGS=(--default-only); fi
  PG_FAMILY="$(aws rds describe-db-engine-versions --engine postgres "${VER_ARGS[@]}" \
    --query 'DBEngineVersions[0].DBParameterGroupFamily' --output text)"
fi
echo "Parameter group family: $PG_FAMILY"

log "Parameter group: ghi log câu query chậm hơn 500 ms (tham số dynamic, áp dụng ngay)"
aws rds create-db-parameter-group \
  --db-parameter-group-name "$DB_PARAM_GROUP" \
  --db-parameter-group-family "$PG_FAMILY" \
  --description "ToiNoi lab: log query cham" \
  --tags "${COMMON_TAGS[@]}" >/dev/null
aws rds modify-db-parameter-group \
  --db-parameter-group-name "$DB_PARAM_GROUP" \
  --parameters "ParameterName=log_min_duration_statement,ParameterValue=500,ApplyMethod=immediate" >/dev/null

log "Tạo instance $DB_ID (mất vài phút)"
ARGS=(
  --db-instance-identifier "$DB_ID"
  --engine postgres
  --db-instance-class "$DB_INSTANCE_CLASS"
  --storage-type gp3
  --allocated-storage 20
  --max-allocated-storage 100
  --storage-encrypted
  --db-name "$DB_NAME"
  --master-username "$DB_USER"
  --manage-master-user-password
  --db-subnet-group-name "$DB_SUBNET_GROUP"
  --vpc-security-group-ids "$SG_DB"
  --db-parameter-group-name "$DB_PARAM_GROUP"
  --no-publicly-accessible
  --no-multi-az
  --backup-retention-period 7
  --preferred-backup-window "17:00-18:00"
  --preferred-maintenance-window "sun:18:00-sun:19:00"
  --copy-tags-to-snapshot
  --enable-cloudwatch-logs-exports postgresql
  --no-deletion-protection
  --tags "${COMMON_TAGS[@]}"
)
if [[ -n "${PG_ENGINE_VERSION:-}" ]]; then ARGS+=(--engine-version "$PG_ENGINE_VERSION"); fi
aws rds create-db-instance "${ARGS[@]}" >/dev/null
aws rds wait db-instance-available --db-instance-identifier "$DB_ID"

log "Thông tin kết nối"
aws rds describe-db-instances --db-instance-identifier "$DB_ID" \
  --query 'DBInstances[0].{Endpoint:Endpoint.Address,Port:Endpoint.Port,Public:PubliclyAccessible,MultiAZ:MultiAZ,Secret:MasterUserSecret.SecretArn,Status:DBInstanceStatus}' \
  --output table
echo "Tiếp theo: dùng Session Manager vào EC2 và chạy ec2-psql.sh (xem README)."
