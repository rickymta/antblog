data "aws_caller_identity" "current" {}

locals {
  proxy_secret_arn = coalesce(var.proxy_secret_arn, aws_rds_cluster.main.master_user_secret[0].secret_arn)
  # Resource ID của proxy (prx-...) nằm ở đoạn cuối của ARN; dùng cho rds-db:connect.
  proxy_resource_id = element(split(":", aws_db_proxy.main.arn), 6)
}

# Role để RDS Proxy đọc secret khi nối vào database.
data "aws_iam_policy_document" "proxy_assume" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["rds.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "proxy_secret" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [local.proxy_secret_arn]
  }

  # Cần khi secret mã hoá bằng KMS key; với key mặc định aws/secretsmanager, điều kiện ViaService giữ quyền hẹp.
  statement {
    actions   = ["kms:Decrypt"]
    resources = ["*"]

    condition {
      test     = "StringEquals"
      variable = "kms:ViaService"
      values   = ["secretsmanager.${var.region}.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "proxy" {
  name               = "${local.name}-proxy"
  assume_role_policy = data.aws_iam_policy_document.proxy_assume.json
}

resource "aws_iam_role_policy" "proxy_secret" {
  name   = "read-db-secret"
  role   = aws_iam_role.proxy.id
  policy = data.aws_iam_policy_document.proxy_secret.json
}

resource "aws_db_proxy" "main" {
  name                   = local.name
  engine_family          = "POSTGRESQL"
  role_arn               = aws_iam_role.proxy.arn
  vpc_subnet_ids         = var.private_subnet_ids
  vpc_security_group_ids = [aws_security_group.proxy.id]
  require_tls            = true
  idle_client_timeout    = 1800
  debug_logging          = false

  auth {
    auth_scheme = "SECRETS"
    iam_auth    = var.iam_auth_for_proxy
    secret_arn  = local.proxy_secret_arn
  }
}

# Hồ kết nối: proxy chỉ mở tối đa 80% max_connections của DB; app có thể mở nhiều hơn nhờ multiplexing.
resource "aws_db_proxy_default_target_group" "main" {
  db_proxy_name = aws_db_proxy.main.name

  connection_pool_config {
    max_connections_percent      = 80
    max_idle_connections_percent = 50
    connection_borrow_timeout    = 120
  }
}

resource "aws_db_proxy_target" "main" {
  db_proxy_name         = aws_db_proxy.main.name
  target_group_name     = aws_db_proxy_default_target_group.main.name
  db_cluster_identifier = aws_rds_cluster.main.cluster_identifier

  depends_on = [
    aws_rds_cluster_instance.writer,
    aws_rds_cluster_instance.reader,
  ]
}

# Endpoint chỉ đọc của proxy: query của CQRS đi đây, command đi endpoint mặc định.
resource "aws_db_proxy_endpoint" "read_only" {
  db_proxy_name          = aws_db_proxy.main.name
  db_proxy_endpoint_name = "${local.name}-ro"
  vpc_subnet_ids         = var.private_subnet_ids
  vpc_security_group_ids = [aws_security_group.proxy.id]
  target_role            = "READ_ONLY"
}

# Policy mẫu cho role của service booking: lấy token IAM để vào proxy với user toinoi_app.
data "aws_iam_policy_document" "app_connect" {
  statement {
    actions   = ["rds-db:connect"]
    resources = ["arn:aws:rds-db:${var.region}:${data.aws_caller_identity.current.account_id}:dbuser:${local.proxy_resource_id}/toinoi_app"]
  }
}

resource "aws_iam_policy" "app_connect" {
  name        = "${local.name}-app-connect"
  description = "Cho phep lay token IAM de vao RDS Proxy voi user toinoi_app"
  policy      = data.aws_iam_policy_document.app_connect.json
}
