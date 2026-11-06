# Cách 1: replication instance + replication task (mặc định).
resource "aws_dms_replication_instance" "this" {
  count = var.use_serverless ? 0 : 1

  replication_instance_id     = "${local.prefix}-dms"
  replication_instance_class  = var.instance_class
  allocated_storage           = var.allocated_storage_gb
  multi_az                    = var.multi_az
  publicly_accessible         = false
  replication_subnet_group_id = aws_dms_replication_subnet_group.this.id
  vpc_security_group_ids      = var.security_group_ids
  apply_immediately           = true

  tags = {
    Service = "migration"
  }
}

resource "aws_dms_replication_task" "trips" {
  count = var.use_serverless ? 0 : 1

  replication_task_id       = "${local.prefix}-trips-full-load-cdc"
  migration_type            = "full-load-and-cdc"
  replication_instance_arn  = aws_dms_replication_instance.this[0].replication_instance_arn
  source_endpoint_arn       = aws_dms_endpoint.source.endpoint_arn
  target_endpoint_arn       = aws_dms_endpoint.target.endpoint_arn
  table_mappings            = file("${path.module}/table-mappings.json")
  replication_task_settings = file("${path.module}/task-settings.json")
  start_replication_task    = var.start_task

  tags = {
    Service = "migration"
  }
}

# Cách 2: DMS Serverless. Không có instance để chọn; đặt dải capacity và để DMS co giãn.
resource "aws_dms_replication_config" "trips" {
  count = var.use_serverless ? 1 : 0

  replication_config_identifier = "${local.prefix}-trips-serverless"
  replication_type              = "full-load-and-cdc"
  source_endpoint_arn           = aws_dms_endpoint.source.endpoint_arn
  target_endpoint_arn           = aws_dms_endpoint.target.endpoint_arn
  table_mappings                = file("${path.module}/table-mappings.json")
  replication_settings          = file("${path.module}/task-settings.json")
  start_replication             = var.start_task

  compute_config {
    replication_subnet_group_id  = aws_dms_replication_subnet_group.this.id
    vpc_security_group_ids       = var.security_group_ids
    min_capacity_units           = var.min_capacity_units
    max_capacity_units           = var.max_capacity_units
    multi_az                     = false
    preferred_maintenance_window = "sun:17:00-sun:18:00"
  }

  tags = {
    Service = "migration"
  }
}
