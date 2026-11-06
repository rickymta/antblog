locals {
  prefix = "toinoi-${var.env}"
}

resource "aws_dms_replication_subnet_group" "this" {
  replication_subnet_group_id          = "${local.prefix}-dms"
  replication_subnet_group_description = "Subnet của DMS cho migration trips"
  subnet_ids                           = var.subnet_ids

  depends_on = [aws_iam_role_policy_attachment.dms_vpc]
}

# Nguồn: PostgreSQL tự vận hành. Cần wal_level=logical, replication slot, user có quyền REPLICATION (xem sql/source-prepare.sql).
resource "aws_dms_endpoint" "source" {
  endpoint_id   = "${local.prefix}-source-pg"
  endpoint_type = "source"
  engine_name   = "postgres"
  database_name = var.source_database_name
  server_name   = var.source_server_name
  port          = 5432
  ssl_mode      = "require"

  secrets_manager_arn             = var.source_secret_arn
  secrets_manager_access_role_arn = aws_iam_role.secrets_access.arn

  # Không đặt slot_name: với full-load-and-cdc, DMS tự tạo slot. slot_name chỉ dùng cho slot có sẵn
  # kèm CdcStartPosition (task chỉ-CDC); đặt tên slot chưa tồn tại thì task báo lỗi.
  # test-decoding có sẵn trong PostgreSQL; pglogical cần cài extension trên nguồn.
  postgres_settings {
    plugin_name      = "test-decoding"
    heartbeat_enable = true
  }

  tags = {
    Service = "migration"
  }

  depends_on = [aws_iam_role_policy.secrets_access]
}

# Đích: Aurora PostgreSQL. Tắt kiểm tra FK và trigger trong lúc nạp bằng session_replication_role = replica.
resource "aws_dms_endpoint" "target" {
  endpoint_id   = "${local.prefix}-target-aurora"
  endpoint_type = "target"
  engine_name   = "aurora-postgresql"
  database_name = var.target_database_name
  server_name   = var.target_server_name
  port          = 5432
  ssl_mode      = "require"

  secrets_manager_arn             = var.target_secret_arn
  secrets_manager_access_role_arn = aws_iam_role.secrets_access.arn

  postgres_settings {
    after_connect_script = "SET session_replication_role = 'replica'"
  }

  tags = {
    Service = "migration"
  }

  depends_on = [aws_iam_role_policy.secrets_access]
}
