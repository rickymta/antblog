# Role cho AWS Backup: một policy managed cho backup, một cho restore.
data "aws_iam_policy_document" "backup_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["backup.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "backup" {
  name               = "${local.prefix}-backup"
  assume_role_policy = data.aws_iam_policy_document.backup_trust.json
}

resource "aws_iam_role_policy_attachment" "backup" {
  role       = aws_iam_role.backup.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSBackupServiceRolePolicyForBackup"
}

resource "aws_iam_role_policy_attachment" "restore" {
  role       = aws_iam_role.backup.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSBackupServiceRolePolicyForRestores"
}

resource "aws_backup_plan" "toinoi" {
  name = "${local.prefix}-backup-plan"

  # Rule 1: backup mỗi đêm 00:00 giờ Việt Nam (17:00 UTC), copy sang region thứ hai.
  rule {
    rule_name         = "daily-with-dr-copy"
    target_vault_name = aws_backup_vault.primary.name
    schedule          = "cron(0 17 * * ? *)"
    start_window      = 60
    completion_window = 360

    lifecycle {
      delete_after = var.daily_retention_days
    }

    copy_action {
      destination_vault_arn = aws_backup_vault.dr.arn

      lifecycle {
        delete_after = var.dr_retention_days
      }
    }

    recovery_point_tags = {
      Project = "ToiNoi"
      Kind    = "daily"
    }
  }

  # Rule 2: continuous backup cho point-in-time restore (RDS, S3...). Giữ tối đa 35 ngày.
  rule {
    rule_name                = "continuous-pitr"
    target_vault_name        = aws_backup_vault.primary.name
    schedule                 = "cron(0 18 * * ? *)"
    enable_continuous_backup = true

    lifecycle {
      delete_after = 35
    }
  }
}

# Chọn tài nguyên theo tag: gắn Backup=daily vào RDS, DynamoDB, EFS... là vào kế hoạch.
resource "aws_backup_selection" "by_tag" {
  name         = "${local.prefix}-by-tag"
  iam_role_arn = aws_iam_role.backup.arn
  plan_id      = aws_backup_plan.toinoi.id

  selection_tag {
    type  = "STRINGEQUALS"
    key   = var.backup_tag_key
    value = var.backup_tag_value
  }
}
