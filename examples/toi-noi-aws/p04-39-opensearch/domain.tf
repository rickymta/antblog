# Managed domain trong VPC (mode = domain). Lab dùng 1 node; production 3 AZ + dedicated master.
resource "aws_opensearch_domain" "search" {
  count = local.is_domain ? 1 : 0

  # 2.19 có hỗ trợ dài nhất trong dòng 2.x; dòng 3.x (3.1 trở lên) cũng đã có, kiểm tương thích instance trước khi đổi.
  domain_name    = "${local.prefix}-search"
  engine_version = "OpenSearch_2.19"

  cluster_config {
    instance_type  = var.domain_instance_type
    instance_count = var.domain_instance_count
    # Production: dedicated_master_enabled = true, dedicated_master_count = 3,
    # zone_awareness_enabled = true với availability_zone_count = 3.
  }

  ebs_options {
    ebs_enabled = true
    volume_type = "gp3"
    volume_size = var.domain_volume_size_gb
  }

  vpc_options {
    subnet_ids         = var.vpc_subnet_ids
    security_group_ids = var.vpc_security_group_ids
  }

  encrypt_at_rest {
    enabled = true
  }

  node_to_node_encryption {
    enabled = true
  }

  domain_endpoint_options {
    enforce_https       = true
    tls_security_policy = "Policy-Min-TLS-1-2-2019-07"
  }

  # Fine-grained access control: master user là IAM principal, không dùng user/password nội bộ.
  advanced_security_options {
    enabled                        = true
    internal_user_database_enabled = false

    master_user_options {
      master_user_arn = var.principal_arn
    }
  }

  tags = {
    Service = "search"
  }
}
