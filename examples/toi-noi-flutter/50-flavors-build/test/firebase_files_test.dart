import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

/// Dạng khoá thật mà một file mẫu không được chứa.
final realSecretPatterns = {
  'Google API key': RegExp(r'AIza[0-9A-Za-z_-]{35}'),
  'private key': RegExp(r'-----BEGIN [A-Z ]*PRIVATE KEY-----'),
  'OAuth client': RegExp(r'\d{12}-[a-z0-9]{20,}\.apps\.googleusercontent\.com'),
};

/// Cần `git` và thư mục lab nằm trong một repo (tải về dạng zip thì không có).
bool insideGitRepo() {
  try {
    final result = Process.runSync('git', [
      'rev-parse',
      '--is-inside-work-tree',
    ], workingDirectory: labRoot);
    return result.exitCode == 0;
  } on ProcessException {
    return false;
  }
}

/// `git check-ignore -q`: 0 nghĩa là bị bỏ qua, 1 là không.
bool ignored(String relative) =>
    Process.runSync('git', [
      'check-ignore',
      '-q',
      relative,
    ], workingDirectory: labRoot).exitCode ==
    0;

void main() {
  final contract = readJson('flavors.json')['apps'] as Map<String, dynamic>;
  String idOf(String app, String flavor) =>
      ((contract[app] as Map)['ids'] as Map)[flavor] as String;

  for (final app in apps) {
    firebaseGroups(app, idOf);
  }
  noRealKeys();
  gitignoreTests();
}

void firebaseGroups(
  String app,
  String Function(String app, String flavor) idOf,
) {
  group('google-services.json.example (Android) của $app', () {
    for (final flavor in flavors) {
      test(
        '$flavor: nằm ở src/$flavor và package_name khớp application ID',
        () {
          final json = jsonDecode(
            read(
              native(
                app,
                'android/app/src/$flavor/google-services.json.example',
              ),
            ),
          ) as Map<String, dynamic>;
          final packages = [
            for (final c in json['client'] as List)
              (((c as Map)['client_info'] as Map)['android_client_info']
                      as Map)['package_name']
                  as String,
          ];
          // Plugin google-services dừng với "No matching client found" nếu
          // không client nào có package_name bằng application ID đang build.
          expect(packages, contains(idOf(app, flavor)));
        },
      );
    }

    test('ba flavor có ba mobilesdk_app_id khác nhau', () {
      final ids = <String>{};
      for (final flavor in flavors) {
        final json = readJson(
          native(app, 'android/app/src/$flavor/google-services.json.example'),
        );
        for (final c in json['client'] as List) {
          ids.add(
            ((c as Map)['client_info'] as Map)['mobilesdk_app_id'] as String,
          );
        }
      }
      expect(ids, hasLength(3));
    });
  });

  group('GoogleService-Info.plist.example (iOS) của $app', () {
    for (final flavor in flavors) {
      test('$flavor: BUNDLE_ID khớp bundle ID của flavor', () {
        final plist = plistStrings(
          read(
            native(
              app,
              'ios/Runner/Firebase/$flavor/GoogleService-Info.plist.example',
            ),
          ),
        );
        expect(plist['BUNDLE_ID'], idOf(app, flavor));
      });
    }
  });
}

void noRealKeys() {
  group('không có khoá thật', () {
    final examples = [
      for (final app in apps)
        for (final flavor in flavors) ...[
          native(app, 'android/app/src/$flavor/google-services.json.example'),
          native(
            app,
            'ios/Runner/Firebase/$flavor/GoogleService-Info.plist.example',
          ),
        ],
    ];

    test('mọi API key trong file mẫu là giá trị giả bắt đầu bằng FAKE_', () {
      for (final file in examples) {
        final text = read(file);
        final keys = RegExp(
          r'(?:current_key"\s*:\s*"|<key>API_KEY</key>\s*<string>)([^"<]*)',
        ).allMatches(text).map((m) => m.group(1)!);
        expect(keys, isNotEmpty, reason: file);
        for (final key in keys) {
          expect(key, startsWith('FAKE_'), reason: file);
        }
      }
    });

    test('không file mẫu nào khớp dạng khoá thật', () {
      for (final file in examples) {
        final text = read(file);
        for (final entry in realSecretPatterns.entries) {
          expect(
            entry.value.hasMatch(text),
            isFalse,
            reason: '${entry.key} trong $file',
          );
        }
      }
    });

    test('bộ quét bắt được khoá giống thật (kiểm chính bộ quét)', () {
      final fake = 'AIza${'a' * 35}';
      expect(realSecretPatterns['Google API key']!.hasMatch(fake), isTrue);
      expect(
        realSecretPatterns['Google API key']!.hasMatch(
          'FAKE_API_KEY_REPLACE_ME',
        ),
        isFalse,
      );
    });

    test('native-config không chứa file Firebase thật', () {
      final real = Directory(path('native-config'))
          .listSync(recursive: true)
          .whereType<File>()
          .map((f) => f.uri.pathSegments.last)
          .where(
            (n) =>
                n == 'google-services.json' || n == 'GoogleService-Info.plist',
          );
      expect(real, isEmpty);
    });

    test('config/*.json không chứa khoá thật', () {
      for (final flavor in flavors) {
        final text = read('config/$flavor.json');
        for (final entry in realSecretPatterns.entries) {
          expect(
            entry.value.hasMatch(text),
            isFalse,
            reason: '$flavor ${entry.key}',
          );
        }
      }
    });
  });
}

void gitignoreTests() {
  group('.gitignore', () {
    final skip = insideGitRepo()
        ? null
        : 'cần git và lab nằm trong một git repo';

    test('chặn file Firebase thật ở chỗ runner đọc', () {
      expect(ignored('android/app/src/dev/google-services.json'), isTrue);
      expect(ignored('android/app/google-services.json'), isTrue);
      expect(
        ignored('ios/Runner/Firebase/dev/GoogleService-Info.plist'),
        isTrue,
      );
    }, skip: skip);

    test('chặn file thật dù lỡ đặt trong native-config', () {
      expect(
        ignored(
          'native-config/customer/android/app/src/dev/google-services.json',
        ),
        isTrue,
      );
    }, skip: skip);

    test('không chặn bản .example', () {
      expect(
        ignored(
          'native-config/customer/android/app/src/dev/google-services.json.example',
        ),
        isFalse,
      );
      expect(
        ignored(
          'native-config/customer/ios/Runner/Firebase/dev/GoogleService-Info.plist.example',
        ),
        isFalse,
      );
    }, skip: skip);

    test('/android/ có neo: không nuốt native-config/customer/android', () {
      expect(ignored('android/app/build.gradle.kts'), isTrue);
      expect(
        ignored('native-config/customer/android/app/build.gradle.kts'),
        isFalse,
      );
      expect(
        ignored('native-config/customer/ios/Flutter/Debug-dev.xcconfig'),
        isFalse,
      );
    }, skip: skip);
  });
}
