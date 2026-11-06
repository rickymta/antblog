locals {
  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = "rollouts-lattice"
    Owner   = "platform"
  }
}

# ---------- VPC Lattice: service network và liên kết VPC ----------

resource "aws_vpclattice_service_network" "this" {
  name      = "toinoi-${var.env}"
  auth_type = "NONE"
}

resource "aws_vpclattice_service_network_vpc_association" "this" {
  vpc_identifier             = var.vpc_id
  service_network_identifier = aws_vpclattice_service_network.this.id
  security_group_ids         = var.security_group_ids
}

# ---------- Service fraud (Lambda) qua Lattice, bắt buộc ký SigV4 ----------

resource "aws_vpclattice_target_group" "fraud" {
  name = "toinoi-${var.env}-fraud"
  type = "LAMBDA"
}

resource "aws_vpclattice_target_group_attachment" "fraud" {
  target_group_identifier = aws_vpclattice_target_group.fraud.id

  target {
    id = var.fraud_lambda_arn
  }
}

resource "aws_lambda_permission" "lattice_invoke" {
  statement_id  = "AllowVpcLatticeInvoke"
  action        = "lambda:InvokeFunction"
  function_name = var.fraud_lambda_arn
  principal     = "vpc-lattice.amazonaws.com"
  source_arn    = aws_vpclattice_target_group.fraud.arn
}

resource "aws_vpclattice_service" "fraud" {
  name      = "toinoi-${var.env}-fraud"
  auth_type = "AWS_IAM"
}

resource "aws_vpclattice_listener" "fraud" {
  name               = "https"
  protocol           = "HTTPS"
  port               = 443
  service_identifier = aws_vpclattice_service.fraud.id

  default_action {
    forward {
      target_groups {
        target_group_identifier = aws_vpclattice_target_group.fraud.id
        weight                  = 100
      }
    }
  }
}

resource "aws_vpclattice_service_network_service_association" "fraud" {
  service_identifier         = aws_vpclattice_service.fraud.id
  service_network_identifier = aws_vpclattice_service_network.this.id
}

data "aws_iam_policy_document" "fraud_auth" {
  statement {
    sid       = "AllowListedCallers"
    effect    = "Allow"
    actions   = ["vpc-lattice-svcs:Invoke"]
    resources = ["*"]

    principals {
      type        = "AWS"
      identifiers = var.caller_role_arns
    }
  }
}

resource "aws_vpclattice_auth_policy" "fraud" {
  resource_identifier = aws_vpclattice_service.fraud.arn
  policy              = data.aws_iam_policy_document.fraud_auth.json
}

# ---------- Pod Identity cho controller Argo Rollouts đọc AMP và CloudWatch ----------

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

data "aws_iam_policy_document" "rollouts" {
  statement {
    sid    = "QueryPrometheus"
    effect = "Allow"
    actions = [
      "aps:QueryMetrics",
      "aps:GetSeries",
      "aps:GetLabels",
      "aps:GetMetricMetadata",
    ]
    resources = [var.amp_workspace_arn]
  }

  statement {
    sid       = "ReadCloudWatchMetrics"
    effect    = "Allow"
    actions   = ["cloudwatch:GetMetricData"]
    resources = ["*"]
  }
}

resource "aws_iam_role" "rollouts" {
  name               = "toinoi-${var.env}-argo-rollouts"
  assume_role_policy = data.aws_iam_policy_document.pods_trust.json
}

resource "aws_iam_role_policy" "rollouts" {
  name   = "analysis-read"
  role   = aws_iam_role.rollouts.id
  policy = data.aws_iam_policy_document.rollouts.json
}

resource "aws_eks_pod_identity_association" "rollouts" {
  cluster_name    = var.cluster_name
  namespace       = "argo-rollouts"
  service_account = "argo-rollouts"
  role_arn        = aws_iam_role.rollouts.arn
}
