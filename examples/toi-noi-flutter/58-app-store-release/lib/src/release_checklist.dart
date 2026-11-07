import 'dart:convert';

import 'metadata.dart';
import 'privacy_manifest.dart';
import 'screenshot.dart';
import 'store_limits.dart';

/// Hai app của Tới Nơi trên App Store: mỗi app một bản ghi riêng, nên mỗi app
/// có metadata, demo account và privacy label riêng.
enum StoreApp {
  customer('vn.toinoi.customer'),
  driver('vn.toinoi.driver');

  const StoreApp(this.bundleId);

  /// Bundle ID bản `prod` (curriculum §3.2).
  final String bundleId;
}

enum Severity { error, warning }

/// Một phát hiện của checklist. [code] ổn định để test so khớp.
final class Finding {
  const Finding(this.code, this.severity, this.message);
  const Finding.error(String code, String message)
    : this(code, Severity.error, message);
  const Finding.warning(String code, String message)
    : this(code, Severity.warning, message);

  final String code;
  final Severity severity;
  final String message;

  @override
  String toString() => '${severity.name.toUpperCase()} $code: $message';
}

/// Mọi thứ checklist cần, đã đọc vào bộ nhớ.
final class ReleaseInput {
  const ReleaseInput({
    required this.app,
    required this.manifest,
    required this.info,
    required this.label,
    required this.metadata,
    this.isFirstVersion = true,
    this.deploymentTarget = 15,
  });

  final StoreApp app;
  final PrivacyManifest manifest;
  final InfoPlist info;
  final PrivacyLabel label;
  final AppMetadata metadata;

  /// "What's New" không có ở phiên bản đầu và bắt buộc từ phiên bản sau.
  final bool isFirstVersion;

  /// `IPHONEOS_DEPLOYMENT_TARGET` (số major); lab không commit `.xcodeproj`
  /// nên nhận giá trị này làm tham số.
  final int deploymentTarget;
}

int _chars(String s) => s.runes.length;
int _bytes(String s) => utf8.encode(s).length;

final RegExp _placeholder = RegExp(
  r'lorem ipsum|\bTODO\b|\bTBD\b|\bFIXME\b|điền sau|placeholder',
  caseSensitive: false,
);

/// Dấu tổ hợp (U+0300..U+036F): chữ "ớ" viết dạng NFD là "o" + hai dấu.
final RegExp _combining = RegExp('[̀-ͯ]');

const List<String> _unverifiableClaims = [
  'số 1',
  'số một',
  '#1',
  'tốt nhất',
  'rẻ nhất',
  'nhanh nhất',
];

const Set<String> _categories = {
  'FOOD_AND_DRINK', 'BUSINESS', 'EDUCATION', 'SOCIAL_NETWORKING', 'BOOKS',
  'SPORTS', 'FINANCE', 'REFERENCE', 'GRAPHICS_AND_DESIGN', 'DEVELOPER_TOOLS',
  'HEALTH_AND_FITNESS', 'MUSIC', 'WEATHER', 'TRAVEL', 'ENTERTAINMENT',
  'STICKERS', 'GAMES', 'LIFESTYLE', 'MEDICAL', 'MAGAZINES_AND_NEWSPAPERS',
  'UTILITIES', 'SHOPPING', 'PRODUCTIVITY', 'NEWS', 'PHOTO_AND_VIDEO',
  'NAVIGATION', //
};

/// Quyền cần mô tả mục đích và kiểu dữ liệu thường đi kèm. Chỉ vị trí là kiểm
/// cứng; còn lại là cảnh báo, vì dùng camera không đồng nghĩa với thu thập ảnh.
const Map<String, String> _usageToDataType = {
  'NSContactsUsageDescription': ManifestVocabulary.contacts,
  'NSMicrophoneUsageDescription': ManifestVocabulary.audioData,
  'NSPhotoLibraryUsageDescription': ManifestVocabulary.photosOrVideos,
  'NSCameraUsageDescription': ManifestVocabulary.photosOrVideos,
};

/// Checklist trước khi bấm "Submit for Review".
final class ReleaseChecklist {
  const ReleaseChecklist(this.input);

  final ReleaseInput input;

