data "aws_iam_policy_document" "dms_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["dms.${var.region}.amazonaws.com"]
    }
  }
}

# Role cho DMS đọc hai secret (nguồn, đích) thay vì ghi mật khẩu vào Terraform.
resource "aws_iam_role" "secrets_access" {
  name               = "toinoi-${var.env}-dms-secrets"
  assume_role_policy = data.aws_iam_policy_document.dms_trust.json
}

data "aws_iam_policy_document" "secrets_access" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [var.source_secret_arn, var.target_secret_arn]
  }
}

resource "aws_iam_role_policy" "secrets_access" {
  name   = "read-dms-secrets"
  role   = aws_iam_role.secrets_access.id
  policy = data.aws_iam_policy_document.secrets_access.json
}

# Hai role mức tài khoản mà DMS yêu cầu theo tên cố định; trust policy dùng principal toàn cục dms.amazonaws.com
# (role đọc secret ở trên dùng principal theo region như tài liệu Secrets Manager của DMS).
data "aws_iam_policy_document" "dms_service_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["dms.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "dms_vpc" {
  count = var.create_service_roles ? 1 : 0

  name               = "dms-vpc-role"
  assume_role_policy = data.aws_iam_policy_document.dms_service_trust.json
}

resource "aws_iam_role_policy_attachment" "dms_vpc" {
  count = var.create_service_roles ? 1 : 0

  role       = aws_iam_role.dms_vpc[0].name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonDMSVPCManagementRole"
}

resource "aws_iam_role" "dms_logs" {
  count = var.create_service_roles ? 1 : 0

  name               = "dms-cloudwatch-logs-role"
  assume_role_policy = data.aws_iam_policy_document.dms_service_trust.json
}

resource "aws_iam_role_policy_attachment" "dms_logs" {
  count = var.create_service_roles ? 1 : 0

  role       = aws_iam_role.dms_logs[0].name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonDMSCloudWatchLogsRole"
}
