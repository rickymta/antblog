#!/usr/bin/env bash
# Dọn service và các revision của task definition tạo bằng AWS CLI.
set -euo pipefail

: "${CLUSTER:=toinoi-lab}"
: "${SERVICE:=toinoi-lab-booking}"
: "${FAMILY:=toinoi-lab-booking}"

aws ecs update-service --cluster "${CLUSTER}" --service "${SERVICE}" --desired-count 0 >/dev/null
aws ecs delete-service --cluster "${CLUSTER}" --service "${SERVICE}" --force >/dev/null
aws ecs wait services-inactive --cluster "${CLUSTER}" --services "${SERVICE}"

# Hủy đăng ký mọi revision đang ACTIVE (revision vẫn tồn tại ở trạng thái INACTIVE).
for arn in $(aws ecs list-task-definitions --family-prefix "${FAMILY}" --status ACTIVE --query 'taskDefinitionArns' --output text); do
  aws ecs deregister-task-definition --task-definition "${arn}" >/dev/null
done
echo "Đã dọn service và hủy đăng ký task definition. Chạy terraform destroy để dọn phần còn lại."