  List<Finding> run() => [
    ..._metadata(),
    ..._manifest(),
    ..._labelAgainstManifest(),
    ..._labelAgainstUsage(),
    ..._review(),
    if (input.deploymentTarget < 13)
      Finding.error(
        'deployment-target-low',
        'IPHONEOS_DEPLOYMENT_TARGET ${input.deploymentTarget} thấp hơn iOS 13 '
            '(Apple yêu cầu từ 9/2026, kiểm tại 11/2026).',
      ),
  ];

  List<Finding> get errors => [
    for (final f in run())
      if (f.severity == Severity.error) f,
  ];

  // ---------------------------------------------------------------- metadata

  List<Finding> _metadata() {
    final out = <Finding>[];
    final m = input.metadata;
    if (m.locales.isEmpty) {
      out.add(
        const Finding.error(
          'metadata-no-locale',
          'Không có thư mục ngôn ngữ nào (ví dụ vi/).',
        ),
      );
    }
    for (final l in m.locales) {
      final at = '[${l.locale}]';
      void need(String? v, String code, String file) {
        if (v == null) {
          out.add(Finding.error(code, '$at thiếu file $file.'));
        } else if (v.trim().isEmpty) {
          out.add(Finding.error('$code-empty', '$at $file rỗng.'));
        }
      }

      need(l.name, 'name-missing', 'name.txt');
      need(l.keywords, 'keywords-missing', 'keywords.txt');
      need(l.description, 'description-missing', 'description.txt');
      need(l.supportUrl, 'support-url-missing', 'support_url.txt');
      need(l.privacyUrl, 'privacy-url-missing', 'privacy_url.txt');

      final name = l.name?.trim() ?? '';
      if (name.isNotEmpty &&
          (_chars(name) < StoreLimits.nameMin ||
              _chars(name) > StoreLimits.nameMax)) {
        out.add(
          Finding.error(
            'name-length',
            '$at tên dài ${_chars(name)} ký tự, cho phép '
                '${StoreLimits.nameMin}..${StoreLimits.nameMax}.',
          ),
        );
      }
      final subtitle = l.subtitle?.trim() ?? '';
      if (_chars(subtitle) > StoreLimits.subtitleMax) {
        out.add(
          Finding.error(
            'subtitle-length',
            '$at subtitle dài ${_chars(subtitle)} ký tự, tối đa '
                '${StoreLimits.subtitleMax}.',
          ),
        );
      }
      final lowered = subtitle.toLowerCase();
      for (final claim in _unverifiableClaims) {
        if (lowered.contains(claim)) {
          out.add(
            Finding.warning(
              'subtitle-claim',
              '$at subtitle có "$claim", dễ bị coi là khẳng định không '
                  'kiểm chứng được (2.3.7).',
            ),
          );
        }
      }
      _keywords(out, at, name, l.keywords);

      final promo = l.promotionalText?.trim() ?? '';
      if (_chars(promo) > StoreLimits.promotionalTextMax) {
        out.add(
          Finding.error(
            'promo-length',
            '$at promotional text dài ${_chars(promo)}, tối đa '
                '${StoreLimits.promotionalTextMax}.',
          ),
        );
      }
      final description = l.description ?? '';
      if (_chars(description) > StoreLimits.descriptionMax) {
        out.add(
          Finding.error(
            'description-length',
            '$at description dài ${_chars(description)}, tối đa '
                '${StoreLimits.descriptionMax}.',
          ),
        );
      }

      final notes = l.releaseNotes?.trim() ?? '';
      if (input.isFirstVersion && notes.isNotEmpty) {
        out.add(
          Finding.warning(
            'release-notes-first-version',
            '$at "What\'s New" không có ở phiên bản đầu tiên.',
          ),
        );
      }
      if (!input.isFirstVersion && notes.isEmpty) {
        out.add(
          Finding.error(
            'release-notes-missing',
            '$at "What\'s New" bắt buộc từ phiên bản thứ hai.',
          ),
        );
      }
      if (_chars(notes) > StoreLimits.releaseNotesMax) {
        out.add(
          Finding.error(
            'release-notes-length',
            '$at release notes dài ${_chars(notes)}, tối đa '
                '${StoreLimits.releaseNotesMax}.',
          ),
        );
      }

      for (final (label, url) in [
        ('support_url', l.supportUrl),
        ('privacy_url', l.privacyUrl),
        ('marketing_url', l.marketingUrl),
      ]) {
        final value = url?.trim() ?? '';
        if (value.isEmpty) continue;
        final uri = Uri.tryParse(value);
        if (uri == null ||
            !(uri.scheme == 'http' || uri.scheme == 'https') ||
            uri.host.isEmpty) {
          out.add(
            Finding.error(
              'url-invalid',
              '$at $label "$value" không phải URL đầy đủ có giao thức.',
            ),
          );
        } else if (uri.scheme != 'https') {
          out.add(
            Finding.warning(
              'url-not-https',
              '$at $label dùng http thay vì https.',
            ),
          );
        }
      }

      final texts = {
        'name': l.name,
        'subtitle': l.subtitle,
        'keywords': l.keywords,
        'description': l.description,
        'promotional_text': l.promotionalText,
        'release_notes': l.releaseNotes,
      };
      for (final e in texts.entries) {
        final v = e.value;
        if (v == null) continue;
        if (_placeholder.hasMatch(v)) {
          out.add(
            Finding.error(
              'placeholder-text',
              '$at ${e.key} còn nội dung tạm (2.1(a)).',
            ),
          );
        }
        if (_combining.hasMatch(v)) {
          out.add(
            Finding.warning(
              'text-not-nfc',
              '$at ${e.key} có dấu tổ hợp (dạng NFD): số ký tự và số byte '
                  'tăng. Lưu file ở dạng NFC.',
            ),
          );
        }
      }
    }

    final copyright = m.copyright?.trim() ?? '';
    if (!RegExp(r'^\d{4} \S').hasMatch(copyright)) {
      out.add(
        const Finding.error(
          'copyright-format',
          'copyright.txt phải dạng "<năm> <chủ sở hữu>", không kèm ©.',
        ),
      );
    }
    final primary = m.primaryCategory?.trim();
    if (primary == null || !_categories.contains(primary)) {
      out.add(
        Finding.error(
          'category-primary',
          'primary_category.txt "$primary" không nằm trong danh mục của deliver.',
        ),
      );
    }
    final secondary = m.secondaryCategory?.trim();
    if (secondary != null && !_categories.contains(secondary)) {
      out.add(
        Finding.error(
          'category-secondary',
          'secondary_category.txt "$secondary" không hợp lệ.',
        ),
      );
    }
    return out;
  }

