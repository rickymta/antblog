import 'package:flutter/foundation.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Bản nháp báo giá: dịch vụ đang chọn, quãng đường đang kéo và cước tính ra.
///
/// Là một [ChangeNotifier]: mọi thay đổi gọi [notifyListeners] đúng một lần,
/// và chỉ khi giá trị thật sự đổi. Cước tính bằng `calculateFare` của
/// `toi_noi_core`, không có công thức thứ hai ở đây.
class QuoteDraft extends ChangeNotifier {
  QuoteDraft({this._service = ServiceKind.ride, double km = 5})
    : _km = km,
      _fare = ValueNotifier(calculateFare(km));

  ServiceKind _service;
  double _km;

  // Cước là dữ liệu suy ra từ km, nhưng được giữ trong ValueNotifier riêng:
  // ValueNotifier chỉ báo khi giá trị mới khác giá trị cũ theo `==`, và
  // Money so sánh theo số đồng, nên đổi km trong 2 km đầu (cước vẫn 12.000)
  // không đánh thức ai đang nghe cước.
  final ValueNotifier<Money> _fare;

  ServiceKind get service => _service;
  double get km => _km;
  Money get fare => _fare.value;

  /// Chỉ-đọc, để widget chỉ quan tâm cước lắng nghe riêng cước.
  ValueListenable<Money> get fareListenable => _fare;

  void selectService(ServiceKind service) {
    if (service == _service) return;
    _service = service;
    notifyListeners();
  }

  /// Đổi quãng đường. Ném [ArgumentError] (từ `calculateFare`) khi ngoài miền
  /// 0..100 km, và khi đó trạng thái không đổi.
  void setKm(double km) {
    if (km == _km) return;
    final fare = calculateFare(km);
    _km = km;
    _fare.value = fare;
    notifyListeners();
  }

  @override
  void dispose() {
    _fare.dispose();
    super.dispose();
  }
}
