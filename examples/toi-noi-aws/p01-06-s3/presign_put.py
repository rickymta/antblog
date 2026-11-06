#!/usr/bin/env python3
"""In ra presigned URL PUT cho một ảnh chứng từ. Cần boto3 và credential của người chạy.

Dùng: python3 presign_put.py <bucket> <key> [expires_seconds]
Service thật (delivery-photo) làm việc này bằng AWS SDK với role của nó (bài 03); đây là bản tối thiểu để thử trên máy.
"""
import sys

import boto3
from botocore.config import Config

bucket, key = sys.argv[1], sys.argv[2]
expires = int(sys.argv[3]) if len(sys.argv) > 3 else 300

s3 = boto3.client("s3", region_name="ap-southeast-1", config=Config(signature_version="s3v4"))
url = s3.generate_presigned_url(
    "put_object",
    # ContentType nằm trong chữ ký: client phải gửi đúng header Content-Type này.
    Params={"Bucket": bucket, "Key": key, "ContentType": "image/jpeg"},
    ExpiresIn=expires,
    HttpMethod="PUT",
)
print(url)
