#!/bin/sh
# App KHÁCH, Swift Package Manager (đường mặc định của Flutter 3.47 stable).
# Báo cho geolocator_apple bỏ đoạn xin quyền "Always", để Info.plist không cần
# NSLocationAlwaysAndWhenInUseUsageDescription (README geolocator 14.1.2):
BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios

# Build từ Xcode.app thì biến môi trường không tới được: permission_handler_apple
# cần PERMISSION_HANDLER_INFO_PLIST trỏ tới ios/Runner/Info.plist, và đổi biến
# môi trường nào cũng phải xoá DerivedData (README permission_handler):
#   launchctl setenv BYPASS_PERMISSION_LOCATION_ALWAYS 1
#   launchctl setenv PERMISSION_HANDLER_INFO_PLIST /đường/dẫn/ios/Runner/Info.plist
#   rm -rf ~/Library/Developer/Xcode/DerivedData
