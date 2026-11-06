locals {
  # Cluster dựng ở lab p05-42 tên toinoi-<env>-eks; role và EFS đặt theo tiền tố toinoi-<env>.
  prefix       = "toinoi-${var.env}"
  cluster_name = "${local.prefix}-eks"
  name         = "${local.prefix}-photo-cache"
}

data "aws_eks_cluster" "this" {
  name = local.cluster_name
}

# Role cho EBS CSI driver và EFS CSI driver. Association được tạo bởi add-on (khối pod_identity_association bên dưới).
module "ebs_csi_pod_identity" {
  source  = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 2.0"

  name = "${local.prefix}-ebs-csi"

  attach_aws_ebs_csi_policy = true

  tags = {
    Service = "platform"
  }
}

module "efs_csi_pod_identity" {
  source  = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 2.0"

  name = "${local.prefix}-efs-csi"

  attach_aws_efs_csi_policy = true

  tags = {
    Service = "platform"
  }
}

resource "aws_eks_addon" "ebs_csi" {
  cluster_name                = data.aws_eks_cluster.this.name
  addon_name                  = "aws-ebs-csi-driver"
  resolve_conflicts_on_update = "OVERWRITE"

  pod_identity_association {
    role_arn        = module.ebs_csi_pod_identity.iam_role_arn
    service_account = "ebs-csi-controller-sa"
  }

  tags = {
    Service = "platform"
  }
}

resource "aws_eks_addon" "efs_csi" {
  cluster_name                = data.aws_eks_cluster.this.name
  addon_name                  = "aws-efs-csi-driver"
  resolve_conflicts_on_update = "OVERWRITE"

  pod_identity_association {
    role_arn        = module.efs_csi_pod_identity.iam_role_arn
    service_account = "efs-csi-controller-sa"
  }

  tags = {
    Service = "platform"
  }
}

# CRD VolumeSnapshot và snapshot controller: EKS cung cấp sẵn dưới dạng add-on snapshot-controller, không cần quyền IAM.
resource "aws_eks_addon" "snapshot_controller" {
  cluster_name                = data.aws_eks_cluster.this.name
  addon_name                  = "snapshot-controller"
  resolve_conflicts_on_update = "OVERWRITE"

  tags = {
    Service = "platform"
  }
}

# EFS cho cache ảnh tạm dùng chung giữa các pod (ReadWriteMany).
resource "aws_efs_file_system" "photo_cache" {
  creation_token   = local.name
  encrypted        = true
  throughput_mode  = "elastic"
  performance_mode = "generalPurpose"

  lifecycle_policy {
    transition_to_ia = "AFTER_30_DAYS"
  }

  tags = {
    Name    = local.name
    Service = "booking"
  }
}

resource "aws_security_group" "efs" {
  name        = "${local.name}-efs"
  description = "NFS vao EFS cache anh tu node EKS"
  vpc_id      = var.vpc_id

  tags = {
    Name    = "${local.name}-efs"
    Service = "booking"
  }
}

resource "aws_vpc_security_group_ingress_rule" "efs_from_nodes" {
  security_group_id            = aws_security_group.efs.id
  referenced_security_group_id = var.node_security_group_id
  description                  = "NFS tu node EKS"
  ip_protocol                  = "tcp"
  from_port                    = 2049
  to_port                      = 2049
}

# Mỗi AZ một mount target để pod mount không phải băng qua AZ khác.
resource "aws_efs_mount_target" "photo_cache" {
  for_each = toset(var.private_subnet_ids)

  file_system_id  = aws_efs_file_system.photo_cache.id
  subnet_id       = each.value
  security_groups = [aws_security_group.efs.id]
}
