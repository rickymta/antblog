import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

void main() {
  final contract = readJson('flavors.json')['apps'] as Map<String, dynamic>;

  String contractId(String app, String flavor) =>
      ((contract[app] as Map)['ids'] as Map)[flavor] as String;
  String contractName(String app, String flavor) =>
      ((contract[app] as Map)['displayNames'] as Map)[flavor] as String;

  for (final app in apps) {
    group('build.gradle.kts của $app', () {
      final gradle = parseGradle(
        read(native(app, 'android/app/build.gradle.kts')),
      );

      test('có đúng ba productFlavors dev, staging, prod', () {
        expect(gradle.flavors.keys, unorderedEquals(flavors));
      });

      test('một chiều flavor duy nhất và cả ba flavor thuộc chiều đó', () {
        expect(gradle.dimensions, hasLength(1));
        for (final flavor in flavors) {
          expect(gradle.flavors[flavor]!.dimension, gradle.dimensions.single);
        }
      });

      test('application ID ghép từ applicationId và hậu tố khớp hợp đồng', () {
        for (final flavor in flavors) {
          expect(
            gradle.idOf(flavor),
            contractId(app, flavor),
            reason: '$app/$flavor',
          );
        }
      });

      test('prod không có hậu tố, dev và staging có', () {
        expect(gradle.flavors['prod']!.suffix, '');
        expect(gradle.flavors['dev']!.suffix, '.dev');
        expect(gradle.flavors['staging']!.suffix, '.staging');
      });

      test('không dùng resValue (AGP 9 tắt mặc định)', () {
        expect(gradle.text, isNot(contains('resValue(')));
      });
    });

    group('tên app của $app', () {
      test('mỗi flavor có strings.xml với app_name khớp hợp đồng', () {
        for (final flavor in flavors) {
          final xml = read(
            native(app, 'android/app/src/$flavor/res/values/strings.xml'),
          );
          expect(appName(xml), contractName(app, flavor), reason: flavor);
        }
      });
    });
  }

  test(
    'namespace không đổi theo flavor (là package của code, không phải ID)',
    () {
      for (final app in apps) {
        final text = read(native(app, 'android/app/build.gradle.kts'));
        expect(text, contains('namespace = "vn.toinoi.$app"'));
      }
    },
  );

  for (final app in apps) {
    test(
      'manifest của $app lấy tên và icon từ tài nguyên, không viết cứng',
      () {
        final manifest = read(
          native(app, 'android/app/src/main/AndroidManifest.xml'),
        );
        expect(manifest, contains('android:label="@string/app_name"'));
        expect(manifest, contains('android:icon="@mipmap/ic_launcher"'));
      },
    );

    test(
      '$app: AGP 9.1.0 của template, plugin google-services và compileSdk',
      () {
        final settings = read(native(app, 'android/settings.gradle.kts'));
        expect(
          settings,
          contains('id("com.android.application") version "9.1.0"'),
        );
        expect(
          settings,
          contains(
            'id("com.google.gms.google-services") version "4.5.0" apply false',
          ),
        );
        final gradle = read(native(app, 'android/app/build.gradle.kts'));
        expect(gradle, contains('compileSdk = flutter.compileSdkVersion'));
        expect(gradle, contains('id("com.google.gms.google-services")'));
      },
    );
  }
}
