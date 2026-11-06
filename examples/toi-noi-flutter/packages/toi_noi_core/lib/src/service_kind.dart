/// Ba dịch vụ của Tới Nơi, dùng chung một đội tài xế.
enum ServiceKind {
  ride('Đặt xe'),
  food('Giao đồ ăn'),
  parcel('Giao bưu kiện');

  const ServiceKind(this.label);

  /// Nhãn tiếng Việt để hiển thị.
  final String label;
}
