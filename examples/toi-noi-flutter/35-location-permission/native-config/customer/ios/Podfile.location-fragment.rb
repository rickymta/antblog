# App KHÁCH, chỉ cho nhánh CocoaPods. Flutter 3.47 stable dùng Swift Package
# Manager mặc định và `flutter create` không sinh Podfile: với SPM, xem
# build-ios.example.sh. Chỉ khi app đã tắt SPM và có ios/Podfile mới ghép vào
# khối post_install.
# Báo cho geolocator_apple bỏ đoạn xin quyền "Always", để app không cần
# NSLocationAlwaysAndWhenInUseUsageDescription (README geolocator 14.1.2).
# permission_handler không bật macro PERMISSION_LOCATION nên không biên dịch
# phần xin quyền vị trí của nó.
installer.pods_project.targets.each do |target|
  if target.name == 'geolocator_apple'
    target.build_configurations.each do |config|
      config.build_settings['GCC_PREPROCESSOR_DEFINITIONS'] ||= ['$(inherited)']
      config.build_settings['GCC_PREPROCESSOR_DEFINITIONS'] << 'BYPASS_PERMISSION_LOCATION_ALWAYS=1'
    end
  end
end
