/// Các đường dẫn nội bộ của app. Một chỗ duy nhất để đổi tên route.
abstract final class AppPaths {
  static const booking = '/';
  static const history = '/history';
  static const account = '/account';
  static const login = '/login';
  static const notFound = '/not-found';

  /// `/trip/T-1002`. Mã được mã hoá để ký tự lạ không làm vỡ đường dẫn.
  static String trip(String id) => '/trip/${Uri.encodeComponent(id)}';

  /// `/login?from=%2Fhistory`: nơi người dùng định tới trước khi bị chặn.
  static String loginFor(String from) =>
      Uri(path: login, queryParameters: {'from': from}).toString();

  /// `/not-found?uri=...`: kèm đường dẫn không hiểu được để hiển thị.
  static String notFoundFor(Uri uri) =>
      Uri(path: notFound, queryParameters: {'uri': uri.toString()}).toString();
}
