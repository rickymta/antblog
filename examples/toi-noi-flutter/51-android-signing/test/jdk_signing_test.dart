import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_51/jdk_tools.dart';

/// Các test này chạy `keytool`, `jarsigner`, `jar` thật trên keystore TEST
/// sinh trong thư mục tạm với mật khẩu ngẫu nhiên; không file nào vào repo.
/// Không có JDK thì bị bỏ qua, kèm lý do.
void main() {
  final skipReason = jdkToolsAvailable
      ? null
      : 'không có JDK: keytool, jarsigner, jar không chạy được';

  group('JDK thật', () {
    late Directory dir;
    late TestKeystore keystore;

    setUp(() async {
      dir = Directory.systemTemp.createTempSync('tn51-jdk');
      keystore = await TestKeystore.generate(
        directory: dir,
        password: randomTestPassword(),
      );
    });
    tearDown(() => dir.deleteSync(recursive: true));

    group('upload keystore', () {
      test('RSA 2048 bit, đúng yêu cầu tối thiểu của Play', () async {
        expect(
          (await keystore.inspect()).keySizeBits,
          greaterThanOrEqualTo(2048),
        );
      });

      test('hiệu lực quá ngày 22/10/2033 mà Play đòi hỏi', () async {
        expect((await keystore.inspect()).validUntilYear, greaterThan(2033));
      });

      test(
        'vân tay SHA-256 gồm 32 byte hex cách nhau bằng dấu hai chấm',
        () async {
          final cert = await keystore.inspect();
          expect(
            cert.sha256,
            matches(RegExp(r'^([0-9A-F]{2}:){31}[0-9A-F]{2}$')),
          );
          stdout.writeln('SHA-256 upload key TEST: ${cert.sha256}');
        },
      );

      test('mật khẩu sai thì keytool lỗi', () async {
        final bad = await Process.run('keytool', [
          '-list',
          '-keystore',
          keystore.file.path,
          '-storepass',
          'sai-mat-khau',
        ]);
        expect(bad.exitCode, isNot(0));
      });
    });

    group('mất upload key: tạo khoá mới và đăng ký chứng chỉ mới', () {
      test('chứng chỉ PEM công khai không chứa khoá riêng', () async {
        final pem = await keystore.exportCertificatePem(
          File('${dir.path}/upload_certificate.pem'),
        );
        final text = pem.readAsStringSync();
        expect(text, startsWith('-----BEGIN CERTIFICATE-----'));
        expect(text, isNot(contains('PRIVATE KEY')));
      });

      test(
        'keystore mới có vân tay khác: Play Console cần đăng ký lại',
        () async {
          final other = Directory('${dir.path}/new')..createSync();
          final replacement = await TestKeystore.generate(
            directory: other,
            password: randomTestPassword(),
          );
          expect(
            (await replacement.inspect()).sha256,
            isNot((await keystore.inspect()).sha256),
          );
        },
      );
    });

    group('AAB và jarsigner (AAB giả, chỉ thử chữ ký)', () {
      test(
        'bundle chưa ký: jarsigner -verify vẫn thoát mã 0 nhưng báo unsigned',
        () async {
          final bundle = await createStandInBundle(dir);
          final result = await verifyBundle(bundle);
          expect(result.status, SignatureStatus.unsigned);
          // Chính vì mã thoát 0 nên CI phải đọc nội dung, không đọc mã thoát.
          final raw = await Process.run('jarsigner', ['-verify', bundle.path]);
          expect(raw.exitCode, 0);
        },
      );

      test(
        'ký xong thì verified và người ký khớp vân tay upload key',
        () async {
          final bundle = await createStandInBundle(dir);
          await keystore.signBundle(bundle);
          expect((await verifyBundle(bundle)).status, SignatureStatus.verified);
          expect(
            await bundleSignerSha256(bundle),
            (await keystore.inspect()).sha256,
          );
        },
      );

      test('sửa nội dung sau khi ký thì chữ ký không còn hợp lệ', () async {
        final bundle = await createStandInBundle(dir);
        await keystore.signBundle(bundle);
        await replaceEntry(
          bundle,
          'base/manifest/AndroidManifest.xml',
          '<manifest evil="1"/>',
        );
        final result = await verifyBundle(bundle);
        expect(result.status, SignatureStatus.invalid);
        expect(result.output, contains('digest error'));
      });

      test('-strict thoát mã 4 vì chứng chỉ upload tự ký', () async {
        final bundle = await createStandInBundle(dir);
        await keystore.signBundle(bundle);
        final strict = await Process.run('jarsigner', [
          '-verify',
          '-strict',
          bundle.path,
        ]);
        expect(strict.exitCode, 4);
      });

      test('bundle bị sửa sau khi ký: jarsigner thoát mã 1', () async {
        final bundle = await createStandInBundle(dir);
        await keystore.signBundle(bundle);
        await replaceEntry(bundle, 'BundleConfig.pb', 'cfg-da-sua');
        final raw = await Process.run('jarsigner', ['-verify', bundle.path]);
        expect(raw.exitCode, 1);
      });

      test('hai upload key khác nhau cho vân tay người ký khác nhau', () async {
        final bundleA = await createStandInBundle(dir);
        await keystore.signBundle(bundleA);

        final dirB = Directory('${dir.path}/b')..createSync();
        final other = await TestKeystore.generate(
          directory: dirB,
          password: randomTestPassword(),
        );
        final bundleB = await createStandInBundle(dirB);
        await other.signBundle(bundleB);

        expect(
          await bundleSignerSha256(bundleA),
          isNot(await bundleSignerSha256(bundleB)),
        );
      });
    });
  }, skip: skipReason);
}
