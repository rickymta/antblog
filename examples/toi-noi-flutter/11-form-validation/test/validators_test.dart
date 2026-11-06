import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_11/trip_request_data.dart';
import 'package:toi_noi_11/validators.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('normalizeAddress', () {
    test('bỏ khoảng trắng đầu cuối và gộp khoảng trắng giữa', () {
      expect(normalizeAddress('  12   Lê  Lợi \n'), '12 Lê Lợi');
    });

    test('chuỗi toàn khoảng trắng thành chuỗi rỗng', () {
      expect(normalizeAddress(' \t\n '), '');
    });
  });

  group('validatePickup', () {
    test('null, rỗng và toàn khoảng trắng đều báo "Nhập điểm đón."', () {
      for (final input in [null, '', '   ']) {
        expect(validatePickup(input), 'Nhập điểm đón.', reason: '$input');
      }
    });

    test('ngắn hơn $minAddressLength ký tự sau chuẩn hoá thì báo quá ngắn', () {
      expect(validatePickup('  ab  '), contains('ít nhất $minAddressLength'));
    });

    test('đúng $minAddressLength ký tự là hợp lệ', () {
      expect(validatePickup('abc'), isNull);
    });

    test('dài đúng $maxAddressLength hợp lệ, dài hơn một ký tự thì lỗi', () {
      expect(validatePickup('a' * maxAddressLength), isNull);
      expect(
        validatePickup('a' * (maxAddressLength + 1)),
        contains('Tối đa $maxAddressLength'),
      );
    });

    test('độ dài tính sau khi gộp khoảng trắng', () {
      final padded = 'a${' ' * 500}b${' ' * 500}c';
      expect(validatePickup(padded), isNull);
    });
  });

  group('validateDestination', () {
    test('rỗng báo "Nhập điểm đến."', () {
      expect(validateDestination(''), 'Nhập điểm đến.');
    });

    test('khác điểm đón thì hợp lệ', () {
      expect(
        validateDestination('Sân bay Tân Sơn Nhất', pickup: '12 Lê Lợi'),
        isNull,
      );
    });

    test('trùng điểm đón bị từ chối, bỏ qua hoa thường và khoảng trắng', () {
      expect(
        validateDestination('  12   LÊ lợi ', pickup: '12 Lê Lợi'),
        'Điểm đến trùng điểm đón.',
      );
    });

    test('lỗi của chính ô đến được báo trước lỗi trùng', () {
      expect(validateDestination('', pickup: ''), 'Nhập điểm đến.');
    });

    test('không có pickup thì không so trùng', () {
      expect(validateDestination('12 Lê Lợi'), isNull);
    });
  });

  group('parseDistanceKm', () {
    final accepted = <String, double>{
      '5': 5,
      '5,5': 5.5,
      '5.5': 5.5,
      ' 2,001 ': 2.001,
      '0': 0,
      '-0': 0,
      '007': 7,
      '100': 100,
      '-3': -3,
    };
    accepted.forEach((input, expected) {
      test('"$input" đọc ra $expected', () {
        expect(parseDistanceKm(input), expected);
      });
    });

    test('"-0" chuẩn hoá thành 0.0 không mang dấu âm', () {
      expect(parseDistanceKm('-0')!.isNegative, isFalse);
    });

    final rejected = [
      null,
      '',
      '   ',
      'abc',
      '5km',
      '1,2,3',
      '1.234,5',
      '1,234.5',
      '5,',
      ',5',
      '.5',
      '5.',
      'NaN',
      'Infinity',
      '1e3',
      '--5',
      '+5',
      '٣', // chữ số Ả Rập, không phải ASCII
    ];
    for (final input in rejected) {
      test('${input == null ? 'null' : '"$input"'} bị từ chối', () {
        expect(parseDistanceKm(input), isNull);
      });
    }
  });

  group('vì sao không dùng double.tryParse', () {
    test('tryParse nhận NaN, Infinity, 1e3 nhưng không nhận dấu phẩy', () {
      expect(double.tryParse('NaN')!.isNaN, isTrue);
      expect(double.tryParse('Infinity'), double.infinity);
      expect(double.tryParse('1e3'), 1000);
      expect(double.tryParse('5,5'), isNull);
    });
  });

  group('validateDistance', () {
    test('rỗng báo thiếu', () {
      expect(validateDistance(''), 'Nhập khoảng cách (km).');
      expect(validateDistance(null), 'Nhập khoảng cách (km).');
      expect(validateDistance('  '), 'Nhập khoảng cách (km).');
    });

    test('sai định dạng báo ví dụ', () {
      expect(validateDistance('abc'), 'Nhập số, ví dụ 5 hoặc 5,5.');
      expect(validateDistance('1,2,3'), 'Nhập số, ví dụ 5 hoặc 5,5.');
    });

    test('hai biên 0 và FarePolicy.maxKm đều hợp lệ', () {
      expect(validateDistance('0'), isNull);
      expect(validateDistance('${FarePolicy.maxKm}'), isNull);
      expect(validateDistance('${FarePolicy.maxKm},0'), isNull);
    });

    test('ngoài miền: âm và lớn hơn maxKm', () {
      final message = 'Khoảng cách phải từ 0 đến ${FarePolicy.maxKm} km.';
      expect(validateDistance('-1'), message);
      expect(validateDistance('100,01'), message);
      expect(validateDistance('100.5'), message);
      expect(validateDistance('9' * 400), message);
    });

    test('mọi số mà validateDistance cho qua đều tính được cước', () {
      for (final input in ['0', '2,001', '5', '99,999', '100']) {
        expect(validateDistance(input), isNull, reason: input);
        expect(() => calculateFare(parseDistanceKm(input)!), returnsNormally);
      }
    });
  });

  group('TripRequestData', () {
    test('bằng nhau theo giá trị', () {
      const a = TripRequestData(pickup: 'A', destination: 'B', distanceKm: 5);
      const b = TripRequestData(pickup: 'A', destination: 'B', distanceKm: 5);
      expect(a, b);
      expect(a.hashCode, b.hashCode);
    });

    test('estimatedFare khớp công thức của toi_noi_core', () {
      const data = TripRequestData(
        pickup: 'A',
        destination: 'B',
        distanceKm: 5,
      );
      expect(data.estimatedFare, calculateFare(5));
      expect(data.estimatedFare.vnd, 25500);
    });
  });
}
