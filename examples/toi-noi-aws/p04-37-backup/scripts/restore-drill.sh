#!/usr/bin/env bash
# Diễn tập restore RDS từ recovery point mới nhất và ĐO THỜI GIAN restore thật (đầu vào cho RTO).
# Script tạo một DB instance mới (tính tiền theo giờ) nên hỏi xác nhận trước khi chạy và in lệnh dọn ở cuối.
# Dùng: AWS_PROFILE=<lab> VAULT=toinoi-lab-primary ROLE_ARN=<arn role backup> ./restore-drill.sh <ten-db-moi>
set -euo pipefail

: "${AWS_PROFILE:?Đặt AWS_PROFILE trỏ vào tài khoản lab}"
: "${VAULT:?Đặt VAULT là tên backup vault, ví dụ toinoi-lab-primary}"
: "${ROLE_ARN:?Đặt ROLE_ARN là role AWS Backup (output backup_role_arn)}"
NEW_DB="${1:?Cần tên DB instance mới, ví dụ toinoi-lab-trips-drill}"
REGION="${AWS_REGION:-ap-southeast-1}"

case "$NEW_DB" in
  toinoi-lab-*) ;;
  *) echo "Từ chối: tên DB mới phải bắt đầu bằng toinoi-lab-" >&2; exit 1 ;;
esac

echo "Tài khoản: $(aws sts get-caller-identity --query Account --output text), vault: ${VAULT}, DB mới: ${NEW_DB}"
read -r -p "Tạo DB instance restore (tính tiền theo giờ) và đo thời gian? [y/N] " ans
[ "$ans" = "y" ] || exit 0

RP_ARN="$(aws backup list-recovery-points-by-backup-vault --region "$REGION" --backup-vault-name "$VAULT" \
  --by-resource-type RDS \
  --query 'sort_by(RecoveryPoints,&CreationDate)[-1].RecoveryPointArn' --output text)"
[ -n "$RP_ARN" ] && [ "$RP_ARN" != "None" ] || { echo "Không có recovery point RDS trong vault." >&2; exit 1; }
echo "Recovery point: ${RP_ARN}"

META_FILE="$(mktemp)"
trap 'rm -f "$META_FILE"' EXIT
# Lấy metadata gốc rồi đổi tên instance; đổi thêm subnet group, security group theo nhu cầu trước khi chạy thật.
aws backup get-recovery-point-restore-metadata --region "$REGION" --backup-vault-name "$VAULT" \
  --recovery-point-arn "$RP_ARN" --query RestoreMetadata --output json \
  | NEW_DB="$NEW_DB" node -e '
      const m = JSON.parse(require("fs").readFileSync(0, "utf8"));
      m.DBInstanceIdentifier = process.env.NEW_DB;
      process.stdout.write(JSON.stringify(m));
    ' >"$META_FILE"

START="$(date +%s)"
JOB_ID="$(aws backup start-restore-job --region "$REGION" --recovery-point-arn "$RP_ARN" \
  --iam-role-arn "$ROLE_ARN" --resource-type RDS --metadata "file://${META_FILE}" \
  --query RestoreJobId --output text)"
echo "Restore job: ${JOB_ID}"

while true; do
  STATUS="$(aws backup describe-restore-job --region "$REGION" --restore-job-id "$JOB_ID" --query Status --output text)"
  echo "$(date +%T) ${STATUS}"
  case "$STATUS" in
    COMPLETED) break ;;
    ABORTED|FAILED) echo "Restore thất bại." >&2; exit 1 ;;
  esac
  sleep 30
done

END="$(date +%s)"
echo "Thời gian restore (đo bằng đồng hồ): $((END - START)) giây."
echo "Bước tiếp theo: nối vào ${NEW_DB} từ một máy trong VPC, đếm dòng và so checksum (xem README), rồi dọn:"
echo "  aws rds delete-db-instance --region ${REGION} --db-instance-identifier ${NEW_DB} --skip-final-snapshot"
