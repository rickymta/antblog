import 'package:toi_noi_core/toi_noi_core.dart';

import 'beta_checklist.dart' show AppRole;

/// Quãng đường của kịch bản thử. Cước lấy từ `calculateFare`, không gõ tay.
const _scenarioKm = 5.0;

/// Văn bản "What to Test" cho một build: tester đọc nó trong app TestFlight
/// trước khi mở bản thử, nên nó nói rõ cần làm gì và cái gì đã biết là lỗi.
///
/// Ném [ArgumentError] khi [buildName] hay [buildNumber] rỗng, vì một ghi chú
/// không nói đang test bản nào thì vô dụng khi phản hồi quay về.
String buildWhatToTest({
  required AppRole role,
  required String buildName,
  required String buildNumber,
  List<String> knownIssues = const [],
}) {
  if (buildName.trim().isEmpty) {
    throw ArgumentError.value(buildName, 'buildName', 'không được rỗng');
  }
  if (buildNumber.trim().isEmpty) {
    throw ArgumentError.value(buildNumber, 'buildNumber', 'không được rỗng');
  }
  final fare = calculateFare(_scenarioKm);
  final steps = switch (role) {
    AppRole.customer => [
      'Mở app, cho phép vị trí khi được hỏi, đặt một chuyến xe '
          '${_scenarioKm.toStringAsFixed(0)} km.',
      'Báo giá phải là $fare (số minh hoạ của bảng giá thử nghiệm).',
      'Đợi thông báo khi tài xế nhận chuyến, bấm vào thông báo để mở đúng '
          'màn hình chuyến.',
    ],
    AppRole.driver => [
      'Bật online, nhận một cuốc thử ${_scenarioKm.toStringAsFixed(0)} km '
          '(cước $fare).',
      'Chuyển sang app dẫn đường giữa chuyến, quay lại sau một phút: vị trí '
          'vẫn được gửi cho khách.',
      'Hoàn thành chuyến và kiểm tra cước hiển thị cho tài xế.',
    ],
  };
  final buffer = StringBuffer()
    ..writeln('Bản $buildName (build $buildNumber).')
    ..writeln('Việc cần thử:');
  for (var i = 0; i < steps.length; i++) {
    buffer.writeln('${i + 1}. ${steps[i]}');
  }
  if (knownIssues.isNotEmpty) {
    buffer.writeln('Lỗi đã biết, không cần báo lại:');
    for (final issue in knownIssues) {
      buffer.writeln('- $issue');
    }
  }
  buffer.write(
    'Báo lỗi: chụp màn hình ngay trong TestFlight để kèm ảnh và mô tả.',
  );
  return buffer.toString();
}
