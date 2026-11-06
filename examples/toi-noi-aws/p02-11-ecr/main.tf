locals {
  repository_name = "toinoi/${var.service}"
}

# Repository riêng cho service đặt chuyến.
# IMMUTABLE: không cho đẩy đè một tag đã tồn tại.
# scan_on_push: quét basic khi push (xem README về scan ở cấp registry).
resource "aws_ecr_repository" "service" {
  name                 = local.repository_name
  image_tag_mutability = "IMMUTABLE"
  force_delete         = true # chỉ cho lab: cho phép destroy dù còn image

  image_scanning_configuration {
    scan_on_push = true
  }

  encryption_configuration {
    encryption_type = "AES256"
  }

  tags = {
    Service = var.service
  }
}

# Lifecycle: xoá untagged sau 7 ngày, giữ 10 image mới nhất.
resource "aws_ecr_lifecycle_policy" "service" {
  repository = aws_ecr_repository.service.name
  policy     = file("${path.module}/lifecycle-policy.json")
}

# Cho mọi tài khoản trong Organization pull (không cho push).
data "aws_iam_policy_document" "org_pull" {
  statement {
    sid    = "AllowOrgPull"
    effect = "Allow"

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    actions = [
      "ecr:BatchCheckLayerAvailability",
      "ecr:BatchGetImage",
      "ecr:GetDownloadUrlForLayer",
    ]

    condition {
      test     = "StringEquals"
      variable = "aws:PrincipalOrgID"
      values   = [var.org_id]
    }
  }
}

resource "aws_ecr_repository_policy" "org_pull" {
  repository = aws_ecr_repository.service.name
  policy     = data.aws_iam_policy_document.org_pull.json
}

# Pull-through cache: image public.ecr.aws/<x> được kéo qua ecr-public/<x> trong registry của ta.
resource "aws_ecr_pull_through_cache_rule" "ecr_public" {
  count = var.enable_pull_through_cache ? 1 : 0

  ecr_repository_prefix = "ecr-public"
  upstream_registry_url = "public.ecr.aws"
}
