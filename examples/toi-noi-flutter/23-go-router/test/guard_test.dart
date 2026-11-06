import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_23/auth.dart';
import 'package:toi_noi_23/guard.dart';

const _alice = Session(userName: 'An');

Future<Session?> _signedOut() async => null;
Future<Session?> _signedIn() async => _alice;

void main() {
  group('requiresLogin', () {
    test('trang chủ, /login, /not-found xem được khi chưa đăng nhập', () {
      expect(requiresLogin('/'), isFalse);
      expect(requiresLogin('/login'), isFalse);
      expect(requiresLogin('/not-found'), isFalse);
    });

    test('mọi đường dẫn khác cần đăng nhập, kể cả đường dẫn lạ', () {
      expect(requiresLogin('/history'), isTrue);
      expect(requiresLogin('/account'), isTrue);
      expect(requiresLogin('/trip/T-1002'), isTrue);
      expect(requiresLogin('/admin'), isTrue);
    });
  });

  group('safeFrom', () {
    test('đường dẫn nội bộ được giữ nguyên, kể cả query', () {
      expect(safeFrom('/history'), '/history');
      expect(safeFrom('/trip/T-1002'), '/trip/T-1002');
      expect(safeFrom('/trip/T-1002?ref=push'), '/trip/T-1002?ref=push');
    });

    test('thiếu hoặc rỗng: về trang chủ', () {
      expect(safeFrom(null), '/');
      expect(safeFrom(''), '/');
    });

    test('link ngoài hoặc có scheme: về trang chủ', () {
      expect(safeFrom('https://evil.example/trip/1'), '/');
      expect(safeFrom('toinoi://trip/T-1002'), '/');
      expect(safeFrom('javascript:alert(1)'), '/');
    });

    test('//host và đường dẫn tương đối: về trang chủ', () {
      expect(safeFrom('//evil.example/x'), '/');
      expect(safeFrom('trip/T-1002'), '/');
    });

    test('chính /login: về trang chủ, tránh vòng lặp', () {
      expect(safeFrom('/login'), '/');
      expect(safeFrom('/login?from=%2Fhistory'), '/');
    });
  });

  group('guardLocation', () {
    test('chưa đăng nhập vào trang riêng: sang /login và nhớ from', () async {
      expect(
        await guardLocation(Uri.parse('/history'), _signedOut),
        '/login?from=%2Fhistory',
      );
      expect(
        await guardLocation(Uri.parse('/trip/T-1002?ref=push'), _signedOut),
        '/login?from=%2Ftrip%2FT-1002%3Fref%3Dpush',
      );
    });

    test('trang công khai: cho vào, không đọc phiên', () async {
      var reads = 0;
      Future<Session?> counting() async {
        reads++;
        return null;
      }

      expect(await guardLocation(Uri.parse('/'), counting), isNull);
      expect(await guardLocation(Uri.parse('/not-found'), counting), isNull);
      expect(reads, 0, reason: 'không chờ kho phiên chỉ để mở trang chủ');
    });

    test('chưa đăng nhập ở /login: ở lại', () async {
      expect(
        await guardLocation(Uri.parse('/login?from=%2Fhistory'), _signedOut),
        isNull,
      );
    });

    test('đã đăng nhập ở /login: tới from, hoặc trang chủ', () async {
      expect(
        await guardLocation(Uri.parse('/login?from=%2Fhistory'), _signedIn),
        '/history',
      );
      expect(await guardLocation(Uri.parse('/login'), _signedIn), '/');
      expect(
        await guardLocation(
          Uri.parse('/login?from=%2F%2Fevil.example'),
          _signedIn,
        ),
        '/',
      );
    });

    test('đã đăng nhập vào trang riêng: cho vào', () async {
      expect(await guardLocation(Uri.parse('/history'), _signedIn), isNull);
      expect(await guardLocation(Uri.parse('/trip/T-1002'), _signedIn), isNull);
    });
  });
}
