locals {
  name = "toinoi-${var.env}-${var.service}"

  # Khung cao điểm của Tới Nơi theo giờ Việt Nam: 11:00-13:00 và 17:00-19:00.
  # Tăng trước giờ cao điểm 15 phút để task kịp khởi động, hạ sau giờ cao điểm 15 phút.
  schedules = {
    lunch-start  = { cron = "cron(45 10 * * ? *)", min = var.peak_min }
    lunch-end    = { cron = "cron(15 13 * * ? *)", min = var.capacity_min }
    dinner-start = { cron = "cron(45 16 * * ? *)", min = var.peak_min }
    dinner-end   = { cron = "cron(15 19 * * ? *)", min = var.capacity_min }
  }
}

# --- Security group: internet -> ALB:80 -> task:8080 ---
resource "aws_security_group" "alb" {
  name        = "${local.name}-alb"
  description = "ALB cua service booking"
  vpc_id      = var.vpc_id

  tags = {
    Name    = "${local.name}-alb"
    Service = var.service
  }
}

resource "aws_security_group" "task" {
  name        = "${local.name}-alb-task"
  description = "Task booking nhan traffic tu ALB"
  vpc_id      = var.vpc_id

  tags = {
    Name    = "${local.name}-alb-task"
    Service = var.service
  }
}

# Lab dùng HTTP cổng 80. Với HTTPS và ACM, xem bài 09.
resource "aws_vpc_security_group_ingress_rule" "alb_http" {
  security_group_id = aws_security_group.alb.id
  ip_protocol       = "tcp"
  from_port         = 80
  to_port           = 80
  cidr_ipv4         = "0.0.0.0/0"
  description       = "HTTP tu Internet (chi cho lab)"
}

resource "aws_vpc_security_group_egress_rule" "alb_to_task" {
  security_group_id            = aws_security_group.alb.id
  ip_protocol                  = "tcp"
  from_port                    = 8080
  to_port                      = 8080
  referenced_security_group_id = aws_security_group.task.id
  description                  = "ALB goi task"
}

resource "aws_vpc_security_group_ingress_rule" "task_from_alb" {
  security_group_id            = aws_security_group.task.id
  ip_protocol                  = "tcp"
  from_port                    = 8080
  to_port                      = 8080
  referenced_security_group_id = aws_security_group.alb.id
  description                  = "Chi nhan tu ALB"
}

resource "aws_vpc_security_group_egress_rule" "task_all" {
  security_group_id = aws_security_group.task.id
  ip_protocol       = "-1"
  cidr_ipv4         = "0.0.0.0/0"
  description       = "Ra ECR, Secrets Manager, CloudWatch Logs, RDS"
}

# --- ALB, target group kiểu ip, listener ---
resource "aws_lb" "main" {
  name               = "toinoi-${var.env}-alb"
  load_balancer_type = "application"
  internal           = false
  security_groups    = [aws_security_group.alb.id]
  subnets            = var.public_subnet_ids

  tags = {
    Service = var.service
  }
}

resource "aws_lb_target_group" "booking" {
  name        = local.name
  port        = 8080
  protocol    = "HTTP"
  vpc_id      = var.vpc_id
  target_type = "ip" # bắt buộc với Fargate (awsvpc): đích là IP của ENI

  # Mặc định 300 giây làm mỗi bước deploy chậm; request của Tới Nơi ngắn nên 30 giây là đủ.
  deregistration_delay = 30

  health_check {
    path                = "/health/ready"
    matcher             = "200"
    interval            = 15
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = 3
  }

  tags = {
    Service = var.service
  }
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.main.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.booking.arn
  }
}

# Alarm lỗi 5xx do target trả về. Dùng làm điều kiện rollback của deployment.
resource "aws_cloudwatch_metric_alarm" "target_5xx" {
  alarm_name          = "${local.name}-target-5xx"
  namespace           = "AWS/ApplicationELB"
  metric_name         = "HTTPCode_Target_5XX_Count"
  statistic           = "Sum"
  period              = 60
  evaluation_periods  = 3
  threshold           = var.alarm_5xx_threshold
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    LoadBalancer = aws_lb.main.arn_suffix
    TargetGroup  = aws_lb_target_group.booking.arn_suffix
  }
}

# --- Service: rolling update, circuit breaker, alarm rollback ---
resource "aws_ecs_service" "booking" {
  name                               = "${local.name}-alb"
  cluster                            = var.cluster_name
  task_definition                    = var.task_definition_arn
  desired_count                      = var.desired_count
  launch_type                        = "FARGATE"
  health_check_grace_period_seconds  = 60
  deployment_minimum_healthy_percent = var.min_percent
  deployment_maximum_percent         = var.max_percent
  enable_execute_command             = true
  enable_ecs_managed_tags            = true
  propagate_tags                     = "SERVICE"

  deployment_circuit_breaker {
    enable   = true
    rollback = true
  }

  alarms {
    alarm_names = [aws_cloudwatch_metric_alarm.target_5xx.alarm_name]
    enable      = true
    rollback    = true
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.booking.arn
    container_name   = "booking"
    container_port   = 8080
  }

  network_configuration {
    subnets          = var.private_subnet_ids
    security_groups  = [aws_security_group.task.id]
    assign_public_ip = false
  }

  depends_on = [aws_lb_listener.http]

  # Auto scaling điều khiển desired_count; deploy bằng CLI đổi task_definition. Đừng để Terraform đảo ngược cả hai.
  lifecycle {
    ignore_changes = [desired_count, task_definition]
  }

  tags = {
    Service = var.service
  }
}

# --- Application Auto Scaling ---
resource "aws_appautoscaling_target" "booking" {
  service_namespace  = "ecs"
  scalable_dimension = "ecs:service:DesiredCount"
  resource_id        = "service/${var.cluster_name}/${aws_ecs_service.booking.name}"
  min_capacity       = var.capacity_min
  max_capacity       = var.capacity_max
}

resource "aws_appautoscaling_policy" "requests" {
  name               = "${local.name}-requests-per-target"
  policy_type        = "TargetTrackingScaling"
  service_namespace  = aws_appautoscaling_target.booking.service_namespace
  scalable_dimension = aws_appautoscaling_target.booking.scalable_dimension
  resource_id        = aws_appautoscaling_target.booking.resource_id

  target_tracking_scaling_policy_configuration {
    target_value       = var.requests_per_target
    scale_out_cooldown = 60
    scale_in_cooldown  = 300

    predefined_metric_specification {
      predefined_metric_type = "ALBRequestCountPerTarget"
      resource_label         = "${aws_lb.main.arn_suffix}/${aws_lb_target_group.booking.arn_suffix}"
    }
  }
}

# Scheduled action đổi min/max của scalable target; target tracking vẫn chạy trong khoảng đó.
resource "aws_appautoscaling_scheduled_action" "peak" {
  for_each = local.schedules

  name               = "${local.name}-${each.key}"
  service_namespace  = aws_appautoscaling_target.booking.service_namespace
  scalable_dimension = aws_appautoscaling_target.booking.scalable_dimension
  resource_id        = aws_appautoscaling_target.booking.resource_id
  schedule           = each.value.cron
  timezone           = "Asia/Ho_Chi_Minh"

  scalable_target_action {
    min_capacity = each.value.min
    max_capacity = var.capacity_max
  }
}
