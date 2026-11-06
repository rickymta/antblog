import 'package:flutter/services.dart' show appFlavor;

import 'flavor.dart';

/// Cấu hình sai. Gom mọi lỗi tìm được một lượt để người sửa không phải chạy
/// lại từng lần một.
final class ConfigException implements Exception {
  ConfigException(Iterable<String> problems)
    : problems = List.unmodifiable(problems);

  final List<String> problems;

  @override
  String toString() =>
      'ConfigException: cấu hình không hợp lệ (${problems.length} lỗi)\n'
      '${problems.map((problem) => ' - $problem').join('\n')}';
}

/// Cấu hình của một bản build, đọc một lần lúc khởi động và bất biến.
///
/// Giá trị đến từ `--dart-define-from-file=config/<flavor>.json`, nên là hằng
/// lúc biên dịch và nằm trong binary: **không để bí mật ở đây**.
final class AppConfig {
  const AppConfig({
    required this.flavor,
    required this.apiBaseUrl,
    required this.logHttp,
    required this.connectTimeout,
    required this.receiveTimeout,
  });

  static const keyFlavor = 'FLAVOR';
  static const keyApiBaseUrl = 'API_BASE_URL';
  static const keyLogHttp = 'LOG_HTTP';
  static const keyConnectTimeoutMs = 'CONNECT_TIMEOUT_MS';
  static const keyReceiveTimeoutMs = 'RECEIVE_TIMEOUT_MS';

  /// Bộ khoá bắt buộc; mỗi file JSON phải có đúng bộ này.
  static const keys = [
    keyFlavor,
    keyApiBaseUrl,
    keyLogHttp,
    keyConnectTimeoutMs,
    keyReceiveTimeoutMs,
  ];

  static const minTimeoutMs = 1000;
  static const maxTimeoutMs = 60000;

  /// Mỗi lời gọi `String.fromEnvironment` phải là `const` và nhận tên khoá là
  /// hằng: chỉ `const` mới được tài liệu đảm bảo. Không `const` thì kết quả
  /// tuỳ nền tảng (trong `flutter test` và `dart compile exe` ta đo được chuỗi
  /// rỗng, âm thầm). Tên khoá lấy từ biến thì không biên dịch được.
  static const Map<String, String> _compileTimeEnvironment = {
    keyFlavor: String.fromEnvironment(keyFlavor),
    keyApiBaseUrl: String.fromEnvironment(keyApiBaseUrl),
    keyLogHttp: String.fromEnvironment(keyLogHttp),
    keyConnectTimeoutMs: String.fromEnvironment(keyConnectTimeoutMs),
    keyReceiveTimeoutMs: String.fromEnvironment(keyReceiveTimeoutMs),
  };

  /// Đọc define của bản build hiện tại và kiểm tra. Ném [ConfigException] nếu
  /// thiếu hoặc sai.
  ///
  /// [nativeFlavor] là flavor mà công cụ gán qua `--flavor` (`appFlavor`);
  /// `null` nghĩa là không build theo flavor native và bỏ qua phép so khớp.
  factory AppConfig.fromEnvironment({String? nativeFlavor = appFlavor}) =>
      AppConfig.fromMap(_compileTimeEnvironment, nativeFlavor: nativeFlavor);

  /// Dựng từ bảng khoá-giá trị chuỗi (cùng dạng công cụ Flutter gửi cho
  /// compiler). Là hàm thuần nên test được mọi ca mà không cần build lại.
  factory AppConfig.fromMap(Map<String, String> env, {String? nativeFlavor}) {
    final problems = <String>[];

    String? read(String key) {
      final value = env[key];
      if (value == null || value.trim().isEmpty) {
        problems.add('thiếu khoá $key (hoặc giá trị rỗng)');
        return null;
      }
      return value;
    }

    final flavor = _flavor(read(keyFlavor), nativeFlavor, problems);
    final apiBaseUrl = _baseUrl(read(keyApiBaseUrl), flavor, problems);
    final logHttp = _bool(keyLogHttp, read(keyLogHttp), problems);
    final connectTimeout = _timeout(
      keyConnectTimeoutMs,
      read(keyConnectTimeoutMs),
      problems,
    );
    final receiveTimeout = _timeout(
      keyReceiveTimeoutMs,
      read(keyReceiveTimeoutMs),
      problems,
    );

    if (flavor == Flavor.prod && logHttp == true) {
      problems.add('prod không được bật $keyLogHttp');
    }
    if (problems.isNotEmpty) {
      throw ConfigException(problems);
    }
    return AppConfig(
      flavor: flavor!,
      apiBaseUrl: apiBaseUrl!,
      logHttp: logHttp!,
      connectTimeout: connectTimeout!,
      receiveTimeout: receiveTimeout!,
    );
  }

