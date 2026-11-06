#!/usr/bin/env bash
# Đổi trạng thái routing control của ARC (drill failover). KHÔNG chạy tự động.
#
# Lúc sự cố chỉ gọi data plane (5 endpoint của cluster, đặt ở nhiều region), KHÔNG gọi control
# plane ở us-west-2. Vì vậy danh sách endpoint phải lưu sẵn từ lúc bình thường:
#   aws route53-recovery-control-config describe-cluster --cluster-arn "$CLUSTER_ARN" \
#     --region us-west-2 --query 'Cluster.ClusterEndpoints[].[Endpoint,Region]' \
#     --output text > arc-endpoints.tsv
# rồi cất tệp đó vào runbook (endpoint của cluster không đổi trong suốt vòng đời cluster).
#
# Dùng: scripts/toggle-routing-control.sh <routing-control-arn> <On|Off> [arc-endpoints.tsv]
set -euo pipefail

ARN="${1:?cần routing control ARN}"
STATE="${2:?cần On hoặc Off}"
ENDPOINTS_FILE="${3:-arc-endpoints.tsv}"

case "$STATE" in
  On | Off) ;;
  *) echo "Trạng thái phải là On hoặc Off" >&2; exit 2 ;;
esac

if [ ! -s "$ENDPOINTS_FILE" ]; then
  echo "Không thấy $ENDPOINTS_FILE: tạo trước bằng lệnh describe-cluster ở đầu script." >&2
  exit 2
fi

# Thử lần lượt từng endpoint data plane; endpoint đầu tiên nhận lệnh là đủ.
while IFS=$'\t' read -r endpoint region; do
  [ -n "$endpoint" ] || continue
  if aws route53-recovery-cluster update-routing-control-state \
    --routing-control-arn "$ARN" --routing-control-state "$STATE" \
    --endpoint-url "$endpoint" --region "$region"; then
    echo "Đã đặt $ARN = $STATE qua $endpoint"
    exit 0
  fi
done <"$ENDPOINTS_FILE"

echo "Không endpoint nào của cluster trả lời" >&2
exit 1
