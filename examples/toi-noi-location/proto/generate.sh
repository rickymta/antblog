#!/usr/bin/env bash
# Sinh lại mã Go từ location.proto. Không cần protoc: dùng buf và protoc-gen-go, cả hai thuần Go.
# Phiên bản pin để kết quả tái lập được.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="/usr/local/go/bin:$PATH"
BIN="$(mktemp -d)"
trap 'rm -rf "$BIN"' EXIT
GOBIN="$BIN" go install github.com/bufbuild/buf/cmd/buf@v1.50.0
GOBIN="$BIN" go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.36.12
export PATH="$BIN:$PATH"
buf lint
buf generate
echo "đã sinh ../go/gen/location/v1/location.pb.go"
