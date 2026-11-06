#!/usr/bin/env bash
# Sinh ảnh bìa rồi hoàn lại ảnh cũ: chạy lại generator đổi byte của mọi ảnh đã có, chỉ ảnh mới được giữ.
# Worktree không có node_modules nên mượn tạm của checkout chính qua symlink, xong thì gỡ.
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
main_root="$(git -C "$root" worktree list --porcelain | sed -n '1s/^worktree //p')"
linked=0
if [[ ! -e "$root/node_modules" ]]; then
  ln -s "$main_root/node_modules" "$root/node_modules"
  linked=1
fi
cleanup() { [[ "$linked" == 1 ]] && rm -f "$root/node_modules"; }
trap cleanup EXIT
node "$root/apps/web/scripts/generate-covers.mjs"
git -C "$root" checkout -- apps/web/public/images/articles/
node "$root/apps/web/scripts/check-covers.mjs"
