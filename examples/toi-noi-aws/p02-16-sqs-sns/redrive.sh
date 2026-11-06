#!/usr/bin/env bash
# Lab bài 16: đưa message từ DLQ về queue gốc (redrive to source) sau khi đã sửa lỗi consumer.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
DLQ_ARN="$(terraform output -json dlq_arns | jq -r '."dispatch-push"')"

aws sqs start-message-move-task --source-arn "$DLQ_ARN"
aws sqs list-message-move-tasks --source-arn "$DLQ_ARN"
