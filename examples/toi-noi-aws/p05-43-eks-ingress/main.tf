locals {
  # Cluster dựng ở lab p05-42 tên toinoi-<env>-eks; role đặt theo tiền tố toinoi-<env>.
  prefix       = "toinoi-${var.env}"
  cluster_name = "${local.prefix}-eks"
}

data "aws_eks_cluster" "this" {
  name = local.cluster_name
}

# Role của AWS Load Balancer Controller, nhận credential qua Pod Identity.
# ServiceAccount aws-load-balancer-controller trong kube-system trùng với tên chart tạo ra (helm/lbc-values.yaml).
module "lb_controller_pod_identity" {
  source  = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 2.0"

  name = "${local.prefix}-lb-controller"

  attach_aws_lb_controller_policy = true

  associations = {
    this = {
      cluster_name    = data.aws_eks_cluster.this.name
      namespace       = "kube-system"
      service_account = "aws-load-balancer-controller"
    }
  }

  tags = {
    Service = "platform"
  }
}

# Role của ExternalDNS: chỉ ghi record trong đúng hosted zone của toinoi.example.
module "external_dns_pod_identity" {
  source  = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 2.0"

  name = "${local.prefix}-external-dns"

  attach_external_dns_policy    = true
  external_dns_hosted_zone_arns = [var.hosted_zone_arn]

  associations = {
    this = {
      cluster_name    = data.aws_eks_cluster.this.name
      namespace       = "kube-system"
      service_account = "external-dns"
    }
  }

  tags = {
    Service = "platform"
  }
}
