/// Ba lớp kích thước cửa sổ, tính theo bề rộng logical pixel.
///
/// Ngưỡng 600 và 840 theo window size classes của Android; ta dùng ba lớp là
/// đủ cho app này. Đây là quy ước thiết kế, không phải hằng của Flutter.
enum WindowClass {
  /// Dưới 600: điện thoại dọc. Thanh điều hướng ở đáy, một cột.
  compact,

  /// 600 đến dưới 840: điện thoại ngang, máy tính bảng dọc. Thanh bên.
  medium,

  /// Từ 840: máy tính bảng ngang. Thanh bên và hai cột danh sách, chi tiết.
  expanded;

  static const double mediumMinWidth = 600;
  static const double expandedMinWidth = 840;

  /// Phân loại theo bề rộng của cả cửa sổ.
  static WindowClass fromWidth(double width) {
    if (width >= expandedMinWidth) return expanded;
    if (width >= mediumMinWidth) return medium;
    return compact;
  }
}
