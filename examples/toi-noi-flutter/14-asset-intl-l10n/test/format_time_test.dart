import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:intl/intl.dart';
import 'package:toi_noi_14/formatting.dart';

/// Khoảng trắng hẹp U+202F mà intl chèn trước AM/PM trong locale en.
const narrowNbsp = '\u202F';

void main() {
  // DateFormat cần dữ liệu ngày tháng của locale. Trong app, delegate của
  // flutter_localizations nạp giúp; trong unit test thuần Dart ta tự nạp.
  setUpAll(initializeDateFormatting);

  // Một thời điểm duy nhất, lưu ở UTC: 16:30 ngày 6/11/2026.
  final utc = DateTime.utc(2026, 11, 6, 16, 30);
  const vietnam = Duration(hours: 7);

  group('formatTripTime đổi UTC sang giờ hiển thị', () {
    test('vi, UTC+7: 23:30 6/11/2026, giờ 24 giờ, ngày/tháng/năm', () {
      expect(formatTripTime(utc, 'vi', offset: vietnam), '6/11/2026 23:30');
    });

    test('en, UTC+7: 11/6/2026 11:30 PM, tháng/ngày/năm, giờ 12 giờ', () {
      expect(
        formatTripTime(utc, 'en', offset: vietnam),
        '11/6/2026 11:30${narrowNbsp}PM',
      );
    });

    test('cùng một thời điểm, ba múi giờ cho ba chuỗi giờ khác nhau', () {
      expect(
        formatTripTime(utc, 'vi', offset: Duration.zero),
        '6/11/2026 16:30',
      );
      expect(formatTripTime(utc, 'vi', offset: vietnam), '6/11/2026 23:30');
      expect(
        formatTripTime(utc, 'vi', offset: const Duration(hours: -5)),
        '6/11/2026 11:30',
      );
    });

    test('qua nửa đêm thì ngày đổi theo: 18:00 UTC là 1:00 ngày hôm sau', () {
      final evening = DateTime.utc(2026, 11, 6, 18);
      expect(formatTripTime(evening, 'vi', offset: vietnam), '7/11/2026 1:00');
      expect(
        formatTripTime(evening, 'en', offset: vietnam),
        '11/7/2026 1:00${narrowNbsp}AM',
      );
    });

    test('múi giờ lệch nửa giờ: UTC+5:30 cho 22:00', () {
      expect(
        formatTripTime(
          utc,
          'vi',
          offset: const Duration(hours: 5, minutes: 30),
        ),
        '6/11/2026 22:00',
      );
    });
  });

  group('múi giờ của máy', () {
    // Giới hạn: nếu máy chạy test đặt múi giờ UTC thì toLocal() không đổi gì
    // và test này vẫn xanh khi bỏ toLocal(); nó chỉ có sức kiểm ở múi giờ khác UTC.
    test('không truyền offset thì dùng toLocal() của chính máy chạy test', () {
      expect(
        formatTripTime(utc, 'vi'),
        formatTripTime(utc, 'vi', offset: utc.toLocal().timeZoneOffset),
      );
    });
  });

  group('lỗi dùng sai', () {
    test('giờ không phải UTC bị từ chối', () {
      final local = DateTime(2026, 11, 6, 23, 30);
      expect(() => formatTripTime(local, 'vi'), throwsArgumentError);
    });

    test('quên đổi múi giờ: DateFormat in nguyên trường của giá trị UTC', () {
      // Đây là lỗi mà formatTripTime tránh: DateFormat không tự đổi múi giờ.
      final naive = DateFormat.yMd('vi').add_jm().format(utc);
      expect(naive, '6/11/2026 16:30');
      expect(naive, isNot(formatTripTime(utc, 'vi', offset: vietnam)));
    });
  });
}
