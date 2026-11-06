locals {
  name = "toinoi-${var.env}-cache"
}

resource "aws_elasticache_subnet_group" "main" {
  name        = local.name
  description = "Private subnet cho cache Tới Nơi"
  subnet_ids  = var.private_subnet_ids
}

# Chỉ security group của client được vào cổng 6379; cache không có endpoint công khai.
resource "aws_security_group" "cache" {
  name        = local.name
  description = "ElastiCache cua Toi Noi"
  vpc_id      = var.vpc_id

  tags = {
    Name    = local.name
    Service = "quote"
  }
}

resource "aws_vpc_security_group_ingress_rule" "from_client" {
  security_group_id            = aws_security_group.cache.id
  ip_protocol                  = "tcp"
  from_port                    = 6379
  to_port                      = 6379
  referenced_security_group_id = var.client_security_group_id
  description                  = "Redis/Valkey tu client"
}

# Phương án 1: theo node. Primary + replica ở hai AZ, failover tự động, mã hoá in-transit và at-rest.
resource "aws_elasticache_replication_group" "node" {
  count = var.mode == "node" ? 1 : 0

  replication_group_id       = local.name
  description                = "Cache bao gia, session tai xe, rate limit cua Toi Noi"
  engine                     = var.engine
  engine_version             = var.engine_version
  node_type                  = var.node_type
  parameter_group_name       = var.parameter_group_name
  port                       = 6379
  num_cache_clusters         = var.num_cache_clusters
  automatic_failover_enabled = var.num_cache_clusters > 1
  multi_az_enabled           = var.num_cache_clusters > 1
  subnet_group_name          = aws_elasticache_subnet_group.main.name
  security_group_ids         = [aws_security_group.cache.id]
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  auth_token                 = var.auth_token
  snapshot_retention_limit   = 0 # cache không phải nguồn sự thật; lab không giữ snapshot
  apply_immediately          = true

  tags = {
    Service = "quote"
  }
}

# Phương án 2: ElastiCache Serverless. Không chọn node; trả theo dung lượng lưu và ECPU đã dùng.
resource "aws_elasticache_serverless_cache" "serverless" {
  count = var.mode == "serverless" ? 1 : 0

  name                 = local.name
  description          = "Cache Toi Noi (serverless)"
  engine               = var.engine
  major_engine_version = var.major_engine_version
  subnet_ids           = var.private_subnet_ids
  security_group_ids   = [aws_security_group.cache.id]

  # Trần để tránh hoá đơn bất ngờ khi tải tăng đột biến.
  cache_usage_limits {
    data_storage {
      maximum = 1
      unit    = "GB"
    }
    ecpu_per_second {
      maximum = 5000
    }
  }

  tags = {
    Service = "quote"
  }
}
