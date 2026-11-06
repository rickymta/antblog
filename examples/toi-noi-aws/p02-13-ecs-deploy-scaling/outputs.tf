output "alb_dns_name" {
  description = "DNS của ALB; thử curl http://<giá trị>/health/ready."
  value       = aws_lb.main.dns_name
}

output "service_name" {
  value = aws_ecs_service.booking.name
}

output "scaling_resource_id" {
  value = aws_appautoscaling_target.booking.resource_id
}
