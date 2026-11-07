/// Trình đọc XML property list đủ dùng cho `Info.plist` và `PrivacyInfo.xcprivacy`.
///
/// Hỗ trợ `dict`, `array`, `string`, `integer`, `real`, `true`, `false`;
/// `data` và `date` được giữ dạng chuỗi. Kết quả là `Map<String, Object?>`,
/// `List<Object?>`, `String`, `int`, `double` hoặc `bool`. Dữ liệu sai ném
/// [FormatException].
library;

final RegExp _token = RegExp(
  r'<!--.*?-->|<\?.*?\?>|<!DOCTYPE[^>]*>|<(/?)([A-Za-z]+)(?:\s[^>]*?)?(/?)>|([^<]+)',
  dotAll: true,
);

/// Đọc [xml] và trả về giá trị gốc (thường là một `dict`).
Object? parsePlist(String xml) {
  final tokens = <_Tag>[];
  for (final m in _token.allMatches(xml)) {
    final name = m.group(2);
    if (name != null) {
      final closing = m.group(1) == '/';
      final selfClosing = m.group(3) == '/';
      tokens.add(_Tag(name, closing: closing, selfClosing: selfClosing));
    } else if (m.group(4) != null) {
      final text = m.group(4)!;
      if (text.trim().isNotEmpty) tokens.add(_Tag.text(_unescape(text)));
    }
  }
  final reader = _Reader(tokens);
  reader.expectOpen('plist');
  final value = reader.readValue();
  reader.expectClose('plist');
  if (!reader.atEnd) {
    throw const FormatException('plist: còn nội dung sau thẻ </plist>');
  }
  return value;
}

String _unescape(String s) => s
    .replaceAll('&lt;', '<')
    .replaceAll('&gt;', '>')
    .replaceAll('&quot;', '"')
    .replaceAll('&apos;', "'")
    .replaceAll('&amp;', '&');

final class _Tag {
  const _Tag(this.name, {required this.closing, required this.selfClosing})
    : text = null;
  const _Tag.text(String this.text)
    : name = '',
      closing = false,
      selfClosing = false;

  final String name;
  final bool closing;
  final bool selfClosing;
  final String? text;
}

final class _Reader {
  _Reader(this._tokens);

  final List<_Tag> _tokens;
  int _i = 0;

  bool get atEnd => _i >= _tokens.length;

  _Tag _next() {
    if (atEnd) throw const FormatException('plist: kết thúc đột ngột');
    return _tokens[_i++];
  }

  _Tag _peek() {
    if (atEnd) throw const FormatException('plist: kết thúc đột ngột');
    return _tokens[_i];
  }

  void expectOpen(String name) {
    final t = _next();
    if (t.text != null || t.closing || t.name != name) {
      throw FormatException('plist: cần <$name>, gặp <${t.name}>');
    }
  }

  void expectClose(String name) {
    final t = _next();
    if (!t.closing || t.name != name) {
      throw FormatException('plist: cần </$name>');
    }
  }

  String _readText(String name) {
    expectOpen(name);
    var value = '';
    if (_peek().text != null) value = _next().text!;
    expectClose(name);
    return value;
  }

  Object? readValue() {
    final t = _peek();
    if (t.text != null || t.closing) {
      throw FormatException('plist: giá trị không hợp lệ ở "${t.name}"');
    }
    switch (t.name) {
      case 'dict':
        return _readDict();
      case 'array':
        return _readArray();
      case 'string':
      case 'data':
      case 'date':
        if (t.selfClosing) {
          _next();
          return '';
        }
        return _readText(t.name);
      case 'integer':
        final text = _readText('integer').trim();
        return int.tryParse(text) ??
            (throw FormatException('plist: integer sai "$text"'));
      case 'real':
        final text = _readText('real').trim();
        return double.tryParse(text) ??
            (throw FormatException('plist: real sai "$text"'));
      case 'true':
        _next();
        return true;
      case 'false':
        _next();
        return false;
    }
    throw FormatException('plist: thẻ <${t.name}> chưa hỗ trợ');
  }

  Map<String, Object?> _readDict() {
    final open = _next();
    final result = <String, Object?>{};
    if (open.selfClosing) return result;
    while (!(_peek().closing && _peek().name == 'dict')) {
      final key = _readText('key');
      if (result.containsKey(key)) {
        throw FormatException('plist: khoá lặp "$key"');
      }
      result[key] = readValue();
    }
    expectClose('dict');
    return result;
  }

  List<Object?> _readArray() {
    final open = _next();
    final result = <Object?>[];
    if (open.selfClosing) return result;
    while (!(_peek().closing && _peek().name == 'array')) {
      result.add(readValue());
    }
    expectClose('array');
    return result;
  }
}
