#!/usr/bin/env bash
# Build, đăng nhập, push image service đặt chuyến lên ECR rồi in digest.
# Chạy trong tài khoản lab của bạn (profile SSO đã `aws sso login`). Script tạo image trong ECR, KHÔNG tạo repository.
set -euo pipefail

REGION="${AWS_REGION:-ap-southeast-1}"
ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"
REGISTRY="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"
REPO="toinoi/booking"
TAG="${1:?Cách dùng: push-image.sh <tag, ví dụ 1.0.0>}"
# Đường dẫn tính từ thư mục scripts/ của lab: lên ba cấp để tới examples/.
CONTEXT="${CONTEXT:-../../../toi-noi-dotnet/31-38-trip-service}"

# Fargate mặc định chạy Linux/X86_64: ép nền tảng khi build trên máy ARM (Apple Silicon).
docker build --platform linux/amd64 -t "${REPO}:${TAG}" "${CONTEXT}"

aws ecr get-login-password --region "${REGION}" \
  | docker login --username AWS --password-stdin "${REGISTRY}"

docker tag "${REPO}:${TAG}" "${REGISTRY}/${REPO}:${TAG}"
docker push "${REGISTRY}/${REPO}:${TAG}"

DIGEST="$(aws ecr describe-images --region "${REGION}" --repository-name "${REPO}" \
  --image-ids imageTag="${TAG}" --query 'imageDetails[0].imageDigest' --output text)"

echo "Image tham chiếu theo digest (dùng cái này trong task definition):"
echo "${REGISTRY}/${REPO}@${DIGEST}"
