# Monitor 1: theo dịch vụ AWS, bắt chi tiêu bất thường ở bất kỳ dịch vụ nào.
resource "aws_ce_anomaly_monitor" "by_service" {
  name              = "toinoi-${var.env}-by-service"
  monitor_type      = "DIMENSIONAL"
  monitor_dimension = "SERVICE"
}

# Monitor 2: theo tag Project=ToiNoi, chỉ nhìn chi phí của dự án.
# Tag cần được kích hoạt cost allocation. monitor_specification là đối tượng Expression (Key là tên tag).
resource "aws_ce_anomaly_monitor" "by_tag" {
  name         = "toinoi-${var.env}-by-project-tag"
  monitor_type = "CUSTOM"

  monitor_specification = jsonencode({
    Tags = {
      Key          = "Project"
      Values       = ["ToiNoi"]
      MatchOptions = ["EQUALS"]
    }
  })
}

# Gửi email tổng hợp hằng ngày khi tác động tuyệt đối vượt ngưỡng.
# Muốn cảnh báo ngay (IMMEDIATE) cần subscriber là SNS topic thay vì email.
resource "aws_ce_anomaly_subscription" "daily" {
  name      = "toinoi-${var.env}-anomaly-daily"
  frequency = "DAILY"

  monitor_arn_list = [
    aws_ce_anomaly_monitor.by_service.arn,
    aws_ce_anomaly_monitor.by_tag.arn,
  ]

  dynamic "subscriber" {
    for_each = toset(var.alert_emails)

    content {
      type    = "EMAIL"
      address = subscriber.value
    }
  }

  threshold_expression {
    dimension {
      key           = "ANOMALY_TOTAL_IMPACT_ABSOLUTE"
      match_options = ["GREATER_THAN_OR_EQUAL"]
      values        = [tostring(var.anomaly_threshold)]
    }
  }
}

# Cost Category: gom dịch vụ AWS thành lớp để báo cáo cho lãnh đạo.
# Tên dịch vụ là tên hiển thị trong Cost Explorer; kiểm lại trong tài khoản của bạn.
resource "aws_ce_cost_category" "layer" {
  name         = "toinoi-${var.env}-layer"
  rule_version = "CostCategoryExpression.v1"

  rule {
    value = "compute"

    rule {
      dimension {
        key           = "SERVICE"
        match_options = ["EQUALS"]
        values        = ["Amazon Elastic Container Service", "AWS Lambda", "Amazon Elastic Kubernetes Service", "Amazon Elastic Compute Cloud - Compute"]
      }
    }
  }

  rule {
    value = "data"

    rule {
      dimension {
        key           = "SERVICE"
        match_options = ["EQUALS"]
        values        = ["Amazon Relational Database Service", "Amazon DynamoDB", "Amazon ElastiCache", "Amazon Simple Storage Service", "Amazon Managed Streaming for Apache Kafka", "Amazon Kinesis"]
      }
    }
  }

  rule {
    value = "edge"

    rule {
      dimension {
        key           = "SERVICE"
        match_options = ["EQUALS"]
        values        = ["Amazon CloudFront", "AWS WAF", "Amazon API Gateway"]
      }
    }
  }

  default_value = "other"
}
