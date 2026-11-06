import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_25/config/app_config.dart';
import 'package:toi_noi_25/config/flavor.dart';

import 'support/config_files.dart';

Map<String, String> _env({
  String flavor = 'staging',
  String url = 'https://api.staging.toinoi.example',
  String logHttp = 'false',
  String connect = '10000',
  String receive = '15000',
}) => {
  AppConfig.keyFlavor: flavor,
  AppConfig.keyApiBaseUrl: url,
  AppConfig.keyLogHttp: logHttp,
  AppConfig.keyConnectTimeoutMs: connect,
  AppConfig.keyReceiveTimeoutMs: receive,
};

List<String> _problems(Map<String, String> env, {String? nativeFlavor}) {
  try {
    AppConfig.fromMap(env, nativeFlavor: nativeFlavor);
  } on ConfigException catch (error) {
    return error.problems;
  }
  fail('đáng lẽ ném ConfigException');
}

void main() {
  group('hợp lệ', () {
    test('map đầy đủ cho ra AppConfig đúng kiểu', () {
      final config = AppConfig.fromMap(_env());
      expect(config.flavor, Flavor.staging);
      expect(
        config.apiBaseUrl,
        Uri.parse('https://api.staging.toinoi.example'),
      );
      expect(config.logHttp, isFalse);
      expect(config.connectTimeout, const Duration(seconds: 10));
      expect(config.receiveTimeout, const Duration(seconds: 15));
      expect(config.isProd, isFalse);
    });

    for (final name in flavorFileNames) {
      test(
        'file config/$name.json qua cùng đường biến thành chuỗi của công cụ',
        () {
          final config = AppConfig.fromMap(readFlavorDefines(name));
          expect(config.flavor.name, name);
        },
      );
    }

    test('dev được dùng http và máy cục bộ', () {
      for (final url in [
        'http://localhost:8787',
        'http://10.0.2.2:8787',
        'http://192.168.1.50:8787',
        'http://[::1]:8787',
      ]) {
        expect(
          AppConfig.fromMap(_env(flavor: 'dev', url: url, logHttp: 'true'))
              .apiBaseUrl,
          Uri.parse(url),
          reason: url,
        );
      }
    });

    test('URL có tiền tố đường dẫn của gateway vẫn hợp lệ', () {
      final config = AppConfig.fromMap(
        _env(flavor: 'prod', url: 'https://api.toinoi.example/v1'),
      );
      expect(config.apiBaseUrl.path, '/v1');
    });

    test('so sánh theo giá trị', () {
      expect(AppConfig.fromMap(_env()), AppConfig.fromMap(_env()));
      expect(
        AppConfig.fromMap(_env()).hashCode,
        AppConfig.fromMap(_env()).hashCode,
      );
      expect(
        AppConfig.fromMap(_env()),
        isNot(AppConfig.fromMap(_env(connect: '9000'))),
      );
    });
  });

  group('thiếu khoá', () {
    test('map rỗng báo đủ năm khoá trong một lượt', () {
      final problems = _problems({});
      expect(problems, hasLength(AppConfig.keys.length));
      for (final key in AppConfig.keys) {
        expect(
          problems.where((p) => p.contains(key)),
          hasLength(1),
          reason: key,
        );
      }
    });

    test('chuỗi rỗng hoặc toàn dấu cách bị coi là thiếu', () {
      // `String.fromEnvironment` trả '' cho khoá không có define; hai trường
      // hợp phải cùng một kết quả.
      expect(
        _problems(_env(url: '')).single,
        contains('thiếu khoá API_BASE_URL'),
      );
      expect(
        _problems(_env(connect: '   ')).single,
        contains('thiếu khoá CONNECT_TIMEOUT_MS'),
      );
    });

    test('khoá thừa không gây lỗi', () {
      expect(
        AppConfig.fromMap({..._env(), 'KHONG_DUNG': 'x'}).flavor,
        Flavor.staging,
      );
    });
  });

  group('FLAVOR', () {
    test('chỉ nhận đúng dev, staging, prod, không đoán hoa thường', () {
      for (final value in ['Prod', 'production', ' prod', 'qa']) {
        expect(
          _problems(_env(flavor: value)).single,
          contains('FLAVOR chỉ nhận dev, staging hoặc prod'),
          reason: value,
        );
      }
    });

    test('FLAVOR sai thì không áp quy tắc theo flavor lên các khoá khác', () {
      final problems = _problems(_env(flavor: 'qa', url: 'http://localhost:1'));
      expect(problems, hasLength(1));
    });
  });

  group('API_BASE_URL', () {
    test('phải là http(s) tuyệt đối có host', () {
      for (final url in [
        'api.toinoi.example',
        '/v1',
        'ftp://x.example',
        'https://',
        'https:///v1',
        'http://[::1',
      ]) {
        expect(
          _problems(_env(url: url, flavor: 'dev')).single,
          contains('phải là địa chỉ http(s) tuyệt đối có host'),
          reason: url,
        );
      }
    });

    test('staging và prod bắt buộc https, dev thì không', () {
      for (final flavor in ['staging', 'prod']) {
        expect(
          _problems(_env(flavor: flavor, url: 'http://api.toinoi.example')),
          ['$flavor phải dùng https cho API_BASE_URL'],
        );
      }
      expect(
        AppConfig.fromMap(
          _env(
            flavor: 'dev',
            url: 'http://api.toinoi.example',
            logHttp: 'true',
          ),
        ).flavor,
        Flavor.dev,
      );
    });

    test('staging và prod không được trỏ về máy cục bộ', () {
      const locals = [
        'https://localhost',
        'https://app.localhost',
        'https://nas.local',
        'https://127.0.0.1',
        'https://10.0.2.2',
        'https://10.1.2.3',
        'https://172.16.0.9',
        'https://172.31.255.1',
        'https://192.168.0.2',
        'https://169.254.1.1',
        'https://0.0.0.0',
        'https://[::1]',
        'https://[fe80::1]',
        'https://[fd00::1]',
      ];
      for (final flavor in ['staging', 'prod']) {
        for (final url in locals) {
          expect(
            _problems(_env(flavor: flavor, url: url)).single,
            contains('không được trỏ API_BASE_URL tới máy cục bộ'),
            reason: '$flavor $url',
          );
        }
      }
    });

    test('địa chỉ công cộng không bị nhận nhầm là cục bộ', () {
      for (final host in [
        'api.toinoi.example',
        'localhost.toinoi.example',
        'fdaccess.toinoi.example',
        '172.32.0.1',
        '172.15.0.1',
        '11.0.0.1',
        '192.169.0.1',
        '8.8.8.8',
      ]) {
        expect(isLocalHost(host), isFalse, reason: host);
      }
    });

    test('không chứa user:password, query hay fragment; thông báo không lộ giá trị', () {
      const password = 'MatKhauRatBiMat';
      final withUser = _problems(
        _env(url: 'https://app:$password@api.toinoi.example'),
      );
      expect(withUser.single, contains('không được chứa user:password'));
      expect(withUser.join(), isNot(contains(password)));

      const key = 'KhoaTrongQuery';
      final withQuery = _problems(
        _env(url: 'https://api.toinoi.example?key=$key'),
      );
      expect(withQuery.single, contains('query hoặc fragment'));
      expect(withQuery.join(), isNot(contains(key)));

      expect(
        _problems(_env(url: 'https://api.toinoi.example/#a')).single,
        contains('query hoặc fragment'),
      );
    });

    test('URL không parse được cũng không bị in lại', () {
      const raw = 'khong phai url:pw@';
      expect(_problems(_env(url: raw)).join(), isNot(contains('pw@')));
    });
  });

  group('LOG_HTTP', () {
    test('chỉ nhận true hoặc false đúng chữ thường', () {
      for (final value in ['True', 'yes', '1', 'null']) {
        expect(
          _problems(_env(logHttp: value)).single,
          contains('LOG_HTTP chỉ nhận true hoặc false'),
          reason: value,
        );
      }
    });

    test('prod không được bật, staging được', () {
      expect(
        _problems(
          _env(
            flavor: 'prod',
            url: 'https://api.toinoi.example',
            logHttp: 'true',
          ),
        ),
        ['prod không được bật LOG_HTTP'],
      );
      expect(AppConfig.fromMap(_env(logHttp: 'true')).logHttp, isTrue);
    });
  });

  group('timeout', () {
    test('biên 1000 và 60000 mili giây', () {
      expect(
        AppConfig.fromMap(_env(connect: '1000')).connectTimeout,
        const Duration(seconds: 1),
      );
      expect(
        AppConfig.fromMap(_env(receive: '60000')).receiveTimeout,
        const Duration(minutes: 1),
      );
      expect(
        _problems(_env(connect: '999')).single,
        contains('CONNECT_TIMEOUT_MS'),
      );
      expect(
        _problems(_env(receive: '60001')).single,
        contains('RECEIVE_TIMEOUT_MS'),
      );
    });

    test('không phải số nguyên thì báo lỗi, không im lặng dùng mặc định', () {
      for (final value in ['abc', '10.5', '-1', '1e4', '10 000']) {
        expect(
          _problems(_env(connect: value)).single,
          contains('CONNECT_TIMEOUT_MS'),
          reason: value,
        );
      }
    });
  });

  group('so khớp với flavor native (appFlavor)', () {
    test('khớp, hoặc không đặt, thì qua', () {
      expect(
        AppConfig.fromMap(_env(), nativeFlavor: 'staging').flavor,
        Flavor.staging,
      );
      expect(AppConfig.fromMap(_env()).flavor, Flavor.staging);
    });

    test('lệch thì báo lỗi nêu cả hai tên', () {
      final problems = _problems(
        _env(flavor: 'dev', url: 'http://localhost:8787', logHttp: 'true'),
        nativeFlavor: 'prod',
      );
      expect(
        problems.single,
        allOf(contains('--flavor prod'), contains('là dev')),
      );
    });
  });

  test('nhiều lỗi cùng lúc được gom vào một ConfigException', () {
    final problems = _problems(
      _env(
        flavor: 'prod',
        url: 'http://localhost:8787',
        logHttp: 'true',
        connect: '5',
      ),
    );
    expect(problems, hasLength(4));
    final text = ConfigException(problems).toString();
    expect(text, contains('(4 lỗi)'));
    expect(text, contains(' - prod phải dùng https'));
  });
}
