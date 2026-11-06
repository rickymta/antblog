# Với MSK dùng IAM, quyền do IAM policy quyết định theo cluster, topic và consumer group.

# Consumer của dispatch: đọc booking.events.v1 trong group dispatch-consumer.
data "aws_iam_policy_document" "dispatch_consumer" {
  statement {
    actions   = ["kafka-cluster:Connect", "kafka-cluster:DescribeCluster"]
    resources = [local.cluster_arn]
  }

  statement {
    actions   = ["kafka-cluster:DescribeTopic", "kafka-cluster:ReadData"]
    resources = ["${local.topic_arn_prefix}/booking.events.v1"]
  }

  statement {
    actions   = ["kafka-cluster:AlterGroup", "kafka-cluster:DescribeGroup"]
    resources = ["${local.group_arn_prefix}/dispatch-consumer"]
  }
}

resource "aws_iam_policy" "dispatch_consumer" {
  name        = "${local.name}-dispatch-consumer"
  description = "Doc booking.events.v1 trong group dispatch-consumer"
  policy      = data.aws_iam_policy_document.dispatch_consumer.json
}

# Producer ghi trực tiếp (ví dụ service dispatch): chỉ ghi, chỉ vào một topic.
# WriteDataIdempotently là quyền trên tài nguyên cluster (producer bật enable.idempotence=true cần nó).
# Vị trí GPS không đi qua MSK trong lộ trình này: xem bài 35 (Kinesis Data Streams).
data "aws_iam_policy_document" "dispatch_producer" {
  statement {
    actions   = ["kafka-cluster:Connect", "kafka-cluster:DescribeCluster", "kafka-cluster:WriteDataIdempotently"]
    resources = [local.cluster_arn]
  }

  statement {
    actions   = ["kafka-cluster:DescribeTopic", "kafka-cluster:WriteData"]
    resources = ["${local.topic_arn_prefix}/dispatch.events.v1"]
  }
}

resource "aws_iam_policy" "dispatch_producer" {
  name        = "${local.name}-dispatch-producer"
  description = "Ghi dispatch.events.v1"
  policy      = data.aws_iam_policy_document.dispatch_producer.json
}
