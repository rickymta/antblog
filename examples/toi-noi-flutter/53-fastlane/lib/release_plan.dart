/// Chọn lane fastlane theo flavor của Tới Nơi.
///
/// Phần này thuần Dart, không gọi fastlane. Fastfile trong `native-config/` là
/// phần thi hành; test đối chiếu hai bên để chúng không lệch nhau.
library;

/// Ba flavor cố định từ bài 25.
enum Flavor { dev, staging, prod }

/// Hai app của Tới Nơi.
enum AppKind { customer, driver }

/// Hai cửa hàng ứng dụng, mỗi cửa hàng có một thư mục fastlane riêng.
enum StorePlatform { ios, android }

/// Application ID (Android) và Bundle ID (iOS): cùng một chuỗi, theo bảng của
/// curriculum §3.2. Flavor `prod` không có hậu tố.
String applicationId(AppKind app, Flavor flavor) => switch (flavor) {
  Flavor.prod => 'vn.toinoi.${app.name}',
  Flavor.dev || Flavor.staging => 'vn.toinoi.${app.name}.${flavor.name}',
};

/// Một lane fastlane có thể chạy: `fastlane <platform> <lane>`.
final class LaneSpec {
  const LaneSpec({
    required this.platform,
    required this.lane,
    required this.requiredEnv,
    this.track,
  });

  final StorePlatform platform;
  final String lane;

  /// Track của Google Play mà lane đẩy lên; `null` với lane iOS.
  final String? track;

  /// Biến môi trường lane đọc; thiếu một biến là lane dừng trước khi build.
  final Set<String> requiredEnv;
}

/// Lane iOS: `match` chỉ đọc, build archive, `gym` xuất IPA, đẩy lên TestFlight.
const LaneSpec iosBeta = LaneSpec(
  platform: StorePlatform.ios,
  lane: 'beta',
  requiredEnv: {
    'ASC_KEY_ID',
    'ASC_ISSUER_ID',
    'ASC_KEY_CONTENT',
    'MATCH_GIT_URL',
    'MATCH_PASSWORD',
  },
);

/// Lane Android: build AAB, đẩy lên track `internal` của Google Play.
const LaneSpec androidInternal = LaneSpec(
  platform: StorePlatform.android,
  lane: 'internal',
  track: 'internal',
  requiredEnv: {'PLAY_JSON_KEY_DATA'},
);

/// Biến xác thực vào repo `match` trên CI: HTTPS dùng biến đầu, deploy key SSH
/// dùng biến sau. Không bắt buộc ở mọi nơi (máy cá nhân có thể đã có SSH hoặc
/// credential helper), nên không nằm trong [requiredEnv].
const Set<String> matchRepoAuthEnv = {
  'MATCH_GIT_BASIC_AUTHORIZATION',
  'MATCH_GIT_PRIVATE_KEY',
};

/// Lane quản trị chỉ chạy trên máy người có quyền, không bao giờ trên CI.
const LaneSpec iosSyncCerts = LaneSpec(
  platform: StorePlatform.ios,
  lane: 'sync_certs',
  requiredEnv: {
    'ASC_KEY_ID',
    'ASC_ISSUER_ID',
    'ASC_KEY_CONTENT',
    'MATCH_GIT_URL',
    'MATCH_PASSWORD',
  },
);

/// Lane phân phối của [flavor].
///
/// `dev` chỉ chạy trên máy lập trình viên (`flutter run`), không qua cửa hàng,
/// nên trả danh sách rỗng. `staging` và `prod` cùng đi qua TestFlight và Play
/// internal: App Store và các track Play sau đó dùng lại bản build đã tải lên
/// (bài 56 đến 58), nên bước đẩy đầu tiên của hai flavor giống nhau.
List<LaneSpec> lanesFor(Flavor flavor) => switch (flavor) {
  Flavor.dev => const [],
  Flavor.staging || Flavor.prod => const [iosBeta, androidInternal],
};

/// Biến môi trường mà mọi lane của [platform] cần, gộp lại.
Set<String> requiredEnv(StorePlatform platform) => {
  for (final spec in [iosBeta, androidInternal, iosSyncCerts])
    if (spec.platform == platform) ...spec.requiredEnv,
};

/// Biến môi trường của [lanes] còn thiếu trong [env] (rỗng cũng tính là thiếu).
Set<String> missingEnv(Iterable<LaneSpec> lanes, Map<String, String> env) => {
  for (final spec in lanes)
    for (final name in spec.requiredEnv)
      if ((env[name] ?? '').trim().isEmpty) name,
};

/// Một lệnh cần chạy, kèm thư mục chạy tính từ gốc project Flutter.
final class LaneInvocation {
  const LaneInvocation({
    required this.spec,
    required this.app,
    required this.flavor,
    required this.buildNumber,
  });

  final LaneSpec spec;
  final AppKind app;
  final Flavor flavor;
  final int buildNumber;

  /// `ios` hoặc `android`: nơi có `Gemfile` và thư mục `fastlane/`.
  String get workingDirectory => spec.platform.name;

  String get command =>
      'bundle exec fastlane ${spec.platform.name} ${spec.lane} '
      'app:${app.name} flavor:${flavor.name} build_number:$buildNumber';

  @override
  String toString() => '(cd $workingDirectory && $command)';
}

/// Danh sách lệnh phát hành [app] ở [flavor] với [buildNumber].
///
/// [only] giới hạn một cửa hàng (ví dụ CI chạy iOS trên macOS, Android trên
/// ubuntu ở hai job riêng, bài 54). Ném [ArgumentError] khi [buildNumber]
/// không dương: `versionCode` của Play và build number của App Store Connect
/// đều phải là số nguyên lớn hơn 0.
List<LaneInvocation> planRelease({
  required Flavor flavor,
  required AppKind app,
  required int buildNumber,
  StorePlatform? only,
}) {
  if (buildNumber < 1) {
    throw ArgumentError.value(
      buildNumber,
      'buildNumber',
      'phải là số nguyên dương',
    );
  }
  return [
    for (final spec in lanesFor(flavor))
      if (only == null || spec.platform == only)
        LaneInvocation(
          spec: spec,
          app: app,
          flavor: flavor,
          buildNumber: buildNumber,
        ),
  ];
}
