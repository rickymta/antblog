#!/usr/bin/env bash
# Cho thấy hash tag quyết định slot: các khoá có cùng {tag} nằm cùng slot nên dùng được MGET, MULTI và Lua cùng nhau.
# Dùng: ENDPOINT=<endpoint cluster mode> ./keyslot.sh
source "$(dirname "$0")/common.sh"

for key in "trip:42:driver" "trip:42:status" "{trip:42}:driver" "{trip:42}:status" "{trip:43}:driver"; do
  printf '%-20s slot %s\n' "$key" "$(cli CLUSTER KEYSLOT "$key")"
done
