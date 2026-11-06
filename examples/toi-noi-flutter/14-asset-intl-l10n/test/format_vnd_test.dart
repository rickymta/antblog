import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_14/formatting.dart';

/// Khoảng trắng không ngắt dòng (U+00A0) mà intl chèn giữa số và ký hiệu ₫
/// trong locale vi. Nhìn giống dấu cách nhưng là ký tự khác.
const nbsp = '\u00A0';

void main() {
  group('formatVnd, locale vi', () {
    test('25.500 ₫: dấu chấm nghìn, ký hiệu đứng sau, cách bằng U+00A0', () {
      expect(formatVnd(const Money(25500), 'vi'), '25.500$nbsp₫');
    });

    test('khoảng trắng là NBSP, không phải dấu cách thường', () {
      final text = formatVnd(const Money(25500), 'vi');
      expect(text.contains(' '), isFalse);
      expect(text.contains(nbsp), isTrue);
      expect(text, isNot('25.500 ₫'));
    });

    test('0 đồng, số tròn trăm nghìn và hàng triệu', () {
      expect(formatVnd(Money.zero, 'vi'), '0$nbsp₫');
      expect(formatVnd(const Money(453000), 'vi'), '453.000$nbsp₫');
      expect(formatVnd(const Money(1234567), 'vi'), '1.234.567$nbsp₫');
    });

    test('vi_VN cho cùng kết quả với vi', () {
      expect(
        formatVnd(const Money(25500), 'vi_VN'),
        formatVnd(const Money(25500), 'vi'),
      );
    });

    test('không có phần lẻ: mọi kết quả kết thúc bằng ₫ ngay sau chữ số', () {
      for (final vnd in [0, 1, 999, 1000, 12005]) {
        expect(
          formatVnd(Money(vnd), 'vi'),
          matches(RegExp('^[0-9.]+$nbsp₫\$')),
        );
      }
    });
  });

  group('formatVnd, locale en', () {
    test(
      '₫25,500: ký hiệu đứng trước, dấu phẩy nghìn, không có khoảng trắng',
      () {
        expect(formatVnd(const Money(25500), 'en'), '₫25,500');
      },
    );

    test('0 đồng và hàng triệu', () {
      expect(formatVnd(Money.zero, 'en'), '₫0');
      expect(formatVnd(const Money(1234567), 'en'), '₫1,234,567');
    });

    test('en_US cho cùng kết quả với en', () {
      expect(
        formatVnd(const Money(453000), 'en_US'),
        formatVnd(const Money(453000), 'en'),
      );
    });
  });

  group('formatVnd với cước tính bằng calculateFare', () {
    // Số tiền lấy từ công thức của toi_noi_core, không gõ tay.
    test('5 km, 2,001 km và 100 km', () {
      expect(formatVnd(calculateFare(5), 'vi'), '25.500$nbsp₫');
      expect(formatVnd(calculateFare(2.001), 'vi'), '12.005$nbsp₫');
      expect(formatVnd(calculateFare(100), 'vi'), '453.000$nbsp₫');
    });
  });

  group('so với Money.toString', () {
    test('toString giữ "VND" và dấu cách thường, formatVnd thì không', () {
      const money = Money(25500);
      expect(money.toString(), '25.500 VND');
      expect(formatVnd(money, 'vi'), isNot(money.toString()));
    });

    test('hai locale cùng số tiền cho hai chuỗi khác nhau', () {
      const money = Money(25500);
      expect(formatVnd(money, 'vi'), isNot(formatVnd(money, 'en')));
    });
  });

  group('locale không hợp lệ', () {
    test('ném ArgumentError thay vì lặng lẽ dùng locale khác', () {
      expect(() => formatVnd(const Money(25500), 'xx_YY'), throwsArgumentError);
    });
  });
}
