output "service_network_arn" {
  description = "ARN service network của Tới Nơi."
  value       = aws_vpclattice_service_network.this.arn
}

output "fraud_service_dns" {
  description = "DNS do Lattice cấp cho service fraud (dùng với SigV4, service vpc-lattice-svcs)."
  value       = aws_vpclattice_service.fraud.dns_entry
}
