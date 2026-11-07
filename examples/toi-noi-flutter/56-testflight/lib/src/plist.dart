// Bản sao của lib/src/plist.dart ở lab 52: mỗi lab tự đứng được, không phụ thuộc lab khác.
import 'package:xml/xml.dart';

/// Đọc một plist dạng XML thành `Map<String, Object?>`.
///
/// Hỗ trợ đúng các kiểu mà profile và file cấu hình ký dùng: `dict`, `array`,
/// `string`, `true`/`false`, `integer`, `real`, `date` (thành [DateTime] UTC)
/// và `data` (giữ nguyên chuỗi base64). Gốc phải là `dict`.
///
/// Ném [FormatException] khi XML hỏng, gốc không phải `dict`, một `dict` có
/// `key` không đi kèm giá trị, hoặc gặp kiểu lạ.
Map<String, Object?> parsePlist(String source) {
  final XmlDocument document;
  try {
    document = XmlDocument.parse(source);
  } on XmlException catch (e) {
    throw FormatException('Không đọc được plist: ${e.message}');
  }
  final root = document.rootElement;
  if (root.name.local != 'plist') {
    throw const FormatException('Gốc XML phải là <plist>');
  }
  final children = root.childElements.toList();
  if (children.length != 1 || children.single.name.local != 'dict') {
    throw const FormatException('Gốc của plist phải là một <dict>');
  }
  return _dict(children.single);
}

Map<String, Object?> _dict(XmlElement element) {
  final children = element.childElements.toList();
  if (children.length.isOdd) {
    throw const FormatException('<dict> có <key> không kèm giá trị');
  }
  final result = <String, Object?>{};
  for (var i = 0; i < children.length; i += 2) {
    final key = children[i];
    if (key.name.local != 'key') {
      throw FormatException(
        'Mong đợi <key>, gặp <${key.name.local}> trong <dict>',
      );
    }
    result[key.innerText] = _value(children[i + 1]);
  }
  return result;
}

Object? _value(XmlElement element) {
  switch (element.name.local) {
    case 'dict':
      return _dict(element);
    case 'array':
      return [for (final child in element.childElements) _value(child)];
    case 'string':
      return element.innerText;
    case 'true':
      return true;
    case 'false':
      return false;
    case 'integer':
      return int.parse(element.innerText.trim());
    case 'real':
      return double.parse(element.innerText.trim());
    case 'date':
      return DateTime.parse(element.innerText.trim()).toUtc();
    case 'data':
      return element.innerText.replaceAll(RegExp(r'\s+'), '');
    default:
      throw FormatException('Kiểu plist lạ: <${element.name.local}>');
  }
}
