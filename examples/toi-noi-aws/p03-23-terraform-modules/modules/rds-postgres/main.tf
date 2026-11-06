resource "aws_db_subnet_group" "this" {
  name       = var.name
  subnet_ids = var.subnet_ids

  tags = {
    Service = "database"
  }
}

resource "aws_security_group" "db" {
  name        = "${var.name}-db"
  description = "PostgreSQL cho ${var.name}. Rule ingress được thêm từ bên ngoài module để tránh vòng phụ thuộc với service"
  vpc_id      = var.vpc_id

  tags = {
    Name    = "${var.name}-db"
    Service = "database"
  }
}

resource "aws_db_instance" "this" {
  identifier = var.name

  engine         = "postgres"
  engine_version = var.engine_version
  instance_class = var.instance_class

  allocated_storage = var.allocated_storage
  storage_type      = "gp3"
  storage_encrypted = true

  db_name  = var.db_name
  username = var.master_username

  # RDS tạo mật khẩu và lưu trong Secrets Manager; không có mật khẩu nào trong code hay tfvars.
  manage_master_user_password = true

  db_subnet_group_name   = aws_db_subnet_group.this.name
  vpc_security_group_ids = [aws_security_group.db.id]
  publicly_accessible    = false
  multi_az               = var.multi_az

  backup_retention_period = var.backup_retention_period
  deletion_protection     = var.deletion_protection

  skip_final_snapshot       = var.skip_final_snapshot
  final_snapshot_identifier = var.skip_final_snapshot ? null : "${var.name}-final"

  tags = {
    Service = "database"
  }
}
