import 'dart:io';

import 'package:toi_noi_51/jdk_tools.dart';

/// Chạy trọn quy trình ký trên keystore TEST trong thư mục tạm: sinh upload
/// keystore, in vân tay SHA-256, ký một AAB giả bằng jarsigner, kiểm lại.
/// Mọi thứ bị xoá khi xong; không file nào vào repository.
Future<void> main() async {
  if (!jdkToolsAvailable) {
    stdout.writeln('Bỏ qua: không có JDK (keytool, jarsigner, jar chạy được).');
    return;
  }
  final dir = Directory.systemTemp.createTempSync('toi-noi-51-');
  try {
    final keystore = await TestKeystore.generate(
      directory: dir,
      password: randomTestPassword(),
    );
    final cert = await keystore.inspect();
    stdout.writeln('Upload key (TEST): ${cert.owner}');
    stdout.writeln(
      '  RSA ${cert.keySizeBits} bit, hết hạn năm ${cert.validUntilYear}',
    );
    stdout.writeln('  SHA-256: ${cert.sha256}');

    final bundle = await createStandInBundle(dir);
    stdout.writeln('Trước khi ký: ${(await verifyBundle(bundle)).status.name}');
    await keystore.signBundle(bundle);
    stdout.writeln('Sau khi ký:   ${(await verifyBundle(bundle)).status.name}');
    stdout.writeln('  người ký SHA-256: ${await bundleSignerSha256(bundle)}');
  } finally {
    dir.deleteSync(recursive: true);
  }
}