  final Flavor flavor;
  final Uri apiBaseUrl;
  final bool logHttp;
  final Duration connectTimeout;
  final Duration receiveTimeout;

  bool get isProd => flavor == Flavor.prod;

  static Flavor? _flavor(
    String? raw,
    String? nativeFlavor,
    List<String> problems,
  ) {
    if (raw == null) {
      return null;
    }
    final flavor = Flavor.tryParse(raw);
    if (flavor == null) {
      problems.add('$keyFlavor chỉ nhận dev, staging hoặc prod');
      return null;
    }
    if (nativeFlavor != null && nativeFlavor != flavor.name) {
      problems.add(
        'build bằng --flavor $nativeFlavor nhưng $keyFlavor trong file cấu '
        'hình là ${flavor.name}: hai nguồn này không khớp',
      );
    }
    return flavor;
  }

  /// Không in lại giá trị của URL trong thông báo: URL sai có thể mang
  /// `user:password@` hoặc `?key=...`, và thông báo này sẽ lên màn hình, log.
  static Uri? _baseUrl(String? raw, Flavor? flavor, List<String> problems) {
    if (raw == null) {
      return null;
    }
    final uri = Uri.tryParse(raw);
    final isHttp =
        uri != null && (uri.scheme == 'http' || uri.scheme == 'https');
    if (uri == null || !isHttp || uri.host.isEmpty) {
      problems.add('$keyApiBaseUrl phải là địa chỉ http(s) tuyệt đối có host');
      return null;
    }
    if (uri.userInfo.isNotEmpty) {
      problems.add('$keyApiBaseUrl không được chứa user:password');
    }
    if (uri.hasQuery || uri.hasFragment) {
      problems.add('$keyApiBaseUrl không được kèm query hoặc fragment');
    }
    if (flavor != null && !flavor.allowsLocalBackend) {
      if (uri.scheme != 'https') {
        problems.add('${flavor.name} phải dùng https cho $keyApiBaseUrl');
      }
      if (isLocalHost(uri.host)) {
        problems.add(
          '${flavor.name} không được trỏ $keyApiBaseUrl tới máy cục bộ '
          '(host ${uri.host})',
        );
      }
    }
    return uri;
  }

  static bool? _bool(String key, String? raw, List<String> problems) {
    if (raw == null) {
      return null;
    }
    if (raw == 'true') {
      return true;
    }
    if (raw == 'false') {
      return false;
    }
    problems.add('$key chỉ nhận true hoặc false');
    return null;
  }

  static Duration? _timeout(String key, String? raw, List<String> problems) {
    if (raw == null) {
      return null;
    }
    final millis = int.tryParse(raw);
    if (millis == null || millis < minTimeoutMs || millis > maxTimeoutMs) {
      problems.add(
        '$key phải là số nguyên mili giây trong khoảng '
        '$minTimeoutMs..$maxTimeoutMs',
      );
      return null;
    }
    return Duration(milliseconds: millis);
  }

  @override
  bool operator ==(Object other) =>
      other is AppConfig &&
      other.flavor == flavor &&
      other.apiBaseUrl == apiBaseUrl &&
      other.logHttp == logHttp &&
      other.connectTimeout == connectTimeout &&
      other.receiveTimeout == receiveTimeout;

  @override
  int get hashCode =>
      Object.hash(flavor, apiBaseUrl, logHttp, connectTimeout, receiveTimeout);

  @override
  String toString() =>
      'AppConfig(${flavor.name}, $apiBaseUrl, logHttp: $logHttp, '
      'connect: ${connectTimeout.inMilliseconds} ms, '
      'receive: ${receiveTimeout.inMilliseconds} ms)';
}

final _ipv4 = RegExp(r'^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$');

/// `true` cho host chỉ có nghĩa trên mạng cục bộ: `localhost`, tên `.local`,
/// loopback, dải riêng của IPv4 (gồm `10.0.2.2` mà Android emulator dùng để
/// gọi máy dev) và vài dạng IPv6 cục bộ.
bool isLocalHost(String host) {
  final name = host.toLowerCase();
  if (name == 'localhost' ||
      name.endsWith('.localhost') ||
      name.endsWith('.local')) {
    return true;
  }
  final match = _ipv4.firstMatch(name);
  if (match != null) {
    final a = int.parse(match[1]!);
    final b = int.parse(match[2]!);
    return a == 0 ||
        a == 10 ||
        a == 127 ||
        (a == 169 && b == 254) ||
        (a == 172 && b >= 16 && b <= 31) ||
        (a == 192 && b == 168);
  }
  if (name.contains(':')) {
    return name == '::1' ||
        name.startsWith('fe80') ||
        name.startsWith('fc') ||
        name.startsWith('fd');
  }
  return false;
}
