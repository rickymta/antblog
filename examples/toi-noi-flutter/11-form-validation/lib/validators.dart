import 'package:toi_noi_core/toi_noi_core.dart' show FarePolicy;

/// Số ký tự tối thiểu và tối đa của một địa chỉ sau khi chuẩn hoá.
const int minAddressLength = 3;
const int maxAddressLength = 120;

/// Bỏ khoảng trắng đầu cuối và gộp mọi dãy khoảng trắng bên trong thành một dấu cách.
String normalizeAddress(String raw) =>
    raw.trim().replaceAll(RegExp(r'\s+'), ' ');

/// Kiểm một địa chỉ; trả `null` khi hợp lệ, trả câu báo lỗi khi sai.
///
/// Dùng chung cho điểm đón và điểm đến, [label] chỉ để câu báo lỗi đúng tên ô.
String? validateAddress(String? value, {required String label}) {
  final text = normalizeAddress(value ?? '');
  if (text.isEmpty) {
    return 'Nhập $label.';
  }
  if (text.length < minAddressLength) {
    return 'Nhập ít nhất $minAddressLength ký tự cho $label.';
  }
  if (text.length > maxAddressLength) {
    return 'Tối đa $maxAddressLength ký tự cho $label.';
  }
  return null;
}

String? validatePickup(String? value) =>
    validateAddress(value, label: 'điểm đón');

/// Kiểm điểm đến. Có tham số [pickup] vì điểm đến không được trùng điểm đón,
/// một quy tắc cần nhìn sang ô khác.
String? validateDestination(String? value, {String? pickup}) {
  final error = validateAddress(value, label: 'điểm đến');
  if (error != null) {
    return error;
  }
  if (pickup != null && _sameAddress(value!, pickup)) {
    return 'Điểm đến trùng điểm đón.';
  }
  return null;
}

bool _sameAddress(String a, String b) =>
    normalizeAddress(a).toLowerCase() == normalizeAddress(b).toLowerCase();

// Chỉ chữ số ASCII, tuỳ chọn dấu trừ ở đầu, tối đa một dấu phân cách thập phân
// (`,` hoặc `.`). Cố ý không dùng `double.tryParse`: nó nhận cả "NaN", "Infinity"
// và "1e3", còn dấu phẩy thì không nhận.
final RegExp _distancePattern = RegExp(r'^-?\d+(?:[.,]\d+)?$');

/// Đọc khoảng cách km từ chuỗi người dùng gõ; trả `null` nếu sai định dạng.
///
/// Nhận cả dấu phẩy (`5,5`, kiểu tiếng Việt) lẫn dấu chấm (`5.5`). Không nhận
/// chuỗi có cả hai dấu hoặc nhiều hơn một dấu phân cách như `1.234,5`.
/// Hàm này chỉ đọc số, việc kiểm miền 0..100 km nằm ở [validateDistance].
double? parseDistanceKm(String? raw) {
  final text = (raw ?? '').trim();
  if (!_distancePattern.hasMatch(text)) {
    return null;
  }
  final km = double.parse(text.replaceFirst(',', '.'));
  // "-0" ra -0.0; đổi về 0.0 để dữ liệu chuẩn hoá không mang dấu âm.
  return km == 0 ? 0.0 : km;
}

/// Kiểm ô khoảng cách. Miền hợp lệ lấy từ [FarePolicy.maxKm] để form và bảng
/// giá không bao giờ lệch nhau.
String? validateDistance(String? raw) {
  if ((raw ?? '').trim().isEmpty) {
    return 'Nhập khoảng cách (km).';
  }
  final km = parseDistanceKm(raw);
  if (km == null) {
    return 'Nhập số, ví dụ 5 hoặc 5,5.';
  }
  // Viết dưới dạng "điều kiện hợp lệ" rồi phủ định, giống calculateFareVnd.
  final isInRange = km >= 0 && km <= FarePolicy.maxKm;
  if (!isInRange) {
    return 'Khoảng cách phải từ 0 đến ${FarePolicy.maxKm} km.';
  }
  return null;
}
