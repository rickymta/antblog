resource "aws_appconfig_application" "quote" {
  name        = local.prefix
  description = "Cấu hình động cho service báo giá của Tới Nơi"
}

resource "aws_appconfig_environment" "this" {
  name           = var.env
  application_id = aws_appconfig_application.quote.id
  description    = "Environment ${var.env}"

  dynamic "monitor" {
    for_each = var.enable_alarm_rollback ? [1] : []

    content {
      alarm_arn      = aws_cloudwatch_metric_alarm.quote_5xx.arn
      alarm_role_arn = aws_iam_role.appconfig_monitor.arn
    }
  }
}

# Profile 1: feature flag. AppConfig tự kiểm cấu trúc và các ràng buộc của attribute.
resource "aws_appconfig_configuration_profile" "flags" {
  application_id = aws_appconfig_application.quote.id
  name           = "pricing-flags"
  description    = "Flag bật giá giờ cao điểm mới"
  location_uri   = "hosted"
  type           = "AWS.AppConfig.FeatureFlags"
}

resource "aws_appconfig_hosted_configuration_version" "flags" {
  application_id           = aws_appconfig_application.quote.id
  configuration_profile_id = aws_appconfig_configuration_profile.flags.configuration_profile_id
  description              = "pricing-flags: peak-pricing-v2 cho 5% khách ở một thành phố"
  content_type             = "application/json"
  content                  = file("${path.module}/flags/pricing-flags.json")
}

# Profile 2: cấu hình tự do cho bảng giá, validator JSON Schema chặn giá âm.
resource "aws_appconfig_configuration_profile" "pricing_table" {
  application_id = aws_appconfig_application.quote.id
  name           = "pricing-table"
  description    = "Bảng giá cơ bản theo loại phương tiện"
  location_uri   = "hosted"
  type           = "AWS.Freeform"

  validator {
    type    = "JSON_SCHEMA"
    content = file("${path.module}/config/pricing-table.schema.json")
  }
}

resource "aws_appconfig_hosted_configuration_version" "pricing_table" {
  application_id           = aws_appconfig_application.quote.id
  configuration_profile_id = aws_appconfig_configuration_profile.pricing_table.configuration_profile_id
  description              = "pricing-table minh hoạ"
  content_type             = "application/json"
  content                  = file("${path.module}/config/pricing-table.json")
}

# Chiến lược cho prod: tăng 10% mỗi bước trong 20 phút, rồi bake 10 phút.
resource "aws_appconfig_deployment_strategy" "linear" {
  name                           = "${local.prefix}-linear20-bake10"
  description                    = "Linear 10% trong 20 phút, bake 10 phút"
  deployment_duration_in_minutes = 20
  final_bake_time_in_minutes     = 10
  growth_factor                  = 10
  growth_type                    = "LINEAR"
  replicate_to                   = "NONE"
}

locals {
  strategy_id = var.fast_lab ? "AppConfig.AllAtOnce" : aws_appconfig_deployment_strategy.linear.id
}

resource "aws_appconfig_deployment" "flags" {
  application_id           = aws_appconfig_application.quote.id
  environment_id           = aws_appconfig_environment.this.environment_id
  configuration_profile_id = aws_appconfig_configuration_profile.flags.configuration_profile_id
  configuration_version    = aws_appconfig_hosted_configuration_version.flags.version_number
  deployment_strategy_id   = local.strategy_id
  description              = "Deploy pricing-flags"
}

# AppConfig chỉ cho một deployment chạy trên một environment tại một thời điểm. Provider tự thử lại khi gặp
# ConflictException (tối đa khoảng 30 phút), nên depends_on giữ thứ tự cố định: flags xong (kể cả bake) mới tới bảng giá.
# Với fast_lab = false (20 phút + bake 10 phút), deployment thứ hai có thể hết thời gian chờ; khi đó chạy lại apply.
resource "aws_appconfig_deployment" "pricing_table" {
  application_id           = aws_appconfig_application.quote.id
  environment_id           = aws_appconfig_environment.this.environment_id
  configuration_profile_id = aws_appconfig_configuration_profile.pricing_table.configuration_profile_id
  configuration_version    = aws_appconfig_hosted_configuration_version.pricing_table.version_number
  deployment_strategy_id   = local.strategy_id
  description              = "Deploy pricing-table"

  depends_on = [aws_appconfig_deployment.flags]
}
