import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/quote_draft.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('QuoteDraft', () {
    late QuoteDraft draft;
    late int notifications;

    setUp(() {
      draft = QuoteDraft();
      notifications = 0;
      draft.addListener(() => notifications++);
      addTearDown(() {
        // Test "dispose" tự huỷ draft, nên chỉ huỷ khi chưa huỷ.
        try {
          draft.dispose();
        } on FlutterError {
          // đã huỷ trong thân test
        }
      });
    });

    test('mặc định: đặt xe, 5 km, 25.500 VND (do calculateFare tính)', () {
      expect(draft.service, ServiceKind.ride);
      expect(draft.km, 5);
      expect(draft.fare, calculateFare(5));
      expect(draft.fare.toString(), '25.500 VND');
    });

    test('đổi dịch vụ báo đúng một lần; chọn lại cái cũ thì không báo', () {
      draft.selectService(ServiceKind.food);
      expect(draft.service, ServiceKind.food);
      expect(notifications, 1);

      draft.selectService(ServiceKind.food);
      expect(notifications, 1);
    });

    test('cước theo km khớp bảng giá: 2,001 km và 100 km', () {
      draft.setKm(2.001);
      expect(draft.fare.vnd, 12005);

      draft.setKm(100);
      expect(draft.fare.vnd, 453000);
      expect(notifications, 2);
    });

    test('km ngoài miền hoặc NaN: ném ArgumentError, trạng thái không đổi', () {
      for (final bad in [101.0, -1.0, double.nan, double.infinity]) {
        expect(() => draft.setKm(bad), throwsArgumentError, reason: '$bad');
      }
      expect(draft.km, 5);
      expect(draft.fare.vnd, 25500);
      expect(notifications, 0);
    });

    test('đổi km trong 2 km đầu: draft báo nhưng cước không báo', () {
      draft.setKm(1);
      var fareNotifications = 0;
      draft.fareListenable.addListener(() => fareNotifications++);
      notifications = 0;

      draft.setKm(1.5); // cước vẫn 12.000
      expect(notifications, 1);
      expect(fareNotifications, 0);
      expect(draft.fare.vnd, 12000);

      draft.setKm(3); // cước 16.500
      expect(notifications, 2);
      expect(fareNotifications, 1);
      expect(draft.fare.vnd, 16500);
    });

    test('sau dispose, thêm listener bị chặn (cả draft lẫn cước)', () {
      draft.dispose();

      expect(() => draft.addListener(() {}), throwsFlutterError);
      expect(() => draft.fareListenable.addListener(() {}), throwsFlutterError);
    });
  });

  group('ValueNotifier', () {
    test(
      'báo khi giá trị thay bằng giá trị khác theo ==, không báo khi bằng',
      () {
        final n = ValueNotifier<int>(1);
        addTearDown(n.dispose);
        var count = 0;
        n.addListener(() => count++);

        n.value = 1;
        expect(count, 0);
        n.value = 2;
        expect(count, 1);
      },
    );

    test('sửa tại chỗ một List thì không báo; thay bằng bản sao thì báo', () {
      final n = ValueNotifier<List<int>>([1]);
      addTearDown(n.dispose);
      var count = 0;
      n.addListener(() => count++);

      n.value.add(2); // cùng một đối tượng, notifier không biết
      expect(count, 0);

      n.value = [...n.value, 3];
      expect(count, 1);
    });
  });
}
