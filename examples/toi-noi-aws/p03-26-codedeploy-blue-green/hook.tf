data "archive_file" "smoke" {
  type        = "zip"
  source_file = "${path.module}/hook/smoke_test.py"
  output_path = "${path.module}/.build/smoke_test.zip"
}

resource "aws_lambda_function" "smoke" {
  function_name    = "${local.name}-smoke"
  description      = "Hook AfterAllowTestTraffic: gọi test listener và báo Succeeded/Failed cho CodeDeploy"
  role             = aws_iam_role.smoke.arn
  runtime          = "python3.13"
  handler          = "smoke_test.handler"
  filename         = data.archive_file.smoke.output_path
  source_code_hash = data.archive_file.smoke.output_base64sha256
  timeout          = 30

  environment {
    variables = {
      TEST_URL = var.smoke_test_url
    }
  }
}
