data "aws_caller_identity" "current" {}

# ---------------------------------------------------------------------------
# 1. Policy: ABAC theo Team, trần quyền cho role Lambda, quyền tạo role có điều kiện
# ---------------------------------------------------------------------------
resource "aws_iam_policy" "abac_team_lambda" {
  name        = "toinoi-${var.env}-abac-team-lambda"
  description = "Team chỉ chạm Lambda function có tag Team trùng tag của principal."
  policy      = file("${path.module}/policies/abac-team-lambda.json")
}

resource "aws_iam_policy" "lambda_dev_boundary" {
  name        = "toinoi-${var.env}-lambda-dev-boundary"
  description = "Permission boundary: trần quyền cho mọi role mà dev tự tạo cho Lambda."
  policy      = file("${path.module}/policies/lambda-dev-boundary.json")
}

resource "aws_iam_policy" "developer_create_role" {
  name        = "toinoi-${var.env}-developer-create-role"
  description = "Cho dev tạo role toinoi-lab-fn-* cho Lambda, chỉ khi gắn đúng boundary."
  policy      = file("${path.module}/policies/developer-create-role.json")
}

# ---------------------------------------------------------------------------
# 2. Role của team: tag Team trên role trở thành aws:PrincipalTag/Team
# ---------------------------------------------------------------------------
data "aws_iam_policy_document" "team_trust" {
  statement {
    sid     = "AssumeFromThisAccount"
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "AWS"
      identifiers = ["arn:aws:iam::${data.aws_caller_identity.current.account_id}:root"]
    }
  }
}

resource "aws_iam_role" "team" {
  for_each = var.teams

  name               = "toinoi-${var.env}-team-${each.key}"
  assume_role_policy = data.aws_iam_policy_document.team_trust.json

  # Tag này là nguồn của aws:PrincipalTag/Team trong policy ABAC.
  tags = {
    Team    = each.key
    Service = each.key
  }
}

resource "aws_iam_role_policy_attachment" "team_abac" {
  for_each = var.teams

  role       = aws_iam_role.team[each.key].name
  policy_arn = aws_iam_policy.abac_team_lambda.arn
}

resource "aws_iam_role_policy_attachment" "team_create_role" {
  for_each = var.teams

  role       = aws_iam_role.team[each.key].name
  policy_arn = aws_iam_policy.developer_create_role.arn
}

# ---------------------------------------------------------------------------
# 3. Role cho đối tác đối soát: external ID chống confused deputy
# ---------------------------------------------------------------------------
data "aws_iam_policy_document" "partner_trust" {
  statement {
    sid     = "PartnerAssumeWithExternalId"
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "AWS"
      identifiers = ["arn:aws:iam::${var.partner_account_id}:root"]
    }

    condition {
      test     = "StringEquals"
      variable = "sts:ExternalId"
      values   = [var.partner_external_id]
    }
  }
}

data "aws_iam_policy_document" "partner_read_reports" {
  statement {
    sid       = "ReadReconcileReports"
    effect    = "Allow"
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::toinoi-${var.env}-reconcile-reports/vixanh/*"]
  }
}

resource "aws_iam_role" "partner" {
  name                 = "toinoi-${var.env}-vixanh-reconcile"
  assume_role_policy   = data.aws_iam_policy_document.partner_trust.json
  max_session_duration = 3600

  tags = {
    Service = "reconcile"
  }
}

resource "aws_iam_role_policy" "partner_read_reports" {
  name   = "read-reconcile-reports"
  role   = aws_iam_role.partner.id
  policy = data.aws_iam_policy_document.partner_read_reports.json
}

# ---------------------------------------------------------------------------
# 4. IAM Access Analyzer: quyền ra ngoài tài khoản và quyền không dùng
# ---------------------------------------------------------------------------
resource "aws_accessanalyzer_analyzer" "external" {
  analyzer_name = "toinoi-${var.env}-external"
  type          = "ACCOUNT"
}

resource "aws_accessanalyzer_analyzer" "unused" {
  count = var.enable_unused_access_analyzer ? 1 : 0

  analyzer_name = "toinoi-${var.env}-unused"
  type          = "ACCOUNT_UNUSED_ACCESS"

  configuration {
    unused_access {
      unused_access_age = var.unused_access_age_days
    }
  }
}
