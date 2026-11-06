# ---------------------------------------------------------------------------
# AWS Network Firewall: chặn dev <-> prod, chỉ cho egress tới tên miền allowlist
# ---------------------------------------------------------------------------
resource "aws_networkfirewall_rule_group" "segment_block" {
  count = var.enable_network_firewall ? 1 : 0

  name     = "toinoi-${var.env}-segment-block"
  type     = "STATEFUL"
  capacity = 50

  rule_group {
    rules_source {
      stateful_rule {
        action = "DROP"
        header {
          direction        = "ANY"
          protocol         = "IP"
          source           = var.spoke_cidrs["dev"]
          source_port      = "ANY"
          destination      = var.spoke_cidrs["prod"]
          destination_port = "ANY"
        }
        rule_option {
          keyword  = "sid"
          settings = ["1"]
        }
      }
    }

    stateful_rule_options {
      rule_order = "STRICT_ORDER"
    }
  }

  tags = {
    Service = "network"
  }
}

resource "aws_networkfirewall_rule_group" "egress_allowlist" {
  count = var.enable_network_firewall ? 1 : 0

  name     = "toinoi-${var.env}-egress-allowlist"
  type     = "STATEFUL"
  capacity = 100

  rule_group {
    # Domain list chỉ xét traffic có nguồn thuộc HOME_NET, mặc định là CIDR của VPC chứa firewall.
    # Firewall đặt ở VPC inspection trung tâm nên phải thêm CIDR của các spoke, nếu không traffic
    # từ dev/prod không khớp allowlist và bị hành động mặc định drop_established chặn hết.
    rule_variables {
      ip_sets {
        key = "HOME_NET"
        ip_set {
          definition = concat(values(var.spoke_cidrs), [var.inspection_cidr])
        }
      }
    }

    rules_source {
      rules_source_list {
        generated_rules_type = "ALLOWLIST"
        target_types         = ["HTTP_HOST", "TLS_SNI"]
        targets              = var.allowed_egress_domains
      }
    }

    stateful_rule_options {
      rule_order = "STRICT_ORDER"
    }
  }

  tags = {
    Service = "network"
  }
}

resource "aws_networkfirewall_firewall_policy" "main" {
  count = var.enable_network_firewall ? 1 : 0

  name = "toinoi-${var.env}-inspection"

  firewall_policy {
    stateless_default_actions          = ["aws:forward_to_sfe"]
    stateless_fragment_default_actions = ["aws:forward_to_sfe"]

    # Strict order + drop established: cái gì không được allow rõ ràng thì bị chặn.
    stateful_default_actions = ["aws:drop_established", "aws:alert_established"]

    stateful_engine_options {
      rule_order = "STRICT_ORDER"
    }

    stateful_rule_group_reference {
      priority     = 10
      resource_arn = aws_networkfirewall_rule_group.segment_block[0].arn
    }

    stateful_rule_group_reference {
      priority     = 20
      resource_arn = aws_networkfirewall_rule_group.egress_allowlist[0].arn
    }
  }

  tags = {
    Service = "network"
  }
}

resource "aws_networkfirewall_firewall" "main" {
  count = var.enable_network_firewall ? 1 : 0

  name                = "toinoi-${var.env}-inspection"
  firewall_policy_arn = aws_networkfirewall_firewall_policy.main[0].arn
  vpc_id              = aws_vpc.inspection.id

  dynamic "subnet_mapping" {
    for_each = aws_subnet.inspection_firewall

    content {
      subnet_id = subnet_mapping.value.id
    }
  }

  tags = {
    Service = "network"
  }
}

resource "aws_cloudwatch_log_group" "firewall_alert" {
  count = var.enable_network_firewall ? 1 : 0

  name              = "/toinoi/${var.env}/network-firewall/alert"
  retention_in_days = 14

  tags = {
    Service = "network"
  }
}

resource "aws_networkfirewall_logging_configuration" "main" {
  count = var.enable_network_firewall ? 1 : 0

  firewall_arn = aws_networkfirewall_firewall.main[0].arn

  logging_configuration {
    log_destination_config {
      log_destination = {
        logGroup = aws_cloudwatch_log_group.firewall_alert[0].name
      }
      log_destination_type = "CloudWatchLogs"
      log_type             = "ALERT"
    }
  }
}
