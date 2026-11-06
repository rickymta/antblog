import 'package:test/test.dart';
import 'package:toi_noi_02/quote.dart';

class _Draft {
  late String id; // không có giá trị khởi tạo: đọc trước khi gán sẽ ném lỗi
}

void main() {
  group('Quote', () {
    test(
      'chưa có tài xế: driverName là null và describe dùng giá trị mặc định',
      () {
        final quote = Quote.fromDistance(5);

        expect(quote.driverName, isNull);
        expect(quote.describe(), '5.0 km | 25.500 VND | chưa có tài xế');
      },
    );

    test('đã có tài xế: describe lấy tên cuối', () {
      final quote = Quote.fromDistance(5, driverName: 'Lê Minh Châu');

      expect(quote.describe(), '5.0 km | 25.500 VND | Châu');
    });

    test('withDriver trả bản sao, báo giá cũ không đổi', () {
      final before = Quote.fromDistance(2);
      final after = before.withDriver('Trần Văn An');

      expect(before.driverName, isNull);
      expect(after.driverName, 'Trần Văn An');
      expect(after.fareVnd, before.fareVnd);
    });

    test('chuỗi rỗng là dữ liệu lỗi, không phải "chưa có tài xế"', () {
      expect(
        () => Quote.fromDistance(5, driverName: '  '),
        throwsArgumentError,
      );
    });

    test('khoảng cách ngoài miền làm fromDistance ném lỗi', () {
      expect(() => Quote.fromDistance(-1), throwsArgumentError);
      expect(() => Quote.fromDistance(double.nan), throwsArgumentError);
    });

    test(
      'late final có initializer: tính khi đọc lần đầu, giữ nguyên sau đó',
      () {
        final quote = Quote.fromDistance(5);

        expect(quote.fareText, '25.500 VND');
        expect(identical(quote.fareText, quote.fareText), isTrue);
      },
    );
  });

  group('late', () {
    test('đọc late chưa gán ném lỗi "has not been initialized"', () {
      final draft = _Draft();

      expect(
        () => draft.id,
        throwsA(
          isA<Error>().having(
            (e) => e.toString(),
            'message',
            contains('has not been initialized'),
          ),
        ),
      );
    });

    test('gán rồi mới đọc thì bình thường', () {
      final draft = _Draft()..id = 'TN-0001';

      expect(draft.id, 'TN-0001');
    });
  });
}
