locals {
  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = "eks-security"
    Owner   = "platform"
  }

  account_id = "123456789012"

  # Mỗi service một ServiceAccount (trùng tên namespace), một role, và chỉ những quyền service đó cần.
  # Ảnh chứng từ nằm dưới prefix proof/ của bucket bài 06.
  services = {
    booking = [
      {
        sid       = "ProofPhotos"
        actions   = ["s3:PutObject", "s3:GetObject"]
        resources = ["arn:aws:s3:::${var.photos_bucket}/proof/*"]
      },
      {
        sid       = "TripEvents"
        actions   = ["sns:Publish"]
        resources = ["arn:aws:sns:${var.region}:${local.account_id}:toinoi-${var.env}-trip-events"]
      },
    ]
    dispatch = [
      {
        sid       = "DispatchJobs"
        actions   = ["sqs:ReceiveMessage", "sqs:DeleteMessage", "sqs:ChangeMessageVisibility", "sqs:GetQueueAttributes"]
        resources = ["arn:aws:sqs:${var.region}:${local.account_id}:toinoi-${var.env}-dispatch-jobs"]
      },
    ]
    tracking = [
      {
        sid       = "ReadProofPhotos"
        actions   = ["s3:GetObject"]
        resources = ["arn:aws:s3:::${var.photos_bucket}/proof/*"]
      },
    ]
  }
}

# Pod Identity: principal là pods.eks.amazonaws.com; TagSession cho phép gắn session tag
# (kubernetes-namespace, kubernetes-service-account...) để dùng làm điều kiện ABAC (bài 51).
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

# ---------- Role cho từng service ----------

resource "aws_iam_role" "service" {
  for_each           = local.services
  name               = "toinoi-${var.env}-${each.key}-pod"
  assume_role_policy = data.aws_iam_policy_document.pods_trust.json
}

data "aws_iam_policy_document" "service" {
  for_each = local.services

  dynamic "statement" {
    for_each = each.value

    content {
      sid       = statement.value.sid
      effect    = "Allow"
      actions   = statement.value.actions
      resources = statement.value.resources
    }
  }
}

resource "aws_iam_role_policy" "service" {
  for_each = local.services
  name     = "least-privilege"
  role     = aws_iam_role.service[each.key].id
  policy   = data.aws_iam_policy_document.service[each.key].json
}

resource "aws_eks_pod_identity_association" "service" {
  for_each        = local.services
  cluster_name    = var.cluster_name
  namespace       = each.key
  service_account = each.key
  role_arn        = aws_iam_role.service[each.key].arn
}

# ---------- Role cho External Secrets Operator ----------
# Một role đọc cho cả controller: ranh giới giữa namespace do ClusterSecretStore.conditions
# và policy Kyverno bảo đảm, không phải IAM. IAM chỉ giới hạn trong prefix toinoi/<env>/.

data "aws_iam_policy_document" "eso" {
  statement {
    sid     = "ReadSecrets"
    effect  = "Allow"
    actions = ["secretsmanager:GetSecretValue", "secretsmanager:DescribeSecret"]
    resources = [
      "arn:aws:secretsmanager:${var.region}:${local.account_id}:secret:toinoi/${var.env}/*",
    ]
  }

  statement {
    sid     = "ReadParameters"
    effect  = "Allow"
    actions = ["ssm:GetParameter", "ssm:GetParameters", "ssm:GetParametersByPath"]
    resources = [
      "arn:aws:ssm:${var.region}:${local.account_id}:parameter/toinoi/${var.env}/*",
    ]
  }
}

resource "aws_iam_role" "eso" {
  name               = "toinoi-${var.env}-external-secrets"
  assume_role_policy = data.aws_iam_policy_document.pods_trust.json
}

resource "aws_iam_role_policy" "eso" {
  name   = "read-toinoi-secrets"
  role   = aws_iam_role.eso.id
  policy = data.aws_iam_policy_document.eso.json
}

resource "aws_eks_pod_identity_association" "eso" {
  cluster_name    = var.cluster_name
  namespace       = "external-secrets"
  service_account = "external-secrets"
  role_arn        = aws_iam_role.eso.arn
}

# ---------- GuardDuty cho EKS (dùng detector đã có) ----------

resource "aws_guardduty_detector_feature" "eks_audit" {
  count       = var.guardduty_detector_id == "" ? 0 : 1
  detector_id = var.guardduty_detector_id
  name        = "EKS_AUDIT_LOGS"
  status      = "ENABLED"
}

# Runtime Monitoring (gộp EKS, ECS, EC2) thay cho EKS_RUNTIME_MONITORING cũ; không bật cả hai trên cùng detector.
resource "aws_guardduty_detector_feature" "runtime" {
  count       = var.guardduty_detector_id == "" ? 0 : 1
  detector_id = var.guardduty_detector_id
  name        = "RUNTIME_MONITORING"
  status      = "ENABLED"

  additional_configuration {
    name   = "EKS_ADDON_MANAGEMENT"
    status = "ENABLED"
  }
}
