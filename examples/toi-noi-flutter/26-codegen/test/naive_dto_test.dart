import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_26/dto/naive_trip_dto.dart';
import 'package:toi_noi_26/dto/trip_dto.dart';
import 'package:toi_noi_api/toi_noi_api.dart' as hand;

import 'support/fixtures.dart';

/// Chỗ bản sinh mã "mặc định" (không converter, không `checked`) lỏng hơn bản
/// viết tay. Mỗi test dưới đây là một dòng của bảng trong bài viết.
NaiveTripDto naive(Map<String, Object?> json) =>
    NaiveTripDto.fromJson(json.cast<String, dynamic>());

void main() {
  group('NaiveTripDto nhận những thứ bản viết tay từ chối', () {
    test('mét lẻ 5000.5 bị cắt thành 5000 mà không báo', () {
      final json = changed(validTripJson(), {'distanceMeters': 5000.5});
      expect(naive(json).distanceMeters, 5000);
      expect(
        () => hand.TripDto.fromJson(json),
        throwsA(isA<hand.DtoFormatException>()),
      );
    });

    test('cước lẻ 25500.5 bị cắt thành 25500: sai tiền, không có lỗi', () {
      final json = changed(validTripJson(), {'fare': 25500.5});
      expect(naive(json).fareVnd, 25500);
      expect(
        () => hand.TripDto.fromJson(json),
        throwsA(isA<hand.DtoFormatException>()),
      );
    });

    test('cước 1e300 thành số nguyên lớn nhất của int 64 bit', () {
      final json = changed(validTripJson(), {'fare': 1e300});
      expect(naive(json).fareVnd, 9223372036854775807);
    });

    test('mét âm, cước âm và version âm đi qua', () {
      expect(
        naive(changed(validTripJson(), {'distanceMeters': -1})).distanceMeters,
        -1,
      );
      expect(naive(changed(validTripJson(), {'fare': -1})).fareVnd, -1);
      expect(naive(changed(validTripJson(), {'version': -1})).version, -1);
    });

    test('tiền tệ USD đi qua', () {
      expect(
        naive(changed(validTripJson(), {'currency': 'USD'})).currency,
        'USD',
      );
    });

    test('ngày tràn bị cuộn sang ngày khác thay vì báo lỗi', () {
      DateTime at(String text) =>
          naive(changed(validTripJson(), {'requestedAt': text})).requestedAt;
      expect(at('2026-13-45T03:30:00Z'), DateTime.utc(2027, 2, 14, 3, 30));
      expect(at('2026-02-30T03:30:00Z'), DateTime.utc(2026, 3, 2, 3, 30));
      expect(at('2026-11-06T25:61:61Z'), DateTime.utc(2026, 11, 7, 2, 2, 1));
      expect(
        at('2026-11-06T03:30:00+99:99'),
        DateTime.utc(2026, 11, 1, 22, 51),
      );
    });

    test('thời điểm thiếu múi giờ thành giờ địa phương, không phải UTC', () {
      final at = naive(
        changed(validTripJson(), {'requestedAt': '2026-11-06T03:30:00'}),
      ).requestedAt;
      expect(at.isUtc, isFalse);
      // Domain (`Trip`) đòi `requestedAt.isUtc`; DTO này không bảo đảm điều đó.
      expect(at.toIso8601String().endsWith('Z'), isFalse);
    });
  });

  group('NaiveTripDto: lỗi không thuộc hợp đồng lỗi của toi_noi_api', () {
    test('kiểu sai ném TypeError, không phải DtoFormatException', () {
      final json = changed(validTripJson(), {'distanceMeters': '5000'});
      expect(() => naive(json), throwsA(isA<TypeError>()));
    });

    test(
      'TypeError là Error, không phải Exception: guardApi ném lại nguyên',
      () async {
        final json = changed(validTripJson(), {'id': absent});
        await expectLater(
          hand.guardApi(() async => naive(json)),
          throwsA(isA<TypeError>()),
        );
      },
    );

    test('enum sai chữ hoa ném ArgumentError (chặt hơn bản viết tay)', () {
      final json = changed(validTripJson(), {'serviceKind': 'RIDE'});
      expect(() => naive(json), throwsArgumentError);
      expect(hand.TripDto.fromJson(json).serviceKind.name, 'ride');
    });

    test(
      'cùng đầu vào hỏng, TripDto có converter thành ParseFailure',
      () async {
        final json = changed(validTripJson(), {'id': absent});
        await expectLater(
          hand.guardApi(() async => TripDto.fromJson(json)),
          throwsA(
            isA<hand.ParseFailure>()
                .having((f) => f.dto, 'dto', 'TripDto')
                .having((f) => f.field, 'field', 'id'),
          ),
        );
      },
    );
  });

  test('tổng kết: bao nhiêu ca bản viết tay từ chối mà bản mặc định nhận', () {
    var handRejects = 0;
    var naiveAccepts = 0;
    var naiveTypeErrors = 0;
    var naiveOtherErrors = 0;
    for (final json in tripCases().values.whereType<Map<String, Object?>>()) {
      try {
        hand.TripDto.fromJson(json);
        continue;
      } on hand.DtoFormatException {
        handRejects++;
      }
      try {
        naive(json);
        naiveAccepts++;
      } on TypeError {
        naiveTypeErrors++;
      } on Object {
        naiveOtherErrors++;
      }
    }
    // Ba nhóm: nhận âm thầm (nguy hiểm nhất), TypeError (lỗi không phân loại
    // được), lỗi khác (ArgumentError, FormatException).
    expect(handRejects, 35);
    expect(naiveAccepts, 16);
    expect(naiveTypeErrors, 13);
    expect(naiveOtherErrors, 6);
  });
}
