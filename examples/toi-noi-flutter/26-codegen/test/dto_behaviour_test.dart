import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_26/dto/trip_dto.dart';
import 'package:toi_noi_api/toi_noi_api.dart' as hand;
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support/fixtures.dart';

/// Những thứ `freezed` và `json_serializable` cho không, mà bản viết tay
/// không có (hoặc phải viết tay từng dòng).
void main() {
  final json = validTripJson();

  group('freezed sinh ==, hashCode, toString, copyWith', () {
    test('== và hashCode so sánh theo giá trị', () {
      final a = TripDto.fromJson(json);
      final b = TripDto.fromJson(json);
      expect(identical(a, b), isFalse);
      expect(a, b);
      expect(a.hashCode, b.hashCode);
    });

    test('bản viết tay không có ==: hai DTO cùng JSON là hai giá trị khác', () {
      expect(
        hand.TripDto.fromJson(json) == hand.TripDto.fromJson(json),
        isFalse,
      );
    });

    test('khác một trường là khác nhau', () {
      expect(
        TripDto.fromJson(json),
        isNot(TripDto.fromJson(changed(json, {'version': 2}))),
      );
    });

    test('toString liệt kê trường', () {
      final text = TripDto.fromJson(json).toString();
      expect(text, startsWith('TripDto('));
      expect(text, contains('id: T-0001'));
      expect(text, contains('fareVnd: 25500'));
    });

    test('copyWith đổi đúng trường được nêu và giữ phần còn lại', () {
      final base = TripDto.fromJson(json);
      final next = base.copyWith(
        status: TripStatus.accepted,
        driverId: 'D-9',
        version: 2,
      );
      expect(next.status, TripStatus.accepted);
      expect(next.driverId, 'D-9');
      expect(next.version, 2);
      expect(next.id, base.id);
      expect(next.fareVnd, 25500);
      expect(base.status, TripStatus.searching, reason: 'bản gốc không đổi');
    });

    test('copyWith phân biệt "không nêu" với "đặt null"', () {
      final withDriver = TripDto.fromJson(
        changed(json, {'status': 'accepted', 'driverId': 'D-9'}),
      );
      expect(withDriver.copyWith(version: 5).driverId, 'D-9');
      expect(withDriver.copyWith(driverId: null).driverId, isNull);
    });
  });

  group('json_serializable sinh fromJson và toJson', () {
    test('toJson cùng khoá, cùng thứ tự, cùng giá trị với bản viết tay', () {
      final generated = TripDto.fromJson(json).toJson();
      final written = hand.TripDto.fromJson(json).toJson();
      expect(generated, written);
      expect(generated.keys.toList(), written.keys.toList());
    });

    test(
      'khoá là fare (theo JsonKey), không phải fareVnd; null được ghi ra',
      () {
        final out = TripDto.fromJson(json).toJson();
        expect(out.containsKey('fare'), isTrue);
        expect(out.containsKey('fareVnd'), isFalse);
        expect(out.containsKey('driverId'), isTrue);
        expect(out['driverId'], isNull);
      },
    );

    test('toJson rồi fromJson trả đúng giá trị ban đầu', () {
      final original = TripDto.fromJson(
        changed(json, {
          'status': 'accepted',
          'driverId': 'D-9',
          'requestedAt': '2026-11-06T10:30:00+07:00',
        }),
      );
      expect(TripDto.fromJson(original.toJson()), original);
    });

    test(
      'thời điểm luôn được ghi ra ở UTC, kể cả khi DTO giữ giờ địa phương',
      () {
        final local = TripDto.fromJson(json)
            .copyWith(requestedAt: DateTime(2026, 11, 6, 10, 30));
        final text = local.toJson()['requestedAt']! as String;
        expect(text.endsWith('Z'), isTrue);
        expect(DateTime.parse(text), local.requestedAt.toUtc());
      },
    );
  });

  group('converter chỉ canh biên JSON, không canh constructor', () {
    test(
      'copyWith(fareVnd: -1) lọt qua constructor, bị chặn ở toJson và toDomain',
      () {
        final broken = TripDto.fromJson(json).copyWith(fareVnd: -1);
        expect(broken.fareVnd, -1);
        // Phát lại JSON này bị từ chối ở biên JSON.
        expect(
          () => TripDto.fromJson(broken.toJson()),
          throwsA(isA<hand.DtoFormatException>()),
        );
        // Money.fromVnd từ chối số âm khi sang domain.
        expect(broken.toDomain, throwsA(isA<hand.DtoFormatException>()));
      },
    );

    test('copyWith(distanceMeters: -5) lọt cả toDomain: ra Trip -0,005 km', () {
      final broken = TripDto.fromJson(json).copyWith(distanceMeters: -5);
      expect(broken.toDomain().distanceKm, -0.005);
      // Chỉ biên JSON mới bắt được.
      expect(
        () => TripDto.fromJson(broken.toJson()),
        throwsA(isA<hand.DtoFormatException>()),
      );
    });

    test('copyWith(currency: "USD") đi qua toJson và toDomain', () {
      final broken = TripDto.fromJson(json).copyWith(currency: 'USD');
      expect(broken.toJson()['currency'], 'USD');
      expect(broken.toDomain().fare.vnd, 25500);
      // Bản viết tay không có trường currency: luôn ghi 'VND'.
      expect(hand.TripDto.fromJson(json).toJson()['currency'], 'VND');
      expect(
        () => TripDto.fromJson(broken.toJson()),
        throwsA(isA<hand.DtoFormatException>()),
      );
    });
  });

  group('cầu nối với domain', () {
    final requestedAt = DateTime.utc(2026, 11, 6, 3, 30);

    test('fromDomain rồi toDomain trả đúng Trip; 5 km là 25.500 VND', () {
      final trip = Trip.request(
        id: 'T-0001',
        serviceKind: ServiceKind.ride,
        distanceKm: 5,
        requestedAt: requestedAt,
      );
      final dto = TripDto.fromDomain(trip, version: 0);
      expect(dto.fareVnd, calculateFare(5).vnd);
      expect(dto.fareVnd, 25500);
      expect(dto.distanceMeters, 5000);
      expect(dto.toDomain(), trip);
    });

    test('2,001 km: 2001 mét và 12.005 VND, giống bản viết tay', () {
      final trip = Trip.request(
        id: 'T-0002',
        serviceKind: ServiceKind.food,
        distanceKm: 2.001,
        requestedAt: requestedAt,
      );
      final generated = TripDto.fromDomain(trip, version: 0);
      final written = hand.TripDto.fromDomain(trip, version: 0);
      expect(generated.distanceMeters, 2001);
      expect(generated.fareVnd, 12005);
      expect(generated.toJson(), written.toJson());
    });

    test('toSnapshot mang version đi cùng chuyến', () {
      final snapshot = TripDto.fromJson(changed(json, {'version': 4}))
          .toSnapshot();
      expect(snapshot.version, 4);
      expect(snapshot.trip.id, 'T-0001');
    });
  });
}