  void _keywords(List<Finding> out, String at, String name, String? raw) {
    if (raw == null || raw.trim().isEmpty) return;
    // Đếm cả dấu phẩy: Apple chỉ nói "100 byte nội dung", đếm thừa là an toàn.
    final bytes = _bytes(raw.trim());
    if (bytes > StoreLimits.keywordsMaxBytes) {
      out.add(
        Finding.error(
          'keywords-bytes',
          '$at keywords chiếm $bytes byte, tối đa '
              '${StoreLimits.keywordsMaxBytes}.',
        ),
      );
    }
    final seen = <String>{};
    for (final part in raw.split(',')) {
      final word = part.trim();
      if (word.isEmpty) {
        out.add(
          Finding.warning(
            'keywords-empty-item',
            '$at keywords có mục rỗng (dấu phẩy thừa).',
          ),
        );
        continue;
      }
      if (_chars(word) < StoreLimits.keywordMinChars) {
        out.add(
          Finding.error(
            'keyword-too-short',
            '$at từ khoá "$word" ngắn hơn ${StoreLimits.keywordMinChars} ký tự.',
          ),
        );
      }
      final lower = word.toLowerCase();
      if (!seen.add(lower)) {
        out.add(
          Finding.warning('keyword-duplicate', '$at từ khoá "$word" bị lặp.'),
        );
      }
      if (name.isNotEmpty && lower.contains(name.toLowerCase())) {
        out.add(
          Finding.error(
            'keyword-repeats-name',
            '$at từ khoá "$word" lặp tên app; tên đã được tìm kiếm sẵn.',
          ),
        );
      }
    }
  }

  // ----------------------------------------------------------- privacy files

