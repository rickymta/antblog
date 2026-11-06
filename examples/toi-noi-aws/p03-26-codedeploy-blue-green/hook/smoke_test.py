"""Hook AfterAllowTestTraffic cho CodeDeploy ECS.

Gọi test listener của ALB (traffic chỉ đi tới task set green), kiểm /health/ready rồi báo
kết quả cho CodeDeploy. Nếu hàm không báo trong thời gian hook cho phép, CodeDeploy coi
hook là thất bại. Smoke test thật của Tới Nơi cần thêm một request có xác thực
(token lấy từ Secrets Manager); bản mẫu này chỉ kiểm sức khoẻ để giữ ví dụ nhỏ.
"""

import json
import os
import urllib.request

import boto3

codedeploy = boto3.client("codedeploy")


def probe(url: str) -> bool:
    request = urllib.request.Request(url, headers={"User-Agent": "toinoi-codedeploy-smoke"})
    with urllib.request.urlopen(request, timeout=5) as response:
        if response.status != 200:
            return False
        body = response.read().decode("utf-8")
    try:
        return json.loads(body).get("status") in ("ready", "Healthy", "healthy")
    except ValueError:
        return True  # endpoint trả text thường: chỉ cần 200


def handler(event, context):
    deployment_id = event["DeploymentId"]
    execution_id = event["LifecycleEventHookExecutionId"]

    status = "Failed"
    try:
        if probe(os.environ["TEST_URL"]):
            status = "Succeeded"
    except Exception as error:  # noqa: BLE001 - mọi lỗi đều làm hook thất bại
        print(f"smoke test lỗi: {error}")

    codedeploy.put_lifecycle_event_hook_execution_status(
        deploymentId=deployment_id,
        lifecycleEventHookExecutionId=execution_id,
        status=status,
    )
    return status
