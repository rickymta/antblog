data "aws_region" "current" {}

locals {
  full_name = "${var.name}-${var.service_name}"

  secrets = var.db_secret_arn == null ? [] : [
    { name = "DB_USERNAME", valueFrom = "${var.db_secret_arn}:username::" },
    { name = "DB_PASSWORD", valueFrom = "${var.db_secret_arn}:password::" },
  ]
}

resource "aws_cloudwatch_log_group" "this" {
  name              = "/toinoi/${var.env}/${var.service_name}"
  retention_in_days = var.log_retention_days

  tags = {
    Service = var.service_name
  }
}

data "aws_iam_policy_document" "ecs_tasks_assume" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

# Execution role: ECS agent kéo image, ghi log và đọc secret lúc khởi động task.
resource "aws_iam_role" "execution" {
  name               = "${local.full_name}-execution"
  assume_role_policy = data.aws_iam_policy_document.ecs_tasks_assume.json

  tags = {
    Service = var.service_name
  }
}

resource "aws_iam_role_policy_attachment" "execution_managed" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

data "aws_iam_policy_document" "read_db_secret" {
  count = var.db_secret_arn == null ? 0 : 1

  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [var.db_secret_arn]
  }
}

resource "aws_iam_role_policy" "read_db_secret" {
  count = var.db_secret_arn == null ? 0 : 1

  name   = "read-db-secret"
  role   = aws_iam_role.execution.id
  policy = data.aws_iam_policy_document.read_db_secret[0].json
}

# Task role: quyền runtime của ứng dụng. Để trống cho tới khi service cần gọi API AWS.
resource "aws_iam_role" "task" {
  name               = "${local.full_name}-task"
  assume_role_policy = data.aws_iam_policy_document.ecs_tasks_assume.json

  tags = {
    Service = var.service_name
  }
}

resource "aws_security_group" "service" {
  name        = "${local.full_name}-svc"
  description = "Task ${local.full_name}"
  vpc_id      = var.vpc_id

  tags = {
    Name    = "${local.full_name}-svc"
    Service = var.service_name
  }
}

resource "aws_vpc_security_group_egress_rule" "all" {
  security_group_id = aws_security_group.service.id
  description       = "Cho phép task đi ra (kéo image, gọi RDS, gọi dịch vụ khác)"
  ip_protocol       = "-1"
  cidr_ipv4         = "0.0.0.0/0"
}

resource "aws_vpc_security_group_ingress_rule" "from_cidr" {
  for_each = toset(var.ingress_cidr_blocks)

  security_group_id = aws_security_group.service.id
  description       = "Gọi vào ${var.service_name}"
  ip_protocol       = "tcp"
  from_port         = var.container_port
  to_port           = var.container_port
  cidr_ipv4         = each.value
}

resource "aws_ecs_task_definition" "this" {
  family                   = local.full_name
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = tostring(var.cpu)
  memory                   = tostring(var.memory)
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.task.arn

  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture        = "X86_64"
  }

  container_definitions = jsonencode([
    {
      name      = var.service_name
      image     = var.image
      essential = true
      portMappings = [
        { containerPort = var.container_port, protocol = "tcp" }
      ]
      environment = [
        for k, v in var.environment : { name = k, value = v }
      ]
      secrets = local.secrets
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.this.name
          "awslogs-region"        = data.aws_region.current.region
          "awslogs-stream-prefix" = var.service_name
        }
      }
    }
  ])

  tags = {
    Service = var.service_name
  }
}

resource "aws_ecs_service" "this" {
  name            = local.full_name
  cluster         = var.cluster_arn
  task_definition = aws_ecs_task_definition.this.arn
  desired_count   = var.desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = var.subnet_ids
    security_groups  = [aws_security_group.service.id]
    assign_public_ip = false
  }

  deployment_circuit_breaker {
    enable   = true
    rollback = true
  }

  # Chỉ gắn vào ALB khi service có target group.
  dynamic "load_balancer" {
    for_each = var.target_group_arn == null ? [] : [var.target_group_arn]

    content {
      target_group_arn = load_balancer.value
      container_name   = var.service_name
      container_port   = var.container_port
    }
  }

  tags = {
    Service = var.service_name
  }
}
