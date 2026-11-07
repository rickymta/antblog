import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_53/native_config_audit.dart';
import 'package:toi_noi_53/release_plan.dart';

import 'support.dart';

void main() {
  final files = readNativeConfig();

  test('native-config có đủ file mà bài nói tới', () {
    expect(files.keys.toSet(), {
      '.env.example',
      'ios/Gemfile',
      'ios/fastlane/Fastfile',
      'ios/fastlane/Appfile',
      'ios/fastlane/Matchfile',
      'android/Gemfile',
      'android/fastlane/Fastfile',
      'ios/gitignore.fragment.example',
      'android/gitignore.fragment.example',
    });
  });

  test('toàn bộ audit đạt trên file thật', () {
    expect(auditNativeConfig(files), isEmpty);
  });

  group('lane đủ và khớp release_plan', () {
    test('iOS có beta và sync_certs, Android có internal', () {
      expect(lanesIn(files[iosFastfile]!), {'beta', 'sync_certs'});
      expect(lanesIn(files[androidFastfile]!), {'internal'});
    });

    test('mọi lane phân phối trong release_plan có mặt trong Fastfile', () {
      for (final flavor in Flavor.values) {
        for (final spec in lanesFor(flavor)) {
          final fastfile =
              files[spec.platform == StorePlatform.ios
                  ? iosFastfile
                  : androidFastfile]!;
          expect(lanesIn(fastfile), contains(spec.lane));
        }
      }
    });

    test('Fastfile chỉ cho phép flavor mà lanesFor có lane', () {
      for (final path in [iosFastfile, androidFastfile]) {
        expect(storeFlavorsIn(files[path]!), {'staging', 'prod'}, reason: path);
      }
    });
  });

  group('bí mật chỉ đến từ ENV', () {
    test('.env.example khai báo đúng các biến mã đọc và để trống hết', () {
      final declared = envDeclared(files[envExample]!);
      expect(declared.keys.toSet(), {
        'ASC_KEY_ID',
        'ASC_ISSUER_ID',
        'ASC_KEY_CONTENT',
        'MATCH_GIT_URL',
        'MATCH_PASSWORD',
        'MATCH_GIT_BASIC_AUTHORIZATION',
        'MATCH_GIT_PRIVATE_KEY',
        'APPLE_TEAM_ID',
        'PLAY_JSON_KEY_DATA',
      });
      expect(declared.values.every((v) => v.isEmpty), isTrue);
    });

    test('API key của App Store Connect lấy từ ENV, không có Apple ID', () {
      final code = codeOf(files[iosFastfile]!);
      final call = callBody(code, 'app_store_connect_api_key')!;
      expect(call, contains('key_id: env!("ASC_KEY_ID")'));
      expect(call, contains('issuer_id: env!("ASC_ISSUER_ID")'));
      expect(call, contains('key_content: env!("ASC_KEY_CONTENT")'));
      expect(call, contains('is_key_content_base64: true'));
      expect(code, isNot(contains('apple_id')));
      expect(code, isNot(contains('FASTLANE_PASSWORD')));
    });

    test('khoá Play đọc từ ENV, không từ file trong repo', () {
      final code = codeOf(files[androidFastfile]!);
      expect(code, contains('env!("PLAY_JSON_KEY_DATA")'));
      expect(code, isNot(contains('json_key_file')));
      expect(code, isNot(contains('service_account')));
    });

    test('không có file bí mật nằm cạnh cấu hình', () {
      for (final path in files.keys) {
        expect(path, isNot(endsWith('.p8')));
        expect(path, isNot(endsWith('.keystore')));
        expect(path, isNot(endsWith('.jks')));
        expect(path, isNot(equals('.env')));
      }
    });
  });

  group('match', () {
    test('Matchfile: kho git, loại appstore, mặc định chỉ đọc', () {
      final code = codeOf(files[iosMatchfile]!);
      expect(code, contains('git_url(ENV["MATCH_GIT_URL"])'));
      expect(code, contains('type("appstore")'));
      expect(code, contains('readonly(true)'));
    });

    test('lane beta gọi match chỉ đọc; chỉ sync_certs được ghi', () {
      final ios = files[iosFastfile]!;
      expect(
        callBody(laneBody(ios, 'beta')!, 'match'),
        contains('readonly: true'),
      );
      expect(
        callBody(laneBody(ios, 'sync_certs')!, 'match'),
        contains('readonly: false'),
      );
    });

    test('Matchfile đọc xác thực repo từ ENV, không phải biến bắt buộc', () {
      final code = codeOf(files[iosMatchfile]!);
      expect(
        code,
        contains(
          'git_basic_authorization(ENV["MATCH_GIT_BASIC_AUTHORIZATION"])',
        ),
      );
      expect(code, contains('git_private_key(ENV["MATCH_GIT_PRIVATE_KEY"])'));
      expect(
        requiredEnv(StorePlatform.ios),
        isNot(contains('MATCH_GIT_PRIVATE_KEY')),
      );
    });

    test('sync_certs từ chối chạy trên CI', () {
      expect(laneBody(files[iosFastfile]!, 'sync_certs'), contains('if is_ci'));
    });
  });

  group('gym và pilot', () {
    test('gym xuất IPA từ archive của flutter build ipa, ký bằng profile của match', () {
      final beta = laneBody(files[iosFastfile]!, 'beta')!;
      final gym = callBody(beta, 'gym')!;
      expect(gym, contains('skip_build_archive: true'));
      expect(
        gym,
        contains('archive_path: "../build/ios/archive/Runner.xcarchive"'),
      );
      expect(gym, contains('export_method: "app-store"'));
      expect(gym, contains('SharedValues::MATCH_PROVISIONING_PROFILE_MAPPING'));
    });

    test('gym truyền scheme: flavor vì project có ba scheme', () {
      final beta = laneBody(files[iosFastfile]!, 'beta')!;
      expect(callBody(beta, 'gym'), contains('scheme: flavor'));
    });

    test('ép Release-<flavor> ký bằng profile của match, trước khi build', () {
      final beta = laneBody(files[iosFastfile]!, 'beta')!;
      final call = callBody(beta, 'update_code_signing_settings')!;
      expect(call, contains('use_automatic_signing: false'));
      expect(call, contains('build_configurations: ["Release-#{flavor}"]'));
      expect(call, contains('MATCH_PROVISIONING_PROFILE_MAPPING][bundle_id]'));
      expect(
        beta.indexOf('match('),
        lessThan(beta.indexOf('update_code_signing_settings(')),
      );
      expect(
        beta.indexOf('update_code_signing_settings('),
        lessThan(beta.indexOf('flutter build ipa')),
      );
    });

    test('lane kiểm archive tồn tại vì flutter build ipa trả thành công cả khi xuất IPA lỗi', () {
      final beta = laneBody(files[iosFastfile]!, 'beta')!;
      expect(
        beta,
        contains('File.exist?("../../build/ios/archive/Runner.xcarchive")'),
      );
    });

    test('upload_to_testflight dùng API key và chỉ nội bộ', () {
      final beta = laneBody(files[iosFastfile]!, 'beta')!;
      final pilot = callBody(beta, 'upload_to_testflight')!;
      expect(pilot, contains('api_key: api_key'));
      expect(pilot, contains('skip_waiting_for_build_processing: true'));
      expect(pilot, isNot(contains('distribute_external')));
    });

    test('có thể dùng IPA có sẵn: ipa:<đường dẫn> bỏ qua match và build', () {
      final beta = laneBody(files[iosFastfile]!, 'beta')!;
      expect(beta, contains('options[:ipa]'));
      final branch = beta.indexOf('if ipa.empty?');
      expect(branch, greaterThan(0));
      for (final needed in [
        'build_number!(options)',
        'env!("MATCH_GIT_URL")',
        'env!("MATCH_PASSWORD")',
        'match(',
      ]) {
        expect(beta.indexOf(needed), greaterThan(branch), reason: needed);
      }
      expect(beta.indexOf('asc_api_key'), lessThan(branch));
    });
  });

  group('.env nằm ngoài repo', () {
    test('mẫu gitignore của ios/ và android/ chặn .env và .env.*', () {
      for (final path in [iosGitignore, androidGitignore]) {
        final lines = codeOf(files[path]!).split('\n').map((l) => l.trim());
        expect(lines, containsAll(['.env', '.env.*']), reason: path);
      }
    });

    test('.env.example nói rõ fastlane đọc .env ở ios/ và android/', () {
      final text = files[envExample]!;
      expect(text, contains('ios/.env'));
      expect(text, contains('android/.env'));
    });
  });

  group('supply', () {
    test('track internal, AAB, không đường vòng lên track khác', () {
      final code = codeOf(files[androidFastfile]!);
      final upload = callBody(code, 'upload_to_play_store')!;
      expect(upload, contains('track: "internal"'));
      expect(upload, contains('aab: "../#{aab}"'));
      expect(upload, isNot(contains('apk:')));
      for (final banned in [
        'production',
        'beta',
        'alpha',
        'track_promote_to',
      ]) {
        expect(code, isNot(contains(banned)), reason: banned);
      }
    });

    test('đường dẫn AAB có flavor đúng quy ước của Flutter', () {
      final code = codeOf(files[androidFastfile]!);
      expect(
        code,
        contains(
          'build/app/outputs/bundle/#{flavor}Release/app-#{flavor}-release.aab',
        ),
      );
    });

    test('chỉ đẩy bản nhị phân; metadata và ảnh cửa hàng thuộc bài sau', () {
      final upload = callBody(
        codeOf(files[androidFastfile]!),
        'upload_to_play_store',
      )!;
      for (final skip in [
        'skip_upload_metadata',
        'skip_upload_images',
        'skip_upload_screenshots',
        'skip_upload_changelogs',
      ]) {
        expect(upload, contains('$skip: true'));
      }
    });
  });

  group(
    'bundle ID khớp curriculum §3.2 (viết tay, độc lập với applicationId)',
    () {
      const expected = {
        'customer/dev': 'vn.toinoi.customer.dev',
        'customer/staging': 'vn.toinoi.customer.staging',
        'customer/prod': 'vn.toinoi.customer',
        'driver/dev': 'vn.toinoi.driver.dev',
        'driver/staging': 'vn.toinoi.driver.staging',
        'driver/prod': 'vn.toinoi.driver',
      };

      test('Fastfile iOS và Android cùng bảng', () {
        expect(appIdsIn(files[iosFastfile]!), expected);
        expect(appIdsIn(files[androidFastfile]!), expected);
      });
    },
  );

  group('build', () {
    test(
      'cả hai lane build với flavor, build number và dart-define-from-file',
      () {
        for (final (path, lane, command) in [
          (iosFastfile, 'beta', 'flutter build ipa'),
          (androidFastfile, 'internal', 'flutter build appbundle'),
        ]) {
          final body = laneBody(files[path]!, lane)!;
          expect(body, contains(command), reason: path);
          expect(body, contains('--flavor #{flavor}'), reason: path);
          expect(
            body,
            contains('--build-number #{build_number}'),
            reason: path,
          );
          expect(
            body,
            contains('--dart-define-from-file=config/#{flavor}.json'),
            reason: path,
          );
        }
      },
    );

    test('Gemfile ghim fastlane theo bản vá (~> 2.241.2) ở cả hai thư mục', () {
      for (final path in [iosGemfile, androidGemfile]) {
        expect(
          files[path],
          contains('gem "fastlane", "~> 2.241.2"'),
          reason: path,
        );
      }
    });
  });

  test('cú pháp Ruby hợp lệ (ruby -c), bỏ qua nếu máy không có ruby', () {
    final which = Process.runSync('which', ['ruby']);
    if (which.exitCode != 0) {
      markTestSkipped('không có ruby trên PATH');
      return;
    }
    for (final path in [
      iosFastfile,
      iosAppfile,
      iosMatchfile,
      androidFastfile,
      iosGemfile,
      androidGemfile,
    ]) {
      final result = Process.runSync('ruby', ['-c', 'native-config/$path']);
      expect(result.exitCode, 0, reason: '$path: ${result.stderr}');
    }
  });
}
