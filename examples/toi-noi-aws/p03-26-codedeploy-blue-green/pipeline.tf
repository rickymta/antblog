# Kết nối GitHub qua CodeConnections. Sau khi apply, kết nối ở trạng thái PENDING:
# phải vào console Developer Tools > Settings > Connections để hoàn tất việc cài GitHub App.
# Dùng resource aws_codeconnections_connection (tên dịch vụ mới); aws_codestarconnections_connection vẫn chạy.
resource "aws_codeconnections_connection" "github" {
  name          = "${local.name}-github"
  provider_type = "GitHub"
}

resource "aws_sns_topic" "notify" {
  name = "${local.name}-pipeline"
}

resource "aws_sns_topic_subscription" "email" {
  count     = var.notification_email == "" ? 0 : 1
  topic_arn = aws_sns_topic.notify.arn
  protocol  = "email"
  endpoint  = var.notification_email
}

resource "aws_codepipeline" "this" {
  name           = local.name
  role_arn       = aws_iam_role.pipeline.arn
  pipeline_type  = "V2"
  execution_mode = "QUEUED"

  artifact_store {
    type     = "S3"
    location = aws_s3_bucket.artifacts.bucket
  }

  stage {
    name = "Source"

    action {
      name             = "GitHub"
      category         = "Source"
      owner            = "AWS"
      provider         = "CodeStarSourceConnection"
      version          = "1"
      output_artifacts = ["source_output"]

      configuration = {
        ConnectionArn        = aws_codeconnections_connection.github.arn
        FullRepositoryId     = var.github_repository
        BranchName           = var.github_branch
        OutputArtifactFormat = "CODE_ZIP"
      }
    }
  }

  stage {
    name = "Build"

    action {
      name             = "BuildImage"
      category         = "Build"
      owner            = "AWS"
      provider         = "CodeBuild"
      version          = "1"
      input_artifacts  = ["source_output"]
      output_artifacts = ["build_output"]

      configuration = {
        ProjectName = aws_codebuild_project.build.name
      }
    }
  }

  dynamic "stage" {
    for_each = var.require_manual_approval ? [1] : []

    content {
      name = "Approval"

      action {
        name     = "ApproveRelease"
        category = "Approval"
        owner    = "AWS"
        provider = "Manual"
        version  = "1"

        configuration = {
          NotificationArn = aws_sns_topic.notify.arn
          CustomData      = "Duyet image trong build_output truoc khi chuyen traffic cua ${var.service}."
        }
      }
    }
  }

  stage {
    name = "Deploy"

    action {
      name            = "BlueGreen"
      category        = "Deploy"
      owner           = "AWS"
      provider        = "CodeDeployToECS"
      version         = "1"
      input_artifacts = ["source_output", "build_output"]

      configuration = {
        ApplicationName                = aws_codedeploy_app.this.name
        DeploymentGroupName            = aws_codedeploy_deployment_group.this.deployment_group_name
        TaskDefinitionTemplateArtifact = "source_output"
        TaskDefinitionTemplatePath     = "taskdef.json"
        AppSpecTemplateArtifact        = "source_output"
        AppSpecTemplatePath            = "appspec.yaml"
        Image1ArtifactName             = "build_output"
        Image1ContainerName            = "IMAGE1_NAME"
      }
    }
  }
}

# Thông báo khi pipeline xong hoặc lỗi. Muốn vào Slack, nối SNS với Amazon Q Developer in chat applications (trước đây là AWS Chatbot).
resource "aws_cloudwatch_event_rule" "pipeline_state" {
  name        = "${local.name}-pipeline-state"
  description = "Pipeline ${local.name} chuyển sang SUCCEEDED hoặc FAILED"

  event_pattern = jsonencode({
    source        = ["aws.codepipeline"]
    "detail-type" = ["CodePipeline Pipeline Execution State Change"]
    detail = {
      state    = ["SUCCEEDED", "FAILED"]
      pipeline = [aws_codepipeline.this.name]
    }
  })
}

resource "aws_cloudwatch_event_target" "pipeline_state" {
  rule = aws_cloudwatch_event_rule.pipeline_state.name
  arn  = aws_sns_topic.notify.arn
}

data "aws_iam_policy_document" "sns_events" {
  statement {
    actions   = ["sns:Publish"]
    resources = [aws_sns_topic.notify.arn]
    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }
  }
}

resource "aws_sns_topic_policy" "notify" {
  arn    = aws_sns_topic.notify.arn
  policy = data.aws_iam_policy_document.sns_events.json
}
