/// Số tiền VND bất biến, lưu bằng số nguyên đồng (không có phần lẻ).
///
/// Không âm: tiền của Tới Nơi chỉ là giá và tổng giá. `==` so sánh theo giá
/// trị nên hai `Money(25500)` bằng nhau dù là hai đối tượng khác nhau.
final class Money implements Comparable<Money> {
  /// Tạo từ số đồng cho hằng và giá trị đã biết chắc.
  ///
  /// Giá trị âm chỉ bị `assert` chặn khi bật assertions (`dart test`,
  /// `--enable-asserts`, Flutter debug). Dữ liệu từ ngoài vào phải dùng
  /// [Money.fromVnd].
  const Money(this.vnd) : assert(vnd >= 0, 'Số tiền không được âm');

  /// Tạo từ số đồng chưa được tin cậy (JSON, form, database).
  ///
  /// Luôn kiểm tra, kể cả khi assertions tắt; ném [ArgumentError] nếu [vnd] âm.
  factory Money.fromVnd(int vnd) {
    if (vnd < 0) {
      throw ArgumentError.value(vnd, 'vnd', 'không được âm');
    }
    return Money(vnd);
  }

  /// 0 đồng, dùng làm giá trị khởi đầu khi cộng dồn.
  static const Money zero = Money(0);

  /// Số đồng.
  final int vnd;

  /// Cộng hai số tiền.
  Money operator +(Money other) => Money(vnd + other.vnd);

  /// Trừ hai số tiền; ném [ArgumentError] nếu kết quả âm.
  Money operator -(Money other) {
    final result = vnd - other.vnd;
    if (result < 0) {
      throw ArgumentError.value(other, 'other', 'lớn hơn số tiền hiện có');
    }
    return Money(result);
  }

  /// Nhân với số lượng nguyên không âm; ném [ArgumentError] nếu [factor] âm.
  Money operator *(int factor) {
    if (factor < 0) {
      throw ArgumentError.value(factor, 'factor', 'không được âm');
    }
    return Money(vnd * factor);
  }

  bool operator <(Money other) => vnd < other.vnd;

  bool operator <=(Money other) => vnd <= other.vnd;

  bool operator >(Money other) => vnd > other.vnd;

  bool operator >=(Money other) => vnd >= other.vnd;

  @override
  int compareTo(Money other) => vnd.compareTo(other.vnd);

  @override
  bool operator ==(Object other) => other is Money && other.vnd == vnd;

  @override
  int get hashCode => vnd.hashCode;

  /// Dạng hiển thị có dấu chấm nghìn: `Money(25500)` thành `"25.500 VND"`.
  ///
  /// Chèn tay để chưa phải dùng package `intl`.
  @override
  String toString() {
    final digits = vnd.abs().toString();
    final buffer = StringBuffer();
    for (var i = 0; i < digits.length; i++) {
      final remaining = digits.length - i;
      if (i > 0 && remaining % 3 == 0) {
        buffer.write('.');
      }
      buffer.write(digits[i]);
    }
    final sign = vnd < 0 ? '-' : '';
    return '$sign$buffer VND';
  }
}
