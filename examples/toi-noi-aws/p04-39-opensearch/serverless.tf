locals {
  prefix          = "toinoi-${var.env}"
  collection_name = "${local.prefix}-search"
  is_serverless   = var.mode == "serverless"
  is_domain       = var.mode == "domain"
}

# OpenSearch Serverless cần ba chính sách trước khi tạo collection: mã hoá, mạng, quyền dữ liệu.
resource "aws_opensearchserverless_security_policy" "encryption" {
  count = local.is_serverless ? 1 : 0

  name = "${local.collection_name}-enc"
  type = "encryption"

  policy = jsonencode({
    Rules = [
      {
        ResourceType = "collection"
        Resource     = ["collection/${local.collection_name}"]
      }
    ]
    AWSOwnedKey = true
  })
}

# Lab cho phép truy cập qua Internet nhưng vẫn bắt buộc ký SigV4 và quyền dữ liệu bên dưới.
# Production nên dùng VPC endpoint (aws_opensearchserverless_vpc_endpoint) và đặt AllowFromPublic = false.
resource "aws_opensearchserverless_security_policy" "network" {
  count = local.is_serverless ? 1 : 0

  name = "${local.collection_name}-net"
  type = "network"

  policy = jsonencode([
    {
      Rules = [
        {
          ResourceType = "collection"
          Resource     = ["collection/${local.collection_name}"]
        }
      ]
      AllowFromPublic = true
    }
  ])
}

resource "aws_opensearchserverless_access_policy" "data" {
  count = local.is_serverless ? 1 : 0

  name = "${local.collection_name}-data"
  type = "data"

  policy = jsonencode([
    {
      Rules = [
        {
          ResourceType = "collection"
          Resource     = ["collection/${local.collection_name}"]
          # Create/Update cần cho POST _aliases và index template (hoán đổi alias, bài 39).
          Permission = [
            "aoss:DescribeCollectionItems",
            "aoss:CreateCollectionItems",
            "aoss:UpdateCollectionItems",
          ]
        },
        {
          ResourceType = "index"
          Resource     = ["index/${local.collection_name}/*"]
          Permission = [
            "aoss:CreateIndex",
            "aoss:UpdateIndex",
            "aoss:DescribeIndex",
            "aoss:ReadDocument",
            "aoss:WriteDocument",
          ]
        }
      ]
      Principal = [var.principal_arn]
    }
  ])
}

resource "aws_opensearchserverless_collection" "search" {
  count = local.is_serverless ? 1 : 0

  name             = local.collection_name
  type             = "SEARCH"
  description      = "Tìm quán và địa chỉ của Tới Nơi (lab bài 39)"
  standby_replicas = "DISABLED"

  depends_on = [
    aws_opensearchserverless_security_policy.encryption,
    aws_opensearchserverless_security_policy.network,
  ]

  tags = {
    Service = "search"
  }
}
