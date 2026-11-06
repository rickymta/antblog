data "aws_caller_identity" "current" {}

locals {
  prefix = "toinoi-${var.env}"
}

# Vault chính nhận backup trong region chính. Dùng khoá mặc định của AWS Backup cho lab.
# Copy sang tài khoản khác: hai tài khoản cùng Organizations, vault đích có access policy backup:CopyIntoBackupVault;
# với RDS, EBS (không do AWS Backup quản lý trọn) vault đích cần khoá customer managed và khoá nguồn phải chia sẻ được.
resource "aws_backup_vault" "primary" {
  name = "${local.prefix}-primary"

  tags = {
    Service = "backup"
  }
}

# Vault ở region thứ hai nhận bản copy từ rule.
resource "aws_backup_vault" "dr" {
  provider = aws.dr
  name     = "${local.prefix}-dr"

  tags = {
    Service = "backup"
    Role    = "dr"
  }
}

# Vault Lock (WORM) cho vault chính. Mặc định tắt vì sau cooling-off không còn gỡ được.
resource "aws_backup_vault_lock_configuration" "primary" {
  count = var.enable_vault_lock ? 1 : 0

  backup_vault_name   = aws_backup_vault.primary.name
  changeable_for_days = var.vault_lock_changeable_for_days
  min_retention_days  = 7
  max_retention_days  = 365
}

# Cảnh báo job lỗi qua SNS.
resource "aws_sns_topic" "backup_events" {
  name = "${local.prefix}-backup-events"

  tags = {
    Service = "backup"
  }
}

data "aws_iam_policy_document" "backup_events" {
  statement {
    sid       = "AllowBackupToPublish"
    actions   = ["sns:Publish"]
    resources = [aws_sns_topic.backup_events.arn]

    principals {
      type        = "Service"
      identifiers = ["backup.amazonaws.com"]
    }
  }
}

resource "aws_sns_topic_policy" "backup_events" {
  arn    = aws_sns_topic.backup_events.arn
  policy = data.aws_iam_policy_document.backup_events.json
}

resource "aws_sns_topic_subscription" "email" {
  count = var.alert_email == "" ? 0 : 1

  topic_arn = aws_sns_topic.backup_events.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

resource "aws_backup_vault_notifications" "primary" {
  backup_vault_name   = aws_backup_vault.primary.name
  sns_topic_arn       = aws_sns_topic.backup_events.arn
  backup_vault_events = ["BACKUP_JOB_FAILED", "COPY_JOB_FAILED", "RESTORE_JOB_COMPLETED"]

  depends_on = [aws_sns_topic_policy.backup_events]
}
