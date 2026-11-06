import 'paths.dart';

/// Quy ước link vào app. Hai dạng cùng tới một màn hình:
///
/// - custom scheme: `toinoi://trip/T-1002`
/// - universal link / app link: `https://toinoi.example/trip/T-1002`
abstract final class DeepLinks {
  static const scheme = 'toinoi';

  /// Tên miền hư cấu của lab; phải khớp `native-config/`.
  static const webHost = 'toinoi.example';

  /// Với custom scheme, "host" của URI là tên màn hình.
  static const _customHosts = {'trip'};

  static Uri customTrip(String id) =>
      Uri(scheme: scheme, host: 'trip', pathSegments: [id]);

  static Uri webTrip(String id) =>
      Uri(scheme: 'https', host: webHost, pathSegments: ['trip', id]);

  /// Đổi một URI tuyệt đối thành đường dẫn nội bộ; `null` nếu [uri] đã là
  /// đường dẫn tương đối (không phải deep link, không cần đổi).
  ///
  /// go_router khớp route theo `uri.path` và bỏ qua scheme lẫn host. Với
  /// `toinoi://trip/T-1002`, `host` là `trip` và `path` chỉ còn `/T-1002`,
  /// không khớp route nào, nên ta phải ghép lại trước khi router tìm route.
  static String? resolve(Uri uri) {
    if (!uri.hasScheme) return null;
    final query = uri.hasQuery ? '?${uri.query}' : '';

    if (uri.scheme == scheme) {
      if (!_customHosts.contains(uri.host)) {
        return AppPaths.notFoundFor(uri);
      }
      return '/${uri.host}${uri.path}$query';
    }

    final isWeb = uri.scheme == 'https' || uri.scheme == 'http';
    if (isWeb && uri.host == webHost) {
      return '${uri.path.isEmpty ? '/' : uri.path}$query';
    }
    // Scheme lạ hoặc tên miền khác: không phải link của ta.
    return AppPaths.notFoundFor(uri);
  }
}
