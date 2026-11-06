import 'package:test/test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  group('Money', () {
    group('toString', () {
      test('chèn dấu chấm nghìn', () {
        expect(const Money(25500).toString(), '25.500 VND');
        expect(const Money(453000).toString(), '453.000 VND');
        expect(const Money(1234567).toString(), '1.234.567 VND');
      });

      test('số dưới 1.000 không có dấu chấm', () {
        expect(Money.zero.toString(), '0 VND');
        expect(const Money(999).toString(), '999 VND');
        expect(const Money(1000).toString(), '1.000 VND');
      });
    });

    group('số học', () {
      test('cộng và nhân trả về Money mới', () {
        expect(const Money(12000) + const Money(13500), const Money(25500));
        expect(const Money(4500) * 3, const Money(13500));
        expect(const Money(4500) * 0, Money.zero);
      });

      test('Money.zero là phần tử trung hoà của phép cộng', () {
        expect(const Money(7000) + Money.zero, const Money(7000));
      });

      test('trừ được khi kết quả không âm', () {
        expect(const Money(25500) - const Money(500), const Money(25000));
        expect(const Money(500) - const Money(500), Money.zero);
      });

      test('trừ ra số âm ném ArgumentError', () {
        expect(() => const Money(500) - const Money(501), throwsArgumentError);
      });

      test('nhân với số âm ném ArgumentError', () {
        expect(() => const Money(500) * -1, throwsArgumentError);
      });

      test('phép toán không sửa toán hạng', () {
        const a = Money(100);
        const b = Money(50);
        final sum = a + b;
        expect(a.vnd, 100);
        expect(b.vnd, 50);
        expect(identical(sum, a), isFalse);
      });
    });

    group('so sánh', () {
      test('<, <=, >, >= theo số đồng', () {
        const small = Money(12000);
        const large = Money(25500);
        expect(small < large, isTrue);
        expect(large > small, isTrue);
        expect(small <= small, isTrue);
        expect(small >= large, isFalse);
        expect(large < small, isFalse);
      });

      test('compareTo cho phép sắp xếp bằng sort', () {
        final sorted = [const Money(300), const Money(100), const Money(200)]
          ..sort();
        expect(sorted, [const Money(100), const Money(200), const Money(300)]);
      });
    });

    group('bằng nhau theo giá trị', () {
      test('== và hashCode cùng số đồng', () {
        expect(const Money(25500), equals(const Money(25500)));
        expect(const Money(25500).hashCode, const Money(25500).hashCode);
        expect(const Money(25500), isNot(equals(const Money(25501))));
      });

      test('khác kiểu thì không bằng, dù cùng số', () {
        // ignore: unrelated_type_equality_checks
        expect(const Money(100) == 100, isFalse);
      });

      test('dùng được làm phần tử Set và khoá Map', () {
        final set = <Money>{}
          ..add(const Money(1))
          ..add(const Money(1))
          ..add(const Money(2));
        expect(set, hasLength(2));
        expect({const Money(5): 'a'}[const Money(5)], 'a');
      });

      test('const cho cùng một instance', () {
        expect(identical(const Money(0), Money.zero), isTrue);
      });
    });

    group('bất biến khởi tạo', () {
      // assert chỉ chạy khi bật assertions: dart test bật sẵn, dart run phải
      // thêm --enable-asserts, bản release và AOT bỏ qua.
      test('const Money âm bị assert chặn khi bật assertions', () {
        expect(() => Money(int.parse('-1')), throwsA(isA<AssertionError>()));
      });

      test('Money.fromVnd luôn kiểm: âm ném ArgumentError', () {
        expect(() => Money.fromVnd(-1), throwsArgumentError);
        expect(
          () => Money.fromVnd(-1),
          throwsA(isA<ArgumentError>().having((e) => e.name, 'name', 'vnd')),
        );
      });

      test('Money.fromVnd nhận 0 và số dương, bằng Money const', () {
        expect(Money.fromVnd(0), Money.zero);
        expect(Money.fromVnd(25500), const Money(25500));
      });
    });
  });
}
