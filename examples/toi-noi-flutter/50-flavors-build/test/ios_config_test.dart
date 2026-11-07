import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

void main() {
  final contract = readJson('flavors.json')['apps'] as Map<String, dynamic>;

  String contractId(String app, String flavor) =>
      ((contract[app] as Map)['ids'] as Map)[flavor] as String;
  String contractName(String app, String flavor) =>
      ((contract[app] as Map)['displayNames'] as Map)[flavor] as String;

  Xcconfig flavorFile(String app, String flavor) =>
      parseXcconfig(read(native(app, 'ios/Flutter/Flavors/$flavor.xcconfig')));

  for (final app in apps) {
    group('Flavors/*.xcconfig của $app', () {
      test('PRODUCT_BUNDLE_IDENTIFIER khớp hợp đồng', () {
        for (final flavor in flavors) {
          expect(
            flavorFile(app, flavor).settings['PRODUCT_BUNDLE_IDENTIFIER'],
            contractId(app, flavor),
            reason: flavor,
          );
        }
      });

      test('APP_DISPLAY_NAME khớp hợp đồng và khác nhau giữa các flavor', () {
        final names = <String>{};
        for (final flavor in flavors) {
          final name = flavorFile(app, flavor).settings['APP_DISPLAY_NAME'];
          expect(name, contractName(app, flavor), reason: flavor);
          names.add(name!);
        }
        expect(names, hasLength(3));
      });

      test('TN_FLAVOR là tên flavor, để script build phase chọn thư mục', () {
        for (final flavor in flavors) {
          expect(flavorFile(app, flavor).settings['TN_FLAVOR'], flavor);
        }
      });
    });
  }

  for (final app in apps) {
    group('chín configuration của $app', () {
      // Debug dựa trên Debug.xcconfig; Profile và Release dựa trên Release.xcconfig.
      const base = {
        'Debug': 'Debug.xcconfig',
        'Profile': 'Release.xcconfig',
        'Release': 'Release.xcconfig',
      };

      test('mỗi <Mode>-<flavor>.xcconfig include base rồi file của flavor', () {
        for (final flavor in flavors) {
          for (final mode in base.keys) {
            final config = parseXcconfig(
              read(native(app, 'ios/Flutter/$mode-$flavor.xcconfig')),
            );
            expect(config.includes, [
              base[mode],
              'Flavors/$flavor.xcconfig',
            ], reason: '$mode-$flavor');
          }
        }
      });

      test(
        'tên configuration viết thường phần flavor, như Flutter yêu cầu',
        () {
          for (final flavor in flavors) {
            expect(flavor, flavor.toLowerCase());
            for (final mode in base.keys) {
              expect(
                exists(native(app, 'ios/Flutter/$mode-$flavor.xcconfig')),
                isTrue,
              );
            }
          }
        },
      );

      test(
        'Debug.xcconfig và Release.xcconfig gốc vẫn include Generated.xcconfig',
        () {
          for (final name in ['Debug', 'Release']) {
            final config = parseXcconfig(
              read(native(app, 'ios/Flutter/$name.xcconfig')),
            );
            expect(config.includes, contains('Generated.xcconfig'));
          }
        },
      );
    });

    group('Info.plist của $app', () {
      final plist = plistStrings(read(native(app, 'ios/Runner/Info.plist')));

      test('tên hiển thị lấy từ APP_DISPLAY_NAME', () {
        expect(plist['CFBundleDisplayName'], r'$(APP_DISPLAY_NAME)');
      });

      test('bundle ID lấy từ PRODUCT_BUNDLE_IDENTIFIER, không viết cứng', () {
        expect(plist['CFBundleIdentifier'], r'$(PRODUCT_BUNDLE_IDENTIFIER)');
      });
    });

    group('AppIcon của $app', () {
      test(
        'mỗi flavor trỏ tới một appiconset có thật, và ba tên khác nhau',
        () {
          final sets = <String>{};
          for (final flavor in flavors) {
            final set = flavorFile(
              app,
              flavor,
            ).settings['ASSETCATALOG_COMPILER_APPICON_NAME']!;
            sets.add(set);
            final contents = readJson(
              native(
                app,
                'ios/Runner/Assets.xcassets/$set.appiconset/Contents.json',
              ),
            );
            final image =
                ((contents['images'] as List).first as Map)['filename'];
            expect(
              exists(
                native(
                  app,
                  'ios/Runner/Assets.xcassets/$set.appiconset/$image',
                ),
              ),
              isTrue,
              reason: '$set thiếu $image',
            );
          }
          expect(sets, {'AppIcon-dev', 'AppIcon-staging', 'AppIcon'});
        },
      );
    });
  }
}
