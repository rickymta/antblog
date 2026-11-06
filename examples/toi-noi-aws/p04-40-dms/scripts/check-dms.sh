#!/usr/bin/env bash
# Xem trạng thái replication task của DMS: tiến độ full load, thống kê từng bảng, trạng thái validation và độ trễ CDC.
# Dùng: AWS_PROFILE=<lab> ./check-dms.sh <replication-task-arn>
# Chỉ đọc, không đổi gì.
set -euo pipefail

: "${AWS_PROFILE:?Đặt AWS_PROFILE trỏ vào tài khoản lab}"
TASK_ARN="${1:?Cần ARN của replication task}"
REGION="${AWS_REGION:-ap-southeast-1}"

echo "== Trạng thái task =="
aws dms describe-replication-tasks --region "$REGION" \
  --filters "Name=replication-task-arn,Values=${TASK_ARN}" \
  --query 'ReplicationTasks[0].{Status:Status,Stop:StopReason,FullLoad:ReplicationTaskStats.FullLoadProgressPercent,TablesLoaded:ReplicationTaskStats.TablesLoaded,TablesErrored:ReplicationTaskStats.TablesErrored}' \
  --output table

echo "== Thống kê từng bảng (số dòng đã nạp, thay đổi CDC, validation) =="
aws dms describe-table-statistics --region "$REGION" --replication-task-arn "$TASK_ARN" \
  --query 'TableStatistics[].{Table:TableName,FullLoadRows:FullLoadRows,Inserts:Inserts,Updates:Updates,Deletes:Deletes,Validation:ValidationState,Failed:ValidationFailedRecords}' \
  --output table

echo "== Metric độ trễ CDC có trong CloudWatch (namespace AWS/DMS) =="
aws cloudwatch list-metrics --region "$REGION" --namespace AWS/DMS --metric-name CDCLatencyTarget \
  --query 'Metrics[].Dimensions[].{Name:Name,Value:Value}' --output table
echo "Lấy giá trị: aws cloudwatch get-metric-statistics với các dimension ở trên (Statistics=Maximum, period 60 giây)."
echo "Điều kiện cutover: CDCLatencyTarget về 0 và validation không còn bản ghi lệch."
