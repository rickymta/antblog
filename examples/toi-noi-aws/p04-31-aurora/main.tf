locals {
  name = "toinoi-${var.env}-trips"
}

resource "aws_db_subnet_group" "main" {
  name       = local.name
  subnet_ids = var.private_subnet_ids
}

# Proxy chỉ nhận cổng 5432 từ nơi gọi; cluster chỉ nhận 5432 từ proxy.
resource "aws_security_group" "proxy" {
  name        = "${local.name}-proxy"
  description = "RDS Proxy cua trips"
  vpc_id      = var.vpc_id
}

resource "aws_security_group" "db" {
  name        = "${local.name}-db"
  description = "Aurora PostgreSQL cua trips"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_ingress_rule" "proxy_from_client" {
  security_group_id            = aws_security_group.proxy.id
  referenced_security_group_id = var.client_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
  description                  = "App vao proxy"
}

resource "aws_vpc_security_group_ingress_rule" "db_from_proxy" {
  security_group_id            = aws_security_group.db.id
  referenced_security_group_id = aws_security_group.proxy.id
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
  description                  = "Proxy vao Aurora"
}

resource "aws_vpc_security_group_egress_rule" "proxy_to_db" {
  security_group_id            = aws_security_group.proxy.id
  referenced_security_group_id = aws_security_group.db.id
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
  description                  = "Proxy ra Aurora"
}

# Hai tham số này cần cho Blue/Green deployment (bài này) và cho CDC với Debezium hoặc DMS (bài 34, 40).
# rds.logical_replication là tham số static: có hiệu lực sau khi reboot instance.
resource "aws_rds_cluster_parameter_group" "main" {
  name        = local.name
  family      = var.parameter_group_family
  description = "Tham so cluster trips: logical replication va log cham"

  parameter {
    name         = "rds.logical_replication"
    value        = "1"
    apply_method = "pending-reboot"
  }

  parameter {
    name         = "log_min_duration_statement"
    value        = "500"
    apply_method = "immediate"
  }
}

resource "aws_rds_cluster" "main" {
  cluster_identifier = local.name
  engine             = "aurora-postgresql"
  engine_mode        = "provisioned"
  engine_version     = var.engine_version
  database_name      = "tripsdb"
  master_username    = "toinoi_admin"

  # RDS tạo mật khẩu master và lưu trong Secrets Manager, tự xoay vòng (như bài 07).
  manage_master_user_password = true

  db_subnet_group_name            = aws_db_subnet_group.main.name
  vpc_security_group_ids          = [aws_security_group.db.id]
  db_cluster_parameter_group_name = aws_rds_cluster_parameter_group.main.name

  storage_type      = var.storage_mode == "aurora-iopt1" ? "aurora-iopt1" : null
  storage_encrypted = true

  backup_retention_period         = 7
  preferred_backup_window         = "17:00-18:00"
  copy_tags_to_snapshot           = true
  enabled_cloudwatch_logs_exports = ["postgresql"]

  serverlessv2_scaling_configuration {
    min_capacity             = var.serverless_min_acu
    max_capacity             = var.serverless_max_acu
    seconds_until_auto_pause = var.serverless_min_acu == 0 ? var.seconds_until_auto_pause : null
  }

  # Lab cố ý cho phép xoá nhanh. Production: deletion_protection = true, skip_final_snapshot = false.
  deletion_protection = false
  skip_final_snapshot = true
  apply_immediately   = true
}

# Writer: promotion tier 0, được chọn đầu tiên khi failover.
resource "aws_rds_cluster_instance" "writer" {
  identifier         = "${local.name}-writer"
  cluster_identifier = aws_rds_cluster.main.id
  engine             = aws_rds_cluster.main.engine
  engine_version     = aws_rds_cluster.main.engine_version
  instance_class     = "db.serverless"
  promotion_tier     = 0
}

# Reader cho service đọc: tier 1, ứng viên thay writer khi failover.
resource "aws_rds_cluster_instance" "reader" {
  identifier         = "${local.name}-reader-1"
  cluster_identifier = aws_rds_cluster.main.id
  engine             = aws_rds_cluster.main.engine
  engine_version     = aws_rds_cluster.main.engine_version
  instance_class     = "db.serverless"
  promotion_tier     = 1

  depends_on = [aws_rds_cluster_instance.writer]
}

# Reader dành riêng cho báo cáo: tier 15, gần như không bao giờ được chọn làm writer.
resource "aws_rds_cluster_instance" "reporting" {
  identifier         = "${local.name}-reporting"
  cluster_identifier = aws_rds_cluster.main.id
  engine             = aws_rds_cluster.main.engine
  engine_version     = aws_rds_cluster.main.engine_version
  instance_class     = "db.serverless"
  promotion_tier     = 15

  depends_on = [aws_rds_cluster_instance.writer]
}

# Custom endpoint: truy vấn báo cáo chỉ đi vào instance reporting, không tranh tài nguyên với reader của app.
resource "aws_rds_cluster_endpoint" "reporting" {
  cluster_identifier          = aws_rds_cluster.main.id
  cluster_endpoint_identifier = "reporting"
  custom_endpoint_type        = "READER"
  static_members              = [aws_rds_cluster_instance.reporting.id]
}

resource "aws_cloudwatch_metric_alarm" "replica_lag" {
  alarm_name          = "${local.name}-replica-lag"
  alarm_description   = "Do tre sao chep toi reader vuot 1 giay trong 5 phut"
  namespace           = "AWS/RDS"
  metric_name         = "AuroraReplicaLagMaximum"
  statistic           = "Maximum"
  period              = 60
  evaluation_periods  = 5
  threshold           = 1000
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    DBClusterIdentifier = aws_rds_cluster.main.cluster_identifier
  }
}
