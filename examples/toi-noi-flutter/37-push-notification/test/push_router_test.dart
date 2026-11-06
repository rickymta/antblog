import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_37/push_router.dart';

void main() {
  const router = PushRouter();

  group('payload hợp lệ', () {
    test('trip_update với tripId đổi thành đường dẫn chuyến', () {
      final decision = router.decide({
        'type': 'trip_update',
        'tripId': 'T-1002',
      });
      expect(decision, const OpenTrip('T-1002'));
      expect((decision as OpenTrip).location, '/trip/T-1002');
    });

    test('khoá thừa bị bỏ qua và không lọt vào payload cục bộ', () {
      final decision = router.decide({
        'type': 'trip_update',
        'tripId': 'T-1002',
        'driverPhone': 'khong-duoc-xuat-hien',
        'fare': '25500',
      });
      final open = decision as OpenTrip;
      expect(open.location, '/trip/T-1002');
      expect(open.toPayload(), isNot(contains('driverPhone')));
      expect(open.toPayload(), isNot(contains('fare')));
    });

    test('mã chuyến dài đúng 64 ký tự vẫn nhận', () {
      final id = 'A${'b' * 63}';
      expect(
        router.decide({'type': 'trip_update', 'tripId': id}),
        OpenTrip(id),
      );
    });
  });

  group('payload lạ bị từ chối', () {
    final cases = <String, Map<String, Object?>?>{
      'null': null,
      'rỗng': <String, Object?>{},
      'thiếu type': {'tripId': 'T-1'},
      'type không phải chuỗi': {'type': 7, 'tripId': 'T-1'},
      'type lạ': {'type': 'promo', 'tripId': 'T-1'},
      'type sai chữ hoa': {'type': 'Trip_Update', 'tripId': 'T-1'},
      'thiếu tripId': {'type': 'trip_update'},
      'tripId rỗng': {'type': 'trip_update', 'tripId': ''},
      'tripId là số': {'type': 'trip_update', 'tripId': 1002},
      'tripId là null': {'type': 'trip_update', 'tripId': null},
      'tripId đi lên thư mục cha': {
        'type': 'trip_update',
        'tripId': '../admin',
      },
      'tripId có dấu gạch chéo': {'type': 'trip_update', 'tripId': 'T/1'},
      'tripId có query': {'type': 'trip_update', 'tripId': 'T-1?x=1'},
      'tripId có %': {'type': 'trip_update', 'tripId': 'T%2F1'},
      'tripId có khoảng trắng': {'type': 'trip_update', 'tripId': 'T 1'},
      'tripId bắt đầu bằng gạch nối': {'type': 'trip_update', 'tripId': '-T1'},
      'tripId dài 65 ký tự': {'type': 'trip_update', 'tripId': 'A${'b' * 64}'},
      'tripId xuống dòng cuối': {'type': 'trip_update', 'tripId': 'T-1\n'},
    };
    for (final entry in cases.entries) {
      test(entry.key, () {
        expect(router.decide(entry.value), isA<IgnorePush>());
      });
    }

    test('lý do không chứa giá trị của payload', () {
      final decision = router.decide({
        'type': 'trip_update',
        'tripId': '../bi-mat-123',
      });
      expect((decision as IgnorePush).reason, isNot(contains('bi-mat-123')));
    });
  });

  group('payload của thông báo cục bộ', () {
    test('toPayload rồi decideFromPayload cho lại cùng chuyến', () {
      const open = OpenTrip('T-1002');
      expect(router.decideFromPayload(open.toPayload()), open);
    });

    for (final raw in [
      null,
      '',
      'không phải json',
      '[1,2]',
      '"T-1"',
      '{"type":"x"}',
    ]) {
      test('chuỗi ${raw == null ? 'null' : "'$raw'"} bị từ chối', () {
        expect(router.decideFromPayload(raw), isA<IgnorePush>());
      });
    }
  });

  test('safeSummary chỉ liệt kê tên khoá, không có giá trị', () {
    final summary = PushRouter.safeSummary({
      'type': 'trip_update',
      'tripId': 'T-1002',
      'secret': 'abc123',
    });
    expect(summary, 'push data keys=[secret, tripId, type]');
    expect(summary, isNot(contains('abc123')));
    expect(summary, isNot(contains('T-1002')));
  });
}
