import 'dart:convert';

import 'package:toi_noi_53/release_plan.dart'
    show AppKind, Flavor, applicationId;

/// Hai nền tảng mà App Distribution nhận.
enum DistributionPlatform { android, ios }

/// Lỗi cấu hình hoặc yêu cầu phân phối không hợp lệ.
final class DistributionException implements Exception {
  const DistributionException(this.message);

  final String message;

  @override
  String toString() => 'DistributionException: $message';
}

/// Một cặp app + nền tảng đã đăng ký trong Firebase, sẵn sàng nhận bản build.
final class DistributionTarget {
  const DistributionTarget({
    required this.app,
    required this.platform,
    required this.firebaseAppId,
    required this.applicationId,
    required this.groupAlias,
  });

  final AppKind app;
  final DistributionPlatform platform;

  /// App ID của Firebase, dạng `1:<số dự án>:<android|ios>:<mã>`.
  final String firebaseAppId;

  /// Application ID (Android) hoặc Bundle ID (iOS).
  final String applicationId;

  /// Alias của nhóm tester, ví dụ `qa-noi-bo`.
  final String groupAlias;

  /// `true` khi App ID là giá trị mẫu (mã bắt đầu bằng `FAKE`).
  bool get isPlaceholder => firebaseAppId.split(':').last.startsWith('FAKE');
}

/// Cấu hình phân phối nội bộ đọc từ `app-distribution.json`.
///
/// Chỉ flavor `staging` có thể xuất hiện trong file: `dev` chỉ chạy trên máy
/// lập trình viên và `prod` đi qua cửa hàng (curriculum §3.2).
final class DistributionConfig {
  DistributionConfig._(this._targets, this.projectId, this.groupAlias);

  /// Đọc và kiểm tra [source]. Ném [DistributionException] ở lỗi đầu tiên.
  factory DistributionConfig.parse(String source) {
    final Object? decoded;
    try {
      decoded = jsonDecode(source);
    } on FormatException catch (e) {
      throw DistributionException('JSON không hợp lệ: ${e.message}');
    }
    final root = _map(decoded, 'gốc');

    final projectId = _string(root, 'firebaseProjectId', 'gốc');
    final groups = _map(root['testerGroups'], 'testerGroups');
    final internal = _map(groups['internal'], 'testerGroups.internal');
    final alias = _string(internal, 'alias', 'testerGroups.internal');
    if (!_alias.hasMatch(alias)) {
      throw DistributionException(
        'alias nhóm "$alias" chỉ gồm chữ thường, số và dấu gạch nối',
      );
    }

    final targets = <(AppKind, DistributionPlatform), DistributionTarget>{};
    final apps = _map(root['apps'], 'apps');
    String? projectNumber;
    for (final appEntry in apps.entries) {
      final app = AppKind.values.where((a) => a.name == appEntry.key);
      if (app.isEmpty) {
        throw DistributionException('app "${appEntry.key}" không tồn tại');
      }
      final flavors = _map(appEntry.value, 'apps.${appEntry.key}');
      for (final flavorEntry in flavors.entries) {
        if (flavorEntry.key != Flavor.staging.name) {
          throw DistributionException(
            'chỉ flavor staging được phân phối, thấy "${flavorEntry.key}" '
            'trong apps.${appEntry.key}',
          );
        }
        final platforms = _map(
          flavorEntry.value,
          'apps.${appEntry.key}.staging',
        );
        for (final platformEntry in platforms.entries) {
          final platform = DistributionPlatform.values.where(
            (p) => p.name == platformEntry.key,
          );
          if (platform.isEmpty) {
            throw DistributionException(
              'nền tảng "${platformEntry.key}" không tồn tại',
            );
          }
          final where = 'apps.${appEntry.key}.staging.${platformEntry.key}';
          final entry = _map(platformEntry.value, where);
          final appId = _string(entry, 'appId', where);
          final match = _firebaseAppId.firstMatch(appId);
          if (match == null || match.group(2) != platform.single.name) {
            throw DistributionException(
              '$where.appId "$appId" không đúng dạng '
              '1:<số dự án>:${platform.single.name}:<mã>',
            );
          }
          projectNumber ??= match.group(1);
          if (match.group(1) != projectNumber) {
            throw DistributionException(
              '$where.appId thuộc dự án khác (${match.group(1)} thay vì '
              '$projectNumber)',
            );
          }
          // Application ID phải là giá trị mà bài 50 cố định cho flavor staging.
          final idKey = platform.single == DistributionPlatform.android
              ? 'packageName'
              : 'bundleId';
          final declared = _string(entry, idKey, where);
          final expected = applicationId(app.single, Flavor.staging);
          if (declared != expected) {
            throw DistributionException(
              '$where.$idKey là "$declared", curriculum §3.2 quy định '
              '"$expected"',
            );
          }
          targets[(app.single, platform.single)] = DistributionTarget(
            app: app.single,
            platform: platform.single,
            firebaseAppId: appId,
            applicationId: expected,
            groupAlias: alias,
          );
        }
      }
    }
    if (targets.isEmpty) {
      throw const DistributionException('apps không có mục nào');
    }
    return DistributionConfig._(targets, projectId, alias);
  }

