#!/usr/bin/env bash
# Dựng cấu hình AppConfig của Tới Nơi bằng AWS CLI: ./cli/appconfig-cli.sh up|read|bad|down
# Script này TẠO tài nguyên thật trong tài khoản của profile đang dùng. Đọc README trước.
set -euo pipefail

cd "$(dirname "$0")/.."
IDS_FILE=".appconfig-ids.env"
APP_NAME="${APP_NAME:-toinoi-lab-quote}"
ENV_NAME="${ENV_NAME:-lab}"
STRATEGY_ID="${STRATEGY_ID:-AppConfig.AllAtOnce}" # prod: dùng chiến lược tự tạo có bake time
TAGS="Project=ToiNoi,Env=lab,Service=quote,Owner=platform"

wait_deployment() {
  local number="$1" state
  while true; do
    state=$(aws appconfig get-deployment --application-id "$APP_ID" --environment-id "$ENV_ID" \
      --deployment-number "$number" --query State --output text)
    echo "deployment $number: $state"
    case "$state" in
      COMPLETE) return 0 ;;
      ROLLED_BACK | REVERTED) echo "deployment bị rollback" >&2; return 1 ;;
    esac
    sleep 10
  done
}

up() {
  APP_ID=$(aws appconfig create-application --name "$APP_NAME" --tags "$TAGS" --query Id --output text)
  ENV_ID=$(aws appconfig create-environment --application-id "$APP_ID" --name "$ENV_NAME" --query Id --output text)

  FLAGS_ID=$(aws appconfig create-configuration-profile --application-id "$APP_ID" \
    --name pricing-flags --location-uri hosted --type AWS.AppConfig.FeatureFlags \
    --query Id --output text)
  FLAGS_VER=$(aws appconfig create-hosted-configuration-version --application-id "$APP_ID" \
    --configuration-profile-id "$FLAGS_ID" --content-type application/json \
    --content fileb://flags/pricing-flags.json --query VersionNumber --output text /dev/null)

  # Validator JSON Schema: chặn giá âm hoặc dưới ngưỡng trước khi cấu hình tới được service.
  VALIDATORS=$(python3 -c 'import json,sys; print(json.dumps([{"Type":"JSON_SCHEMA","Content":open(sys.argv[1]).read()}]))' config/pricing-table.schema.json)
  TABLE_ID=$(aws appconfig create-configuration-profile --application-id "$APP_ID" \
    --name pricing-table --location-uri hosted --type AWS.Freeform \
    --validators "$VALIDATORS" --query Id --output text)
  TABLE_VER=$(aws appconfig create-hosted-configuration-version --application-id "$APP_ID" \
    --configuration-profile-id "$TABLE_ID" --content-type application/json \
    --content fileb://config/pricing-table.json --query VersionNumber --output text /dev/null)

  printf 'APP_ID=%s\nENV_ID=%s\nFLAGS_ID=%s\nTABLE_ID=%s\n' "$APP_ID" "$ENV_ID" "$FLAGS_ID" "$TABLE_ID" > "$IDS_FILE"

  for pair in "$FLAGS_ID:$FLAGS_VER" "$TABLE_ID:$TABLE_VER"; do
    number=$(aws appconfig start-deployment --application-id "$APP_ID" --environment-id "$ENV_ID" \
      --deployment-strategy-id "$STRATEGY_ID" --configuration-profile-id "${pair%%:*}" \
      --configuration-version "${pair##*:}" --query DeploymentNumber --output text)
    wait_deployment "$number"
  done
  echo "Xong. ID đã lưu ở $IDS_FILE"
}

read_config() {
  # shellcheck disable=SC1090
  source "$IDS_FILE"
  token=$(aws appconfigdata start-configuration-session --application-identifier "$APP_ID" \
    --environment-identifier "$ENV_ID" --configuration-profile-identifier "$FLAGS_ID" \
    --query InitialConfigurationToken --output text)
  aws appconfigdata get-latest-configuration --configuration-token "$token" /dev/stdout
}

bad() {
  # Version có giá âm: validator phải chặn ở bước tạo version hoặc, muộn nhất, ở bước deploy.
  # shellcheck disable=SC1090
  source "$IDS_FILE"
  if version=$(aws appconfig create-hosted-configuration-version --application-id "$APP_ID" \
    --configuration-profile-id "$TABLE_ID" --content-type application/json \
    --content fileb://config/pricing-table.bad.json --query VersionNumber --output text /dev/null); then
    echo "Version $version đã được tạo; thử deploy, validator phải chặn ở bước này."
    if aws appconfig start-deployment --application-id "$APP_ID" --environment-id "$ENV_ID" \
      --deployment-strategy-id "$STRATEGY_ID" --configuration-profile-id "$TABLE_ID" \
      --configuration-version "$version"; then
      echo "CẢNH BÁO: deployment không bị chặn, kiểm lại validator." >&2
    else
      echo "Deployment bị từ chối như mong đợi."
    fi
  else
    echo "Việc tạo version bị từ chối như mong đợi."
  fi
}

down() {
  # shellcheck disable=SC1090
  source "$IDS_FILE"
  for profile in "$FLAGS_ID" "$TABLE_ID"; do
    for version in $(aws appconfig list-hosted-configuration-versions --application-id "$APP_ID" \
      --configuration-profile-id "$profile" --query 'Items[].VersionNumber' --output text); do
      aws appconfig delete-hosted-configuration-version --application-id "$APP_ID" \
        --configuration-profile-id "$profile" --version-number "$version"
    done
    aws appconfig delete-configuration-profile --application-id "$APP_ID" --configuration-profile-id "$profile"
  done
  aws appconfig delete-environment --application-id "$APP_ID" --environment-id "$ENV_ID"
  aws appconfig delete-application --application-id "$APP_ID"
  rm -f "$IDS_FILE"
}

case "${1:-}" in
  up) up ;;
  read) read_config ;;
  bad) bad ;;
  down) down ;;
  *) echo "Dùng: $0 up|read|bad|down" >&2; exit 2 ;;
esac
