/// Đọc các quyền `<uses-permission>` từ một AndroidManifest.xml.
///
/// Không dùng thư viện XML: manifest của lab nhỏ, và ta chỉ cần tên quyền.
/// Hàm bỏ qua chú thích XML, bỏ qua `<uses-permission-sdk-23>`, và không tính
/// quyền có `tools:node="remove"` (cách app loại quyền mà plugin tự thêm).
library;

const String backgroundLocation =
    'android.permission.ACCESS_BACKGROUND_LOCATION';
const String coarseLocation = 'android.permission.ACCESS_COARSE_LOCATION';
const String fineLocation = 'android.permission.ACCESS_FINE_LOCATION';
const String foregroundServiceLocation =
    'android.permission.FOREGROUND_SERVICE_LOCATION';

final RegExp _comment = RegExp(r'<!--.*?-->', dotAll: true);
final RegExp _usesPermission = RegExp(
  r'<uses-permission(?=[\s/>])[^>]*>',
  dotAll: true,
);
final RegExp _nameAttr = RegExp(r'''android:name\s*=\s*["']([^"']+)["']''');
final RegExp _nodeRemove = RegExp(r'''tools:node\s*=\s*["']remove["']''');

/// Tập tên quyền mà manifest [xml] khai.
Set<String> readPermissions(String xml) {
  final stripped = xml.replaceAll(_comment, '');
  final result = <String>{};
  for (final tag in _usesPermission.allMatches(stripped)) {
    final text = tag.group(0)!;
    if (_nodeRemove.hasMatch(text)) continue;
    final name = _nameAttr.firstMatch(text)?.group(1);
    if (name != null) result.add(name);
  }
  return result;
}

/// Khai báo phải làm riêng trong Play Console (mục App content), ngoài việc có
/// dòng `<uses-permission>`.
enum ConsoleDeclaration { backgroundLocation, foregroundServiceLocation }

/// Các khai báo Console mà [permissions] kéo theo.
Set<ConsoleDeclaration> consoleDeclarationsFor(Set<String> permissions) => {
  if (permissions.contains(backgroundLocation))
    ConsoleDeclaration.backgroundLocation,
  if (permissions.contains(foregroundServiceLocation))
    ConsoleDeclaration.foregroundServiceLocation,
};
