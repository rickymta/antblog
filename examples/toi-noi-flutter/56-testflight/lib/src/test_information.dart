import 'dart:convert';

import 'beta_checklist.dart' show Severity;

class InfoIssue {
  const InfoIssue(this.code, this.severity, this.field, this.message);

  final String code;
  final Severity severity;
  final String field;
  final String message;

  @override
  String toString() => '${severity.name} $code [$field]: $message';
}

/// Thông tin Apple đòi trước khi gửi build cho tester external: mô tả beta,
/// email nhận phản hồi, liên hệ cho người review và, nếu app đòi đăng nhập,
/// một tài khoản demo.
///
/// Tài khoản và mật khẩu thật nằm trong App Store Connect (hoặc biến môi trường
/// của CI), không nằm trong repo; file mẫu dùng giá trị giữ chỗ dạng `<...>`.
class TestInformation {
  const TestInformation({
    required this.betaAppDescription,
    required this.feedbackEmail,
    required this.contactFirstName,
    required this.contactLastName,
    required this.contactEmail,
    required this.contactPhone,
    required this.requiresSignIn,
    this.demoAccountName = '',
    this.demoAccountPassword = '',
    this.notes = '',
  });

  /// Đọc từ JSON. Ném [FormatException] khi thiếu khoá hoặc sai kiểu.
  factory TestInformation.fromJson(String source) {
    final Object? decoded;
    try {
      decoded = jsonDecode(source);
    } on FormatException catch (e) {
      throw FormatException('JSON hỏng: ${e.message}');
    }
    if (decoded is! Map<String, Object?>) {
      throw const FormatException('Gốc phải là một object JSON');
    }
    final json = decoded;
    String text(String key, {bool optional = false}) {
      final v = json[key];
      if (v == null && optional) return '';
      if (v is! String) {
        throw FormatException('Khoá "$key" phải là chuỗi');
      }
      return v;
    }

    final requires = json['requiresSignIn'];
    if (requires is! bool) {
      throw const FormatException('Khoá "requiresSignIn" phải là true/false');
    }
    return TestInformation(
      betaAppDescription: text('betaAppDescription'),
      feedbackEmail: text('feedbackEmail'),
      contactFirstName: text('contactFirstName'),
      contactLastName: text('contactLastName'),
      contactEmail: text('contactEmail'),
      contactPhone: text('contactPhone'),
      requiresSignIn: requires,
      demoAccountName: text('demoAccountName', optional: true),
      demoAccountPassword: text('demoAccountPassword', optional: true),
      notes: text('notes', optional: true),
    );
  }

  final String betaAppDescription;
  final String feedbackEmail;
  final String contactFirstName;
  final String contactLastName;
  final String contactEmail;
  final String contactPhone;
  final bool requiresSignIn;
  final String demoAccountName;
  final String demoAccountPassword;
  final String notes;

  /// Giá trị giữ chỗ: rỗng hoặc dạng `<điền ở đây>`.
  static bool isUnfilled(String v) {
    final t = v.trim();
    return t.isEmpty || (t.startsWith('<') && t.endsWith('>'));
  }

  /// Giới hạn Notes mà trang App Review information của Apple nêu (kiểm tại
  /// 11/2026). Trang đó nói về thông tin review của bản App Store; ta dùng làm
  /// mức cảnh báo cho TestFlight vì chưa tìm thấy con số riêng.
  static const notesMaxBytes = 4000;

  List<InfoIssue> validateForExternal() {
    final issues = <InfoIssue>[];
    void required(String field, String value) {
      if (isUnfilled(value)) {
        issues.add(
          InfoIssue(
            'info-unfilled',
            Severity.error,
            field,
            '$field chưa điền (rỗng hoặc còn giá trị giữ chỗ).',
          ),
        );
      }
    }

    required('betaAppDescription', betaAppDescription);
    required('feedbackEmail', feedbackEmail);
    required('contactFirstName', contactFirstName);
    required('contactLastName', contactLastName);
    required('contactEmail', contactEmail);
    required('contactPhone', contactPhone);

    for (final (field, value) in [
      ('feedbackEmail', feedbackEmail),
      ('contactEmail', contactEmail),
    ]) {
      if (!isUnfilled(value) && !_email.hasMatch(value.trim())) {
        issues.add(
          InfoIssue(
            'email-invalid',
            Severity.error,
            field,
            '"$value" không phải địa chỉ email.',
          ),
        );
      }
    }

    if (!isUnfilled(contactPhone) && !contactPhone.trim().startsWith('+')) {
      issues.add(
        const InfoIssue(
          'contact-phone-format',
          Severity.warning,
          'contactPhone',
          'Trang App Review information của Apple đòi số điện thoại dạng quốc '
              'tế, bắt đầu bằng dấu +.',
        ),
      );
    }

    if (requiresSignIn) {
      required('demoAccountName', demoAccountName);
      required('demoAccountPassword', demoAccountPassword);
    }

    if (utf8.encode(notes).length > notesMaxBytes) {
      issues.add(
        const InfoIssue(
          'notes-too-long',
          Severity.warning,
          'notes',
          'Notes dài hơn 4000 byte (đếm theo UTF-8, tiếng Việt có dấu tốn '
              'nhiều byte hơn số ký tự).',
        ),
      );
    }
    return issues;
  }

  static final _email = RegExp(r'^[^@\s]+@[^@\s]+\.[^@\s]+$');
}
