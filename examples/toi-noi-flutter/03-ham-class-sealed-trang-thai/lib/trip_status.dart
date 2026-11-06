/// Tám trạng thái của một chuyến, theo hồ sơ Tới Nơi.
enum TripStatus {
  requested('Đã đặt'),
  searching('Đang tìm tài xế'),
  accepted('Tài xế đã nhận'),
  arriving('Tài xế đang tới'),
  pickedUp('Đã đón / lấy hàng'),
  inTransit('Đang chở / giao'),
  completed('Hoàn thành'),
  cancelled('Đã huỷ');

  const TripStatus(this.label);

  /// Nhãn tiếng Việt để hiển thị.
  final String label;

  /// Khách hoặc tài xế chỉ được huỷ ở ba trạng thái này.
  ///
  /// Liệt kê đủ tám trạng thái, không dùng `_`: thêm trạng thái mới mà chưa
  /// quyết định huỷ được hay không sẽ khiến `dart analyze` báo lỗi.
  bool get canCancel => switch (this) {
    searching || accepted || arriving => true,
    requested || pickedUp || inTransit || completed || cancelled => false,
  };

  /// Trạng thái cuối: không còn sự kiện nào hợp lệ.
  bool get isTerminal => switch (this) {
    completed || cancelled => true,
    requested ||
    searching ||
    accepted ||
    arriving ||
    pickedUp ||
    inTransit => false,
  };
}
