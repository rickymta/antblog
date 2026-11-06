import 'auth.dart';
import 'deep_links.dart';
import 'paths.dart';

/// Các đường dẫn xem được khi chưa đăng nhập. Mọi đường dẫn khác cần đăng
/// nhập: quên khai báo thì app chặn thay vì để lộ.
const _publicPaths = {AppPaths.booking, AppPaths.login, AppPaths.notFound};

bool requiresLogin(String path) => !_publicPaths.contains(path);

/// Chỉ nhận `from` là đường dẫn nội bộ. Mọi thứ khác (link ngoài, scheme,
/// `//host`, chính `/login`) quay về trang chủ, để tham số `from` do người
/// khác tạo ra không đưa người dùng đi nơi lạ sau khi đăng nhập.
String safeFrom(String? from) {
  if (from == null || !from.startsWith('/') || from.startsWith('//')) {
    return AppPaths.booking;
  }
  final uri = Uri.tryParse(from);
  if (uri == null || uri.hasScheme || uri.hasAuthority) {
    return AppPaths.booking;
  }
  if (uri.path == AppPaths.login) return AppPaths.booking;
  return from;
}

/// Đường dẫn + query của [uri], bỏ scheme và host.
String _locationOf(Uri uri) =>
    Uri(path: uri.path, query: uri.hasQuery ? uri.query : null).toString();

/// Hàm `redirect` ở cấp router. Trả về `null` nghĩa là "cho vào".
///
/// [readSession] là hàm async vì lúc khởi động phiên còn đang được đọc từ
/// kho; redirect chờ đọc xong thay vì đoán.
Future<String?> guardLocation(
  Uri uri,
  Future<Session?> Function() readSession,
) async {
  // 1. Link vào app: đổi thành đường dẫn nội bộ rồi để vòng redirect chạy
  //    lại với đường dẫn mới (go_router lặp tới khi redirect trả null).
  final link = DeepLinks.resolve(uri);
  if (link != null) return link;

  // 2. Trang công khai (trừ /login): vào luôn, không chờ kho phiên.
  final path = uri.path;
  if (path != AppPaths.login && !requiresLogin(path)) return null;

  final signedIn = await readSession() != null;

  // 3. Đã đăng nhập mà còn ở /login: đi tiếp tới nơi định tới.
  if (path == AppPaths.login) {
    return signedIn ? safeFrom(uri.queryParameters['from']) : null;
  }

  // 4. Chưa đăng nhập mà vào trang riêng: sang /login, nhớ nơi định tới.
  if (!signedIn && requiresLogin(path)) {
    return AppPaths.loginFor(_locationOf(uri));
  }
  return null;
}
