#!/usr/bin/env bash
# Thiết lập phía AWS cho bài 25. Script này CHỈ IN lệnh ra màn hình, không tự gọi AWS.
# Đọc kỹ, đổi org/repo, rồi tự chạy từng lệnh bằng profile SSO của tài khoản lab.
set -euo pipefail

ACCOUNT_ID="${ACCOUNT_ID:-123456789012}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat <<CMDS
# 1. Tạo OIDC identity provider cho GitHub (một lần cho mỗi tài khoản).
aws iam create-open-id-connect-provider \\
  --url https://token.actions.githubusercontent.com \\
  --client-id-list sts.amazonaws.com

# 2. Role build: trust giới hạn nhánh main, quyền chỉ push ECR repo toinoi/quote.
aws iam create-role \\
  --role-name toinoi-github-quote-build \\
  --assume-role-policy-document file://${HERE}/iam/trust-policy-build.json
aws iam put-role-policy \\
  --role-name toinoi-github-quote-build \\
  --policy-name ecr-push-quote \\
  --policy-document file://${HERE}/iam/policy-build-ecr-push.json

# 3. Role deploy prod: trust giới hạn environment prod, quyền chỉ update service quote.
aws iam create-role \\
  --role-name toinoi-github-quote-deploy-prod \\
  --assume-role-policy-document file://${HERE}/iam/trust-policy-deploy-prod.json
aws iam put-role-policy \\
  --role-name toinoi-github-quote-deploy-prod \\
  --policy-name ecs-deploy-quote \\
  --policy-document file://${HERE}/iam/policy-deploy-ecs.json

# Tài khoản đang dùng: ${ACCOUNT_ID}
CMDS
