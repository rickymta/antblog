import 'dart:convert';

import 'plist.dart';

/// Một kiểu dữ liệu app thu thập, đúng cấu trúc trong `PrivacyInfo.xcprivacy`
/// và cũng là một dòng của privacy label.
final class CollectedData {
  const CollectedData({
    required this.type,
    required this.linked,
    required this.tracking,
    required this.purposes,
  });

  final String type;
  final bool linked;
  final bool tracking;
  final Set<String> purposes;

  @override
  String toString() => '$type(linked: $linked, tracking: $tracking)';
}

/// Một nhóm required reason API cùng các lý do đã khai.
final class AccessedApi {
  const AccessedApi({required this.category, required this.reasons});

  final String category;
  final List<String> reasons;
}

/// Nội dung `PrivacyInfo.xcprivacy` của app.
final class PrivacyManifest {
  const PrivacyManifest({
    required this.tracking,
    required this.trackingDomains,
    required this.collected,
    required this.accessedApis,
  });

  /// Đọc file plist. Thiếu một khoá cấp cao nhất là lỗi, vì Xcode sinh đủ bốn
  /// khoá và thiếu khoá thường là file sửa tay hỏng.
  factory PrivacyManifest.parse(String xml) {
    final root = parsePlist(xml);
    if (root is! Map<String, Object?>) {
      throw const FormatException('PrivacyInfo: gốc phải là dict');
    }
    Object field(String key) =>
        root[key] ?? (throw FormatException('PrivacyInfo: thiếu khoá $key'));

    final tracking = field('NSPrivacyTracking');
    final domains = field('NSPrivacyTrackingDomains');
    final collected = field('NSPrivacyCollectedDataTypes');
    final apis = field('NSPrivacyAccessedAPITypes');
    if (tracking is! bool || domains is! List || collected is! List) {
      throw const FormatException('PrivacyInfo: sai kiểu ở khoá cấp cao nhất');
    }
    if (apis is! List) {
      throw const FormatException(
        'PrivacyInfo: NSPrivacyAccessedAPITypes sai kiểu',
      );
    }
    return PrivacyManifest(
      tracking: tracking,
      trackingDomains: [for (final d in domains) d.toString()],
      collected: [
        for (final entry in collected)
          if (entry is Map<String, Object?>)
            CollectedData(
              type: '${entry['NSPrivacyCollectedDataType']}',
              linked: entry['NSPrivacyCollectedDataTypeLinked'] == true,
              tracking: entry['NSPrivacyCollectedDataTypeTracking'] == true,
              purposes: {
                for (final p
                    in (entry['NSPrivacyCollectedDataTypePurposes']
                            as List<Object?>? ??
                        const []))
                  '$p',
              },
            ),
      ],
      accessedApis: [
        for (final entry in apis)
          if (entry is Map<String, Object?>)
            AccessedApi(
              category: '${entry['NSPrivacyAccessedAPIType']}',
              reasons: [
                for (final r
                    in (entry['NSPrivacyAccessedAPITypeReasons']
                            as List<Object?>? ??
                        const []))
                  '$r',
              ],
            ),
      ],
    );
  }

  final bool tracking;
  final List<String> trackingDomains;
  final List<CollectedData> collected;
  final List<AccessedApi> accessedApis;

  Set<String> get collectedTypes => {for (final c in collected) c.type};
}

/// Phần của `Info.plist` mà việc phát hành App Store quan tâm.
final class InfoPlist {
  const InfoPlist({
    required this.usageDescriptions,
    required this.backgroundModes,
  });

  factory InfoPlist.parse(String xml) {
    final root = parsePlist(xml);
    if (root is! Map<String, Object?>) {
      throw const FormatException('Info.plist: gốc phải là dict');
    }
    final modes = root['UIBackgroundModes'];
    return InfoPlist(
      usageDescriptions: {
        for (final e in root.entries)
          if (e.key.startsWith('NS') &&
              e.key.endsWith('UsageDescription') &&
              e.value is String)
            e.key: e.value! as String,
      },
      backgroundModes: modes is List ? [for (final m in modes) '$m'] : const [],
    );
  }

  /// Khoá `NS...UsageDescription` và chuỗi mô tả người dùng thấy.
  final Map<String, String> usageDescriptions;
  final List<String> backgroundModes;

  bool get usesLocation =>
      usageDescriptions.keys.any((k) => k.startsWith('NSLocation'));
  bool get usesBackgroundLocation => backgroundModes.contains('location');
}

/// Câu trả lời ta sẽ nhập vào mục App Privacy của App Store Connect.
///
/// App Store Connect không đọc file này; nó là bản ghi trong git để đối chiếu
/// với manifest và `Info.plist`. Mỗi dòng có `source`: `app` nếu chính app
/// thu thập (phải có trong manifest của app), `sdk:<tên>` nếu do SDK bên thứ
/// ba (nằm trong manifest của SDK, không bắt buộc có trong manifest của app).
final class PrivacyLabel {
  const PrivacyLabel({required this.tracking, required this.entries});

  factory PrivacyLabel.parse(String json) {
    final root = jsonDecode(json);
    if (root is! Map<String, Object?>) {
      throw const FormatException('label: gốc phải là object');
    }
    final list = root['dataTypes'];
    if (root['tracking'] is! bool || list is! List) {
      throw const FormatException('label: cần "tracking" và "dataTypes"');
    }
    return PrivacyLabel(
      tracking: root['tracking']! as bool,
      entries: [
        for (final e in list)
          if (e is Map<String, Object?>)
            LabelEntry(
              data: CollectedData(
                type: '${e['type']}',
                linked: e['linked'] == true,
                tracking: e['tracking'] == true,
                purposes: {
                  for (final p in (e['purposes'] as List<Object?>? ?? const []))
                    '$p',
                },
              ),
              source: '${e['source'] ?? 'app'}',
            ),
      ],
    );
  }

  final bool tracking;
  final List<LabelEntry> entries;

  Set<String> get types => {for (final e in entries) e.data.type};
}

/// Một dòng của [PrivacyLabel].
final class LabelEntry {
  const LabelEntry({required this.data, required this.source});

  final CollectedData data;
  final String source;

  bool get fromApp => source == 'app';
}
