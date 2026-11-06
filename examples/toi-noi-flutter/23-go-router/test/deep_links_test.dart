import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_23/deep_links.dart';

void main() {
  group('DeepLinks.resolve', () {
    test(
      'custom scheme: host là tên màn hình, được ghép lại thành đường dẫn',
      () {
        final uri = Uri.parse('toinoi://trip/T-1002');
        // Bằng chứng vì sao cần ghép: path chỉ còn phần sau host.
        expect(uri.host, 'trip');
        expect(uri.path, '/T-1002');
        expect(DeepLinks.resolve(uri), '/trip/T-1002');
      },
    );

    test('custom scheme giữ query', () {
      expect(
        DeepLinks.resolve(Uri.parse('toinoi://trip/T-1002?ref=push')),
        '/trip/T-1002?ref=push',
      );
    });

    test('custom scheme với host lạ: màn hình không tìm thấy', () {
      expect(
        DeepLinks.resolve(Uri.parse('toinoi://payment/confirm')),
        '/not-found?uri=toinoi%3A%2F%2Fpayment%2Fconfirm',
      );
    });

    test(
      'custom scheme thiếu mã chuyến: ghép ra /trip để router báo không khớp',
      () {
        expect(DeepLinks.resolve(Uri.parse('toinoi://trip')), '/trip');
      },
    );

    test('universal link của ta: bỏ origin, giữ đường dẫn và query', () {
      expect(
        DeepLinks.resolve(Uri.parse('https://toinoi.example/trip/T-1002')),
        '/trip/T-1002',
      );
      expect(
        DeepLinks.resolve(Uri.parse('https://toinoi.example/trip/T-1002?a=1')),
        '/trip/T-1002?a=1',
      );
      expect(DeepLinks.resolve(Uri.parse('https://toinoi.example')), '/');
    });

    test('https với tên miền khác hoặc scheme lạ: không phải link của ta', () {
      expect(
        DeepLinks.resolve(Uri.parse('https://evil.example/trip/T-1002')),
        startsWith('/not-found?uri='),
      );
      expect(
        DeepLinks.resolve(Uri.parse('ftp://toinoi.example/trip/T-1002')),
        startsWith('/not-found?uri='),
      );
    });

    test('đường dẫn tương đối: không phải deep link, trả về null', () {
      expect(DeepLinks.resolve(Uri.parse('/trip/T-1002')), isNull);
      expect(DeepLinks.resolve(Uri.parse('/')), isNull);
    });

    test('customTrip và webTrip dựng đúng link, mã lạ được mã hoá', () {
      expect(DeepLinks.customTrip('T-1002').toString(), 'toinoi://trip/T-1002');
      expect(
        DeepLinks.webTrip('T-1002').toString(),
        'https://toinoi.example/trip/T-1002',
      );
      expect(DeepLinks.customTrip('a/b').toString(), 'toinoi://trip/a%2Fb');
    });
  });
}
