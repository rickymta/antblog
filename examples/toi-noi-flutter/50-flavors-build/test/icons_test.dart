import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

const densities = {
  'mdpi': 48,
  'hdpi': 72,
  'xhdpi': 96,
  'xxhdpi': 144,
  'xxxhdpi': 192,
};

/// Android: prod dùng icon của `main`; dev và staging ghi đè ở `src/<flavor>`.
String androidSourceSet(String flavor) => flavor == 'prod' ? 'main' : flavor;

List<int> androidIcon(String app, String flavor, String density) => File(
  path(
    native(
      app,
      'android/app/src/${androidSourceSet(flavor)}/res/mipmap-$density/ic_launcher.png',
    ),
  ),
).readAsBytesSync();

String iosSet(String flavor) =>
    flavor == 'prod' ? 'AppIcon' : 'AppIcon-$flavor';

List<int> iosIcon(String app, String flavor) => File(
  path(
    native(
      app,
      'ios/Runner/Assets.xcassets/${iosSet(flavor)}.appiconset/icon-1024.png',
    ),
  ),
).readAsBytesSync();

void main() {
  for (final app in apps) {
    group('Android của $app', () {
      test('mỗi mật độ có PNG đúng cỡ cho cả ba flavor', () {
        for (final entry in densities.entries) {
          for (final flavor in flavors) {
            final header = pngHeader(androidIcon(app, flavor, entry.key));
            expect(header, isNotNull, reason: '$flavor ${entry.key}');
            expect(header!.width, entry.value, reason: '$flavor ${entry.key}');
            expect(header.height, entry.value, reason: '$flavor ${entry.key}');
          }
        }
      });

      test('ở mỗi mật độ, ba flavor có ba icon khác nhau', () {
        for (final density in densities.keys) {
          final contents = {
            for (final flavor in flavors)
              flavor: String.fromCharCodes(androidIcon(app, flavor, density)),
          };
          expect(contents.values.toSet(), hasLength(3), reason: density);
        }
      });

      test('dev và staging ghi đè mọi mật độ, không bỏ sót mật độ nào', () {
        // Thiếu một mật độ ở src/<flavor> thì máy mật độ đó rơi về icon của main.
        for (final flavor in ['dev', 'staging']) {
          final dirs =
              Directory(path(native(app, 'android/app/src/$flavor/res')))
                  .listSync()
                  .map(
                    (e) => e.uri.pathSegments.where((s) => s.isNotEmpty).last,
                  )
                  .where((n) => n.startsWith('mipmap-'))
                  .toSet();
          expect(
            dirs,
            densities.keys.map((d) => 'mipmap-$d').toSet(),
            reason: flavor,
          );
        }
      });

      test('không có mipmap-anydpi: icon thích ứng (adaptive) là phạm vi chưa làm', () {
        for (final flavor in flavors) {
          expect(
            Directory(
              path(
                native(
                  app,
                  'android/app/src/${androidSourceSet(flavor)}/res/mipmap-anydpi-v26',
                ),
              ),
            ).existsSync(),
            isFalse,
          );
        }
      });
    });

    group('iOS của $app', () {
      test('mỗi bộ AppIcon có PNG 1024x1024 RGB', () {
        for (final flavor in flavors) {
          final header = pngHeader(iosIcon(app, flavor));
          expect(header, isNotNull, reason: flavor);
          expect(header!.width, 1024, reason: flavor);
          expect(header.height, 1024, reason: flavor);
          expect(header.colorType, 2, reason: flavor);
        }
      });

      test('ba flavor có ba icon khác nhau', () {
        final contents = {
          for (final flavor in flavors)
            String.fromCharCodes(iosIcon(app, flavor)),
        };
        expect(contents, hasLength(3));
      });

      test('Contents.json khai báo icon đơn kích thước cho iOS', () {
        for (final flavor in flavors) {
          final contents = readJson(
            native(
              app,
              'ios/Runner/Assets.xcassets/${iosSet(flavor)}.appiconset/Contents.json',
            ),
          );
          final image = (contents['images'] as List).single as Map;
          expect(image['filename'], 'icon-1024.png');
          expect(image['idiom'], 'universal');
          expect(image['platform'], 'ios');
          expect(image['size'], '1024x1024');
        }
      });
    });
  }

  test('bộ đọc PNG từ chối file không phải PNG', () {
    expect(
      pngHeader('không phải PNG, dài hơn 26 byte một chút'.codeUnits),
      isNull,
    );
    expect(pngHeader(const [1, 2, 3]), isNull);
  });
}
