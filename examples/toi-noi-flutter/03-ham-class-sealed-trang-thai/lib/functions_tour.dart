import 'service_kind.dart';
import 'trip.dart';

/// Tham số vị trí; `[unit]` là tham số vị trí tuỳ chọn có giá trị mặc định.
String formatVnd(int vnd, [String unit = 'VND']) {
  final digits = vnd.abs().toString();
  final buffer = StringBuffer(vnd < 0 ? '-' : '');
  for (var i = 0; i < digits.length; i++) {
    if (i > 0 && (digits.length - i) % 3 == 0) buffer.write('.');
    buffer.write(digits[i]);
  }
  return '$buffer $unit';
}

/// Tham số đặt tên: `kind` bắt buộc, `fareVnd` có thể thiếu, `unit` có mặc định.
String tripSummary(
  String id, {
  required ServiceKind kind,
  int? fareVnd,
  String unit = 'VND',
}) {
  final fare = fareVnd == null ? 'chưa có giá' : formatVnd(fareVnd, unit);
  return '$id | ${kind.label} | $fare';
}

/// Hàm mũi tên: thân chỉ là một biểu thức.
bool isLongTrip(double distanceKm) => distanceKm > 10;

/// Hàm có tên cùng chữ ký `bool Function(Trip)`, truyền được như tear-off.
bool isCancellable(Trip trip) => trip.status.canCancel;

/// Kiểu hàm: nhận một chuyến, trả bool.
typedef TripFilter = bool Function(Trip trip);

/// Hàm là giá trị: nhận một hàm khác làm tham số.
int countWhere(Iterable<Trip> trips, TripFilter test) {
  var count = 0;
  for (final trip in trips) {
    if (test(trip)) count++;
  }
  return count;
}
