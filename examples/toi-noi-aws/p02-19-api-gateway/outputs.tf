output "http_api_endpoint" {
  value = aws_apigatewayv2_api.http.api_endpoint
}

output "websocket_url" {
  value = "${aws_apigatewayv2_api.ws.api_endpoint}/${aws_apigatewayv2_stage.ws.name}"
}

output "websocket_management_endpoint" {
  description = "Endpoint cho aws apigatewaymanagementapi post-to-connection."
  value       = "https://${aws_apigatewayv2_api.ws.id}.execute-api.${var.region}.amazonaws.com/${aws_apigatewayv2_stage.ws.name}"
}

output "connections_table" {
  value = aws_dynamodb_table.connections.name
}
