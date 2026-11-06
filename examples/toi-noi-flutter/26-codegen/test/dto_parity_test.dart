import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_26/dto/quote_dto.dart' as gen;
import 'package:toi_noi_26/dto/trip_dto.dart' as gen;
import 'package:toi_noi_api/toi_noi_api.dart' as hand;

import 'support/fixtures.dart';

/// Kết quả của một lần giải JSON, ở dạng so sánh được: hoặc `toJson()` của DTO,
/// hoặc `DtoFormatException <DTO>.<trường>`. Mọi loại lỗi khác (TypeError,
/// ArgumentError...) làm test hỏng, vì chúng không thuộc hợp đồng lỗi.
Object outcome(Object? Function() decode) {
  try {
    final dto = decode()! as dynamic;
    return dto.toJson() as Object;
  } on hand.DtoFormatException catch (e) {
    return 'DtoFormatException ${e.dto}.${e.field}';
  }
}

void main() {
  group('TripDto: bản sinh mã cùng hành vi với bản viết tay', () {
    final cases = tripCases();

    for (final entry in cases.entries) {
      test(entry.key, () {
        final fromHand = outcome(() => hand.TripDto.fromJson(entry.value));
        final fromGen = outcome(() => gen.TripDto.fromJson(entry.value));
        expect(fromGen, fromHand);
      });
    }

    test(
      'hai lỗi cùng lúc: cả hai bản báo lỗi, nhưng tên trường có thể khác',
      () {
        // Bản viết tay kiểm `currency` trước mọi trường khác; bản sinh mã
        // báo trường đầu tiên theo thứ tự khai báo. Cùng loại lỗi, khác trường.
        String field(Object? Function() decode) {
          try {
            decode();
          } on hand.DtoFormatException catch (e) {
            return e.field;
          }
          fail('phải bị từ chối');
        }

        final twoErrors = changed(validTripJson(), {
          'currency': 'USD',
          'id': absent,
        });
        expect(field(() => hand.TripDto.fromJson(twoErrors)), 'currency');
        expect(field(() => gen.TripDto.fromJson(twoErrors)), 'id');
        expect(
          field(() => hand.TripDto.fromJson(<String, Object?>{})),
          'currency',
        );
        expect(field(() => gen.TripDto.fromJson(<String, Object?>{})), 'id');
        // Không có currency trong cuộc chơi: cùng một trường.
        final other = changed(validTripJson(), {'id': absent, 'version': -1});
        expect(field(() => hand.TripDto.fromJson(other)), 'id');
        expect(field(() => gen.TripDto.fromJson(other)), 'id');
      },
    );

    test('bảng ca có cả ca nhận lẫn ca từ chối', () {
      // Chặn kiểu hỏng âm thầm: bảng toàn ca lỗi (hoặc toàn ca nhận) sẽ làm
      // phép so sánh ở trên chẳng chứng minh gì.
      final results = [
        for (final json in cases.values)
          outcome(() => hand.TripDto.fromJson(json)),
      ];
      expect(results.whereType<Map<Object?, Object?>>().length, 8);
      expect(results.whereType<String>().length, cases.length - 8);
    });

    test('lỗi nêu đúng tên trường của JSON (fare, không phải fareVnd)', () {
      expect(
        () => gen.TripDto.fromJson(changed(validTripJson(), {'fare': 25500.5})),
        throwsA(
          isA<hand.DtoFormatException>()
              .having((e) => e.dto, 'dto', 'TripDto')
              .having((e) => e.field, 'field', 'fare'),
        ),
      );
    });

    test('toDomain giữ bất biến: accepted mà không có tài xế', () {
      final json = changed(validTripJson(), {'status': 'accepted'});
      for (final decode in <Object Function()>[
        () => hand.TripDto.fromJson(json).toDomain(),
        () => gen.TripDto.fromJson(json).toDomain(),
      ]) {
        expect(
          decode,
          throwsA(
            isA<hand.DtoFormatException>().having(
              (e) => e.field,
              'field',
              'driverId',
            ),
          ),
        );
      }
    });

    test('toDomain và toSnapshot cho cùng Trip, cùng version', () {
      final json = changed(validTripJson(), {
        'status': 'accepted',
        'driverId': 'D-9',
        'version': 3,
      });
      final fromHand = hand.TripDto.fromJson(json).toSnapshot();
      final fromGen = gen.TripDto.fromJson(json).toSnapshot();
      expect(fromGen, fromHand);
      expect(fromGen.version, 3);
      expect(fromGen.trip.fare.vnd, 25500);
    });
  });

  group('QuoteDto: bản sinh mã cùng hành vi với bản viết tay', () {
    final cases = <String, Object?>{
      'báo giá hợp lệ': validQuoteJson(),
      'cước 25500.0': changed(validQuoteJson(), {'fare': 25500.0}),
      'tên dịch vụ hoa': changed(validQuoteJson(), {'serviceKind': 'FOOD'}),
      'khoá thừa': changed(validQuoteJson(), {'note': 'x'}),
      'thiếu fare': changed(validQuoteJson(), {'fare': absent}),
      'thiếu currency': changed(validQuoteJson(), {'currency': absent}),
      'cước lẻ': changed(validQuoteJson(), {'fare': 25500.5}),
      'cước âm': changed(validQuoteJson(), {'fare': -5}),
      'tiền tệ USD': changed(validQuoteJson(), {'currency': 'USD'}),
      'mét lẻ': changed(validQuoteJson(), {'distanceMeters': 1.5}),
      'dịch vụ lạ': changed(validQuoteJson(), {'serviceKind': 'bike'}),
      'mảng': <Object?>[],
      'null': null,
    };

    for (final entry in cases.entries) {
      test(entry.key, () {
        final fromHand = outcome(() => hand.QuoteDto.fromJson(entry.value));
        final fromGen = outcome(() => gen.QuoteDto.fromJson(entry.value));
        expect(fromGen, fromHand);
      });
    }

    test('toMoney: 25.500 VND, cùng giá trị với bản viết tay', () {
      final fromGen = gen.QuoteDto.fromJson(validQuoteJson());
      final fromHand = hand.QuoteDto.fromJson(validQuoteJson());
      expect(fromGen.toMoney(), fromHand.toMoney());
      expect(fromGen.toMoney().toString(), '25.500 VND');
    });
  });
}
