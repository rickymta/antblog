output "application_id" {
  value = aws_appconfig_application.quote.id
}

output "environment_id" {
  value = aws_appconfig_environment.this.environment_id
}

output "flags_profile_id" {
  value = aws_appconfig_configuration_profile.flags.configuration_profile_id
}

output "pricing_table_profile_id" {
  value = aws_appconfig_configuration_profile.pricing_table.configuration_profile_id
}

output "reader_function" {
  description = "Tên Lambda đọc cấu hình, hoặc null nếu chưa cung cấp layer ARN."
  value       = local.create_reader ? aws_lambda_function.reader[0].function_name : null
}
