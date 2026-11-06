#!/usr/bin/env bash
# Đăng ký task definition và tạo service ECS Fargate bằng AWS CLI v2.
# Cần chạy Terraform với manage_workload=false trước để có cluster, role, log group, security group.
# Script tạo tài nguyên tính tiền (task Fargate). Chỉ chạy trong tài khoản lab; dọn bằng scripts/cleanup.sh.
set -euo pipefail

: "${AWS_REGION:=ap-southeast-1}"
: "${CLUSTER:=toinoi-lab}"
: "${SERVICE:=toinoi-lab-booking}"
: "${LOG_GROUP:=/ecs/toinoi-lab-booking}"
: "${AUTH_AUTHORITY:=https://id.toinoi.example}"
: "${AUTH_AUDIENCE:=toinoi-api}"
: "${EXECUTION_ROLE_ARN:?Lấy từ terraform output execution_role_arn}"
: "${TASK_ROLE_ARN:?Lấy từ terraform output task_role_arn}"
: "${IMAGE_DIGEST:?Dạng <registry>/toinoi/booking@sha256:...}"
: "${DB_SECRET_ARN:?ARN đầy đủ của secret connection string}"
: "${SUBNETS:?Danh sách private subnet, cách nhau bằng dấu phẩy, không khoảng trắng}"
: "${SECURITY_GROUP:?Lấy từ terraform output task_security_group_id}"

export AWS_REGION EXECUTION_ROLE_ARN TASK_ROLE_ARN IMAGE_DIGEST DB_SECRET_ARN LOG_GROUP AUTH_AUTHORITY AUTH_AUDIENCE

HERE="$(cd "$(dirname "$0")/.." && pwd)"
RENDERED="$(mktemp)"
trap 'rm -f "${RENDERED}"' EXIT

# Chỉ thay đúng các biến đã khai báo, không đụng tới ký tự $ khác.
envsubst '${EXECUTION_ROLE_ARN} ${TASK_ROLE_ARN} ${IMAGE_DIGEST} ${DB_SECRET_ARN} ${LOG_GROUP} ${AWS_REGION} ${AUTH_AUTHORITY} ${AUTH_AUDIENCE}' \
  < "${HERE}/task-definition.json" > "${RENDERED}"

# Mỗi lần register tạo một revision mới của family; revision cũ vẫn còn.
TASK_DEF_ARN="$(aws ecs register-task-definition --cli-input-json "file://${RENDERED}" \
  --query 'taskDefinition.taskDefinitionArn' --output text)"
echo "Đã đăng ký: ${TASK_DEF_ARN}"

aws ecs create-service \
  --cluster "${CLUSTER}" \
  --service-name "${SERVICE}" \
  --task-definition "${TASK_DEF_ARN}" \
  --desired-count 1 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[${SUBNETS}],securityGroups=[${SECURITY_GROUP}],assignPublicIp=DISABLED}" \
  --enable-execute-command \
  --enable-ecs-managed-tags \
  --propagate-tags SERVICE \
  --tags key=Project,value=ToiNoi key=Env,value=lab key=Service,value=booking key=Owner,value=platform

aws ecs wait services-stable --cluster "${CLUSTER}" --services "${SERVICE}"
echo "Service ổn định. Xem task:"
aws ecs list-tasks --cluster "${CLUSTER}" --service-name "${SERVICE}"