  List<Finding> _manifest() {
    final out = <Finding>[];
    final m = input.manifest;
    if (m.tracking && m.trackingDomains.isEmpty) {
      out.add(
        const Finding.error(
          'manifest-tracking-domains',
          'NSPrivacyTracking là true nhưng NSPrivacyTrackingDomains rỗng.',
        ),
      );
    }
    for (final c in m.collected) {
      if (!ManifestVocabulary.dataTypes.contains(c.type)) {
        out.add(
          Finding.error(
            'manifest-data-type-unknown',
            'Kiểu dữ liệu "${c.type}" không có trong danh mục của Apple.',
          ),
        );
      }
      if (c.purposes.isEmpty) {
        out.add(
          Finding.error(
            'manifest-purposes-empty',
            '${c.type} không khai mục đích.',
          ),
        );
      }
      for (final p in c.purposes) {
        if (!ManifestVocabulary.purposes.contains(p)) {
          out.add(
            Finding.error(
              'manifest-purpose-unknown',
              '${c.type}: mục đích "$p" không có trong danh mục của Apple.',
            ),
          );
        }
      }
      if (c.tracking && !m.tracking) {
        out.add(
          Finding.error(
            'manifest-tracking-mismatch',
            '${c.type} đánh dấu tracking nhưng NSPrivacyTracking là false.',
          ),
        );
      }
    }
    final categories = <String>{};
    for (final api in m.accessedApis) {
      final allowed = RequiredReasonApi.reasonsByCategory[api.category];
      if (allowed == null) {
        out.add(
          Finding.error(
            'manifest-api-category-unknown',
            'Nhóm API "${api.category}" không có trong danh mục của Apple.',
          ),
        );
        continue;
      }
      if (!categories.add(api.category)) {
        out.add(
          Finding.warning(
            'manifest-api-category-duplicate',
            '${api.category} xuất hiện hai lần; gộp lý do vào một mục.',
          ),
        );
      }
      if (api.reasons.isEmpty) {
        out.add(
          Finding.error(
            'manifest-api-reasons-empty',
            '${api.category} không khai lý do nào.',
          ),
        );
      }
      for (final reason in api.reasons) {
        if (!allowed.contains(reason)) {
          out.add(
            Finding.error(
              'manifest-api-reason-invalid',
              '$reason không phải lý do hợp lệ của ${api.category}.',
            ),
          );
        } else if (RequiredReasonApi.sdkOnlyReasons.contains(reason)) {
          out.add(
            Finding.error(
              'manifest-api-sdk-only-reason',
              '$reason chỉ SDK bên thứ ba được khai, app không được.',
            ),
          );
        }
      }
    }
    return out;
  }

  List<Finding> _labelAgainstManifest() {
    final out = <Finding>[];
    final label = input.label;
    final manifest = input.manifest;
    if (label.tracking != manifest.tracking) {
      out.add(
        Finding.error(
          'label-tracking-mismatch',
          'Label tracking=${label.tracking}, manifest tracking=${manifest.tracking}.',
        ),
      );
    }
    final byType = {for (final e in label.entries) e.data.type: e};
    for (final c in manifest.collected) {
      final entry = byType[c.type];
      if (entry == null) {
        out.add(
          Finding.error(
            'label-missing-type',
            'Manifest có ${c.type} nhưng label chưa khai.',
          ),
        );
        continue;
      }
      final d = entry.data;
      if (d.linked != c.linked ||
          d.tracking != c.tracking ||
          !d.purposes.containsAll(c.purposes)) {
        out.add(
          Finding.error(
            'label-manifest-mismatch',
            '${c.type}: label $d khác manifest $c hoặc thiếu mục đích.',
          ),
        );
      }
    }
    for (final e in label.entries) {
      if (e.fromApp && !manifest.collectedTypes.contains(e.data.type)) {
        out.add(
          Finding.error(
            'label-app-type-not-in-manifest',
            '${e.data.type} nguồn "app" nhưng manifest của app không có.',
          ),
        );
      }
    }
    return out;
  }

