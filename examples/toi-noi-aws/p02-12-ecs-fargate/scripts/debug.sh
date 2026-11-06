#!/usr/bin/env bash
# Xem lý do task dừng và vào container bằng ECS Exec.
set -euo pipefail

: "${CLUSTER:=toinoi-lab}"
: "${SERVICE:=toinoi-lab-booking}"

echo "== Task đã dừng gần đây và stoppedReason =="
STOPPED="$(aws ecs list-tasks --cluster "${CLUSTER}" --service-name "${SERVICE}" --desired-status STOPPED --query 'taskArns' --output text)"
if [ -n "${STOPPED}" ] && [ "${STOPPED}" != "None" ]; then
  # shellcheck disable=SC2086
  aws ecs describe-tasks --cluster "${CLUSTER}" --tasks ${STOPPED} \
    --query 'tasks[].{task:taskArn,stopCode:stopCode,reason:stoppedReason,container:containers[0].reason}' --output table
fi

echo "== Vào container (cần Session Manager plugin và image có /bin/sh) =="
TASK="$(aws ecs list-tasks --cluster "${CLUSTER}" --service-name "${SERVICE}" --desired-status RUNNING --query 'taskArns[0]' --output text)"
aws ecs execute-command --cluster "${CLUSTER}" --task "${TASK}" --container booking --interactive --command "/bin/sh"
