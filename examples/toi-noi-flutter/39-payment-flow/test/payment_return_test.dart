import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_39/payment_return.dart';

void main() {
  group('PaymentReturn.tryParse', () {
    test('đọc mã phiên và lời khẳng định chưa xác thực', () {
      final ret = PaymentReturn.tryParse(
        Uri.parse('toinoi://payment/return?session=s-1&status=success'),
      );
      expect(ret!.sessionId, 's-1');
      expect(ret.claimed, 'success');
    });

    test('status là tuỳ chọn', () {
      final ret = PaymentReturn.tryParse(
        Uri.parse('toinoi://payment/return?session=s-1'),
      );
      expect(ret!.sessionId, 's-1');
      expect(ret.claimed, isNull);
    });

    test('scheme và host không phân biệt hoa thường (Uri chuẩn hoá)', () {
      expect(
        PaymentReturn.tryParse(
          Uri.parse('TOINOI://Payment/return?session=s-1'),
        ),
        isNotNull,
      );
    });

    test('không phải link quay về: scheme, host hoặc đường dẫn khác', () {
      for (final raw in [
        'https://toinoi.example/payment/return?session=s-1',
        'https://payment/return?session=s-1',
        'toinoi://trip/T-1002',
        'toinoi://payment/other?session=s-1',
        'toinoi://payment/return/extra?session=s-1',
        'toinoi://evil/return?session=s-1',
      ]) {
        expect(PaymentReturn.tryParse(Uri.parse(raw)), isNull, reason: raw);
      }
    });

    test('thiếu hoặc rỗng mã phiên thì từ chối', () {
      for (final raw in [
        'toinoi://payment/return',
        'toinoi://payment/return?status=success',
        'toinoi://payment/return?session=',
      ]) {
        expect(PaymentReturn.tryParse(Uri.parse(raw)), isNull, reason: raw);
      }
    });

    test('matches chỉ xét scheme, host, đường dẫn (không xét tham số)', () {
      expect(PaymentReturn.matches(Uri.parse('toinoi://payment/return')), true);
      expect(
        PaymentReturn.matches(Uri.parse('toinoi://payment/return?x=1')),
        true,
      );
      expect(PaymentReturn.matches(Uri.parse('toinoi://payment/')), false);
    });
  });
}
