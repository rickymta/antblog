data "aws_caller_identity" "current" {}

locals {
  # Cluster dựng ở lab p05-42 tên toinoi-<env>-eks; role đặt theo tiền tố toinoi-<env>.
  prefix       = "toinoi-${var.env}"
  cluster_name = "${local.prefix}-eks"
}

data "aws_eks_cluster" "this" {
  name = local.cluster_name
}

# Karpenter tìm subnet và security group theo tag này (khớp selector trong k8s/50-karpenter-nodepools.yaml).
resource "aws_ec2_tag" "discovery_subnets" {
  for_each = toset(var.private_subnet_ids)

  resource_id = each.value
  key         = "karpenter.sh/discovery"
  value       = local.cluster_name
}

resource "aws_ec2_tag" "discovery_node_sg" {
  resource_id = var.node_security_group_id
  key         = "karpenter.sh/discovery"
  value       = local.cluster_name
}

# Hạ tầng AWS cho Karpenter: role của controller (Pod Identity), role của node, hàng đợi SQS nhận sự kiện Spot và các rule EventBridge.
module "karpenter" {
  source  = "terraform-aws-modules/eks/aws//modules/karpenter"
  version = "~> 21.0"

  cluster_name = data.aws_eks_cluster.this.name

  # Tên cố định: controller theo tiền tố toinoi-<env>, node role để EC2NodeClass tham chiếu được.
  iam_role_name                 = "${local.prefix}-karpenter-controller"
  iam_role_use_name_prefix      = false
  node_iam_role_name            = "${local.prefix}-karpenter-node"
  node_iam_role_use_name_prefix = false

  # Cho phép node dùng Session Manager (bài 05).
  node_iam_role_additional_policies = {
    AmazonSSMManagedInstanceCore = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
  }

  # Controller chạy trong kube-system với ServiceAccount karpenter; module tạo Pod Identity association cho cặp này.
  namespace       = "kube-system"
  service_account = "karpenter"

  tags = {
    Service = "platform"
  }
}

# KEDA operator đọc độ sâu hàng đợi SQS bằng Pod Identity: chỉ được lấy thuộc tính hàng đợi, không đọc hay xoá message.
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

data "aws_iam_policy_document" "keda_sqs" {
  statement {
    sid       = "ReadQueueDepth"
    effect    = "Allow"
    actions   = ["sqs:GetQueueAttributes", "sqs:GetQueueUrl"]
    resources = ["arn:aws:sqs:${var.region}:${data.aws_caller_identity.current.account_id}:${var.dispatch_queue_name}"]
  }
}

resource "aws_iam_role" "keda" {
  name               = "${local.prefix}-keda-operator"
  assume_role_policy = data.aws_iam_policy_document.pod_identity_trust.json

  tags = {
    Service = "platform"
  }
}

resource "aws_iam_role_policy" "keda_sqs" {
  name   = "read-queue-depth"
  role   = aws_iam_role.keda.id
  policy = data.aws_iam_policy_document.keda_sqs.json
}

resource "aws_eks_pod_identity_association" "keda" {
  cluster_name    = data.aws_eks_cluster.this.name
  namespace       = "keda"
  service_account = "keda-operator"
  role_arn        = aws_iam_role.keda.arn

  tags = {
    Service = "platform"
  }
}
