import 'package:args/args.dart';

import 'faults.dart';

/// Tuỳ chọn dòng lệnh của `bin/server.dart`, tách ra để test không cần socket.
final class ServerOptions {
  const ServerOptions({
    required this.host,
    required this.port,
    required this.faults,
    required this.showHelp,
    this.withAuth = false,
  });

  final String host;
  final int port;
  final FaultConfig faults;
  final bool showHelp;

  /// Bật `AuthModule` (OTP, JWT, refresh token) và đòi Bearer ở `/trips`.
  final bool withAuth;
}

ArgParser _parser() => ArgParser()
  ..addOption(
    'host',
    defaultsTo: '127.0.0.1',
    help: 'Địa chỉ lắng nghe. Dùng 0.0.0.0 để thiết bị thật trong LAN gọi tới.',
  )
  ..addOption('port', defaultsTo: '8787', help: 'Cổng lắng nghe.')
  ..addOption(
    'latency-ms',
    defaultsTo: '0',
    help: 'Chờ chừng này mili giây trước mỗi request.',
  )
  ..addOption(
    'fail-first',
    defaultsTo: '0',
    help: 'N request đầu tiên trả 500.',
  )
  ..addOption(
    'fail-every',
    defaultsTo: '0',
    help: 'Cứ mỗi N request thì một request trả 500 (0 là tắt).',
  )
  ..addFlag(
    'with-auth',
    negatable: false,
    help:
        'Bật xác thực giả (OTP cố định, JWT không ký) và đòi Bearer ở /trips.',
  )
  ..addFlag('help', abbr: 'h', negatable: false, help: 'Hiện trợ giúp.');

/// Nội dung trợ giúp của dòng lệnh.
String get serverUsage => _parser().usage;

/// Đọc [args]; ném [FormatException] khi tuỳ chọn sai hoặc ngoài miền.
ServerOptions parseServerOptions(List<String> args) {
  final ArgResults results;
  try {
    results = _parser().parse(args);
  } on ArgParserException catch (e) {
    throw FormatException(e.message);
  }

  int number(String name, {required int min, int max = 1 << 31}) {
    final text = results.option(name)!;
    final value = int.tryParse(text);
    if (value == null || value < min || value > max) {
      throw FormatException(
        '--$name phải là số nguyên từ $min tới $max, nhận "$text"',
      );
    }
    return value;
  }

  return ServerOptions(
    host: results.option('host')!,
    port: number('port', min: 0, max: 65535),
    faults: FaultConfig(
      latency: Duration(milliseconds: number('latency-ms', min: 0)),
      failFirst: number('fail-first', min: 0),
      failEvery: number('fail-every', min: 0),
    ),
    showHelp: results.flag('help'),
    withAuth: results.flag('with-auth'),
  );
}
