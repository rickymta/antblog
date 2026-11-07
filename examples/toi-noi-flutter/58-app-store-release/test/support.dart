import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:toi_noi_58/toi_noi_58.dart';

/// Gói phát hành thật của [app] trong thư mục lab (test chạy từ gốc lab).
ReleaseInput loadSample(StoreApp app, {bool isFirstVersion = true}) =>
    loadReleaseInput(Directory.current, app, isFirstVersion: isFirstVersion);

/// Bản sao của [base] với vài trường thay thế.
ReleaseInput change(
  ReleaseInput base, {
  PrivacyManifest? manifest,
  InfoPlist? info,
  PrivacyLabel? label,
  AppMetadata? metadata,
  bool? isFirstVersion,
  int? deploymentTarget,
}) => ReleaseInput(
  app: base.app,
  manifest: manifest ?? base.manifest,
  info: info ?? base.info,
  label: label ?? base.label,
  metadata: metadata ?? base.metadata,
  isFirstVersion: isFirstVersion ?? base.isFirstVersion,
  deploymentTarget: deploymentTarget ?? base.deploymentTarget,
);

/// Bản sao của metadata [base] với locale đầu tiên được sửa; trường nào truyền
/// `null` thì giữ nguyên, trường truyền chuỗi (kể cả rỗng) thì thay.
AppMetadata withLocale(
  AppMetadata base, {
  String? name,
  String? subtitle,
  String? keywords,
  String? description,
  String? promotionalText,
  String? releaseNotes,
  String? supportUrl,
  String? marketingUrl,
  String? privacyUrl,
  String? copyright,
  String? primaryCategory,
  String? secondaryCategory,
  ReviewInformation? review,
}) {
  final l = base.locales.first;
  return AppMetadata(
    locales: [
      LocalizedMetadata(
        locale: l.locale,
        name: name ?? l.name,
        subtitle: subtitle ?? l.subtitle,
        keywords: keywords ?? l.keywords,
        description: description ?? l.description,
        promotionalText: promotionalText ?? l.promotionalText,
        releaseNotes: releaseNotes ?? l.releaseNotes,
        supportUrl: supportUrl ?? l.supportUrl,
        marketingUrl: marketingUrl ?? l.marketingUrl,
        privacyUrl: privacyUrl ?? l.privacyUrl,
      ),
    ],
    review: review ?? base.review,
    copyright: copyright ?? base.copyright,
    primaryCategory: primaryCategory ?? base.primaryCategory,
    secondaryCategory: secondaryCategory ?? base.secondaryCategory,
  );
}

/// Bản sao của [r] với vài trường thay thế.
ReviewInformation withReview(
  ReviewInformation r, {
  String? phoneNumber,
  String? demoUser,
  String? demoPassword,
  String? notes,
  String? firstName,
}) => ReviewInformation(
  firstName: firstName ?? r.firstName,
  lastName: r.lastName,
  phoneNumber: phoneNumber ?? r.phoneNumber,
  emailAddress: r.emailAddress,
  demoUser: demoUser ?? r.demoUser,
  demoPassword: demoPassword ?? r.demoPassword,
  notes: notes ?? r.notes,
);

/// Mã của mọi phát hiện, để so khớp ngắn gọn.
Set<String> codes(Iterable<Finding> findings) => {
  for (final f in findings) f.code,
};

/// Mã của các phát hiện cấp lỗi.
Set<String> errorCodes(Iterable<Finding> findings) => {
  for (final f in findings)
    if (f.severity == Severity.error) f.code,
};

const String _head =
    '<?xml version="1.0" encoding="UTF-8"?>\n'
    '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" '
    '"http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n';

/// Dựng XML `PrivacyInfo.xcprivacy` từ cấu trúc Dart (không phụ thuộc file thật).
String manifestXml({
  bool tracking = false,
  List<String> domains = const [],
  List<({String type, bool linked, bool tracking, List<String> purposes})>
      collected =
      const [],
  List<({String category, List<String> reasons})> apis = const [],
}) {
  String strings(List<String> xs) => xs.isEmpty
      ? '<array/>'
      : '<array>${xs.map((x) => '<string>$x</string>').join()}</array>';
  final data = collected
      .map(
        (c) =>
            '<dict><key>NSPrivacyCollectedDataType</key><string>${c.type}</string>'
            '<key>NSPrivacyCollectedDataTypeLinked</key><${c.linked}/>'
            '<key>NSPrivacyCollectedDataTypeTracking</key><${c.tracking}/>'
            '<key>NSPrivacyCollectedDataTypePurposes</key>${strings(c.purposes)}</dict>',
      )
      .join();
  final api = apis
      .map(
        (a) =>
            '<dict><key>NSPrivacyAccessedAPIType</key><string>${a.category}</string>'
            '<key>NSPrivacyAccessedAPITypeReasons</key>${strings(a.reasons)}</dict>',
      )
      .join();
  return '$_head<plist version="1.0"><dict>'
      '<key>NSPrivacyTracking</key><$tracking/>'
      '<key>NSPrivacyTrackingDomains</key>${strings(domains)}'
      '<key>NSPrivacyCollectedDataTypes</key>${data.isEmpty ? '<array/>' : '<array>$data</array>'}'
      '<key>NSPrivacyAccessedAPITypes</key>${api.isEmpty ? '<array/>' : '<array>$api</array>'}'
      '</dict></plist>';
}

/// Dựng XML `Info.plist` với các chuỗi mô tả quyền và background mode.
String infoXml({
  Map<String, String> usage = const {},
  List<String> backgroundModes = const [],
}) {
  final u = usage.entries
      .map((e) => '<key>${e.key}</key><string>${e.value}</string>')
      .join();
  final b = backgroundModes.isEmpty
      ? ''
      : '<key>UIBackgroundModes</key><array>${backgroundModes.map((m) => '<string>$m</string>').join()}</array>';
  return '$_head<plist version="1.0"><dict>$u$b</dict></plist>';
}

/// Một dòng của label dạng JSON.
Map<String, Object?> labelEntry(
  String type, {
  String source = 'app',
  bool linked = true,
  bool tracking = false,
  List<String> purposes = const [
    'NSPrivacyCollectedDataTypePurposeAppFunctionality',
  ],
}) => {
  'type': type,
  'linked': linked,
  'tracking': tracking,
  'purposes': purposes,
  'source': source,
};

PrivacyLabel labelOf(
  List<Map<String, Object?>> entries, {
  bool tracking = false,
}) => PrivacyLabel.parse(
  jsonEncode({'tracking': tracking, 'dataTypes': entries}),
);

/// Dựng file PNG tối thiểu: chữ ký, IHDR, (tRNS), IDAT, IEND. CRC để 0: lab
/// chỉ đọc kích thước và colour type, không giải mã ảnh.
Uint8List png(int width, int height, {int colorType = 2, bool trns = false}) {
  final out = BytesBuilder();
  void chunk(String type, List<int> data) {
    final length = ByteData(4)..setUint32(0, data.length);
    out
      ..add(length.buffer.asUint8List())
      ..add(type.codeUnits)
      ..add(data)
      ..add(const [0, 0, 0, 0]);
  }

  out.add(const [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
  final ihdr = ByteData(13)
    ..setUint32(0, width)
    ..setUint32(4, height)
    ..setUint8(8, 8)
    ..setUint8(9, colorType);
  chunk('IHDR', ihdr.buffer.asUint8List());
  if (trns) chunk('tRNS', const [0, 0]);
  chunk('IDAT', const [0]);
  chunk('IEND', const []);
  return out.toBytes();
}
