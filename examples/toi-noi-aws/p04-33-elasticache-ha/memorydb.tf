# MemoryDB: dữ liệu ghi vào transaction log đa AZ trước khi trả lời, nên bền vững như một database.
# Chỉ tạo khi create_memorydb = true. Dùng cho trạng thái dispatch không được mất khi failover
# (ví dụ khoá chào chuyến cho tài xế), không dùng cho cache báo giá.
resource "aws_memorydb_subnet_group" "main" {
  count = var.create_memorydb ? 1 : 0

  name       = "toinoi-${var.env}-dispatch"
  subnet_ids = var.private_subnet_ids
}

resource "aws_security_group" "memorydb" {
  count = var.create_memorydb ? 1 : 0

  name        = "toinoi-${var.env}-dispatch-memorydb"
  description = "MemoryDB trang thai dispatch"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_ingress_rule" "memorydb_from_client" {
  count = var.create_memorydb ? 1 : 0

  security_group_id            = aws_security_group.memorydb[0].id
  referenced_security_group_id = var.client_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 6379
  to_port                      = 6379
  description                  = "Service dispatch vao MemoryDB"
}

# User xác thực bằng IAM (không có mật khẩu trong code). Chỉ chạm khoá dispatch:* và kênh dispatch:*.
# MemoryDB luôn chạy cluster mode: client cần CLUSTER SLOTS/SHARDS để khám phá shard, mà hai lệnh này
# không thuộc các nhóm @read/@write/@connection (đã thử ACL DRYRUN trên Redis 7), nên cấp riêng.
resource "aws_memorydb_user" "dispatch" {
  count = var.create_memorydb ? 1 : 0

  user_name     = "dispatch-service"
  access_string = "on ~dispatch:* &dispatch:* -@all +@read +@write +@pubsub +@stream +@connection +cluster|slots +cluster|shards"

  authentication_mode {
    type = "iam"
  }
}

resource "aws_memorydb_acl" "dispatch" {
  count = var.create_memorydb ? 1 : 0

  name       = "toinoi-${var.env}-dispatch"
  user_names = [aws_memorydb_user.dispatch[0].user_name]
}

resource "aws_memorydb_cluster" "dispatch" {
  count = var.create_memorydb ? 1 : 0

  name                     = "toinoi-${var.env}-dispatch"
  description              = "Trang thai dispatch ben vung"
  engine                   = "valkey"
  engine_version           = var.memorydb_engine_version
  node_type                = var.memorydb_node_type
  num_shards               = var.memorydb_num_shards
  num_replicas_per_shard   = var.memorydb_replicas_per_shard
  acl_name                 = aws_memorydb_acl.dispatch[0].name
  subnet_group_name        = aws_memorydb_subnet_group.main[0].name
  security_group_ids       = [aws_security_group.memorydb[0].id]
  tls_enabled              = true
  snapshot_retention_limit = 1
}
