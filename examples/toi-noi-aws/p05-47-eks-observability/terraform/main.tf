locals {
  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = "observability"
    Owner   = "platform"
  }

  account_id = "123456789012"
}

# ---------- Amazon Managed Service for Prometheus ----------

resource "aws_prometheus_workspace" "this" {
  alias = "toinoi-${var.env}-metrics"
}

resource "aws_prometheus_rule_group_namespace" "booking" {
  name         = "booking-alerts"
  workspace_id = aws_prometheus_workspace.this.id
  data         = file("${path.module}/rules/booking-alerts.yaml")
}

# ---------- Cảnh báo: Alertmanager của AMP -> SNS ----------

resource "aws_sns_topic" "alerts" {
  name = "toinoi-${var.env}-eks-alerts"
}

resource "aws_sns_topic_subscription" "email" {
  count     = var.alert_email == "" ? 0 : 1
  topic_arn = aws_sns_topic.alerts.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

data "aws_iam_policy_document" "sns_from_amp" {
  statement {
    sid       = "AllowAmpAlertManagerPublish"
    effect    = "Allow"
    actions   = ["sns:Publish", "sns:GetTopicAttributes"]
    resources = [aws_sns_topic.alerts.arn]

    principals {
      type        = "Service"
      identifiers = ["aps.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [local.account_id]
    }

    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = [aws_prometheus_workspace.this.arn]
    }
  }
}

resource "aws_sns_topic_policy" "alerts" {
  arn    = aws_sns_topic.alerts.arn
  policy = data.aws_iam_policy_document.sns_from_amp.json
}

resource "aws_prometheus_alert_manager_definition" "this" {
  workspace_id = aws_prometheus_workspace.this.id
  definition   = <<-EOT
    alertmanager_config: |
      route:
        receiver: toinoi-sns
        group_by: [alertname, namespace]
        group_wait: 30s
        group_interval: 5m
        repeat_interval: 4h
      receivers:
        - name: toinoi-sns
          sns_configs:
            - topic_arn: ${aws_sns_topic.alerts.arn}
              sigv4:
                region: ${var.region}
              send_resolved: true
  EOT
}

# ---------- Pod Identity: ADOT collector ghi metric (AMP) và trace (X-Ray) ----------

data "aws_iam_policy_document" "pods_trust" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole", "sts:TagSession"]

    principals {
      type        = "Service"
      identifiers = ["pods.eks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "adot" {
  name               = "toinoi-${var.env}-adot-collector"
  assume_role_policy = data.aws_iam_policy_document.pods_trust.json
}

data "aws_iam_policy_document" "adot" {
  statement {
    sid       = "RemoteWriteToWorkspace"
    effect    = "Allow"
    actions   = ["aps:RemoteWrite"]
    resources = [aws_prometheus_workspace.this.arn]
  }
}

resource "aws_iam_role_policy" "adot" {
  name   = "amp-remote-write"
  role   = aws_iam_role.adot.id
  policy = data.aws_iam_policy_document.adot.json
}

resource "aws_iam_role_policy_attachment" "adot_xray" {
  role       = aws_iam_role.adot.name
  policy_arn = "arn:aws:iam::aws:policy/AWSXrayWriteOnlyAccess"
}

resource "aws_eks_pod_identity_association" "adot" {
  cluster_name    = var.cluster_name
  namespace       = "observability"
  service_account = "adot-collector"
  role_arn        = aws_iam_role.adot.arn
}

# ---------- Fluent Bit -> CloudWatch Logs, mỗi namespace một log group ----------

resource "aws_cloudwatch_log_group" "app" {
  for_each          = toset(var.namespaces)
  name              = "/toinoi/${var.env}/eks/${each.value}"
  retention_in_days = var.log_retention_days
}

data "aws_iam_policy_document" "fluent_bit" {
  statement {
    sid     = "WriteApplicationLogs"
    effect  = "Allow"
    actions = ["logs:CreateLogStream", "logs:PutLogEvents", "logs:DescribeLogStreams"]
    resources = flatten([
      for g in aws_cloudwatch_log_group.app : [g.arn, "${g.arn}:*"]
    ])
  }
}

resource "aws_iam_role" "fluent_bit" {
  name               = "toinoi-${var.env}-fluent-bit"
  assume_role_policy = data.aws_iam_policy_document.pods_trust.json
}

resource "aws_iam_role_policy" "fluent_bit" {
  name   = "write-app-logs"
  role   = aws_iam_role.fluent_bit.id
  policy = data.aws_iam_policy_document.fluent_bit.json
}

resource "aws_eks_pod_identity_association" "fluent_bit" {
  cluster_name    = var.cluster_name
  namespace       = "observability"
  service_account = "fluent-bit"
  role_arn        = aws_iam_role.fluent_bit.arn
}

# ---------- Tuỳ chọn: Container Insights enhanced observability ----------

resource "aws_iam_role" "cloudwatch_agent" {
  count              = var.enable_container_insights ? 1 : 0
  name               = "toinoi-${var.env}-cloudwatch-agent"
  assume_role_policy = data.aws_iam_policy_document.pods_trust.json
}

resource "aws_iam_role_policy_attachment" "cloudwatch_agent" {
  count      = var.enable_container_insights ? 1 : 0
  role       = aws_iam_role.cloudwatch_agent[0].name
  policy_arn = "arn:aws:iam::aws:policy/CloudWatchAgentServerPolicy"
}

resource "aws_eks_addon" "container_insights" {
  count        = var.enable_container_insights ? 1 : 0
  cluster_name = var.cluster_name
  addon_name   = "amazon-cloudwatch-observability"

  pod_identity_association {
    role_arn        = aws_iam_role.cloudwatch_agent[0].arn
    service_account = "cloudwatch-agent"
  }
}

# ---------- Tuỳ chọn: Amazon Managed Grafana (đăng nhập bằng IAM Identity Center) ----------

data "aws_iam_policy_document" "grafana_trust" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["grafana.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "grafana" {
  count              = var.create_grafana ? 1 : 0
  name               = "toinoi-${var.env}-grafana"
  assume_role_policy = data.aws_iam_policy_document.grafana_trust.json
}

data "aws_iam_policy_document" "grafana_read" {
  statement {
    sid    = "QueryPrometheus"
    effect = "Allow"
    actions = [
      "aps:QueryMetrics",
      "aps:GetSeries",
      "aps:GetLabels",
      "aps:GetMetricMetadata",
      "aps:ListWorkspaces",
      "aps:DescribeWorkspace",
    ]
    resources = ["*"]
  }

  statement {
    sid    = "ReadTraces"
    effect = "Allow"
    actions = [
      "xray:GetTraceSummaries",
      "xray:BatchGetTraces",
      "xray:GetServiceGraph",
    ]
    resources = ["*"]
  }

  statement {
    sid    = "ReadCloudWatch"
    effect = "Allow"
    actions = [
      "cloudwatch:GetMetricData",
      "cloudwatch:ListMetrics",
      "cloudwatch:DescribeAlarms",
      "logs:DescribeLogGroups",
      "logs:StartQuery",
      "logs:GetQueryResults",
    ]
    resources = ["*"]
  }
}

resource "aws_iam_role_policy" "grafana" {
  count  = var.create_grafana ? 1 : 0
  name   = "read-observability"
  role   = aws_iam_role.grafana[0].id
  policy = data.aws_iam_policy_document.grafana_read.json
}

resource "aws_grafana_workspace" "this" {
  count                    = var.create_grafana ? 1 : 0
  name                     = "toinoi-${var.env}"
  account_access_type      = "CURRENT_ACCOUNT"
  authentication_providers = ["AWS_SSO"]
  permission_type          = "CUSTOMER_MANAGED"
  role_arn                 = aws_iam_role.grafana[0].arn
  data_sources             = ["PROMETHEUS", "CLOUDWATCH", "XRAY"]
}
