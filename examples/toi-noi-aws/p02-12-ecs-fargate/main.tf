locals {
  name        = "toinoi-${var.env}-${var.service}"
  account_id  = data.aws_caller_identity.current.account_id
  secret_arns = [var.db_secret_arn]
}

# Cluster: nhóm logic của service và task, không có máy chủ nào để quản lý khi dùng Fargate.
resource "aws_ecs_cluster" "main" {
  name = "toinoi-${var.env}"
}

resource "aws_cloudwatch_log_group" "service" {
  name              = "/ecs/${local.name}"
  retention_in_days = 14
}

# Trust policy chuẩn của execution role theo tài liệu ECS: chỉ principal ecs-tasks.amazonaws.com.
data "aws_iam_policy_document" "ecs_execution_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

# Trust policy của task role: AWS khuyên thêm aws:SourceAccount và aws:SourceArn để chặn "confused deputy".
# aws:SourceArn chưa hỗ trợ khoá theo một cluster cụ thể, nên dùng wildcard toàn bộ ECS trong account/region.
data "aws_iam_policy_document" "ecs_tasks_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [local.account_id]
    }

    condition {
      test     = "ArnLike"
      variable = "aws:SourceArn"
      values   = ["arn:aws:ecs:${var.region}:${local.account_id}:*"]
    }
  }
}

# Execution role: quyền của ECS agent, dùng lúc khởi động task (pull image, ghi log, đọc secret).
resource "aws_iam_role" "execution" {
  name               = "${local.name}-execution"
  assume_role_policy = data.aws_iam_policy_document.ecs_execution_trust.json
}

resource "aws_iam_role_policy_attachment" "execution_managed" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

data "aws_iam_policy_document" "execution_secrets" {
  statement {
    sid       = "ReadBookingSecrets"
    actions   = ["secretsmanager:GetSecretValue"]
    resources = local.secret_arns
  }

  dynamic "statement" {
    for_each = var.kms_key_arn == null ? [] : [var.kms_key_arn]

    content {
      sid       = "DecryptBookingSecrets"
      actions   = ["kms:Decrypt"]
      resources = [statement.value]
    }
  }
}

resource "aws_iam_role_policy" "execution_secrets" {
  name   = "read-startup-secrets"
  role   = aws_iam_role.execution.id
  policy = data.aws_iam_policy_document.execution_secrets.json
}

# Task role: quyền runtime của ứng dụng (code gọi AWS API bằng role này).
resource "aws_iam_role" "task" {
  name               = "${local.name}-task"
  assume_role_policy = data.aws_iam_policy_document.ecs_tasks_trust.json
}

data "aws_iam_policy_document" "task" {
  statement {
    sid       = "ProofPhotosReadWrite"
    actions   = ["s3:PutObject", "s3:GetObject"]
    resources = ["arn:aws:s3:::${var.photos_bucket}/proof/*"]
  }

  # Cần cho ECS Exec (Session Manager). Chỉ giữ nếu bạn bật enable_execute_command.
  statement {
    sid = "EcsExecChannels"
    actions = [
      "ssmmessages:CreateControlChannel",
      "ssmmessages:CreateDataChannel",
      "ssmmessages:OpenControlChannel",
      "ssmmessages:OpenDataChannel",
    ]
    resources = ["*"]
  }
}

resource "aws_iam_role_policy" "task" {
  name   = "booking-runtime"
  role   = aws_iam_role.task.id
  policy = data.aws_iam_policy_document.task.json
}

# Mỗi task Fargate (awsvpc) có ENI riêng và gắn security group này.
# Không có ingress: bài 13 thêm ingress từ ALB. Hãy thêm 5432 từ security group này vào security group của RDS (bài 07).
resource "aws_security_group" "task" {
  name        = "${local.name}-task"
  description = "Task Fargate service booking"
  vpc_id      = var.vpc_id

  tags = {
    Name    = "${local.name}-task"
    Service = var.service
  }
}

resource "aws_vpc_security_group_egress_rule" "task_all" {
  security_group_id = aws_security_group.task.id
  ip_protocol       = "-1"
  cidr_ipv4         = "0.0.0.0/0"
  description       = "Ra ECR, Secrets Manager, CloudWatch Logs, RDS (qua NAT hoặc VPC endpoint)"
}

resource "aws_ecs_task_definition" "booking" {
  count = var.manage_workload ? 1 : 0

  family                   = local.name
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "512"
  memory                   = "1024"
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.task.arn

  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture        = "X86_64"
  }

  container_definitions = jsonencode([
    {
      name      = "booking"
      image     = var.image
      essential = true

      portMappings = [{ containerPort = 8080, protocol = "tcp" }]

      environment = [
        { name = "ASPNETCORE_ENVIRONMENT", value = "Production" },
        { name = "ASPNETCORE_HTTP_PORTS", value = "8080" },
        { name = "Auth__Authority", value = var.auth_authority },
        { name = "Auth__Audience", value = var.auth_audience },
      ]

      secrets = [
        { name = "ConnectionStrings__Trips", valueFrom = var.db_secret_arn },
      ]

      # Cần curl trong image. Image chiseled hoặc image không có curl phải bỏ khối này và dựa vào health check của ALB.
      healthCheck = {
        command     = ["CMD-SHELL", "curl -f http://localhost:8080/health/ready || exit 1"]
        interval    = 15
        timeout     = 5
        retries     = 3
        startPeriod = 30
      }

      stopTimeout = 30

      linuxParameters = { initProcessEnabled = true }

      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.service.name
          "awslogs-region"        = var.region
          "awslogs-stream-prefix" = "booking"
        }
      }
    }
  ])

  tags = {
    Service = var.service
  }
}

resource "aws_ecs_service" "booking" {
  count = var.manage_workload ? 1 : 0

  name                    = local.name
  cluster                 = aws_ecs_cluster.main.id
  task_definition         = aws_ecs_task_definition.booking[0].arn
  desired_count           = var.desired_count
  launch_type             = "FARGATE"
  enable_execute_command  = true
  enable_ecs_managed_tags = true
  propagate_tags          = "SERVICE"

  network_configuration {
    subnets          = var.private_subnet_ids
    security_groups  = [aws_security_group.task.id]
    assign_public_ip = false
  }

  tags = {
    Service = var.service
  }
}
