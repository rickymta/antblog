# ---------------------------------------------------------------------------
# AWS Config: recorder, delivery channel, rule managed và remediation SSM Automation
# ---------------------------------------------------------------------------
data "aws_iam_policy_document" "config_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["config.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "config" {
  count = var.enable_config ? 1 : 0

  name               = "toinoi-${var.env}-config-recorder"
  assume_role_policy = data.aws_iam_policy_document.config_assume.json

  tags = {
    Service = "security"
  }
}

resource "aws_iam_role_policy_attachment" "config" {
  count = var.enable_config ? 1 : 0

  role       = aws_iam_role.config[0].name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWS_ConfigRole"
}

resource "aws_config_configuration_recorder" "main" {
  count = var.enable_config ? 1 : 0

  name     = "default"
  role_arn = aws_iam_role.config[0].arn

  recording_group {
    all_supported = true
  }
}

resource "aws_config_delivery_channel" "main" {
  count = var.enable_config ? 1 : 0

  name           = "default"
  s3_bucket_name = aws_s3_bucket.logs.id
  s3_key_prefix  = "config"

  depends_on = [aws_config_configuration_recorder.main, aws_s3_bucket_policy.logs]
}

resource "aws_config_configuration_recorder_status" "main" {
  count = var.enable_config ? 1 : 0

  name       = aws_config_configuration_recorder.main[0].name
  is_enabled = true

  depends_on = [aws_config_delivery_channel.main]
}

locals {
  config_rules = {
    s3-bucket-public-read-prohibited   = "S3_BUCKET_PUBLIC_READ_PROHIBITED"
    s3-bucket-public-write-prohibited  = "S3_BUCKET_PUBLIC_WRITE_PROHIBITED"
    rds-storage-encrypted              = "RDS_STORAGE_ENCRYPTED"
    encrypted-volumes                  = "ENCRYPTED_VOLUMES"
    vpc-flow-logs-enabled              = "VPC_FLOW_LOGS_ENABLED"
    cloud-trail-enabled                = "CLOUD_TRAIL_ENABLED"
    cloudtrail-s3-dataevents-enabled   = "CLOUDTRAIL_S3_DATAEVENTS_ENABLED"
    root-account-mfa-enabled           = "ROOT_ACCOUNT_MFA_ENABLED"
    kms-cmk-not-scheduled-for-deletion = "KMS_CMK_NOT_SCHEDULED_FOR_DELETION"
  }
}

resource "aws_config_config_rule" "managed" {
  for_each = var.enable_config ? local.config_rules : {}

  name = "toinoi-${var.env}-${each.key}"

  source {
    owner             = "AWS"
    source_identifier = each.value
  }

  tags = {
    Service = "security"
  }

  depends_on = [aws_config_configuration_recorder_status.main]
}

# Remediation: đóng bucket public bằng SSM Automation. automatic = false: người duyệt bấm chạy,
# không để tự sửa tài nguyên production khi chưa hiểu tác động.
data "aws_iam_policy_document" "remediation_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ssm.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "remediation_s3" {
  statement {
    effect = "Allow"
    actions = [
      "s3:GetBucketAcl",
      "s3:PutBucketAcl",
      "s3:GetBucketPublicAccessBlock",
      "s3:PutBucketPublicAccessBlock",
    ]
    resources = ["arn:aws:s3:::toinoi-*"]
  }
}

resource "aws_iam_role" "remediation" {
  count = var.enable_config ? 1 : 0

  name               = "toinoi-${var.env}-config-remediation"
  assume_role_policy = data.aws_iam_policy_document.remediation_assume.json

  tags = {
    Service = "security"
  }
}

resource "aws_iam_role_policy" "remediation" {
  count = var.enable_config ? 1 : 0

  name   = "close-public-buckets"
  role   = aws_iam_role.remediation[0].id
  policy = data.aws_iam_policy_document.remediation_s3.json
}

resource "aws_config_remediation_configuration" "s3_public_read" {
  count = var.enable_config ? 1 : 0

  config_rule_name = aws_config_config_rule.managed["s3-bucket-public-read-prohibited"].name
  resource_type    = "AWS::S3::Bucket"
  target_type      = "SSM_DOCUMENT"
  target_id        = "AWS-DisableS3BucketPublicReadWrite"
  target_version   = "1"
  automatic        = false

  parameter {
    name         = "AutomationAssumeRole"
    static_value = aws_iam_role.remediation[0].arn
  }

  parameter {
    name           = "S3BucketName"
    resource_value = "RESOURCE_ID"
  }
}
