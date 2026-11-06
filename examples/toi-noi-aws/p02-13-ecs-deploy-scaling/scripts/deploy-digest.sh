#!/usr/bin/env bash
# Rolling deploy một image digest mới: đăng ký revision mới từ revision hiện tại rồi update-service.
# Cần jq. Script làm service chạy task mới (tính tiền Fargate) và quan sát events tới khi ổn định hoặc rollback.
set -euo pipefail

: "${CLUSTER:=toinoi-lab}"
: "${SERVICE:=toinoi-lab-booking-alb}"
: "${FAMILY:=toinoi-lab-booking}"
NEW_IMAGE="${1:?Cách dùng: deploy-digest.sh <registry>/toinoi/booking@sha256:...}"

case "${NEW_IMAGE}" in
  *@sha256:*) ;;
  *) echo "Image phải tham chiếu theo digest (@sha256:...), không dùng tag." >&2; exit 1 ;;
esac

TMP="$(mktemp)"
trap 'rm -f "${TMP}"' EXIT

# Lấy revision mới nhất của family, bỏ các trường do ECS tự sinh, đổi image.
aws ecs describe-task-definition --task-definition "${FAMILY}" --query taskDefinition --output json \
  | jq --arg img "${NEW_IMAGE}" '
      del(.taskDefinitionArn, .revision, .status, .requiresAttributes, .compatibilities, .registeredAt, .registeredBy)
      | .containerDefinitions[0].image = $img' > "${TMP}"

NEW_ARN="$(aws ecs register-task-definition --cli-input-json "file://${TMP}" \
  --query 'taskDefinition.taskDefinitionArn' --output text)"
echo "Revision mới: ${NEW_ARN}"

aws ecs update-service --cluster "${CLUSTER}" --service "${SERVICE}" --task-definition "${NEW_ARN}" >/dev/null

# Quan sát: số task, rolloutState của deployment và 5 event mới nhất.
for _ in $(seq 1 20); do
  aws ecs describe-services --cluster "${CLUSTER}" --services "${SERVICE}" \
    --query 'services[0].deployments[].{state:rolloutState,running:runningCount,desired:desiredCount,pending:pendingCount,td:taskDefinition}' \
    --output table
  STATE="$(aws ecs describe-services --cluster "${CLUSTER}" --services "${SERVICE}" \
    --query 'services[0].deployments[?status==`PRIMARY`].rolloutState | [0]' --output text)"
  [ "${STATE}" = "COMPLETED" ] || [ "${STATE}" = "FAILED" ] && break
  sleep 15
done

aws ecs describe-services --cluster "${CLUSTER}" --services "${SERVICE}" \
  --query 'services[0].events[:5].[createdAt,message]' --output table
echo "rolloutState cuối: ${STATE}"
