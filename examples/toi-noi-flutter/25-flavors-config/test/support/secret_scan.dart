/// Quét một bảng cấu hình tìm thứ trông giống bí mật. Trả về mô tả lỗi theo
/// tên khoá, **không bao giờ** chứa giá trị, vì kết quả sẽ lên màn hình CI.
///
/// Đây là lưới an toàn thô, không thay được quy tắc "không để bí mật vào file
/// cấu hình": nó bắt được dạng phổ biến, không bắt được mọi dạng.
List<String> findSecretLike(Map<String, Object?> config) {
  final findings = <String>[];
  for (final entry in config.entries) {
    final key = entry.key;
    if (_secretKeyName.hasMatch(key)) {
      findings.add('$key: tên khoá gợi ý bí mật');
    }
    final value = entry.value;
    if (value is String) {
      for (final rule in _valueRules) {
        if (rule.pattern.hasMatch(value)) {
          findings.add('$key: giá trị giống ${rule.name}');
        }
      }
    } else if (value is Map || value is List) {
      findings.add('$key: giá trị lồng nhau sẽ thành chuỗi khó đoán');
    }
  }
  return findings;
}

final _secretKeyName = RegExp(
  r'secret|token|passw|private|credential|api[_-]?key|auth|signature',
  caseSensitive: false,
);

class _Rule {
  const _Rule(this.name, this.pattern);

  final String name;
  final RegExp pattern;
}

final _valueRules = [
  _Rule('khoá PEM', RegExp('-----BEGIN [A-Z ]+-----')),
  _Rule('access key AWS', RegExp(r'\bAKIA[0-9A-Z]{16}\b')),
  _Rule('API key Google', RegExp(r'\bAIza[0-9A-Za-z_-]{35}\b')),
  _Rule('JWT', RegExp(r'^eyJ[\w-]+\.[\w-]+\.[\w-]*$')),
  _Rule('token GitHub', RegExp(r'\bgh[pousr]_[0-9A-Za-z]{20,}')),
  _Rule('khoá Stripe', RegExp(r'\b[sr]k_(live|test)_[0-9A-Za-z]{10,}')),
  _Rule('user:password trong URL', RegExp(r'://[^/\s:@]+:[^/\s@]+@')),
  _Rule('chuỗi ngẫu nhiên dài', RegExp(r'^[A-Za-z0-9+/_=-]{32,}$')),
];
