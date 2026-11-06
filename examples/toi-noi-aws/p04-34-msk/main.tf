locals {
  name       = "toinoi-${var.env}-events"
  is_prov    = var.mode == "provisioned"
  is_express = startswith(var.broker_instance_type, "express.")

  cluster_arn = coalesce(
    one(aws_msk_cluster.main[*].arn),
    one(aws_msk_serverless_cluster.main[*].arn),
  )

  # Quy ước ARN của MSK: .../cluster/<tên>/<uuid>, .../topic/<tên>/<uuid>/<topic>, .../group/<tên>/<uuid>/<group>
  topic_arn_prefix = replace(local.cluster_arn, ":cluster/", ":topic/")
  group_arn_prefix = replace(local.cluster_arn, ":cluster/", ":group/")
}

resource "aws_security_group" "msk" {
  name        = "${local.name}-msk"
  description = "MSK cua Toi Noi: chi nhan 9098 tu client va connector"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_ingress_rule" "msk_from_client" {
  security_group_id            = aws_security_group.msk.id
  referenced_security_group_id = var.client_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 9098
  to_port                      = 9098
  description                  = "Service vao Kafka bang IAM"
}

resource "aws_vpc_security_group_ingress_rule" "msk_self" {
  security_group_id            = aws_security_group.msk.id
  referenced_security_group_id = aws_security_group.msk.id
  ip_protocol                  = "-1"
  description                  = "Broker noi chuyen voi nhau"
}

resource "aws_vpc_security_group_egress_rule" "msk_self" {
  security_group_id            = aws_security_group.msk.id
  referenced_security_group_id = aws_security_group.msk.id
  ip_protocol                  = "-1"
  description                  = "Broker noi chuyen voi nhau"
}

resource "aws_cloudwatch_log_group" "msk" {
  name              = "/toinoi/${var.env}/msk/${local.name}"
  retention_in_days = 14
}

# Cấu hình broker cho MSK Provisioned với broker Standard. Broker Express đặt sẵn RF và min ISR an toàn,
# chỉ cho sửa một số thuộc tính, nên lab không gắn cấu hình này cho Express.
resource "aws_msk_configuration" "main" {
  count = local.is_prov && !local.is_express ? 1 : 0

  name           = local.name
  kafka_versions = [var.kafka_version]

  server_properties = <<-EOT
    auto.create.topics.enable=false
    default.replication.factor=3
    min.insync.replicas=2
    num.partitions=12
    log.retention.hours=168
    unclean.leader.election.enable=false
  EOT
}

resource "aws_msk_cluster" "main" {
  count = local.is_prov ? 1 : 0

  cluster_name           = local.name
  kafka_version          = var.kafka_version
  number_of_broker_nodes = 3
  enhanced_monitoring    = "PER_BROKER"

  broker_node_group_info {
    instance_type   = var.broker_instance_type
    client_subnets  = var.private_subnet_ids
    security_groups = [aws_security_group.msk.id]

    dynamic "storage_info" {
      for_each = local.is_express ? [] : [1]
      content {
        ebs_storage_info {
          volume_size = var.broker_volume_gb
        }
      }
    }

    # MSK không có endpoint công khai theo mặc định; đặt DISABLED để ý định rõ ràng.
    connectivity_info {
      public_access {
        type = "DISABLED"
      }
    }
  }

  client_authentication {
    unauthenticated = false

    sasl {
      iam = true
    }
  }

  encryption_info {
    encryption_in_transit {
      client_broker = "TLS"
      in_cluster    = true
    }
  }

  dynamic "configuration_info" {
    for_each = local.is_express ? [] : [1]
    content {
      arn      = aws_msk_configuration.main[0].arn
      revision = aws_msk_configuration.main[0].latest_revision
    }
  }

  logging_info {
    broker_logs {
      cloudwatch_logs {
        enabled   = true
        log_group = aws_cloudwatch_log_group.msk.name
      }
    }
  }
}

# MSK Serverless: chỉ IAM, AWS tự quản lý broker và đĩa; partition tính theo giới hạn dịch vụ.
resource "aws_msk_serverless_cluster" "main" {
  count = local.is_prov ? 0 : 1

  cluster_name = local.name

  vpc_config {
    subnet_ids         = var.private_subnet_ids
    security_group_ids = [aws_security_group.msk.id]
  }

  client_authentication {
    sasl {
      iam {
        enabled = true
      }
    }
  }
}

# UnderReplicatedPartitions có dimension Cluster Name và Broker ID: mỗi broker một alarm (broker ID 1..3).
resource "aws_cloudwatch_metric_alarm" "under_replicated" {
  for_each = local.is_prov ? toset(["1", "2", "3"]) : toset([])

  alarm_name          = "${local.name}-broker-${each.key}-under-replicated"
  alarm_description   = "Co partition thieu replica dong bo: 1 broker cham hoac mat"
  namespace           = "AWS/Kafka"
  metric_name         = "UnderReplicatedPartitions"
  statistic           = "Maximum"
  period              = 60
  evaluation_periods  = 5
  threshold           = 0
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    "Cluster Name" = local.name
    "Broker ID"    = each.key
  }
}
