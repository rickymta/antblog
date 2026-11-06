import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_51/key_properties.dart';

void main() {
  group('KeyProperties.parse', () {
    test('đọc bốn khoá, bỏ comment và dòng trống', () {
      final p = KeyProperties.parse('''
# ghi chú
! cũng là ghi chú

storeFile=/keys/upload-keystore.jks
storePassword = abc123
keyAlias:upload
keyPassword=abc123
''');
      expect(p['storeFile'], '/keys/upload-keystore.jks');
      expect(p['storePassword'], 'abc123');
      expect(p['keyAlias'], 'upload');
      expect(p.isComplete, isTrue);
      expect(p.problem, isNull);
    });

    test('thiếu hoặc rỗng thì báo đúng tên khoá, theo thứ tự cố định', () {
      final p = KeyProperties.parse('keyAlias=upload\nstorePassword=\n');
      expect(p.missingKeys, ['storeFile', 'storePassword', 'keyPassword']);
      expect(
        p.problem,
        'key.properties thiếu khoá: storeFile, storePassword, keyPassword',
      );
    });

    test('giá trị chỉ có khoảng trắng coi như rỗng', () {
      expect(
        KeyProperties.parse('keyAlias=   \n').missingKeys,
        contains('keyAlias'),
      );
    });

    test('tách ở dấu = đầu tiên, giá trị được phép chứa =', () {
      expect(
        KeyProperties.parse('storePassword=a=b=c').values['storePassword'],
        'a=b=c',
      );
    });

    test('hai dấu \\ trên Windows thành một dấu \\', () {
      final p = KeyProperties.parse(
        r'storeFile=C:\\Users\\quan\\upload-keystore.jks',
      );
      expect(p['storeFile'], r'C:\Users\quan\upload-keystore.jks');
    });

    // Kết quả của java.util.Properties.load trên JDK 21 (chạy tay một lần).
    test(r'một dấu \ rồi chữ u thì lỗi, như Gradle: C:\Users\quan\up.jks', () {
      expect(
        () => KeyProperties.parse(r'storeFile=C:\Users\quan\up.jks'),
        throwsA(
          isA<FormatException>().having(
            (e) => e.message,
            'message',
            contains('uxxxx'),
          ),
        ),
      );
    });

    // java.util.Properties trên JDK 21 cho C:Usersquankeys.jks (chạy tay).
    test(r'các dấu \ khác bị nuốt âm thầm, không báo lỗi', () {
      final p = KeyProperties.parse(r'storeFile=C:\Users\quan\keys.jks');
      expect(p['storeFile'], 'C:Usersquankeys.jks');
    });

    test(r'\t là ký tự tab', () {
      expect(KeyProperties.parse(r'a=b\tc')['a'], 'b\tc');
    });

    test('giữ khoảng trắng ở cuối giá trị, như Java', () {
      expect(KeyProperties.parse('k = v  ')['k'], 'v  ');
    });
  });

  group('native-config/key.properties.example', () {
    late KeyProperties example;
    setUp(() {
      example = KeyProperties.parse(
        File('native-config/key.properties.example').readAsStringSync(),
      );
    });

    test('đủ bốn khoá bắt buộc', () {
      expect(example.isComplete, isTrue, reason: example.problem);
    });

    test('mật khẩu chỉ là chỗ giữ chỗ, không phải bí mật thật', () {
      expect(example['storePassword'], 'CHANGE_ME');
      expect(example['keyPassword'], 'CHANGE_ME');
    });

    test('alias theo tài liệu Flutter và keystore nằm ngoài repository', () {
      expect(example['keyAlias'], 'upload');
      expect(example['storeFile'], startsWith('/'));
    });
  });
}
