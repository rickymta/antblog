# App TÀI XẾ, chỉ cho nhánh CocoaPods (xem ghi chú ở bản của app khách). Với
# SPM không cần gì: Package.swift của permission_handler_apple đọc Info.plist
# và tự bật macro theo khoá có mặt.
# Bước nâng lên "Always" đi qua permission_handler, và code xin quyền Always của
# nó chỉ được biên dịch khi macro PERMISSION_LOCATION=1 (README permission_handler).
# Không đặt BYPASS_PERMISSION_LOCATION_ALWAYS cho geolocator_apple ở app này.
installer.pods_project.targets.each do |target|
  if target.name == 'permission_handler_apple'
    target.build_configurations.each do |config|
      config.build_settings['GCC_PREPROCESSOR_DEFINITIONS'] ||= ['$(inherited)']
      config.build_settings['GCC_PREPROCESSOR_DEFINITIONS'] << 'PERMISSION_LOCATION=1'
    end
  end
end
