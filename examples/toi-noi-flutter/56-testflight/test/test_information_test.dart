import 'dart:convert';

import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';

TestInformation filled({
  bool requiresSignIn = true,
  String demoName = 'demo.taixe@toinoi.example',
  String demoPassword = 'mat-khau-demo-gia',
  String notes = 'Ghi chú.',
  String phone = '+84000000000',
  String feedbackEmail = 'beta@toinoi.example',
  String contactEmail = 'lienhe@toinoi.example',
  String description = 'Bản thử của Tới Nơi.',
}) => TestInformation(
  betaAppDescription: description,
  feedbackEmail: feedbackEmail,
  contactFirstName: 'An',
  contactLastName: 'Nguyễn',
  contactEmail: contactEmail,
  contactPhone: phone,
  requiresSignIn: requiresSignIn,
  demoAccountName: demoName,
  demoAccountPassword: demoPassword,
  notes: notes,
);

List<String> codes(TestInformation t) => [
  for (final i in t.validateForExternal()) '${i.code}:${i.field}',
];

void main() {
  test('đủ thông tin thì không có phát hiện nào', () {
    expect(filled().validateForExternal(), isEmpty);
  });

  test('isUnfilled: rỗng, khoảng trắng và giá trị giữ chỗ <...>', () {
    expect(TestInformation.isUnfilled(''), isTrue);
    expect(TestInformation.isUnfilled('  \n'), isTrue);
    expect(TestInformation.isUnfilled('<điền ở đây>'), isTrue);
    expect(TestInformation.isUnfilled('  <x> '), isTrue);
    expect(TestInformation.isUnfilled('có giá trị'), isFalse);
    expect(TestInformation.isUnfilled('a <b>'), isFalse);
  });

  test('mô tả beta rỗng là lỗi, vì Apple ghi trường này bắt buộc', () {
    expect(codes(filled(description: ' ')), [
      'info-unfilled:betaAppDescription',
    ]);
  });

  test('app đòi đăng nhập: thiếu tài khoản demo hoặc mật khẩu là lỗi', () {
    expect(codes(filled(demoName: '', demoPassword: '<điền>')), [
      'info-unfilled:demoAccountName',
      'info-unfilled:demoAccountPassword',
    ]);
  });

  test('app không đòi đăng nhập: không cần tài khoản demo', () {
    expect(
      filled(
        requiresSignIn: false,
        demoName: '',
        demoPassword: '',
      ).validateForExternal(),
      isEmpty,
    );
  });

  test('email phản hồi và email liên hệ phải có dạng email', () {
    expect(codes(filled(feedbackEmail: 'khong-phai-email')), [
      'email-invalid:feedbackEmail',
    ]);
    expect(codes(filled(contactEmail: 'a@b')), ['email-invalid:contactEmail']);
  });

  test('email còn giữ chỗ chỉ báo chưa điền, không báo thêm sai dạng', () {
    expect(codes(filled(feedbackEmail: '<email>')), [
      'info-unfilled:feedbackEmail',
    ]);
  });

  test('số điện thoại không bắt đầu bằng + là cảnh báo, không phải lỗi', () {
    final issues = filled(phone: '0900000000').validateForExternal();
    expect(issues.single.code, 'contact-phone-format');
    expect(issues.single.severity, Severity.warning);
  });

  test(
    'Notes: đúng 4000 byte qua, 4001 byte cảnh báo; đếm theo byte UTF-8',
    () {
      expect(filled(notes: 'a' * 4000).validateForExternal(), isEmpty);
      expect(
        filled(notes: 'a' * 4001).validateForExternal().single.code,
        'notes-too-long',
      );
      // "ế" tốn 3 byte UTF-8: 1334 ký tự là 4002 byte dù ít hơn 4000 ký tự.
      final vn = 'ế' * 1334;
      expect(vn.length, lessThan(4000));
      expect(utf8.encode(vn).length, 4002);
      expect(
        filled(notes: vn).validateForExternal().single.code,
        'notes-too-long',
      );
    },
  );

  group('fromJson', () {
    String json(Map<String, Object?> m) => jsonEncode(m);

    Map<String, Object?> base() => {
      'betaAppDescription': 'd',
      'feedbackEmail': 'a@b.vn',
      'contactFirstName': 'f',
      'contactLastName': 'l',
      'contactEmail': 'c@d.vn',
      'contactPhone': '+84',
      'requiresSignIn': false,
    };

    test('đọc đủ khoá, khoá tuỳ chọn mặc định rỗng', () {
      final t = TestInformation.fromJson(json(base()));
      expect(t.requiresSignIn, isFalse);
      expect(t.demoAccountName, '');
      expect(t.notes, '');
    });

    test('thiếu khoá bắt buộc hoặc sai kiểu ném FormatException', () {
      for (final key in base().keys) {
        final m = base()..remove(key);
        expect(
          () => TestInformation.fromJson(json(m)),
          throwsA(isA<FormatException>()),
          reason: 'thiếu $key',
        );
      }
      expect(
        () =>
            TestInformation.fromJson(json(base()..['requiresSignIn'] = 'true')),
        throwsA(isA<FormatException>()),
      );
      expect(
        () => TestInformation.fromJson(json(base()..['notes'] = 5)),
        throwsA(isA<FormatException>()),
      );
    });

    test('JSON hỏng hoặc gốc không phải object ném FormatException', () {
      expect(() => TestInformation.fromJson('{'), throwsFormatException);
      expect(() => TestInformation.fromJson('[]'), throwsFormatException);
    });
  });
}
