locals {
  rg_id = "toinoi-${var.env}-ha"

  # Tên node trong cluster mode theo quy ước <replication-group-id>-<shard 4 chữ số>-<node 3 chữ số>.
  # Dùng để đặt alarm cho từng node mà không phụ thuộc giá trị chỉ biết sau khi apply.
  node_ids = toset(flatten([
    for shard in range(1, var.num_node_groups + 1) : [
      for node in range(1, var.replicas_per_node_group + 2) : format("%s-%04d-%03d", local.rg_id, shard, node)
    ]
  ]))
}

resource "aws_elasticache_subnet_group" "main" {
  name       = local.rg_id
  subnet_ids = var.private_subnet_ids
}

resource "aws_security_group" "cache" {
  name        = "${local.rg_id}-cache"
  description = "ElastiCache cluster mode cua Toi Noi"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_ingress_rule" "cache_from_client" {
  security_group_id            = aws_security_group.cache.id
  referenced_security_group_id = var.client_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 6379
  to_port                      = 6379
  description                  = "Service vao cache"
}

# cluster-enabled = yes là thứ biến replication group thành cluster mode enabled.
resource "aws_elasticache_parameter_group" "main" {
  name        = local.rg_id
  family      = var.parameter_group_family
  description = "Cluster mode, eviction va slow log cho cache Toi Noi"

  parameter {
    name  = "cluster-enabled"
    value = "yes"
  }

  parameter {
    name  = "maxmemory-policy"
    value = var.maxmemory_policy
  }

  parameter {
    name  = "reserved-memory-percent"
    value = tostring(var.reserved_memory_percent)
  }

  parameter {
    name  = "slowlog-log-slower-than"
    value = "10000"
  }
}

resource "aws_cloudwatch_log_group" "slow" {
  name              = "/toinoi/${var.env}/elasticache/${local.rg_id}/slow-log"
  retention_in_days = 14
}

resource "aws_elasticache_replication_group" "main" {
  replication_group_id = local.rg_id
  description          = "Cache va rate limit, cluster mode, Multi-AZ"

  engine                  = var.engine
  engine_version          = var.engine_version
  node_type               = var.node_type
  parameter_group_name    = aws_elasticache_parameter_group.main.name
  port                    = 6379
  subnet_group_name       = aws_elasticache_subnet_group.main.name
  security_group_ids      = [aws_security_group.cache.id]
  num_node_groups         = var.num_node_groups
  replicas_per_node_group = var.replicas_per_node_group

  automatic_failover_enabled = true
  multi_az_enabled           = true

  at_rest_encryption_enabled = true
  transit_encryption_enabled = true

  # Cache không phải nguồn sự thật nên không giữ snapshot (bài 15). Đổi nếu dùng cho dữ liệu cần phục hồi.
  snapshot_retention_limit = 0
  apply_immediately        = true

  log_delivery_configuration {
    destination      = aws_cloudwatch_log_group.slow.name
    destination_type = "cloudwatch-logs"
    log_format       = "json"
    log_type         = "slow-log"
  }
}

resource "aws_cloudwatch_metric_alarm" "memory" {
  for_each = local.node_ids

  alarm_name          = "${each.key}-memory-high"
  alarm_description   = "DatabaseMemoryUsagePercentage vuot 80%"
  namespace           = "AWS/ElastiCache"
  metric_name         = "DatabaseMemoryUsagePercentage"
  statistic           = "Maximum"
  period              = 60
  evaluation_periods  = 5
  threshold           = 80
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    CacheClusterId = each.key
  }

  depends_on = [aws_elasticache_replication_group.main]
}

resource "aws_cloudwatch_metric_alarm" "evictions" {
  for_each = local.node_ids

  alarm_name          = "${each.key}-evictions"
  alarm_description   = "Co khoa bi loai vi het bo nho"
  namespace           = "AWS/ElastiCache"
  metric_name         = "Evictions"
  statistic           = "Sum"
  period              = 60
  evaluation_periods  = 5
  threshold           = 0
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    CacheClusterId = each.key
  }

  depends_on = [aws_elasticache_replication_group.main]
}
