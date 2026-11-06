# Cấu hình tflint. Chạy `tflint --init` một lần để tải plugin, rồi `tflint --recursive`.
# Phiên bản plugin là ví dụ (bản mới nhất lúc review 11/2026); kiểm trang releases của terraform-linters/tflint-ruleset-aws.
plugin "aws" {
  enabled = true
  version = "0.50.0"
  source  = "github.com/terraform-linters/tflint-ruleset-aws"
}

rule "terraform_required_version" {
  enabled = true
}

rule "terraform_required_providers" {
  enabled = true
}

rule "terraform_unused_declarations" {
  enabled = true
}
