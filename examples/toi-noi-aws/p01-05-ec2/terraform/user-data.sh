#!/bin/bash
# User data cho instance toinoi-lab-quote (Amazon Linux 2023). cloud-init chạy script này MỘT LẦN, bằng root,
# ở lần boot đầu. Log: /var/log/cloud-init-output.log
# Container nginx dưới đây chỉ là PLACEHOLDER cho service quote thật (image thật đẩy lên ECR ở bài 11).
set -euxo pipefail

dnf install -y docker
systemctl enable --now docker

mkdir -p /opt/toinoi/quote
cat >/opt/toinoi/quote/default.conf <<'NGINX'
server {
  listen 8080;
  location = /health/ready {
    default_type text/plain;
    return 200 'ready\n';
  }
  location = /v1/quote {
    default_type application/json;
    return 200 '{"service":"quote","note":"placeholder","distance_km":5,"fare_vnd":25500}\n';
  }
}
NGINX

docker run -d --name quote --restart unless-stopped -p 8080:8080 \
  -v /opt/toinoi/quote/default.conf:/etc/nginx/conf.d/default.conf:ro \
  nginx:stable-alpine
