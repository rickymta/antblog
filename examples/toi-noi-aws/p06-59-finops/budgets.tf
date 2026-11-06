# Budget theo tag Project=ToiNoi. Tag phải được kích hoạt làm cost allocation tag (bài 10).
# Dùng filter_expression (đối tượng Expression của Budgets, như FilterExpression ở bài 10) thay cho
# cost_filter kiểu cũ; filter_expression đi kèm metrics.
resource "aws_budgets_budget" "project_monthly" {
  name         = "toinoi-${var.env}-monthly"
  budget_type  = "COST"
  limit_amount = var.monthly_limit
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  metrics = ["UnblendedCost"]

  filter_expression {
    tags {
      key           = "Project"
      values        = ["ToiNoi"]
      match_options = ["EQUALS"]
    }
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 80
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.alert_emails
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 100
    threshold_type             = "PERCENTAGE"
    notification_type          = "FORECASTED"
    subscriber_email_addresses = var.alert_emails
  }
}

# Hai budget dưới đây chỉ có nghĩa khi đã có Savings Plans; mặc định không tạo.
resource "aws_budgets_budget" "sp_utilization" {
  count = var.enable_commitment_budgets ? 1 : 0

  name         = "toinoi-${var.env}-sp-utilization"
  budget_type  = "SAVINGS_PLANS_UTILIZATION"
  limit_amount = "100.0"
  limit_unit   = "PERCENTAGE"
  time_unit    = "DAILY"

  notification {
    comparison_operator        = "LESS_THAN"
    threshold                  = var.commitment_utilization_floor
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.alert_emails
  }
}

resource "aws_budgets_budget" "sp_coverage" {
  count = var.enable_commitment_budgets ? 1 : 0

  name         = "toinoi-${var.env}-sp-coverage"
  budget_type  = "SAVINGS_PLANS_COVERAGE"
  limit_amount = "100.0"
  limit_unit   = "PERCENTAGE"
  time_unit    = "DAILY"

  notification {
    comparison_operator        = "LESS_THAN"
    threshold                  = var.commitment_coverage_floor
    threshold_type             = "PERCENTAGE"
    notification_type          = "ACTUAL"
    subscriber_email_addresses = var.alert_emails
  }
}
