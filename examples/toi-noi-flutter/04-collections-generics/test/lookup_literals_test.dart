import 'package:test/test.dart';
import 'package:toi_noi_04/driver_lookup.dart';
import 'package:toi_noi_04/literals.dart';
import 'package:toi_noi_04/model.dart';

import 'fixtures.dart';

const _drivers = [
  Driver(id: 'D-17', name: 'Lê Minh An', city: 'Hà Nội', isOnline: true),
  Driver(id: 'D-23', name: 'Trần Bảo Bình', city: 'Đà Nẵng', isOnline: true),
  Driver(id: 'D-31', name: 'Phạm Chí Công', city: 'Hà Nội', isOnline: false),
  Driver(id: 'D-58', name: 'Đỗ Gia Em', city: 'Đà Nẵng', isOnline: true),
];

void main() {
  group('Set', () {
    test('onlineCities loại trùng và bỏ tài xế offline', () {
      final cities = onlineCities(_drivers);
      expect(cities, {'Hà Nội', 'Đà Nẵng'});
      expect(
        cities.length,
        2,
      ); // Đà Nẵng có hai tài xế nhưng chỉ xuất hiện một lần
    });

    test('Set.contains trả lời "có không" và add trùng trả false', () {
      final cities = onlineCities(_drivers);
      expect(cities.contains('Đà Nẵng'), isTrue);
      expect(cities.contains('TP.HCM'), isFalse);
      expect(cities.add('Đà Nẵng'), isFalse);
      expect(cities.add('TP.HCM'), isTrue);
    });

    test('không có ai online thì Set rỗng', () {
      expect(onlineCities(_drivers.where((d) => !d.isOnline)), isEmpty);
    });
  });

  group('Map', () {
    final byId = indexById(_drivers);

    test('indexById tra theo id', () {
      expect(byId['D-17']!.name, 'Lê Minh An');
      expect(byId.length, 4);
    });

    test('key không có trong Map trả null, không ném lỗi', () {
      expect(byId['D-99'], isNull);
      expect(findDriver(byId, 'D-99'), isNull);
    });

    test(
      'findDriver với id null trả null; driverOf dùng driverId của chuyến',
      () {
        expect(findDriver(byId, null), isNull);
        final assigned = makeTrip(
          'T1',
          TripStatus.accepted,
          1,
          driverId: 'D-23',
        );
        final unassigned = makeTrip('T2', TripStatus.searching, 1);
        expect(driverOf(assigned, byId)?.name, 'Trần Bảo Bình');
        expect(driverOf(unassigned, byId), isNull);
      },
    );

    test('trùng id: bản sau ghi đè bản trước', () {
      final dup = indexById([
        _drivers[0],
        const Driver(id: 'D-17', name: 'Bản mới', city: 'Huế', isOnline: false),
      ]);
      expect(dup.length, 1);
      expect(dup['D-17']!.name, 'Bản mới');
    });
  });

  group('any, firstWhere', () {
    test('hasOnlineDriverIn', () {
      expect(hasOnlineDriverIn(_drivers, 'Đà Nẵng'), isTrue);
      expect(
        hasOnlineDriverIn(_drivers.where((d) => !d.isOnline), 'Hà Nội'),
        isFalse,
      );
      expect(hasOnlineDriverIn(_drivers, 'Huế'), isFalse);
    });

    test('firstOnlineIn trả null khi không có', () {
      expect(firstOnlineIn(_drivers, 'Hà Nội')!.id, 'D-17');
      expect(firstOnlineIn(_drivers, 'Huế'), isNull);
    });

    test(
      'firstWhere không orElse ném StateError, có orElse thì dùng dự phòng',
      () {
        expect(
          () => _drivers.firstWhere((d) => d.city == 'Huế'),
          throwsStateError,
        );
        const fallback = Driver(
          id: 'D-00',
          name: 'Dự phòng',
          city: 'Huế',
          isOnline: true,
        );
        expect(
          firstOnlineOrFallback(_drivers, 'Huế', fallback),
          same(fallback),
        );
        expect(firstOnlineOrFallback(_drivers, 'Hà Nội', fallback).id, 'D-17');
      },
    );
  });

  group('collection literal', () {
    test('collection-if và spread', () {
      expect(homeTabs(isDriverApp: true), ['Chuyến', 'Thu nhập']);
      expect(homeTabs(isDriverApp: false, extra: ['Hồ sơ', 'Trợ giúp']), [
        'Chuyến',
        'Ưu đãi',
        'Hồ sơ',
        'Trợ giúp',
      ]);
    });

    test('collection-for dựng Map đủ 8 trạng thái, mỗi trạng thái bằng 0', () {
      final counts = emptyStatusCounts();
      expect(counts.length, 8);
      expect(counts.values.every((n) => n == 0), isTrue);
    });

    test('const Set bất biến và khớp isTerminal', () {
      expect(
        terminalStatuses,
        TripStatus.values.where((s) => s.isTerminal).toSet(),
      );
      expect(
        () => terminalStatuses.add(TripStatus.requested),
        throwsUnsupportedError,
      );
    });

    test('canCancel đúng ba trạng thái', () {
      expect(TripStatus.values.where((s) => s.canCancel), [
        TripStatus.searching,
        TripStatus.accepted,
        TripStatus.arriving,
      ]);
    });
  });
}
