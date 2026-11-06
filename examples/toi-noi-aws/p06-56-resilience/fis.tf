# Alarm làm stop condition: nếu lỗi 5xx của ALB vượt ngưỡng, FIS dừng thí nghiệm.
resource "aws_cloudwatch_metric_alarm" "booking_5xx" {
  alarm_name          = "toinoi-${var.env}-booking-fis-stop-5xx"
  alarm_description   = "Stop condition của FIS: 5xx từ target của ALB booking quá ngưỡng."
  namespace           = "AWS/ApplicationELB"
  metric_name         = "HTTPCode_Target_5XX_Count"
  statistic           = "Sum"
  period              = 60
  evaluation_periods  = 1
  threshold           = var.error_alarm_threshold
  comparison_operator = "GreaterThanOrEqualToThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    LoadBalancer = var.alb_arn_suffix
  }
}

data "aws_iam_policy_document" "fis_assume" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["fis.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "fis_permissions" {
  statement {
    sid       = "EcsStopTask"
    actions   = ["ecs:ListTasks", "ecs:DescribeTasks", "ecs:StopTask"]
    resources = ["*"]

    condition {
      test     = "ArnEquals"
      variable = "ecs:cluster"
      values   = [var.ecs_cluster_arn]
    }
  }

  statement {
    sid = "NetworkDisrupt"
    actions = [
      "ec2:DescribeSubnets",
      "ec2:DescribeNetworkAcls",
      "ec2:CreateNetworkAcl",
      "ec2:CreateNetworkAclEntry",
      "ec2:DeleteNetworkAcl",
      "ec2:ReplaceNetworkAclAssociation",
      "ec2:CreateTags",
      "ec2:DeleteTags",
    ]
    resources = ["*"]
  }
}

resource "aws_iam_role" "fis" {
  name               = "toinoi-${var.env}-fis"
  assume_role_policy = data.aws_iam_policy_document.fis_assume.json
}

resource "aws_iam_role_policy" "fis" {
  name   = "toinoi-${var.env}-fis"
  role   = aws_iam_role.fis.id
  policy = data.aws_iam_policy_document.fis_permissions.json
}

resource "aws_fis_experiment_template" "booking_resilience" {
  description = "Game day: dừng một phần task booking và làm mất kết nối một AZ, có stop condition."
  role_arn    = aws_iam_role.fis.arn

  stop_condition {
    source = "aws:cloudwatch:alarm"
    value  = aws_cloudwatch_metric_alarm.booking_5xx.arn
  }

  action {
    name      = "stop-booking-tasks"
    action_id = "aws:ecs:stop-task"

    target {
      key   = "Tasks"
      value = "booking-tasks"
    }
  }

  action {
    name        = "disrupt-az-network"
    action_id   = "aws:network:disrupt-connectivity"
    start_after = ["stop-booking-tasks"]

    parameter {
      key   = "scope"
      value = "all"
    }

    parameter {
      key   = "duration"
      value = var.experiment_duration
    }

    target {
      key   = "Subnets"
      value = "private-subnets-in-az"
    }
  }

  target {
    name           = "booking-tasks"
    resource_type  = "aws:ecs:task"
    selection_mode = "PERCENT(${var.stop_task_percent})"

    parameters = {
      cluster = var.ecs_cluster_name
      service = var.ecs_service_name
    }
  }

  target {
    name           = "private-subnets-in-az"
    resource_type  = "aws:ec2:subnet"
    selection_mode = "ALL"

    resource_tag {
      key   = "Tier"
      value = var.private_subnet_tag_value
    }

    filter {
      path   = "AvailabilityZone"
      values = [var.target_az]
    }
  }

  tags = {
    Name    = "toinoi-${var.env}-booking-resilience"
    Service = "booking"
  }
}
