locals {
  prefix = "toinoi-${var.env}"
}

data "aws_iam_policy_document" "lambda_trust" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

# ---------- HTTP API: GET /v1/quotes (JWT) và GET /health (công khai) ----------

data "archive_file" "quote" {
  type        = "zip"
  source_dir  = "${path.module}/src/quote"
  output_path = "${path.module}/build/quote.zip"
}

resource "aws_iam_role" "quote" {
  name               = "${local.prefix}-quote-fn"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust.json
}

resource "aws_iam_role_policy_attachment" "quote_logs" {
  role       = aws_iam_role.quote.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_function" "quote" {
  function_name    = "${local.prefix}-quote"
  role             = aws_iam_role.quote.arn
  runtime          = "nodejs22.x"
  architectures    = ["arm64"]
  handler          = "index.handler"
  filename         = data.archive_file.quote.output_path
  source_code_hash = data.archive_file.quote.output_base64sha256
  timeout          = 5
  memory_size      = 256
}

resource "aws_cloudwatch_log_group" "http_access" {
  name              = "/aws/apigateway/${local.prefix}-http-access"
  retention_in_days = 14
}

resource "aws_apigatewayv2_api" "http" {
  name          = "${local.prefix}-http"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = var.cors_allow_origins
    allow_methods = ["GET", "OPTIONS"]
    allow_headers = ["authorization", "content-type"]
    max_age       = 600
  }
}

resource "aws_apigatewayv2_integration" "quote" {
  api_id                 = aws_apigatewayv2_api.http.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.quote.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_authorizer" "jwt" {
  api_id           = aws_apigatewayv2_api.http.id
  name             = "toinoi-jwt"
  authorizer_type  = "JWT"
  identity_sources = ["$request.header.Authorization"]

  jwt_configuration {
    issuer   = var.jwt_issuer
    audience = var.jwt_audience
  }
}

resource "aws_apigatewayv2_route" "quotes" {
  api_id             = aws_apigatewayv2_api.http.id
  route_key          = "GET /v1/quotes"
  target             = "integrations/${aws_apigatewayv2_integration.quote.id}"
  authorization_type = "JWT"
  authorizer_id      = aws_apigatewayv2_authorizer.jwt.id
}

resource "aws_apigatewayv2_route" "health" {
  api_id             = aws_apigatewayv2_api.http.id
  route_key          = "GET /health"
  target             = "integrations/${aws_apigatewayv2_integration.quote.id}"
  authorization_type = "NONE"
}

resource "aws_apigatewayv2_stage" "http_default" {
  api_id      = aws_apigatewayv2_api.http.id
  name        = "$default"
  auto_deploy = true

  # Trần chung cho cả stage; route quan trọng có trần riêng bên dưới.
  default_route_settings {
    throttling_rate_limit  = 100
    throttling_burst_limit = 200
  }

  route_settings {
    route_key              = aws_apigatewayv2_route.quotes.route_key
    throttling_rate_limit  = var.route_rate_limit
    throttling_burst_limit = var.route_burst_limit
  }

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.http_access.arn
    format = jsonencode({
      requestId          = "$context.requestId"
      ip                 = "$context.identity.sourceIp"
      requestTime        = "$context.requestTime"
      httpMethod         = "$context.httpMethod"
      routeKey           = "$context.routeKey"
      status             = "$context.status"
      responseLatency    = "$context.responseLatency"
      integrationStatus  = "$context.integrationStatus"
      integrationLatency = "$context.integrationLatency"
      authorizerError    = "$context.authorizer.error"
    })
  }
}

resource "aws_lambda_permission" "http_invoke_quote" {
  statement_id  = "AllowHttpApiInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.quote.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.http.execution_arn}/*/*"
}

