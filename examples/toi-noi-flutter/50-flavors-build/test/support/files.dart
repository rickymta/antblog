import 'dart:convert';
import 'dart:io';

import 'package:xml/xml.dart';

/// `flutter test` chạy với thư mục làm việc là gốc của lab.
final labRoot = Directory.current.path;

String path(String relative) => '$labRoot/$relative';

String read(String relative) => File(path(relative)).readAsStringSync();

bool exists(String relative) => File(path(relative)).existsSync();

Map<String, dynamic> readJson(String relative) =>
    jsonDecode(read(relative)) as Map<String, dynamic>;

const flavors = ['dev', 'staging', 'prod'];
const apps = ['customer', 'driver'];

/// Thư mục `native-config/<app>/...`.
String native(String app, String relative) => 'native-config/$app/$relative';

/// Một `create("...") { ... }` trong `productFlavors`.
class GradleFlavor {
  GradleFlavor({required this.dimension, required this.suffix});
  final String? dimension;
  final String suffix;
}

class GradleConfig {
  GradleConfig({
    required this.applicationId,
    required this.dimensions,
    required this.flavors,
    required this.text,
  });

  final String? applicationId;
  final List<String> dimensions;
  final Map<String, GradleFlavor> flavors;
  final String text;

  /// Application ID mà Gradle ghép: `applicationId` + `applicationIdSuffix`.
  String idOf(String flavor) => '$applicationId${flavors[flavor]!.suffix}';
}

/// Đọc đúng khuôn của file mẫu (một khối `productFlavors`, mỗi flavor một khối
/// `create`). Không phải bộ phân tích Kotlin.
GradleConfig parseGradle(String text) {
  String? one(RegExp pattern, String source) =>
      pattern.firstMatch(source)?.group(1);

  final block =
      RegExp(r'productFlavors\s*\{([\s\S]*?)\n    \}')
          .firstMatch(text)
          ?.group(1) ??
      '';
  final found = <String, GradleFlavor>{};
  for (final m in RegExp(
    r'create\("([^"]+)"\)\s*\{([^}]*)\}',
  ).allMatches(block)) {
    found[m.group(1)!] = GradleFlavor(
      dimension: one(RegExp(r'dimension\s*=\s*"([^"]+)"'), m.group(2)!),
      suffix:
          one(RegExp(r'applicationIdSuffix\s*=\s*"([^"]*)"'), m.group(2)!) ??
          '',
    );
  }
  return GradleConfig(
    applicationId: one(RegExp(r'applicationId\s*=\s*"([^"]+)"'), text),
    dimensions: [
      for (final m in RegExp(
        r'flavorDimensions\s*\+=\s*"([^"]+)"',
      ).allMatches(text))
        m.group(1)!,
    ],
    flavors: found,
    text: text,
  );
}

/// Một file `.xcconfig`: các dòng `KEY = value` và danh sách `#include`.
class Xcconfig {
  Xcconfig(this.settings, this.includes);
  final Map<String, String> settings;
  final List<String> includes;
}

Xcconfig parseXcconfig(String text) {
  final settings = <String, String>{};
  final includes = <String>[];
  for (final raw in text.split('\n')) {
    final line = raw.trim();
    if (line.isEmpty || line.startsWith('//')) continue;
    final include = RegExp(r'^#include\??\s+"([^"]+)"$').firstMatch(line);
    if (include != null) {
      includes.add(include.group(1)!);
      continue;
    }
    final kv = RegExp(r'^([A-Z0-9_]+)\s*=\s*(.*)$').firstMatch(line);
    if (kv != null) settings[kv.group(1)!] = kv.group(2)!;
  }
  return Xcconfig(settings, includes);
}

/// Từ điển khoá -> chuỗi của một plist phẳng (chỉ lấy các giá trị `<string>`).
Map<String, String> plistStrings(String text) {
  final dict = XmlDocument.parse(text).findAllElements('dict').first;
  final children = dict.childElements.toList();
  return {
    for (var i = 0; i + 1 < children.length; i += 2)
      if (children[i + 1].name.local == 'string')
        children[i].innerText: children[i + 1].innerText,
  };
}

/// Đọc `app_name` trong một `strings.xml`.
String? appName(String stringsXml) {
  for (final e in XmlDocument.parse(stringsXml).findAllElements('string')) {
    if (e.getAttribute('name') == 'app_name') return e.innerText;
  }
  return null;
}

/// Chiều rộng, chiều cao và kiểu màu của một PNG, đọc từ khối IHDR.
class PngHeader {
  PngHeader(this.width, this.height, this.colorType);
  final int width;
  final int height;
  final int colorType;
}

PngHeader? pngHeader(List<int> bytes) {
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (bytes.length < 26) return null;
  for (var i = 0; i < 8; i++) {
    if (bytes[i] != signature[i]) return null;
  }
  int u32(int at) =>
      (bytes[at] << 24) |
      (bytes[at + 1] << 16) |
      (bytes[at + 2] << 8) |
      bytes[at + 3];
  return PngHeader(u32(16), u32(20), bytes[25]);
}

/// Chép đệ quy [from] sang [to] (tạo thư mục khi cần).
void copyTree(Directory from, Directory to) {
  to.createSync(recursive: true);
  for (final entity in from.listSync()) {
    final name = entity.uri.pathSegments.where((s) => s.isNotEmpty).last;
    if (entity is Directory) {
      if (name == '.dart_tool' || name == 'build') continue;
      copyTree(entity, Directory('${to.path}/$name'));
    } else if (entity is File) {
      entity.copySync('${to.path}/$name');
    }
  }
}