  List<Finding> _labelAgainstUsage() {
    final out = <Finding>[];
    final info = input.info;
    final labelTypes = input.label.types;
    final hasLocationLabel =
        labelTypes.contains(ManifestVocabulary.preciseLocation) ||
        labelTypes.contains(ManifestVocabulary.coarseLocation);

    if (info.usesLocation && !hasLocationLabel) {
      out.add(
        const Finding.error(
          'label-missing-location',
          'Info.plist xin quyền vị trí nhưng label không khai Precise/Coarse Location.',
        ),
      );
    }
    if (!info.usesLocation && hasLocationLabel) {
      out.add(
        const Finding.error(
          'location-without-usage-description',
          'Label khai vị trí nhưng Info.plist không có NSLocation...UsageDescription.',
        ),
      );
    }
    if (info.usesBackgroundLocation && !info.usesLocation) {
      out.add(
        const Finding.error(
          'background-location-without-usage',
          'UIBackgroundModes có location nhưng không có mô tả quyền vị trí.',
        ),
      );
    }
    if (info.usageDescriptions.containsKey(
          'NSLocationAlwaysAndWhenInUseUsageDescription',
        ) &&
        !info.usesBackgroundLocation) {
      out.add(
        const Finding.warning(
          'always-without-background-mode',
          'Có quyền "luôn cho phép" nhưng UIBackgroundModes không có location.',
        ),
      );
    }
    for (final e in _usageToDataType.entries) {
      if (info.usageDescriptions.containsKey(e.key) &&
          !labelTypes.contains(e.value)) {
        out.add(
          Finding.warning(
            'label-missing-for-usage',
            '${e.key} có trong Info.plist; kiểm xem ${e.value} có cần khai không.',
          ),
        );
      }
    }
    for (final e in info.usageDescriptions.entries) {
      final text = e.value.trim();
      if (_placeholder.hasMatch(text)) {
        out.add(
          Finding.error(
            'usage-description-placeholder',
            '${e.key} còn nội dung tạm.',
          ),
        );
      } else if (_chars(text) < 20) {
        out.add(
          Finding.warning(
            'usage-description-short',
            '${e.key} chỉ ${_chars(text)} ký tự; mô tả mục đích đủ rõ để '
                'người duyệt hiểu (5.1.1(ii)).',
          ),
        );
      }
    }
    final att = info.usageDescriptions.containsKey(
      'NSUserTrackingUsageDescription',
    );
    if (input.label.tracking && !att) {
      out.add(
        const Finding.error(
          'tracking-without-att-description',
          'Label khai tracking nhưng thiếu NSUserTrackingUsageDescription.',
        ),
      );
    }
    if (!input.label.tracking && att) {
      out.add(
        const Finding.warning(
          'att-description-without-tracking',
          'Có NSUserTrackingUsageDescription nhưng label khai không tracking.',
        ),
      );
    }
    return out;
  }

  // ------------------------------------------------------------------ review

  List<Finding> _review() {
    final out = <Finding>[];
    final r = input.metadata.review;
    void need(String? v, String code, String file) {
      if (v == null || v.trim().isEmpty) {
        out.add(
          Finding.error(code, 'review_information/$file trống hoặc thiếu.'),
        );
      }
    }

    need(r.firstName, 'review-contact-missing', 'first_name.txt');
    need(r.lastName, 'review-contact-missing', 'last_name.txt');
    need(r.emailAddress, 'review-contact-missing', 'email_address.txt');
    need(r.phoneNumber, 'review-contact-missing', 'phone_number.txt');
    final phone = r.phoneNumber?.trim() ?? '';
    if (phone.isNotEmpty && !RegExp(r'^\+\d[\d ]{6,}$').hasMatch(phone)) {
      out.add(
        const Finding.error(
          'review-phone-format',
          'Số điện thoại phải dạng quốc tế: dấu + rồi mã quốc gia.',
        ),
      );
    }
    need(r.demoUser, 'review-demo-missing', 'demo_user.txt');
    need(r.demoPassword, 'review-demo-missing', 'demo_password.txt');

    final notes = r.notes?.trim() ?? '';
    if (notes.isEmpty) {
      out.add(
        const Finding.error(
          'review-notes-missing',
          'Thiếu ghi chú cho App Review (notes.txt).',
        ),
      );
      return out;
    }
    if (_bytes(notes) > StoreLimits.reviewNotesMaxBytes) {
      out.add(
        Finding.error(
          'review-notes-bytes',
          'Ghi chú review chiếm ${_bytes(notes)} byte, tối đa '
              '${StoreLimits.reviewNotesMaxBytes}.',
        ),
      );
    }
    if (_placeholder.hasMatch(notes)) {
      out.add(
        const Finding.error(
          'placeholder-text',
          'Ghi chú review còn nội dung tạm (2.1(a)).',
        ),
      );
    }
    if (!notes.toLowerCase().contains('xoá tài khoản')) {
      out.add(
        const Finding.warning(
          'review-notes-account-deletion',
          'Ghi chú chưa nói chỗ tìm tính năng xoá tài khoản (5.1.1(v)).',
        ),
      );
    }
    final lower = notes.toLowerCase();
    if (input.info.usesBackgroundLocation &&
        !(lower.contains('vị trí nền') || lower.contains('background'))) {
      out.add(
        const Finding.warning(
          'review-notes-background-location',
          'App dùng vị trí nền nhưng ghi chú không giải thích (5.1.5, 2.5.4).',
        ),
      );
    }
    return out;
  }
}

