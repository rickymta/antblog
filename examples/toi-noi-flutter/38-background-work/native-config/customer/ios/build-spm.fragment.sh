# Chỉ áp dụng khi dự án iOS dùng Swift Package Manager (mặc định trên Flutter
# stable gần đây; geolocator_apple có Package.swift). Cờ truyền bằng biến môi
# trường lúc build, thay cho đoạn Podfile (theo README của geolocator).
BYPASS_PERMISSION_LOCATION_ALWAYS=1 flutter build ios
