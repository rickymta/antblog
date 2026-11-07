#!/bin/sh
# Run Script build phase của target Runner: chọn GoogleService-Info.plist theo flavor.
# Đặt SAU "Copy Bundle Resources". Cần TN_FLAVOR (từ Flavors/<flavor>.xcconfig).
#
# Xcode tự đặt PROJECT_DIR, TARGET_BUILD_DIR, UNLOCALIZED_RESOURCES_FOLDER_PATH và
# PRODUCT_BUNDLE_IDENTIFIER khi chạy phase này.
set -eu

: "${TN_FLAVOR:?TN_FLAVOR chưa đặt: Flavors/<flavor>.xcconfig chưa được áp vào configuration này}"

source_plist="${PROJECT_DIR}/Runner/Firebase/${TN_FLAVOR}/GoogleService-Info.plist"
target_dir="${TARGET_BUILD_DIR}/${UNLOCALIZED_RESOURCES_FOLDER_PATH}"

if [ ! -f "$source_plist" ]; then
  echo "error: thiếu $source_plist. Tải file của flavor ${TN_FLAVOR} từ Firebase console (file thật không commit)." >&2
  exit 1
fi

# Bắt lỗi chép nhầm file của flavor khác: BUNDLE_ID trong plist phải bằng bundle ID đang build.
# PlistBuddy có sẵn trên macOS; thiếu thì bỏ qua phép so này.
if [ -x /usr/libexec/PlistBuddy ]; then
  plist_bundle_id=$(/usr/libexec/PlistBuddy -c "Print :BUNDLE_ID" "$source_plist")
  if [ "$plist_bundle_id" != "${PRODUCT_BUNDLE_IDENTIFIER}" ]; then
    echo "error: $source_plist có BUNDLE_ID $plist_bundle_id nhưng đang build ${PRODUCT_BUNDLE_IDENTIFIER}." >&2
    exit 1
  fi
fi

mkdir -p "$target_dir"
cp "$source_plist" "$target_dir/GoogleService-Info.plist"
