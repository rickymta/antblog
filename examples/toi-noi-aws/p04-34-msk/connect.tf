# MSK Connect chạy Debezium PostgreSQL đọc bảng outbox của Aurora (bài 31) rồi ghi vào Kafka.
# Chỉ tạo khi create_connector = true và mode = provisioned.
locals {
  connector_enabled = var.create_connector && local.is_prov
}

resource "aws_security_group" "connector" {
  count = local.connector_enabled ? 1 : 0

  name        = "${local.name}-connect"
  description = "MSK Connect: ra Kafka 9098 va database 5432"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_egress_rule" "connector_to_msk" {
  count = local.connector_enabled ? 1 : 0

  security_group_id            = aws_security_group.connector[0].id
  referenced_security_group_id = aws_security_group.msk.id
  ip_protocol                  = "tcp"
  from_port                    = 9098
  to_port                      = 9098
  description                  = "Connector ghi vao Kafka"
}

resource "aws_vpc_security_group_ingress_rule" "msk_from_connector" {
  count = local.connector_enabled ? 1 : 0

  security_group_id            = aws_security_group.msk.id
  referenced_security_group_id = aws_security_group.connector[0].id
  ip_protocol                  = "tcp"
  from_port                    = 9098
  to_port                      = 9098
  description                  = "Connector vao Kafka bang IAM"
}

resource "aws_vpc_security_group_egress_rule" "connector_to_db" {
  count = local.connector_enabled && var.db_security_group_id != null ? 1 : 0

  security_group_id            = aws_security_group.connector[0].id
  referenced_security_group_id = var.db_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
  description                  = "Connector doc WAL cua database"
}

resource "aws_vpc_security_group_ingress_rule" "db_from_connector" {
  count = local.connector_enabled && var.db_security_group_id != null ? 1 : 0

  security_group_id            = var.db_security_group_id
  referenced_security_group_id = aws_security_group.connector[0].id
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
  description                  = "MSK Connect vao database"
}

# Cần đường ra HTTPS tới Secrets Manager (NAT hoặc VPC endpoint) nếu dùng config provider.
resource "aws_vpc_security_group_egress_rule" "connector_https" {
  count = local.connector_enabled ? 1 : 0

  security_group_id = aws_security_group.connector[0].id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
  description       = "Secrets Manager qua NAT hoac VPC endpoint"
}

data "aws_iam_policy_document" "connector_assume" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["kafkaconnect.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "connector" {
  # WriteDataIdempotently nằm ở tài nguyên cluster: producer của Kafka Connect 3.x bật idempotence mặc định.
  statement {
    actions   = ["kafka-cluster:Connect", "kafka-cluster:DescribeCluster", "kafka-cluster:WriteDataIdempotently"]
    resources = [local.cluster_arn]
  }

  # Connector ghi các topic nghiệp vụ và tạo topic nội bộ của Kafka Connect.
  # Thu hẹp lại theo tên topic cụ thể sau khi biết tên topic nội bộ trong môi trường của bạn.
  statement {
    actions = [
      "kafka-cluster:CreateTopic",
      "kafka-cluster:DescribeTopic",
      "kafka-cluster:WriteData",
      "kafka-cluster:ReadData",
    ]
    resources = ["${local.topic_arn_prefix}/*"]
  }

  statement {
    actions   = ["kafka-cluster:AlterGroup", "kafka-cluster:DescribeGroup"]
    resources = ["${local.group_arn_prefix}/*"]
  }

  statement {
    actions   = ["secretsmanager:GetSecretValue", "secretsmanager:DescribeSecret"]
    resources = ["arn:aws:secretsmanager:${var.region}:*:secret:toinoi-${var.env}/debezium-*"]
  }
}

resource "aws_iam_role" "connector" {
  count = local.connector_enabled ? 1 : 0

  name               = "${local.name}-connector"
  assume_role_policy = data.aws_iam_policy_document.connector_assume.json
}

resource "aws_iam_role_policy" "connector" {
  count = local.connector_enabled ? 1 : 0

  name   = "msk-connect"
  role   = aws_iam_role.connector[0].id
  policy = data.aws_iam_policy_document.connector.json
}

resource "aws_cloudwatch_log_group" "connector" {
  count = local.connector_enabled ? 1 : 0

  name              = "/toinoi/${var.env}/msk-connect/outbox"
  retention_in_days = 14
}

resource "aws_mskconnect_custom_plugin" "debezium" {
  count = local.connector_enabled ? 1 : 0

  name         = "${local.name}-debezium-postgres"
  content_type = "ZIP"

  location {
    s3 {
      bucket_arn = var.plugin_bucket_arn
      file_key   = var.plugin_file_key
    }
  }
}

# Worker: converter mặc định và config provider để đọc mật khẩu từ Secrets Manager.
# Tên lớp provider theo hướng dẫn "config providers" của MSK Connect (gói aws-samples/msk-config-providers);
# plugin ZIP phải chứa cả Debezium lẫn jar của provider.
resource "aws_mskconnect_worker_configuration" "main" {
  count = local.connector_enabled ? 1 : 0

  name = "${local.name}-worker"

  properties_file_content = <<-EOT
    key.converter=org.apache.kafka.connect.storage.StringConverter
    value.converter=org.apache.kafka.connect.json.JsonConverter
    config.providers=secretsmanager
    config.providers.secretsmanager.class=com.amazonaws.kafka.config.providers.SecretsManagerConfigProvider
    config.providers.secretsmanager.param.region=${var.region}
  EOT
}

resource "aws_mskconnect_connector" "outbox" {
  count = local.connector_enabled ? 1 : 0

  name                 = "${local.name}-outbox"
  kafkaconnect_version = var.kafkaconnect_version

  capacity {
    provisioned_capacity {
      mcu_count    = 1
      worker_count = 1
    }
  }

  # Debezium PostgreSQL chỉ chạy một task, nên một worker là đủ.
  connector_configuration = merge(
    jsondecode(file("${path.module}/debezium-outbox.json")),
    { "database.hostname" = var.db_host },
  )

  kafka_cluster {
    apache_kafka_cluster {
      bootstrap_servers = aws_msk_cluster.main[0].bootstrap_brokers_sasl_iam

      vpc {
        security_groups = [aws_security_group.connector[0].id]
        subnets         = var.private_subnet_ids
      }
    }
  }

  kafka_cluster_client_authentication {
    authentication_type = "IAM"
  }

  kafka_cluster_encryption_in_transit {
    encryption_type = "TLS"
  }

  plugin {
    custom_plugin {
      arn      = aws_mskconnect_custom_plugin.debezium[0].arn
      revision = aws_mskconnect_custom_plugin.debezium[0].latest_revision
    }
  }

  worker_configuration {
    arn      = aws_mskconnect_worker_configuration.main[0].arn
    revision = aws_mskconnect_worker_configuration.main[0].latest_revision
  }

  service_execution_role_arn = aws_iam_role.connector[0].arn

  log_delivery {
    worker_log_delivery {
      cloudwatch_logs {
        enabled   = true
        log_group = aws_cloudwatch_log_group.connector[0].name
      }
    }
  }
}
