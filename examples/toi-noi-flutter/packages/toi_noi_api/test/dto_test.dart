import 'package:test/test.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Payload hợp lệ theo hợp đồng của fake API / BFF.
Map<String, Object?> _tripJson({Map<String, Object?> override = const {}}) => {
  'id': 'T-1',
  'serviceKind': 'ride',
  'status': 'requested',
  'distanceMeters': 5000,
  'fare': 25500,
  'currency': 'VND',
  'driverId': null,
  'requestedAt': '2026-11-06T03:30:00.000Z',
  'version': 0,
  ...override,
};

Matcher _failsOn(String dto, String field) => throwsA(
  isA<DtoFormatException>()
      .having((e) => e.dto, 'dto', dto)
      .having((e) => e.field, 'field', field),
);

void main() {
  group('QuoteRequestDto', () {
    test('toJson dùng tên enum và km như khách nhập', () {
      const dto = QuoteRequestDto(
        serviceKind: ServiceKind.parcel,
        distanceKm: 2.001,
      );
      expect(dto.toJson(), {'serviceKind': 'parcel', 'distanceKm': 2.001});
    });

    test('fromJson(toJson) trả lại đúng giá trị, kể cả km nguyên', () {
      const dto = QuoteRequestDto(serviceKind: ServiceKind.food, distanceKm: 5);
      final back = QuoteRequestDto.fromJson(dto.toJson());
      expect(back.serviceKind, ServiceKind.food);
      expect(back.distanceKm, 5.0);
      // JSON số nguyên (5) cũng được đọc thành double.
      expect(
        QuoteRequestDto.fromJson({'serviceKind': 'ride', 'distanceKm': 5})
            .distanceKm,
        5.0,
      );
    });

    test('toJson từ chối NaN và vô cực thay vì sinh JSON hỏng', () {
      for (final km in [double.nan, double.infinity]) {
        final dto = QuoteRequestDto(
          serviceKind: ServiceKind.ride,
          distanceKm: km,
        );
        expect(dto.toJson, throwsArgumentError);
      }
    });

    test('fromJson báo đúng trường sai', () {
      expect(
        () => QuoteRequestDto.fromJson({'serviceKind': 'ride'}),
        _failsOn('QuoteRequestDto', 'distanceKm'),
      );
      expect(
        () => QuoteRequestDto.fromJson({
          'serviceKind': 'ride',
          'distanceKm': '5',
        }),
        _failsOn('QuoteRequestDto', 'distanceKm'),
      );
      expect(
        () =>
            QuoteRequestDto.fromJson({'serviceKind': 'taxi', 'distanceKm': 5}),
        _failsOn('QuoteRequestDto', 'serviceKind'),
      );
      expect(
        () => QuoteRequestDto.fromJson([1, 2]),
        _failsOn('QuoteRequestDto', r'$'),
      );
    });
  });

  group('QuoteDto', () {
    test('đọc cước và đổi sang Money', () {
      final dto = QuoteDto.fromJson({
        'serviceKind': 'ride',
        'distanceMeters': 5000,
        'fare': 25500,
        'currency': 'VND',
      });
      expect(dto.fareVnd, 25500);
      expect(dto.toMoney(), const Money(25500));
      expect(dto.toMoney().toString(), '25.500 VND');
      expect(dto.distanceMeters, 5000);
    });

    test('toJson rồi fromJson cho lại cùng giá trị', () {
      const dto = QuoteDto(
        serviceKind: ServiceKind.ride,
        distanceMeters: 2001,
        fareVnd: 12005,
      );
      final back = QuoteDto.fromJson(dto.toJson());
      expect(back.distanceMeters, 2001);
      expect(back.fareVnd, 12005);
    });

    test('tiền tệ khác VND bị từ chối, không đoán', () {
      expect(
        () => QuoteDto.fromJson({
          'serviceKind': 'ride',
          'distanceMeters': 5000,
          'fare': 25500,
          'currency': 'USD',
        }),
        _failsOn('QuoteDto', 'currency'),
      );
    });

    test('cước âm hoặc có phần lẻ bị từ chối', () {
      for (final fare in [-1, 25500.5, 'nhiều']) {
        expect(
          () => QuoteDto.fromJson({
            'serviceKind': 'ride',
            'distanceMeters': 5000,
            'fare': fare,
            'currency': 'VND',
          }),
          _failsOn('QuoteDto', 'fare'),
          reason: '$fare',
        );
      }
    });
  });

  group('TripDto.fromJson', () {
    test('payload hợp lệ', () {
      final dto = TripDto.fromJson(_tripJson());
      expect(dto.id, 'T-1');
      expect(dto.serviceKind, ServiceKind.ride);
      expect(dto.status, TripStatus.requested);
      expect(dto.distanceMeters, 5000);
      expect(dto.fareVnd, 25500);
      expect(dto.driverId, isNull);
      expect(dto.requestedAt, DateTime.utc(2026, 11, 6, 3, 30));
      expect(dto.requestedAt.isUtc, isTrue);
      expect(dto.version, 0);
    });

    test('trường lạ được bỏ qua (server thêm trường không làm app hỏng)', () {
      final dto = TripDto.fromJson(_tripJson(override: {'customerId': 'C-9'}));
      expect(dto.id, 'T-1');
    });

    test('driverId vắng mặt coi như null', () {
      final json = _tripJson()..remove('driverId');
      expect(TripDto.fromJson(json).driverId, isNull);
    });

    for (final field in [
      'id',
      'serviceKind',
      'status',
      'distanceMeters',
      'fare',
      'currency',
      'requestedAt',
      'version',
    ]) {
      test('thiếu $field -> báo đúng trường', () {
        final json = _tripJson()..remove(field);
        expect(() => TripDto.fromJson(json), _failsOn('TripDto', field));
      });
    }

    test('sai kiểu từng trường', () {
      final wrong = <String, Object?>{
        'id': 7,
        'distanceMeters': '5000',
        'version': 1.5,
        'driverId': 12,
        'serviceKind': 3,
        'status': true,
      };
      for (final entry in wrong.entries) {
        expect(
          () => TripDto.fromJson(_tripJson(override: {entry.key: entry.value})),
          _failsOn('TripDto', entry.key),
          reason: entry.key,
        );
      }
    });

    test('mét là double (5000.0) bị từ chối vì hợp đồng nói int', () {
      expect(
        () => TripDto.fromJson(_tripJson(override: {'distanceMeters': 5000.0})),
        _failsOn('TripDto', 'distanceMeters'),
      );
    });

    test('số âm bị từ chối', () {
      for (final field in ['distanceMeters', 'fare', 'version']) {
        expect(
          () => TripDto.fromJson(_tripJson(override: {field: -1})),
          _failsOn('TripDto', field),
          reason: field,
        );
      }
    });

    test(
      'trạng thái: camelCase của fake API và PascalCase của .NET đều nhận',
      () {
        expect(
          TripDto.fromJson(_tripJson(override: {'status': 'pickedUp'})).status,
          TripStatus.pickedUp,
        );
        expect(
          TripDto.fromJson(_tripJson(override: {'status': 'PickedUp'})).status,
          TripStatus.pickedUp,
        );
        expect(
          TripDto.fromJson(_tripJson(override: {'status': 'Requested'})).status,
          TripStatus.requested,
        );
      },
    );

    test('trạng thái lạ nêu rõ tập giá trị hợp lệ', () {
      expect(
        () => TripDto.fromJson(_tripJson(override: {'status': 'teleporting'})),
        throwsA(
          isA<DtoFormatException>()
              .having((e) => e.field, 'field', 'status')
              .having((e) => e.message, 'message', contains('inTransit')),
        ),
      );
    });

    test('số quá lớn bị từ chối thay vì tràn int', () {
      // 1e300.toInt() là 9223372036854775807; mức chặn là 2^53.
      for (final fare in [1e300, 9007199254740994.0, 9007199254740993]) {
        expect(
          () => TripDto.fromJson(_tripJson(override: {'fare': fare})),
          _failsOn('TripDto', 'fare'),
          reason: '$fare',
        );
      }
      for (final field in ['distanceMeters', 'version']) {
        expect(
          () =>
              TripDto.fromJson(_tripJson(override: {field: 9007199254740993})),
          _failsOn('TripDto', field),
          reason: field,
        );
      }
      expect(
        TripDto.fromJson(_tripJson(override: {'fare': 9007199254740992}))
            .fareVnd,
        9007199254740992,
      );
    });

    test('cước dạng 25500.0 nhận được, 25500.5 thì không', () {
      expect(
        TripDto.fromJson(_tripJson(override: {'fare': 25500.0})).fareVnd,
        25500,
      );
      expect(
        () => TripDto.fromJson(_tripJson(override: {'fare': 25500.5})),
        _failsOn('TripDto', 'fare'),
      );
    });

    test('tiền tệ khác VND bị từ chối', () {
      expect(
        () => TripDto.fromJson(_tripJson(override: {'currency': 'USD'})),
        _failsOn('TripDto', 'currency'),
      );
    });

    group('requestedAt', () {
      test('múi giờ +07:00 được đổi về UTC', () {
        final dto = TripDto.fromJson(
          _tripJson(override: {'requestedAt': '2026-11-06T10:30:00+07:00'}),
        );
        expect(dto.requestedAt, DateTime.utc(2026, 11, 6, 3, 30));
        expect(dto.requestedAt.isUtc, isTrue);
      });

      test('thiếu múi giờ bị từ chối vì phụ thuộc máy đọc', () {
        expect(
          () => TripDto.fromJson(
            _tripJson(override: {'requestedAt': '2026-11-06T03:30:00'}),
          ),
          _failsOn('TripDto', 'requestedAt'),
        );
      });

      test('không phải ngày giờ bị từ chối', () {
        for (final text in ['hôm qua', '', '2026-13-45T00:00:00Z']) {
          expect(
            () => TripDto.fromJson(_tripJson(override: {'requestedAt': text})),
            _failsOn('TripDto', 'requestedAt'),
            reason: text,
          );
        }
      });

      test('ngày giờ tràn bị từ chối thay vì cuộn sang ngày khác', () {
        // DateTime.parse('2026-13-45T00:00:00Z') ra 2027-02-14, không báo lỗi.
        expect(DateTime.tryParse('2026-13-45T00:00:00Z'), isNotNull);
        for (final text in [
          '2026-13-45T00:00:00Z',
          '2026-02-30T00:00:00Z',
          '2026-11-06T24:00:00Z',
          '2026-11-06T03:60:00Z',
          '2026-11-06T03:30:61Z',
          '2026-11-06T03:30:00+99:99',
          '2026-11-06T03:30:00+24:00',
          '2026-11-06T03:30:00-05:60',
        ]) {
          expect(
            () => TripDto.fromJson(_tripJson(override: {'requestedAt': text})),
            _failsOn('TripDto', 'requestedAt'),
            reason: text,
          );
        }
      });

      test('định dạng .NET (+00:00, 7 chữ số lẻ) đọc được', () {
        final dto = TripDto.fromJson(
          _tripJson(
            override: {'requestedAt': '2026-11-06T03:30:00.1234567+00:00'},
          ),
        );
        expect(dto.requestedAt.isUtc, isTrue);
        expect(dto.requestedAt.hour, 3);
      });

      test('micro giây từ server được giữ nguyên', () {
        final dto = TripDto.fromJson(
          _tripJson(override: {'requestedAt': '2026-11-06T03:30:00.123456Z'}),
        );
        expect(dto.requestedAt.microsecond, 456);
      });
    });

    test('view hiện tại của service .NET thiếu trường: báo trường đầu tiên thiếu', () {
      // TripView(Id, CustomerId, Fare, Currency, Status, DriverId, Version)
      // serialize camelCase; không có serviceKind, distanceMeters, requestedAt.
      final dotnetView = <String, Object?>{
        'id': '6f1f4d55-1b60-4b9a-8f3c-0f0b1c4a9a11',
        'customerId': '0d6a3f0e-5c1c-4a58-9d0e-6a8f1f3e2b77',
        'fare': 25500,
        'currency': 'VND',
        'status': 'Requested',
        'driverId': null,
        'version': 0,
      };
      expect(
        () => TripDto.fromJson(dotnetView),
        _failsOn('TripDto', 'serviceKind'),
      );
    });

    test('payload không phải object bị từ chối', () {
      for (final bad in [null, 'chuỗi', 42, <Object?>[]]) {
        expect(() => TripDto.fromJson(bad), _failsOn('TripDto', r'$'));
      }
    });
  });

  group('TripDto.toDomain', () {
    test('map sang Trip của toi_noi_core: km từ mét, Money từ đồng', () {
      final trip = TripDto.fromJson(_tripJson()).toDomain();
      expect(
        trip,
        Trip.request(
          id: 'T-1',
          serviceKind: ServiceKind.ride,
          distanceKm: 5,
          requestedAt: DateTime.utc(2026, 11, 6, 3, 30),
        ),
      );
      expect(trip.fare, calculateFare(5));
      expect(trip.fare.toString(), '25.500 VND');
    });

    test('2001 mét -> 2,001 km -> cước 12.005', () {
      final trip = TripDto.fromJson(
        _tripJson(override: {'distanceMeters': 2001, 'fare': 12005}),
      ).toDomain();
      expect(trip.distanceKm, 2.001);
      expect(trip.fare, calculateFare(trip.distanceKm));
    });

    test(
      'mọi trạng thái: domain -> DTO -> JSON -> DTO -> domain giữ nguyên',
      () {
        final requested = Trip.request(
          id: 'T-2',
          serviceKind: ServiceKind.parcel,
          distanceKm: 12.345,
          requestedAt: DateTime.utc(2026, 11, 6, 3),
        );
        final searching = requested.apply(const SearchStarted());
        final accepted = searching.apply(const DriverAccepted('D-7'));
        final arriving = accepted.apply(const DriverArrived());
        final pickedUp = arriving.apply(const PickedUp());
        final inTransit = pickedUp.apply(const TripStarted());
        final completed = inTransit.apply(const Completed());
        final cancelled = accepted.apply(
          const Cancelled(by: CancelledBy.driver),
        );
        final all = [
          requested,
          searching,
          accepted,
          arriving,
          pickedUp,
          inTransit,
          completed,
          cancelled,
        ];
        expect(all.map((t) => t.status).toSet(), TripStatus.values.toSet());

        for (final trip in all) {
          final dto = TripDto.fromDomain(trip, version: 3);
          final back = TripDto.fromJson(dto.toJson()).toDomain();
          expect(back, trip, reason: trip.status.name);
        }
      },
    );

    test('accepted mà không có tài xế -> DtoFormatException, không phải ArgumentError', () {
      final dto = TripDto.fromJson(
        _tripJson(override: {'status': 'accepted', 'driverId': null}),
      );
      expect(
        dto.toDomain,
        throwsA(
          isA<DtoFormatException>()
              .having((e) => e.field, 'field', 'driverId')
              .having((e) => e.message, 'message', contains('accepted')),
        ),
      );
    });

    test('searching mà đã có tài xế -> DtoFormatException', () {
      final dto = TripDto.fromJson(
        _tripJson(override: {'status': 'searching', 'driverId': 'D-1'}),
      );
      expect(dto.toDomain, throwsA(isA<DtoFormatException>()));
    });
  });

  group('TripSnapshot', () {
    test('toSnapshot giữ trip và version đi cùng nhau', () {
      final snapshot = TripDto.fromJson(_tripJson(override: {'version': 4}))
          .toSnapshot();
      expect(snapshot.version, 4);
      expect(snapshot.trip.id, 'T-1');
      expect(snapshot.trip.fare, const Money(25500));
    });

    test('bằng nhau theo giá trị', () {
      final a = TripDto.fromJson(_tripJson()).toSnapshot();
      final b = TripDto.fromJson(_tripJson()).toSnapshot();
      expect(a, b);
      expect(a.hashCode, b.hashCode);
      expect(
        a,
        isNot(
          TripDto.fromJson(_tripJson(override: {'version': 1})).toSnapshot(),
        ),
      );
    });

    test(
      'payload vi phạm bất biến của Trip: toSnapshot ném DtoFormatException',
      () {
        final dto = TripDto.fromJson(
          _tripJson(override: {'status': 'accepted', 'driverId': null}),
        );
        expect(dto.toSnapshot, throwsA(isA<DtoFormatException>()));
      },
    );
  });
}