/// Chia ảnh theo thiết bị từ độ phân giải, như `fastlane deliver` làm với thư
/// mục `screenshots/<ngôn ngữ>/`: cỡ iPad vào nhóm iPad, còn lại vào iPhone.
({List<PngInfo> iphone, List<PngInfo> ipad}) splitByDevice(
  Iterable<PngInfo> shots,
) {
  bool isIpad(PngInfo s) =>
      ScreenshotSizes.matches(
        ScreenshotSizes.ipadRequired,
        s.width,
        s.height,
      ) ||
      ScreenshotSizes.matches(ScreenshotSizes.ipadOther, s.width, s.height);
  return (
    iphone: [
      for (final s in shots)
        if (!isIpad(s)) s,
    ],
    ipad: [
      for (final s in shots)
        if (isIpad(s)) s,
    ],
  );
}

/// Kiểm ảnh chụp theo "Screenshot specifications".
List<Finding> checkScreenshots({
  required List<PngInfo> iphone,
  List<PngInfo> ipad = const [],
  required bool supportsIpad,
}) {
  final out = <Finding>[];

  void group(
    String name,
    List<PngInfo> shots,
    List<(int, int)> required,
    List<(int, int)> other,
  ) {
    final perSize = <(int, int), int>{};
    for (final s in shots) {
      final known =
          ScreenshotSizes.matches(required, s.width, s.height) ||
          ScreenshotSizes.matches(other, s.width, s.height);
      if (!known) {
        out.add(
          Finding.warning(
            'screenshot-size-unknown',
            '$name ${s.width}x${s.height} không có trong danh sách của lab; '
                'đối chiếu trang Apple.',
          ),
        );
      }
      if (s.hasAlpha) {
        out.add(
          Finding.error(
            'screenshot-alpha',
            '$name ${s.width}x${s.height} có kênh alpha hoặc trong suốt; '
                'App Store Connect không nhận.',
          ),
        );
      }
      final key = s.width <= s.height
          ? (s.width, s.height)
          : (s.height, s.width);
      perSize[key] = (perSize[key] ?? 0) + 1;
    }
    for (final e in perSize.entries) {
      if (e.value > StoreLimits.screenshotsMaxPerSize) {
        out.add(
          Finding.error(
            'screenshot-too-many',
            '$name ${e.key.$1}x${e.key.$2}: ${e.value} ảnh, tối đa '
                '${StoreLimits.screenshotsMaxPerSize}.',
          ),
        );
      }
    }
    if (!shots.any(
      (s) => ScreenshotSizes.matches(required, s.width, s.height),
    )) {
      out.add(
        Finding.error(
          'screenshot-$name-required-size',
          'Chưa có ảnh $name đúng cỡ bắt buộc '
              '${required.map((s) => '${s.$1}x${s.$2}').join(' hoặc ')}.',
        ),
      );
    }
  }

  group(
    'iphone',
    iphone,
    ScreenshotSizes.iphoneRequired,
    ScreenshotSizes.iphoneOther,
  );
  if (supportsIpad) {
    group(
      'ipad',
      ipad,
      ScreenshotSizes.ipadRequired,
      ScreenshotSizes.ipadOther,
    );
  } else if (ipad.isNotEmpty) {
    out.add(
      const Finding.warning(
        'screenshot-ipad-unused',
        'App không hỗ trợ iPad nhưng có ảnh iPad.',
      ),
    );
  }
  return out;
}
