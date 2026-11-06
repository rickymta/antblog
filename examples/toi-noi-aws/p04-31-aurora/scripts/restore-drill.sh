#!/usr/bin/env bash
# Diễn tập khôi phục: PITR hoặc fast clone ra cluster mới, đo thời gian tới khi instance sẵn sàng.
# Dùng:
#   ./restore-drill.sh pitr  [2026-11-06T09:50:00Z]   # full-copy, về thời điểm cho trước (mặc định: thời điểm mới nhất)
#   ./restore-drill.sh clone                         # copy-on-write (fast clone) từ thời điểm mới nhất
#   ./restore-drill.sh cleanup                       # xoá cluster diễn tập
# Restore luôn tạo cluster MỚI và chưa có instance: script tạo thêm một instance db.serverless.
source "$(dirname "$0")/common.sh"

TARGET="${CLUSTER}-drill"
mode="${1:?pitr | clone | cleanup}"

case "$mode" in
  pitr | clone)
    if [[ "$mode" == "clone" ]]; then
      restore_type="copy-on-write"
    else
      restore_type="full-copy"
    fi
    if [[ "$mode" == "pitr" && $# -ge 2 ]]; then
      when=(--restore-to-time "$2")
    else
      when=(--use-latest-restorable-time)
    fi

    start="$(now)"
    aws rds restore-db-cluster-to-point-in-time \
      --source-db-cluster-identifier "$CLUSTER" \
      --db-cluster-identifier "$TARGET" \
      --restore-type "$restore_type" \
      "${when[@]}" \
      --db-subnet-group-name "$SUBNET_GROUP" \
      --vpc-security-group-ids "$DB_SG" \
      --serverless-v2-scaling-configuration MinCapacity=0.5,MaxCapacity=4 \
      --tags Key=Project,Value=ToiNoi Key=Env,Value=lab Key=Service,Value=trips-db Key=Owner,Value=platform >/dev/null

    aws rds create-db-instance \
      --db-instance-identifier "${TARGET}-1" \
      --db-cluster-identifier "$TARGET" \
      --engine aurora-postgresql \
      --db-instance-class db.serverless >/dev/null

    aws rds wait db-instance-available --db-instance-identifier "${TARGET}-1"
    echo "Cluster $TARGET ($restore_type) sẵn sàng sau $(($(now) - start)) giây."
    aws rds describe-db-clusters --db-cluster-identifier "$TARGET" \
      --query 'DBClusters[0].[Endpoint,Status]' --output text
    echo "Đây mới chỉ là thời gian hạ tầng. RTO thực gồm cả kiểm dữ liệu, đổi cấu hình app và warm-up."
    ;;
  cleanup)
    aws rds delete-db-instance --db-instance-identifier "${TARGET}-1" >/dev/null || true
    aws rds wait db-instance-deleted --db-instance-identifier "${TARGET}-1" || true
    aws rds delete-db-cluster --db-cluster-identifier "$TARGET" --skip-final-snapshot >/dev/null
    echo "Đã xoá $TARGET."
    ;;
  *)
    echo "Dùng: $0 pitr [thời-điểm-UTC] | clone | cleanup" >&2
    exit 1
    ;;
esac
