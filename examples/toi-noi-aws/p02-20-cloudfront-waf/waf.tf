# Web ACL cho CloudFront phải ở us-east-1.
resource "aws_wafv2_web_acl" "edge" {
  provider    = aws.use1
  name        = "${local.prefix}-edge"
  description = "Lớp chắn biên của Tới Nơi: managed rule, chống brute force đăng nhập"
  scope       = "CLOUDFRONT"

  default_action {
    allow {}
  }

  custom_response_body {
    key          = "too-many-requests"
    content      = jsonencode({ error = "too_many_requests", message = "Thử lại sau vài phút." })
    content_type = "APPLICATION_JSON"
  }

  # Rate-based: chặn IP gọi /auth/login quá nhiều lần.
  rule {
    name     = "login-rate-limit"
    priority = 1

    action {
      block {
        custom_response {
          response_code            = 429
          custom_response_body_key = "too-many-requests"

          response_header {
            name  = "Retry-After"
            value = "300"
          }
        }
      }
    }

    statement {
      rate_based_statement {
        limit                 = var.login_rate_limit
        aggregate_key_type    = "IP"
        evaluation_window_sec = 300

        scope_down_statement {
          byte_match_statement {
            search_string         = "/auth/login"
            positional_constraint = "STARTS_WITH"

            field_to_match {
              uri_path {}
            }

            text_transformation {
              priority = 0
              type     = "NONE"
            }
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "login-rate-limit"
      sampled_requests_enabled   = true
    }
  }

  # Managed rule group. Khi mới bật nên đổi override_action thành count {} để quan sát trước khi chặn.
  rule {
    name     = "aws-common"
    priority = 10

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "aws-common"
      sampled_requests_enabled   = true
    }
  }

  rule {
    name     = "aws-known-bad-inputs"
    priority = 20

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesKnownBadInputsRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "aws-known-bad-inputs"
      sampled_requests_enabled   = true
    }
  }

  rule {
    name     = "aws-sqli"
    priority = 30

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesSQLiRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "aws-sqli"
      sampled_requests_enabled   = true
    }
  }

  # Geo match ở chế độ count: chỉ đo lưu lượng ngoài Việt Nam, chưa chặn.
  rule {
    name     = "outside-vn-count"
    priority = 40

    action {
      count {}
    }

    statement {
      not_statement {
        statement {
          geo_match_statement {
            country_codes = ["VN"]
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "outside-vn-count"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "${local.prefix}-edge"
    sampled_requests_enabled   = true
  }
}

# Log WAF vào S3: tên bucket bắt buộc bắt đầu bằng aws-waf-logs- và bucket phải cùng account với web ACL.
# Đặt ở us-east-1 cạnh web ACL cho gọn; WAF tự gắn bucket policy cho delivery.logs.amazonaws.com khi bật log.
resource "aws_s3_bucket" "waf_logs" {
  provider = aws.use1
  bucket   = var.waf_log_bucket_name
}

resource "aws_s3_bucket_public_access_block" "waf_logs" {
  provider                = aws.use1
  bucket                  = aws_s3_bucket.waf_logs.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_lifecycle_configuration" "waf_logs" {
  provider = aws.use1
  bucket   = aws_s3_bucket.waf_logs.id

  rule {
    id     = "expire-waf-logs"
    status = "Enabled"

    filter {}

    expiration {
      days = 30
    }
  }
}

resource "aws_wafv2_web_acl_logging_configuration" "edge" {
  provider                = aws.use1
  resource_arn            = aws_wafv2_web_acl.edge.arn
  log_destination_configs = [aws_s3_bucket.waf_logs.arn]

  # Không ghi token đăng nhập vào log.
  redacted_fields {
    single_header {
      name = "authorization"
    }
  }
}
