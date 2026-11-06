variable "region" {
  description = "Region của pipeline và ECS service."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod."
  type        = string
  default     = "lab"
}

variable "service" {
  description = "Tên service được deploy."
  type        = string
  default     = "booking"
}

variable "owner" {
  description = "Team sở hữu, dùng cho tag Owner."
  type        = string
  default     = "platform"
}

variable "github_repository" {
  description = "FullRepositoryId dạng org/repo. Ví dụ minh hoạ, đổi sang repo của bạn."
  type        = string
  default     = "toinoi-org/toi-noi"
}

variable "github_branch" {
  description = "Branch kích hoạt pipeline."
  type        = string
  default     = "main"
}

variable "ecr_repository_name" {
  description = "Repository ECR đã có (bài 11) để CodeBuild đẩy image lên."
  type        = string
  default     = "toinoi/booking"
}

variable "ecs_cluster_name" {
  description = "ECS cluster đã có (bài 12)."
  type        = string
  default     = "toinoi-lab"
}

variable "ecs_service_name" {
  description = "ECS service đã tạo với deploymentController = CODE_DEPLOY (xem ecs-service.json)."
  type        = string
  default     = "toinoi-lab-booking"
}

variable "production_listener_arn" {
  description = "Listener production của ALB (ví dụ HTTPS 443)."
  type        = string
}

variable "test_listener_arn" {
  description = "Listener test của ALB (ví dụ HTTP/HTTPS 8443) để chạy smoke test trước khi nhận traffic thật."
  type        = string
}

variable "blue_target_group_name" {
  description = "Tên target group thứ nhất (đang gắn vào service)."
  type        = string
  default     = "toinoi-lab-booking-blue"
}

variable "green_target_group_name" {
  description = "Tên target group thứ hai (nhận task set mới)."
  type        = string
  default     = "toinoi-lab-booking-green"
}

variable "ecs_passrole_arns" {
  description = "ARN execution role và task role mà task definition tham chiếu. CodePipeline chỉ được PassRole cho đúng các role này."
  type        = list(string)
}

variable "rollback_alarm_names" {
  description = "Tên CloudWatch alarm làm CodeDeploy dừng và rollback khi chuyển traffic (ví dụ 5xx của target group green)."
  type        = list(string)
  default     = []
}

variable "deployment_config_name" {
  description = "Cấu hình dịch chuyển traffic của CodeDeploy cho ECS."
  type        = string
  default     = "CodeDeployDefault.ECSCanary10Percent5Minutes"
}

variable "smoke_test_url" {
  description = "URL qua test listener mà hook AfterAllowTestTraffic gọi. Ví dụ http://<alb-dns>:8443/health/ready."
  type        = string
}

variable "require_manual_approval" {
  description = "Thêm stage Approval thủ công trước khi deploy."
  type        = bool
  default     = true
}

variable "notification_email" {
  description = "Email nhận thông báo pipeline và approval. Để trống thì không tạo subscription."
  type        = string
  default     = ""
}