# Custom domain chỉ tạo khi có chứng chỉ ACM cùng region.
resource "aws_apigatewayv2_domain_name" "api" {
  count       = var.certificate_arn == "" ? 0 : 1
  domain_name = var.domain_name

  domain_name_configuration {
    certificate_arn = var.certificate_arn
    endpoint_type   = "REGIONAL"
    security_policy = "TLS_1_2"
  }
}

resource "aws_apigatewayv2_api_mapping" "api" {
  count       = var.certificate_arn == "" ? 0 : 1
  api_id      = aws_apigatewayv2_api.http.id
  domain_name = aws_apigatewayv2_domain_name.api[0].id
  stage       = aws_apigatewayv2_stage.http_default.id
}

# ---------- WebSocket API: khách xem vị trí tài xế ----------

resource "aws_dynamodb_table" "connections" {
  name         = "${local.prefix}-ws-connections"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "connectionId"

  attribute {
    name = "connectionId"
    type = "S"
  }

  # Kết nối bị rớt không gọi $disconnect vẫn tự biến mất nhờ TTL.
  ttl {
    attribute_name = "expiresAt"
    enabled        = true
  }
}

data "archive_file" "ws" {
  type        = "zip"
  source_dir  = "${path.module}/src/ws"
  output_path = "${path.module}/build/ws.zip"
}

resource "aws_iam_role" "ws" {
  name               = "${local.prefix}-ws-fn"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust.json
}

resource "aws_iam_role_policy_attachment" "ws_logs" {
  role       = aws_iam_role.ws.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

data "aws_iam_policy_document" "ws_table" {
  statement {
    effect    = "Allow"
    actions   = ["dynamodb:PutItem", "dynamodb:DeleteItem"]
    resources = [aws_dynamodb_table.connections.arn]
  }
}

resource "aws_iam_role_policy" "ws_table" {
  name   = "connections-table"
  role   = aws_iam_role.ws.id
  policy = data.aws_iam_policy_document.ws_table.json
}

resource "aws_lambda_function" "ws" {
  function_name    = "${local.prefix}-ws-handler"
  role             = aws_iam_role.ws.arn
  runtime          = "nodejs22.x"
  architectures    = ["arm64"]
  handler          = "index.handler"
  filename         = data.archive_file.ws.output_path
  source_code_hash = data.archive_file.ws.output_base64sha256
  timeout          = 5
  memory_size      = 256

  environment {
    variables = {
      TABLE_NAME = aws_dynamodb_table.connections.name
    }
  }
}

resource "aws_apigatewayv2_api" "ws" {
  name                       = "${local.prefix}-tracking-ws"
  protocol_type              = "WEBSOCKET"
  route_selection_expression = "$request.body.action"
}

resource "aws_apigatewayv2_integration" "ws" {
  api_id           = aws_apigatewayv2_api.ws.id
  integration_type = "AWS_PROXY"
  integration_uri  = aws_lambda_function.ws.invoke_arn
}

resource "aws_apigatewayv2_route" "ws_connect" {
  api_id    = aws_apigatewayv2_api.ws.id
  route_key = "$connect"
  target    = "integrations/${aws_apigatewayv2_integration.ws.id}"
}

resource "aws_apigatewayv2_route" "ws_disconnect" {
  api_id    = aws_apigatewayv2_api.ws.id
  route_key = "$disconnect"
  target    = "integrations/${aws_apigatewayv2_integration.ws.id}"
}

resource "aws_apigatewayv2_route" "ws_default" {
  api_id    = aws_apigatewayv2_api.ws.id
  route_key = "$default"
  target    = "integrations/${aws_apigatewayv2_integration.ws.id}"
}

resource "aws_apigatewayv2_stage" "ws" {
  api_id      = aws_apigatewayv2_api.ws.id
  name        = "lab"
  auto_deploy = true

  default_route_settings {
    throttling_rate_limit  = 100
    throttling_burst_limit = 200
  }
}

resource "aws_lambda_permission" "ws_invoke" {
  statement_id  = "AllowWebSocketApiInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.ws.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.ws.execution_arn}/*/*"
}
