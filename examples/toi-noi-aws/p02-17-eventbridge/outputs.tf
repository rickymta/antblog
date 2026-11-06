output "event_bus_name" {
  value = aws_cloudwatch_event_bus.trips.name
}

output "event_bus_arn" {
  value = aws_cloudwatch_event_bus.trips.arn
}

output "archive_arn" {
  value = aws_cloudwatch_event_archive.trips.arn
}

output "settlement_queue_url" {
  value = aws_sqs_queue.settlement.url
}

output "rule_dlq_url" {
  value = aws_sqs_queue.rule_dlq.url
}
