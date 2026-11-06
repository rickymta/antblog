# Lambda mẫu đọc cấu hình qua AppConfig Agent Lambda extension (localhost:2772).
# Chỉ tạo khi bạn cung cấp ARN layer của extension trong region.
locals {
  create_reader = var.appconfig_extension_layer_arn != ""
}

data "archive_file" "reader" {
  count       = local.create_reader ? 1 : 0
  type        = "zip"
  source_file = "${path.module}/reader/handler.py"
  output_path = "${path.module}/.build/reader.zip"
}

data "aws_iam_policy_document" "assume_lambda" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "reader" {
  # Extension gọi StartConfigurationSession rồi GetLatestConfiguration trên đúng profile.
  statement {
    actions = [
      "appconfig:StartConfigurationSession",
      "appconfig:GetLatestConfiguration",
    ]
    resources = [
      "arn:aws:appconfig:${var.region}:${data.aws_caller_identity.current.account_id}:application/${aws_appconfig_application.quote.id}/environment/${aws_appconfig_environment.this.environment_id}/configuration/${aws_appconfig_configuration_profile.flags.configuration_profile_id}",
      "arn:aws:appconfig:${var.region}:${data.aws_caller_identity.current.account_id}:application/${aws_appconfig_application.quote.id}/environment/${aws_appconfig_environment.this.environment_id}/configuration/${aws_appconfig_configuration_profile.pricing_table.configuration_profile_id}",
    ]
  }
}

resource "aws_iam_role" "reader" {
  count              = local.create_reader ? 1 : 0
  name               = "${local.prefix}-reader"
  assume_role_policy = data.aws_iam_policy_document.assume_lambda.json
}

resource "aws_iam_role_policy_attachment" "reader_logs" {
  count      = local.create_reader ? 1 : 0
  role       = aws_iam_role.reader[0].name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "reader" {
  count  = local.create_reader ? 1 : 0
  name   = "read-appconfig"
  role   = aws_iam_role.reader[0].id
  policy = data.aws_iam_policy_document.reader.json
}

resource "aws_lambda_function" "reader" {
  count            = local.create_reader ? 1 : 0
  function_name    = "${local.prefix}-reader"
  description      = "Báo giá mẫu đọc bảng giá và flag từ AppConfig"
  role             = aws_iam_role.reader[0].arn
  runtime          = "python3.13"
  handler          = "handler.handler"
  filename         = data.archive_file.reader[0].output_path
  source_code_hash = data.archive_file.reader[0].output_base64sha256
  timeout          = 10
  layers           = [var.appconfig_extension_layer_arn]

  environment {
    variables = {
      APPCONFIG_APPLICATION                         = aws_appconfig_application.quote.name
      APPCONFIG_ENVIRONMENT                         = aws_appconfig_environment.this.name
      AWS_APPCONFIG_EXTENSION_POLL_INTERVAL_SECONDS = "45"
    }
  }
}
