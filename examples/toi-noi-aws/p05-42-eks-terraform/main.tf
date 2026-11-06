data "aws_caller_identity" "current" {}

locals {
  # Tiền tố tài nguyên toinoi-<env>; cluster đặt tên toinoi-<env>-eks.
  prefix = "toinoi-${var.env}"
  name   = "${local.prefix}-eks"

  # Mọi namespace theo bounded context mà role CI được sửa.
  ci_namespaces = ["booking", "dispatch"]

  subnet_tags = merge(
    { for id in var.public_subnet_ids : id => { "kubernetes.io/role/elb" = "1" } },
    { for id in var.private_subnet_ids : id => { "kubernetes.io/role/internal-elb" = "1" } },
  )
}

# Subnet do lab p03-23 tạo nên không có tag cho load balancer; aws_ec2_tag thêm tag mà không sở hữu subnet.
resource "aws_ec2_tag" "lb_role" {
  for_each = {
    for pair in flatten([
      for id, tags in local.subnet_tags : [
        for k, v in tags : { key = "${id}/${k}", id = id, tag = k, value = v }
      ]
    ]) : pair.key => pair
  }

  resource_id = each.value.id
  key         = each.value.tag
  value       = each.value.value
}

resource "aws_ec2_tag" "cluster_shared" {
  for_each = toset(concat(var.public_subnet_ids, var.private_subnet_ids))

  resource_id = each.value
  key         = "kubernetes.io/cluster/${local.name}"
  value       = "shared"
}

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 21.0"

  name               = local.name
  kubernetes_version = var.kubernetes_version

  vpc_id                   = var.vpc_id
  subnet_ids               = var.private_subnet_ids
  control_plane_subnet_ids = var.private_subnet_ids

  # Endpoint: node gọi qua đường private; người dùng gọi qua public nhưng chỉ từ dải IP cho phép.
  endpoint_private_access      = true
  endpoint_public_access       = true
  endpoint_public_access_cidrs = var.api_allowed_cidrs

  # Chỉ ghi log api và audit để giữ chi phí CloudWatch Logs thấp.
  enabled_log_types                      = ["api", "audit"]
  cloudwatch_log_group_retention_in_days = 14

  # Xác thực chỉ qua access entries, không dùng ConfigMap aws-auth. Người chạy terraform apply được cấp quyền admin.
  authentication_mode                      = "API"
  enable_cluster_creator_admin_permissions = true

  access_entries = {
    ci = {
      principal_arn = var.ci_role_arn
      policy_associations = {
        edit = {
          policy_arn = "arn:aws:eks::aws:cluster-access-policy/AmazonEKSEditPolicy"
          access_scope = {
            type       = "namespace"
            namespaces = local.ci_namespaces
          }
        }
      }
    }
    dev = {
      principal_arn = var.dev_role_arn
      policy_associations = {
        view = {
          policy_arn = "arn:aws:eks::aws:cluster-access-policy/AmazonEKSViewPolicy"
          access_scope = {
            type = "cluster"
          }
        }
      }
    }
  }

  addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    # Hai add-on này phải có trước khi node khởi động: CNI cấp IP cho pod, agent cấp credential cho pod.
    vpc-cni = {
      most_recent    = true
      before_compute = true
    }
    eks-pod-identity-agent = {
      most_recent    = true
      before_compute = true
    }
  }

  eks_managed_node_groups = {
    # Node on-demand ổn định cho add-on và service không chịu được gián đoạn.
    base = {
      ami_type       = "AL2023_ARM_64_STANDARD"
      instance_types = ["m7g.large"]
      capacity_type  = "ON_DEMAND"

      min_size     = 2
      max_size     = 3
      desired_size = var.base_node_desired_size

      labels = {
        "toinoi.example/capacity" = "on-demand"
      }

      update_config = {
        max_unavailable_percentage = 33
      }
    }

    # Node Spot: nhiều loại instance cùng cỡ để có nhiều vùng giá và ít bị thu hồi đồng loạt.
    spot = {
      ami_type       = "AL2023_ARM_64_STANDARD"
      instance_types = ["m7g.large", "m6g.large", "m7gd.large"]
      capacity_type  = "SPOT"

      min_size     = 0
      max_size     = 6
      desired_size = var.spot_node_desired_size

      labels = {
        "toinoi.example/capacity" = "spot"
      }

      # Chỉ pod khai tolerations mới chạy ở đây; service chịu gián đoạn mới được lên Spot.
      taints = {
        spot = {
          key    = "toinoi.example/capacity"
          value  = "spot"
          effect = "NO_SCHEDULE"
        }
      }

      update_config = {
        max_unavailable_percentage = 33
      }
    }
  }

  tags = {
    Service = "platform"
  }
}

# Pod Identity cho service booking: role trust principal pods.eks.amazonaws.com, quyền S3 giới hạn proof/*.
data "aws_iam_policy_document" "pod_identity_trust" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole", "sts:TagSession"]

    principals {
      type        = "Service"
      identifiers = ["pods.eks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "booking" {
  name               = "${local.prefix}-booking-pod"
  assume_role_policy = data.aws_iam_policy_document.pod_identity_trust.json

  tags = {
    Service = "booking"
  }
}

data "aws_iam_policy_document" "booking_s3" {
  statement {
    sid       = "ProofObjects"
    effect    = "Allow"
    actions   = ["s3:PutObject", "s3:GetObject"]
    resources = ["arn:aws:s3:::${var.proof_bucket_name}/proof/*"]
  }
}

resource "aws_iam_role_policy" "booking_s3" {
  name   = "proof-photos"
  role   = aws_iam_role.booking.id
  policy = data.aws_iam_policy_document.booking_s3.json
}

resource "aws_eks_pod_identity_association" "booking" {
  cluster_name    = module.eks.cluster_name
  namespace       = "booking"
  service_account = "booking"
  role_arn        = aws_iam_role.booking.arn

  tags = {
    Service = "booking"
  }

  depends_on = [module.eks]
}
