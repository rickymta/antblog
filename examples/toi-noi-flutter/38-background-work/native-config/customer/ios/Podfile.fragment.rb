# Dán vào khối post_install của ios/Podfile (theo README của geolocator).
# Cờ này bỏ lời gọi requestAlwaysAuthorization khỏi geolocator_apple, nhờ đó
# app chỉ-khi-đang-dùng không phải khai NSLocationAlways... trong Info.plist.
post_install do |installer|
  installer.pods_project.targets.each do |target|
    if target.name == "geolocator_apple"
      target.build_configurations.each do |config|
        config.build_settings['GCC_PREPROCESSOR_DEFINITIONS'] ||= ['$(inherited)', 'BYPASS_PERMISSION_LOCATION_ALWAYS=1']
      end
    end
  end
end