  final Map<(AppKind, DistributionPlatform), DistributionTarget> _targets;

  /// ID dự án Firebase (chỉ để đọc, App Distribution dùng App ID).
  final String projectId;

  /// Alias nhóm tester nội bộ.
  final String groupAlias;

  /// Mọi mục đã khai báo.
  Iterable<DistributionTarget> get targets => _targets.values;

  /// Mục của [app] trên [platform] ở [flavor].
  ///
  /// Ném [DistributionException] với flavor khác `staging`, hoặc khi app chưa
  /// đăng ký nền tảng đó.
  DistributionTarget target(
    AppKind app,
    Flavor flavor,
    DistributionPlatform platform,
  ) {
    if (flavor != Flavor.staging) {
      throw DistributionException(
        'flavor ${flavor.name} không đi qua App Distribution: dev chạy trên '
        'máy lập trình viên, prod đi qua cửa hàng',
      );
    }
    final found = _targets[(app, platform)];
    if (found == null) {
      throw DistributionException(
        '${app.name} chưa khai báo ${platform.name} trong cấu hình',
      );
    }
    return found;
  }

  /// Tham số cho `firebase`, theo trang CLI của App Distribution:
  /// `appdistribution:distribute <file> --app ... --groups ... --release-notes-file ...`.
  List<String> cliArguments(
    DistributionTarget target, {
    required String binaryPath,
    required String releaseNotesFile,
  }) {
    final extension = binaryPath.split('.').last.toLowerCase();
    final allowed = switch (target.platform) {
      DistributionPlatform.android => const {'apk', 'aab'},
      DistributionPlatform.ios => const {'ipa'},
    };
    if (!allowed.contains(extension)) {
      throw DistributionException(
        'file .$extension không hợp lệ cho ${target.platform.name}',
      );
    }
    return [
      'appdistribution:distribute',
      binaryPath,
      '--app',
      target.firebaseAppId,
      '--groups',
      target.groupAlias,
      '--release-notes-file',
      releaseNotesFile,
    ];
  }
}

final RegExp _alias = RegExp(r'^[a-z0-9]+(-[a-z0-9]+)*$');
final RegExp _firebaseAppId = RegExp(
  r'^1:(\d{1,20}):(android|ios):([A-Za-z0-9-]+)$',
);

Map<String, Object?> _map(Object? value, String where) {
  if (value is Map<String, Object?>) return value;
  throw DistributionException('$where phải là một object JSON');
}

String _string(Map<String, Object?> map, String key, String where) {
  final value = map[key];
  if (value is String && value.trim().isNotEmpty) return value;
  throw DistributionException('$where.$key phải là chuỗi không rỗng');
}
