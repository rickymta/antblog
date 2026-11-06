#!/usr/bin/env python3
"""Tạo index quán, nạp 6 quán mẫu và chạy truy vấn "bun bo" trong bán kính 3 km.

Chạy sau `terraform apply`. Cần: pip install opensearch-py boto3.
Dùng:
  AWS_PROFILE=<lab> python3 seed_and_search.py --endpoint https://<id>.ap-southeast-1.aoss.amazonaws.com --service aoss
  (domain managed: --service es, endpoint https://vpc-....ap-southeast-1.es.amazonaws.com, chạy từ máy trong VPC)
Script chỉ ghi vào index toinoi-merchants-v1. Không in credential.
"""
import argparse
import json
import pathlib
import sys

import boto3
from opensearchpy import AWSV4SignerAuth, OpenSearch, RequestsHttpConnection

HERE = pathlib.Path(__file__).resolve().parent.parent
INDEX = "toinoi-merchants-v1"


def load(path):
    with open(HERE / path, encoding="utf-8") as handle:
        return json.load(handle)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", required=True, help="URL endpoint https://...")
    parser.add_argument("--service", choices=["aoss", "es"], default="aoss")
    parser.add_argument("--region", default="ap-southeast-1")
    args = parser.parse_args()

    host = args.endpoint.replace("https://", "").rstrip("/")
    credentials = boto3.Session().get_credentials()
    auth = AWSV4SignerAuth(credentials, args.region, args.service)
    client = OpenSearch(
        hosts=[{"host": host, "port": 443}],
        http_auth=auth,
        use_ssl=True,
        verify_certs=True,
        connection_class=RequestsHttpConnection,
    )

    if not client.indices.exists(index=INDEX):
        client.indices.create(index=INDEX, body=load("requests/merchants-index.json"))
        print(f"Đã tạo index {INDEX}")

    for doc in load("data/merchants.json"):
        client.index(index=INDEX, id=doc["merchant_id"], body=doc)
    print("Đã nạp quán mẫu. Serverless có thể cần vài giây để tìm thấy (refresh).")

    result = client.search(index=INDEX, body=load("requests/search-nearby.json"))
    for hit in result["hits"]["hits"]:
        distance = hit.get("sort", [None])[0]
        print(f"{hit['_source']['name']:<28} {distance:.0f} m")
    return 0


if __name__ == "__main__":
    sys.exit(main())
