import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

Future<void> main(List<String> args) async {
  final ServerOptions options;
  try {
    options = parseServerOptions(args);
  } on FormatException catch (e) {
    stderr
      ..writeln(e.message)
      ..writeln(serverUsage);
    exitCode = 64;
    return;
  }
  if (options.showHelp) {
    stdout.writeln(serverUsage);
    return;
  }

  final auth = options.withAuth ? AuthModule() : null;
  final handler = const Pipeline()
      .addMiddleware(logRequests())
      .addHandler(
        createFakeApiHandler(
          faults: options.faults,
          modules: [?auth],
          middleware: [?auth?.requireBearer],
        ),
      );
  final server = await shelf_io.serve(handler, options.host, options.port);
  stdout.writeln(
    'Tới Nơi fake API: http://${server.address.host}:${server.port} '
    '(Ctrl+C để dừng)',
  );
  if (auth != null) {
    stdout.writeln(
      'Xác thực giả đang bật: mã OTP cố định $fakeOtpCode (CHỈ DÀNH CHO FAKE), '
      '/trips đòi Authorization: Bearer.',
    );
  }
}
