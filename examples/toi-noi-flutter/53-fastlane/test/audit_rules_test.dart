import 'package:test/test.dart';
import 'package:toi_noi_53/native_config_audit.dart';

import 'support.dart';

/// Ca sai: mỗi test sửa đúng một chỗ của file thật và kỳ vọng audit đỏ đúng
/// mã tương ứng. Nếu audit lỏng, test này đỏ.
void main() {
  final real = readNativeConfig();

  // Giá trị giả ghép từ mảnh để repo không chứa chuỗi giống bí mật.
  final fakePem = '-----BEGIN ${'PRIVATE'} KEY-----';
  final fakeToken = 'a' * 20 + 'B' * 20 + '0' * 4;

  group('bí mật', () {
    test('khoá PEM trong Fastfile', () {
      final files = mutated(
        real,
        iosFastfile,
        'default_platform(:ios)',
        'KEY = "$fakePem"',
      );
      expect(codesOf(files), contains('bí-mật'));
    });

    test('chuỗi dài giống token trong Fastfile', () {
      final files = mutated(
        real,
        androidFastfile,
        'default_platform(:android)',
        'TOKEN = "$fakeToken"',
      );
      expect(codesOf(files), contains('bí-mật'));
    });

    test('giá trị điền sẵn trong .env.example', () {
      final files = mutated(
        real,
        envExample,
        'MATCH_PASSWORD=\n',
        'MATCH_PASSWORD=abc\n',
      );
      expect(codesOf(files), contains('bí-mật'));
    });

    test('key_id gõ literal thay vì ENV', () {
      final files = mutated(
        real,
        iosFastfile,
        'key_id: env!("ASC_KEY_ID")',
        'key_id: "ABC123"',
      );
      expect(codesOf(files), contains('bí-mật-literal'));
    });

    test('khoá Play gõ literal', () {
      final files = mutated(
        real,
        androidFastfile,
        'json_key_data: env!("PLAY_JSON_KEY_DATA")',
        'json_key_data: "{}"',
      );
      expect(codesOf(files), contains('bí-mật-literal'));
    });

    test('chuỗi giống bí mật nằm trong comment không làm đỏ audit', () {
      final files = mutated(
        real,
        iosFastfile,
        'default_platform(:ios)',
        '# KEY = "$fakePem"\ndefault_platform(:ios)',
      );
      expect(auditNativeConfig(files), isEmpty);
    });
  });

  group('ENV', () {
    test('đọc biến chưa khai trong .env.example', () {
      final files = mutated(
        real,
        androidFastfile,
        'env!("PLAY_JSON_KEY_DATA")',
        'env!("PLAY_KEY")',
      );
      expect(
        codesOf(files),
        containsAll(['env-chưa-khai', 'env-thừa', 'env-lane']),
      );
    });

    test('đọc ENV[...] trần, bỏ kiểm rỗng của env!', () {
      final files = mutated(
        real,
        iosFastfile,
        'env!("ASC_KEY_CONTENT")',
        'ENV["ASC_KEY_CONTENT"]',
      );
      expect(codesOf(files), contains('env-lane'));
    });
  });

  group('bundle ID', () {
    test('sai hậu tố staging ở iOS', () {
      final files = mutated(
        real,
        iosFastfile,
        '"vn.toinoi.customer.staging"',
        '"vn.toinoi.customer.stg"',
      );
      expect(codesOf(files), contains('bundle-id'));
    });

    test('prod mang hậu tố ở Android', () {
      final files = mutated(
        real,
        androidFastfile,
        '"vn.toinoi.driver"',
        '"vn.toinoi.driver.prod"',
      );
      expect(codesOf(files), contains('bundle-id'));
    });
  });

  group('flavor', () {
    test('thêm dev vào danh sách flavor được phân phối', () {
      final files = mutated(
        real,
        iosFastfile,
        '%w[staging prod]',
        '%w[dev staging prod]',
      );
      expect(codesOf(files), contains('flavor'));
    });
  });

  group('Android', () {
    test('track production', () {
      final files = mutated(
        real,
        androidFastfile,
        'track: "internal"',
        'track: "production"',
      );
      expect(codesOf(files), containsAll(['android-track']));
    });

    test('đẩy APK thay AAB', () {
      final files = mutated(
        real,
        androidFastfile,
        'aab: "../#{aab}"',
        'apk: "../#{aab}"',
      );
      expect(codesOf(files), contains('android-aab'));
    });

    test('promote sang track khác', () {
      final files = mutated(
        real,
        androidFastfile,
        'track: "internal",',
        'track: "internal",\n      track_promote_to: "beta",',
      );
      expect(codesOf(files), contains('android-track'));
    });
  });

  group('iOS', () {
    test('match không chỉ đọc trong lane beta', () {
      final files = mutated(
        real,
        iosFastfile,
        'match(type: "appstore", app_identifier: bundle_id, readonly: true, api_key: api_key)',
        'match(type: "appstore", app_identifier: bundle_id, readonly: false, api_key: api_key)',
      );
      expect(codesOf(files), containsAll(['ios-match']));
    });

    test('sync_certs bỏ kiểm CI', () {
      final files = mutated(
        real,
        iosFastfile,
        '    UI.user_error!("sync_certs chỉ chạy trên máy quản trị, không chạy trên CI") if is_ci\n',
        '',
      );
      expect(codesOf(files), contains('ios-sync'));
    });

    test('đăng nhập bằng Apple ID', () {
      final files = mutated(
        real,
        iosAppfile,
        'team_id(',
        'apple_id("dev@toinoi.example")\nteam_id(',
      );
      expect(codesOf(files), contains('ios-mật-khẩu'));
    });

    test('tự phát cho tester ngoài', () {
      final files = mutated(
        real,
        iosFastfile,
        'skip_waiting_for_build_processing: true,',
        'skip_waiting_for_build_processing: true,\n      distribute_external: true,',
      );
      expect(codesOf(files), contains('ios-pilot'));
    });

    test('Matchfile bỏ readonly mặc định', () {
      final files = mutated(
        real,
        iosMatchfile,
        'readonly(true)',
        'readonly(false)',
      );
      expect(codesOf(files), contains('matchfile'));
    });

    test('quên flavor khi build IPA', () {
      final files = mutated(real, iosFastfile, '--flavor #{flavor} ', '');
      expect(codesOf(files), contains('ios-build'));
    });
  });

  group('ký và scheme', () {
    test('gym thiếu scheme', () {
      final files = mutated(real, iosFastfile, '        scheme: flavor,\n', '');
      expect(codesOf(files), contains('ios-scheme'));
    });

    test(
      'bỏ update_code_signing_settings: staging sẽ ký bằng profile ad-hoc',
      () {
        final files = mutated(
          real,
          iosFastfile,
          'build_configurations: ["Release-#{flavor}"]',
          'build_configurations: ["Release"]',
        );
        expect(codesOf(files), contains('ios-signing'));
      },
    );

    test('update_code_signing_settings đặt sau flutter build ipa', () {
      final text = real[iosFastfile]!;
      final start = text.indexOf('      update_code_signing_settings(');
      final end = text.indexOf('      sh("cd ../..');
      final moved =
          text.substring(0, start) +
          text.substring(end, text.indexOf('      # flutter build ipa trả')) +
          text.substring(start, end) +
          text.substring(text.indexOf('      # flutter build ipa trả'));
      expect(codesOf({...real, iosFastfile: moved}), contains('ios-signing'));
    });

    test('đòi build_number ngoài nhánh build', () {
      final files = mutated(
        real,
        iosFastfile,
        '    api_key = asc_api_key\n',
        '    api_key = asc_api_key\n    build_number!(options)\n',
      );
      expect(codesOf(files), contains('ios-ipa-branch'));
    });

    test('git_basic_authorization gõ literal trong Matchfile', () {
      final files = mutated(
        real,
        iosMatchfile,
        'git_basic_authorization(ENV["MATCH_GIT_BASIC_AUTHORIZATION"])',
        'git_basic_authorization("dXNlcjp0b2tlbg==")',
      );
      expect(codesOf(files), contains('bí-mật-literal'));
    });
  });

  group('Gemfile và file thiếu', () {
    test('Gemfile ghim theo minor nhận cả 2.999.0', () {
      final files = mutated(real, iosGemfile, '"~> 2.241.2"', '"~> 2.241"');
      expect(codesOf(files), contains('gemfile'));
    });

    test('mẫu gitignore thiếu .env.*', () {
      final files = mutated(real, androidGitignore, '.env.*\n', '');
      expect(codesOf(files), contains('gitignore'));
    });

    test('Gemfile không ghim phiên bản fastlane', () {
      final files = mutated(
        real,
        iosGemfile,
        'gem "fastlane", "~> 2.241.2"',
        'gem "fastlane"',
      );
      expect(codesOf(files), contains('gemfile'));
    });

    test('thiếu Fastfile Android', () {
      final files = {...real}..remove(androidFastfile);
      expect(codesOf(files), contains('thiếu-file'));
    });
  });

  group('trình phân tích', () {
    test('laneBody trả null khi không có lane', () {
      expect(laneBody(real[iosFastfile]!, 'khong_co'), isNull);
    });

    test('callBody xử lý ngoặc lồng nhau', () {
      expect(callBody('a(b(c), d) e(f)', 'a'), 'b(c), d');
    });

    test('codeOf bỏ dòng comment thụt lề', () {
      expect(codeOf('  # x\ny'), 'y');
    });
  });
}
