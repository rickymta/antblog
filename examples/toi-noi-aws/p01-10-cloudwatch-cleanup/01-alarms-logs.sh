#!/usr/bin/env bash
set -euo pipefail
# SNS topic + email, log group có retention, metric filter đếm ERROR, và ba alarm.
# Alarm tính tiền theo số alarm mỗi tháng; metric tuỳ chỉnh (ToiNoi/Quote) và log lưu trữ cũng tính tiền.
source "$(dirname "$0")/common.sh"
confirm_identity
: "${ALERT_EMAIL:?}"

log "SNS topic và đăng ký email (phải bấm Confirm trong thư)"
aws sns create-topic --name "$TOPIC_NAME" --tags "${TAGS_KV[@]}" >/dev/null
aws sns subscribe --topic-arn "$TOPIC_ARN" --protocol email --notification-endpoint "$ALERT_EMAIL" >/dev/null

log "Log group, retention 14 ngày, tag"
if [[ -z "$(aws logs describe-log-groups --log-group-name-prefix "$LOG_GROUP" \
  --query "logGroups[?logGroupName=='$LOG_GROUP'].logGroupName" --output text)" ]]; then
  aws logs create-log-group --log-group-name "$LOG_GROUP"
fi
aws logs put-retention-policy --log-group-name "$LOG_GROUP" --retention-in-days 14
aws logs tag-resource --resource-arn "arn:aws:logs:${AWS_REGION}:${ACCOUNT_ID}:log-group:${LOG_GROUP}" \
  --tags Project=ToiNoi,Env=lab,Service=quote,Owner=platform

log "Metric filter: mỗi dòng log chứa ERROR cộng 1 vào QuoteErrorCount"
aws logs put-metric-filter --log-group-name "$LOG_GROUP" --filter-name quote-error-count \
  --filter-pattern "ERROR" \
  --metric-transformations "metricName=QuoteErrorCount,metricNamespace=$METRIC_NS,metricValue=1,defaultValue=0"

LB="$(alb_dim)"; TG="$(tg_dim)"
if [[ -z "$LB" || -z "$TG" ]]; then
  echo "Chưa có ALB/target group của lab 08: bỏ qua alarm ALB." >&2
else
  log "Alarm: 5xx từ target (3 trong 5 phút liên tiếp, không có dữ liệu = không vi phạm)"
  aws cloudwatch put-metric-alarm --alarm-name toinoi-lab-quote-target-5xx \
    --alarm-description "ToiNoi lab: service quote tra nhieu loi 5xx" \
    --namespace AWS/ApplicationELB --metric-name HTTPCode_Target_5XX_Count \
    --dimensions "Name=LoadBalancer,Value=$LB" \
    --statistic Sum --period 60 --evaluation-periods 5 --datapoints-to-alarm 3 \
    --threshold 10 --comparison-operator GreaterThanOrEqualToThreshold \
    --treat-missing-data notBreaching \
    --alarm-actions "$TOPIC_ARN" --ok-actions "$TOPIC_ARN" --tags "${TAGS_KV[@]}"

  log "Alarm: có target không khoẻ"
  aws cloudwatch put-metric-alarm --alarm-name toinoi-lab-quote-unhealthy-hosts \
    --alarm-description "ToiNoi lab: co target khong khoe trong target group quote" \
    --namespace AWS/ApplicationELB --metric-name UnHealthyHostCount \
    --dimensions "Name=TargetGroup,Value=$TG" "Name=LoadBalancer,Value=$LB" \
    --statistic Maximum --period 60 --evaluation-periods 2 --datapoints-to-alarm 2 \
    --threshold 1 --comparison-operator GreaterThanOrEqualToThreshold \
    --treat-missing-data missing \
    --alarm-actions "$TOPIC_ARN" --ok-actions "$TOPIC_ARN" --tags "${TAGS_KV[@]}"
fi

log "Alarm: số dòng ERROR trong log"
aws cloudwatch put-metric-alarm --alarm-name toinoi-lab-quote-log-errors \
  --alarm-description "ToiNoi lab: log service quote co nhieu dong ERROR" \
  --namespace "$METRIC_NS" --metric-name QuoteErrorCount \
  --statistic Sum --period 60 --evaluation-periods 5 --datapoints-to-alarm 3 \
  --threshold 5 --comparison-operator GreaterThanOrEqualToThreshold \
  --treat-missing-data notBreaching \
  --alarm-actions "$TOPIC_ARN" --ok-actions "$TOPIC_ARN" --tags "${TAGS_KV[@]}"

aws cloudwatch describe-alarms --alarm-name-prefix toinoi-lab- \
  --query 'MetricAlarms[].[AlarmName,StateValue]' --output table
