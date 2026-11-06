#!/usr/bin/env bash
# Biến dùng chung. Cần KAFKA_HOME trỏ tới thư mục bản phân phối Kafka (có bin/kafka-topics.sh).
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
export AWS_DEFAULT_REGION="$AWS_REGION"
export AWS_PAGER=""
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC2034
CLIENT_PROPS="${CLIENT_PROPS:-$HERE/../client.properties}"
# shellcheck disable=SC2034  # các biến này được dùng bởi script gọi source
KAFKA_BIN="${KAFKA_HOME:?đặt KAFKA_HOME=<thư mục Kafka>}/bin"

# Lấy bootstrap brokers (IAM, cổng 9098) từ ARN cluster nếu chưa đặt BOOTSTRAP.
# shellcheck disable=SC2034
if [[ -z "${BOOTSTRAP:-}" ]]; then
  BOOTSTRAP="$(aws kafka get-bootstrap-brokers --cluster-arn "${CLUSTER_ARN:?đặt CLUSTER_ARN hoặc BOOTSTRAP}" \
    --query 'BootstrapBrokerStringSaslIam' --output text)"
fi
