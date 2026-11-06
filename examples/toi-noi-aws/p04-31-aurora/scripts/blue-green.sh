#!/usr/bin/env bash
# Blue/Green deployment cho nâng major version.
# Dùng:
#   ./blue-green.sh create <target-engine-version> <target-cluster-parameter-group>
#   ./blue-green.sh status <blue-green-deployment-identifier>
#   ./blue-green.sh switchover <blue-green-deployment-identifier>
#   ./blue-green.sh delete <blue-green-deployment-identifier>
# Điều kiện (xem User Guide): cluster nguồn bật rds.logical_replication = 1 và đã reboot.
# Parameter group đích phải thuộc family của phiên bản đích (ví dụ aurora-postgresql18), tạo trước.
# Blue/Green không hỗ trợ cluster dùng mật khẩu master do RDS quản lý (Secrets Manager): script kiểm và dừng nếu còn bật.
source "$(dirname "$0")/common.sh"

cmd="${1:?create | status | switchover | delete}"

case "$cmd" in
  create)
    version="${2:?target engine version}"
    pg="${3:?target cluster parameter group}"
    managed="$(aws rds describe-db-clusters --db-cluster-identifier "$CLUSTER" \
      --query 'DBClusters[0].MasterUserSecret.SecretArn' --output text)"
    if [[ "$managed" != "None" ]]; then
      echo "Cluster $CLUSTER đang dùng mật khẩu master do RDS quản lý; Blue/Green không hỗ trợ cấu hình này." >&2
      echo "Đổi proxy sang secret riêng rồi tắt: aws rds modify-db-cluster --no-manage-master-user-password (xem README)." >&2
      exit 1
    fi
    account="$(aws sts get-caller-identity --query Account --output text)"
    aws rds create-blue-green-deployment \
      --blue-green-deployment-name "${CLUSTER}-upgrade" \
      --source "arn:aws:rds:${AWS_REGION}:${account}:cluster:${CLUSTER}" \
      --target-engine-version "$version" \
      --target-db-cluster-parameter-group-name "$pg" \
      --query 'BlueGreenDeployment.BlueGreenDeploymentIdentifier' --output text
    ;;
  status)
    aws rds describe-blue-green-deployments \
      --blue-green-deployment-identifier "${2:?id}" \
      --query 'BlueGreenDeployments[0].[Status,SwitchoverDetails]' --output json
    ;;
  switchover)
    aws rds switchover-blue-green-deployment \
      --blue-green-deployment-identifier "${2:?id}" \
      --switchover-timeout 300 >/dev/null
    echo "Đã gửi lệnh switchover. Theo dõi bằng: $0 status ${2}"
    ;;
  delete)
    aws rds delete-blue-green-deployment \
      --blue-green-deployment-identifier "${2:?id}" >/dev/null
    echo "Đã xoá deployment (cluster blue cũ vẫn còn cho tới khi bạn tự xoá)."
    ;;
  *)
    echo "Lệnh không hợp lệ: $cmd" >&2
    exit 1
    ;;
esac
