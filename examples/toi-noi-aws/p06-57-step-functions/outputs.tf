output "saga_state_machine_arn" {
  description = "ARN state machine saga đặt chuyến."
  value       = aws_sfn_state_machine.saga.arn
}

output "saga_alias_arn" {
  description = "ARN alias live; truyền cho start-execution để luôn dùng phiên bản đã chọn."
  value       = aws_sfn_alias.saga_live.arn
}

output "recon_state_machine_arn" {
  description = "ARN state machine đối soát đêm."
  value       = aws_sfn_state_machine.recon.arn
}
