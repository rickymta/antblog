data "aws_caller_identity" "current" {}
data "aws_region" "current" {}

locals {
  name         = "toinoi-${var.env}"
  ecr_registry = "${data.aws_caller_identity.current.account_id}.dkr.ecr.${data.aws_region.current.region}.amazonaws.com"
  db_services  = { for k, v in var.services : k => v if v.needs_db }
}

module "network" {
  source = "../../modules/network"
  # Khi module nằm ở repo riêng, ghim theo git tag:
  # source = "git::https://git.toinoi.example/platform/terraform-modules.git//network?ref=v1.2.0"

  name               = local.name
  vpc_cidr           = var.vpc_cidr
  az_count           = 2
  enable_nat_gateway = var.enable_nat_gateway
}

resource "aws_ecs_cluster" "this" {
  name = local.name

  tags = {
    Service = "platform"
  }
}

module "service" {
  source   = "../../modules/ecs-service"
  for_each = var.services

  name         = local.name
  env          = var.env
  service_name = each.key
  cluster_arn  = aws_ecs_cluster.this.arn
  vpc_id       = module.network.vpc_id
  subnet_ids   = module.network.private_subnet_ids

  image          = "${local.ecr_registry}/${var.ecr_repository_prefix}/${each.key}:${each.value.image_tag}"
  container_port = each.value.container_port
  cpu            = each.value.cpu
  memory         = each.value.memory
  desired_count  = each.value.desired_count

  environment = merge(
    {
      ENVIRONMENT  = var.env
      SERVICE_NAME = each.key
    },
    each.value.needs_db ? {
      DB_HOST = module.rds.address
      DB_PORT = tostring(module.rds.port)
      DB_NAME = module.rds.db_name
    } : {}
  )

  db_secret_arn = each.value.needs_db ? module.rds.master_user_secret_arn : null
}

module "rds" {
  source = "../../modules/rds-postgres"

  name       = "${local.name}-trips"
  vpc_id     = module.network.vpc_id
  subnet_ids = module.network.private_subnet_ids

  multi_az            = var.db_multi_az
  deletion_protection = var.db_deletion_protection
  skip_final_snapshot = var.db_skip_final_snapshot
}

# Rule ingress đặt ở môi trường, không đặt trong module rds: nếu để module rds nhận security group của service
# làm input, hai module phụ thuộc vòng về thiết kế (service cần secret và địa chỉ của rds), dễ thành lỗi Cycle
# khi rule viết inline và không tách được state database với state service.
# Chỉ service có needs_db = true mới được mở cổng 5432.
resource "aws_vpc_security_group_ingress_rule" "db_from_service" {
  for_each = local.db_services

  security_group_id            = module.rds.security_group_id
  referenced_security_group_id = module.service[each.key].security_group_id
  description                  = "PostgreSQL từ ${each.key}"
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
}
